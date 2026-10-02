use clipflow_common::{RationalTime, TimeRange};
use clipflow_media::audio::{
    ClockAnchor, HysteresisSyncComparator, MasterClockProvider, MonotonicClampedClock, SyncAction,
    TransportState,
};
use clipflow_media::decode::{MockVideoDecoder, VideoTextureProvider};
use clipflow_media::render::{DeviceLostWatchdog, DeviceState, ProxyGovernor, ProxyResolution};
use clipflow_timeline::models::{
    AudioProperties, CanvasSize, Clip, ClipPayload, Sequence, Track, TrackKind, Transform2D,
};
use clipflow_ui::interaction::{ShuttleController, ShuttleDirection};
use std::time::Duration;
use uuid::Uuid;

fn create_clip(name: &str, start_ticks: i64, dur_ticks: i64, timescale: u32) -> Clip {
    Clip {
        id: Uuid::new_v4(),
        name: name.to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(
            RationalTime::new(0, timescale),
            RationalTime::new(dur_ticks, timescale),
        ),
        timeline_range: TimeRange::new(
            RationalTime::new(start_ticks, timescale),
            RationalTime::new(dur_ticks, timescale),
        ),
        speed: 1.0,
        transform: Transform2D::default(),
        audio_props: AudioProperties::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    }
}

#[test]
fn test_m1_long_run_playback_and_drift_benchmark() {
    println!("=== 启动 M1 60秒 3600帧 端到端音画同步仿真基准 ===");

    // 1. 装配数据模型：包含 1 条视频轨、1 条音频轨
    let mut seq = Sequence::new("M1 验收工程", CanvasSize::P1080_16_9, 60, 1);
    let clip_v = create_clip("4k_sample.mp4", 0, 3600, 60);
    let clip_a = create_clip("audio_master.wav", 0, 3600, 60);

    let v_track = Track {
        id: Uuid::new_v4(),
        name: "V1".to_string(),
        kind: TrackKind::Video,
        mute: false,
        solo: false,
        locked: false,
        visible: true,
        audio_props: None,
        clips: vec![clip_v],
    };
    let a_track = Track {
        id: Uuid::new_v4(),
        name: "A1".to_string(),
        kind: TrackKind::Audio,
        mute: false,
        solo: false,
        locked: false,
        visible: true,
        audio_props: None,
        clips: vec![clip_a],
    };
    seq.tracks.push(v_track);
    seq.tracks.push(a_track);

    // 2. 初始化核心引擎与时钟
    let period_ns = 16_666_667u64; // 60 FPS (约 16.666ms)
    let clock = MonotonicClampedClock::new(period_ns);
    clock.set_state(TransportState::Playing);

    let mut comparator = HysteresisSyncComparator::new();
    let mut decoder = MockVideoDecoder::new(1920, 1080, 60);
    let mut governor = ProxyGovernor::new();
    let watchdog = DeviceLostWatchdog::new();

    // 3. 仿真 3600 帧播放推进
    let mut max_abs_drift_ns: i64 = 0;
    let mut total_rendered_frames = 0;
    let mut total_held_frames = 0;

    for frame_idx in 0..3600 {
        // 模拟音频硬件时钟周期中断（更新 DAC 硬件锚点）
        let audio_dac_pos_ns = frame_idx as u64 * period_ns;
        clock.update_anchor(ClockAnchor {
            dac_pos_ns: audio_dac_pos_ns,
            qpc_ns: audio_dac_pos_ns, // 理想硬件同步
            period_ns,
        });

        let master_pts_ns = clock.now_ns() as i64;
        let video_pts_ns = frame_idx as i64 * period_ns as i64;

        // 计算音画时延 delta_ns = video_pts - master_pts
        let delta_ns = video_pts_ns - master_pts_ns;
        if delta_ns.abs() > max_abs_drift_ns {
            max_abs_drift_ns = delta_ns.abs();
        }

        // 施密特迟滞决策
        let action = comparator.evaluate(delta_ns);
        match action {
            SyncAction::RenderCurrentFrame => {
                total_rendered_frames += 1;
                let frame = decoder.poll_next_frame();
                assert!(frame.is_some());
                governor.record_render_frame(Duration::from_millis(5));
            }
            SyncAction::HoldPreviousFrame => {
                total_held_frames += 1;
            }
            SyncAction::DropCurrentFrame => {
                let _ = decoder.poll_next_frame();
            }
        }

        assert!(master_pts_ns >= 0);
    }

    let max_drift_ms = max_abs_drift_ns as f64 / 1_000_000.0;
    println!(
        "仿真结束: 总帧数=3600, 渲染帧={}, 轻微持帧={}, 最大音画漂移={:.4}ms",
        total_rendered_frames, total_held_frames, max_drift_ms
    );

    // 断言 1: 累积音画漂移 <= 2.0ms
    assert!(
        max_drift_ms <= 2.0,
        "音画漂移超标: {:.4}ms > 2.0ms",
        max_drift_ms
    );

    // 断言 2: 渲染呈现率 >= 95%
    assert!(
        total_rendered_frames >= 3400,
        "正常渲染帧率偏低: {}",
        total_rendered_frames
    );

    // 断言 3: 容灾看门狗保持正常
    assert_eq!(watchdog.state(), DeviceState::Normal);
    assert_eq!(governor.resolution(), ProxyResolution::Full);
}

