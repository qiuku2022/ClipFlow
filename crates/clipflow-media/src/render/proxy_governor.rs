use std::time::Duration;

/// 监视器代理分辨率档位
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProxyResolution {
    /// 全分辨率原始渲染
    Full,
    /// 1/4 代理分辨率渲染 (高负荷/掉帧/高速飞梭时启用)
    Quarter,
}

/// 监视器 1/4 代理流控总督
///
/// 监控回放负载、倍速走带及渲染耗时。
/// 当连续 3 帧渲染超时或播放倍速 > 2x 时，动态降级为 1/4 代理并激活半透明黄色警告框；
/// 并在负载平稳 (连续 10 帧正常) 且倍速 <= 2x 时无感回切 Full。
#[derive(Debug, Clone)]
pub struct ProxyGovernor {
    resolution: ProxyResolution,
    warning_border: bool,
    consecutive_dropped_frames: usize,
    consecutive_normal_frames: usize,
    target_budget: Duration,
    high_speed_active: bool,
}

impl Default for ProxyGovernor {
    fn default() -> Self {
        Self::new()
    }
}

impl ProxyGovernor {
    pub fn new() -> Self {
        Self {
            resolution: ProxyResolution::Full,
            warning_border: false,
            consecutive_dropped_frames: 0,
            consecutive_normal_frames: 0,
            target_budget: Duration::from_nanos(16_666_667), // 60 FPS 预算 16.67ms
            high_speed_active: false,
        }
    }

    pub fn resolution(&self) -> ProxyResolution {
        self.resolution
    }

    pub fn is_warning_border_active(&self) -> bool {
        self.warning_border
    }

    /// 根据播放倍速与外部掉帧报告更新流控
    pub fn update_playback_conditions(&mut self, speed_multiplier: u32, _dropped_frames: usize) {
        if speed_multiplier > 2 {
            self.high_speed_active = true;
            self.resolution = ProxyResolution::Quarter;
            self.warning_border = true;
        } else {
            self.high_speed_active = false;
            if self.consecutive_dropped_frames < 3 {
                self.resolution = ProxyResolution::Full;
                self.warning_border = false;
            }
        }
    }

    /// 记录单帧 GPU 提交与呈现总耗时
    pub fn record_render_frame(&mut self, render_time: Duration) {
        if render_time > self.target_budget {
            self.consecutive_dropped_frames += 1;
            self.consecutive_normal_frames = 0;

            if self.consecutive_dropped_frames >= 3 {
                self.resolution = ProxyResolution::Quarter;
                self.warning_border = true;
            }
        } else {
            self.consecutive_dropped_frames = 0;
            self.consecutive_normal_frames += 1;

            if self.consecutive_normal_frames >= 10 && !self.high_speed_active {
                self.resolution = ProxyResolution::Full;
                self.warning_border = false;
            }
        }
    }
}
