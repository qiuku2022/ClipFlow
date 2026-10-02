use clipflow_media::audio::{
    AudioWatchdog, ClockAnchor, HysteresisSyncComparator, MasterClockProvider,
    MonotonicClampedClock, SyncAction, TransportController, TransportState,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_monotonic_clamped_clock_zero_inversion() {
    let clock = Arc::new(MonotonicClampedClock::new(10_000_000)); // 10ms 周期
    clock.set_state(TransportState::Playing);

    // 初始锚点：DAC 0ns，QPC 1_000_000ns
    clock.update_anchor(ClockAnchor {
        dac_pos_ns: 0,
        qpc_ns: 1_000_000,
        period_ns: 10_000_000,
    });

    let running = Arc::new(AtomicBool::new(true));
    let mut handles = vec![];

    // 启动 4 个并发线程高频查询 now_ns()，断言时间永远单调非递减
    for _ in 0..4 {
        let c = clock.clone();
        let r = running.clone();
        handles.push(thread::spawn(move || {
            let mut last = 0u64;
            let mut samples = 0;
            while r.load(Ordering::Relaxed) && samples < 25_000 {
                let now = c.now_ns();
                assert!(
                    now >= last,
                    "Time inversion detected! now: {} < last: {}",
                    now,
                    last
                );
                last = now;
                samples += 1;
            }
        }));
    }

    // 主线程模拟音频中断：故意注入跳跃和轻度欠载锚点
    for i in 1..=20 {
        thread::sleep(Duration::from_millis(2));
        clock.update_anchor(ClockAnchor {
            dac_pos_ns: i * 2_000_000,
            qpc_ns: 1_000_000 + i * 2_000_000,
            period_ns: 10_000_000,
        });
    }

    running.store(false, Ordering::Relaxed);
    for h in handles {
        h.join().unwrap();
    }
}

#[test]
fn test_monotonic_clock_clamp_upper_bound() {
    let clock = MonotonicClampedClock::new(10_000_000); // 10ms
    clock.set_state(TransportState::Playing);

    clock.update_anchor(ClockAnchor {
        dac_pos_ns: 100_000_000,
        qpc_ns: 100_000_000,
        period_ns: 10_000_000,
    });

    // 睡眠 50ms（远大于 15ms 箝位），外推不应该超过 15ms = 15_000_000ns
    thread::sleep(Duration::from_millis(50));
    let now = clock.now_ns();
    let extrapolation = now - 100_000_000;
    assert!(
        extrapolation <= 16_000_000, // 允许少量误差
        "Extrapolation exceeded clamp ceiling: {} ns",
        extrapolation
    );
}

#[test]
fn test_hysteresis_sync_comparator_decision_loop() {
    let mut comparator = HysteresisSyncComparator::new();

    // 1. 理想对齐 Δt = 0ms -> InLock
    let action = comparator.evaluate(0);
    assert_eq!(action, SyncAction::RenderCurrentFrame);
    assert!(comparator.is_in_lock());

    // 2. 偏差轻微扩大到 +10ms（未超出 12ms 退出阈值）-> 维持 InLock
    let action = comparator.evaluate(10_000_000);
    assert_eq!(action, SyncAction::RenderCurrentFrame);
    assert!(comparator.is_in_lock());

    // 3. 偏差穿透 +14ms -> 退出 InLock，切入 HoldPreviousFrame
    let action = comparator.evaluate(14_000_000);
    assert_eq!(action, SyncAction::HoldPreviousFrame);
    assert!(!comparator.is_in_lock());

    // 4. 偏差回落至 +10ms（未达到 8ms 恢复阈值）-> 依然处于调整态
    let action = comparator.evaluate(10_000_000);
    assert_eq!(action, SyncAction::HoldPreviousFrame);
    assert!(!comparator.is_in_lock());

    // 5. 偏差收敛至 +6ms（达到 8ms 恢复阈值）-> 重新切回 InLock
    let action = comparator.evaluate(6_000_000);
    assert_eq!(action, SyncAction::RenderCurrentFrame);
    assert!(comparator.is_in_lock());

    // 6. 严重滞后 -45ms -> DropCurrentFrame
    let action = comparator.evaluate(-45_000_000);
    assert_eq!(action, SyncAction::DropCurrentFrame);
}

#[test]
fn test_transport_controller_state_machine() {
    let mut controller = TransportController::new();
    assert_eq!(controller.state(), TransportState::Paused);

    controller.play();
    assert_eq!(controller.state(), TransportState::Playing);

    controller.pause();
    assert_eq!(controller.state(), TransportState::Paused);

    controller.start_scrubbing(50_000_000);
    assert_eq!(controller.state(), TransportState::Scrubbing);
    assert_eq!(controller.scrub_target_ns(), 50_000_000);

    controller.stop_scrubbing();
    assert_eq!(controller.state(), TransportState::Paused);
}

#[test]
fn test_audio_watchdog_timeout_fallback() {
    let mut watchdog = AudioWatchdog::new(Duration::from_millis(50));
    assert!(!watchdog.is_fallback_active());

    watchdog.tick_heartbeat();
    assert!(!watchdog.check_timeout());

    thread::sleep(Duration::from_millis(70));
    // 超时后切入 fallback 模式
    assert!(watchdog.check_timeout());
    assert!(watchdog.is_fallback_active());

    // 重新收到心跳自愈
    watchdog.tick_heartbeat();
    assert!(!watchdog.is_fallback_active());
}
