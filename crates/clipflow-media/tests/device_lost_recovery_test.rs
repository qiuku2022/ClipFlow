use clipflow_media::render::{
    DeviceLostWatchdog, DeviceState, ProxyGovernor, ProxyResolution,
};
use std::time::Duration;

#[test]
fn test_proxy_governor_high_speed_downgrade() {
    let mut governor = ProxyGovernor::new();
    assert_eq!(governor.resolution(), ProxyResolution::Full);
    assert!(!governor.is_warning_border_active());

    // 1x, 2x 保持 Full 分辨率
    governor.update_playback_conditions(1, 0);
    assert_eq!(governor.resolution(), ProxyResolution::Full);

    governor.update_playback_conditions(2, 0);
    assert_eq!(governor.resolution(), ProxyResolution::Full);

    // > 2x (4x, 8x, 16x) 自动降级为 1/4 代理并激活黄色边框提示
    governor.update_playback_conditions(4, 0);
    assert_eq!(governor.resolution(), ProxyResolution::Quarter);
    assert!(governor.is_warning_border_active());

    // 恢复为 1x 播放，平滑回到 Full 分辨率并消除边框
    governor.update_playback_conditions(1, 0);
    assert_eq!(governor.resolution(), ProxyResolution::Full);
    assert!(!governor.is_warning_border_active());
}

#[test]
fn test_proxy_governor_dropped_frame_threshold() {
    let mut governor = ProxyGovernor::new();

    // 连续掉帧测试：阈值为连续 3 帧
    governor.record_render_frame(Duration::from_millis(10)); // 正常
    governor.record_render_frame(Duration::from_millis(20)); // 超时 1
    governor.record_render_frame(Duration::from_millis(22)); // 超时 2
    assert_eq!(governor.resolution(), ProxyResolution::Full);

    governor.record_render_frame(Duration::from_millis(25)); // 连续超时第 3 帧 -> 触发降级
    assert_eq!(governor.resolution(), ProxyResolution::Quarter);
    assert!(governor.is_warning_border_active());

    // 连续 10 帧正常后自动恢复
    for _ in 0..9 {
        governor.record_render_frame(Duration::from_millis(8));
        assert_eq!(governor.resolution(), ProxyResolution::Quarter);
    }
    governor.record_render_frame(Duration::from_millis(8)); // 第 10 帧正常 -> 恢复 Full
    assert_eq!(governor.resolution(), ProxyResolution::Full);
    assert!(!governor.is_warning_border_active());
}

#[test]
fn test_device_lost_watchdog_silence_window_and_rebuild() {
    let mut watchdog = DeviceLostWatchdog::new();
    assert_eq!(watchdog.state(), DeviceState::Normal);
    assert_eq!(watchdog.recovery_count(), 0);

    // 触发设备丢失（如驱动崩溃或 wgpu 设备被移除）
    watchdog.trigger_device_lost("DXGI_ERROR_DEVICE_RESET");
    assert_eq!(watchdog.state(), DeviceState::Recovering);
    assert_eq!(watchdog.last_error_reason(), Some("DXGI_ERROR_DEVICE_RESET"));

    // 静默观察期为 300ms：200ms 时不可重建
    assert!(!watchdog.is_ready_to_rebuild(Duration::from_millis(200)));

    // 超过 300ms (如 350ms) 后准许安全静默重建
    assert!(watchdog.is_ready_to_rebuild(Duration::from_millis(350)));

    // 完成重建
    watchdog.complete_rebuild();
    assert_eq!(watchdog.state(), DeviceState::Normal);
    assert_eq!(watchdog.recovery_count(), 1);
}