#[test]
fn test_m1_shuttle_cascade_and_proxy_governor_integration() {
    let mut shuttle = ShuttleController::new();
    let mut governor = ProxyGovernor::new();

    // 1x 播放
    shuttle.press_l();
    assert_eq!(shuttle.speed_multiplier(), 1);
    governor.update_playback_conditions(shuttle.speed_multiplier(), 0);
    assert_eq!(governor.resolution(), ProxyResolution::Full);
    assert!(!governor.is_warning_border_active());

    // 连续按 L 加速到 4x, 8x
    shuttle.press_l(); // 2x
    shuttle.press_l(); // 4x
    assert_eq!(shuttle.speed_multiplier(), 4);

    governor.update_playback_conditions(shuttle.speed_multiplier(), 0);
    assert_eq!(governor.resolution(), ProxyResolution::Quarter);
    assert!(governor.is_warning_border_active());

    // 急停重置
    shuttle.press_k();
    assert_eq!(shuttle.speed_multiplier(), 0);
    assert_eq!(shuttle.direction(), ShuttleDirection::Stopped);

    // 恢复常规 1x 播放
    shuttle.press_space();
    assert_eq!(shuttle.speed_multiplier(), 1);
    governor.update_playback_conditions(shuttle.speed_multiplier(), 0);
    assert_eq!(governor.resolution(), ProxyResolution::Full);
    assert!(!governor.is_warning_border_active());
}

#[test]
fn test_m1_burst_dropped_frame_recovery_integration() {
    let mut comparator = HysteresisSyncComparator::new();
    let mut governor = ProxyGovernor::new();

    // 注入极端滞后: 视频滞后 -45ms (即 delta_ns = -45_000_000)
    let lagging_delta_ns = -45_000_000i64;
    let decision = comparator.evaluate(lagging_delta_ns);
    assert_eq!(decision, SyncAction::DropCurrentFrame);

    // 记录连续 3 帧高延迟，触发 1/4 代理保护
    governor.record_render_frame(Duration::from_millis(30));
    governor.record_render_frame(Duration::from_millis(32));
    governor.record_render_frame(Duration::from_millis(28));
    assert_eq!(governor.resolution(), ProxyResolution::Quarter);

    // 视频追上音频 (进入 [-8ms, +8ms] 完美同步区间)
    let sync_delta_ns = 2_000_000i64; // +2ms
    let sync_decision = comparator.evaluate(sync_delta_ns);
    assert_eq!(sync_decision, SyncAction::RenderCurrentFrame);
}
