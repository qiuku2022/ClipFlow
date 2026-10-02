use std::time::Duration;

/// GPU 渲染设备生命周期状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceState {
    /// 正常工作
    Normal,
    /// 崩溃捕获与静默恢复中
    Recovering,
}

/// wgpu / D3D11 DeviceLost 容灾看门狗
///
/// 当显卡发生 TDR 重置、驱动崩溃或外接显卡插拔触发 DeviceLost 时：
/// 1. 立即隔离并销毁已有渲染管线与资源；
/// 2. 维持 300ms 观察窗口期，等待 GPU 驱动内核重置就绪；
/// 3. 超出 300ms 后准许触发无感静默热重建；
/// 4. 统计故障与恢复计数。
#[derive(Debug, Clone)]
pub struct DeviceLostWatchdog {
    state: DeviceState,
    last_error: Option<String>,
    recovery_count: usize,
    silence_window: Duration,
}

impl Default for DeviceLostWatchdog {
    fn default() -> Self {
        Self::new()
    }
}

impl DeviceLostWatchdog {
    pub fn new() -> Self {
        Self {
            state: DeviceState::Normal,
            last_error: None,
            recovery_count: 0,
            silence_window: Duration::from_millis(300),
        }
    }

    pub fn state(&self) -> DeviceState {
        self.state
    }

    pub fn recovery_count(&self) -> usize {
        self.recovery_count
    }

    pub fn last_error_reason(&self) -> Option<&str> {
        self.last_error.as_deref()
    }

    /// 触发设备丢失事件
    pub fn trigger_device_lost(&mut self, reason: &str) {
        self.state = DeviceState::Recovering;
        self.last_error = Some(reason.to_string());
    }

    /// 检查是否已度过静默观察期并准许重建管线
    pub fn is_ready_to_rebuild(&self, elapsed: Duration) -> bool {
        self.state == DeviceState::Recovering && elapsed >= self.silence_window
    }

    /// 完成静默热重建，重置设备状态
    pub fn complete_rebuild(&mut self) {
        self.state = DeviceState::Normal;
        self.recovery_count += 1;
        self.last_error = None;
    }
}
