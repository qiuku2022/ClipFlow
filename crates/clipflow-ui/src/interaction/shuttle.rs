use clipflow_common::RationalTime;

/// 走带方向
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShuttleDirection {
    Forward,
    Backward,
    Stopped,
}

/// J-K-L 动态飞梭走带控制器
///
/// 遵循经典 NLE 交互模式：
/// - L: 正向播放，连续按加速 (1x -> 2x -> 4x -> 8x -> 16x 封顶)
/// - J: 反向播放，连续按加速 (1x -> 2x -> 4x -> 8x -> 16x 封顶)
/// - K: 瞬时急停
/// - Space: 暂停与 1x 切换
/// - K + L / K + J: 逐帧单步前进 / 后退
#[derive(Debug, Clone)]
pub struct ShuttleController {
    direction: ShuttleDirection,
    speed: u32,
}

impl Default for ShuttleController {
    fn default() -> Self {
        Self::new()
    }
}

impl ShuttleController {
    pub fn new() -> Self {
        Self {
            direction: ShuttleDirection::Stopped,
            speed: 0,
        }
    }

    pub fn speed_multiplier(&self) -> u32 {
        self.speed
    }

    pub fn direction(&self) -> ShuttleDirection {
        self.direction
    }

    pub fn is_playing(&self) -> bool {
        self.speed > 0 && self.direction != ShuttleDirection::Stopped
    }

    /// 按 L 键：正向倍速升级，或反向降速
    pub fn press_l(&mut self) {
        match self.direction {
            ShuttleDirection::Stopped => {
                self.direction = ShuttleDirection::Forward;
                self.speed = 1;
            }
            ShuttleDirection::Forward => {
                if self.speed == 0 {
                    self.speed = 1;
                } else if self.speed < 16 {
                    self.speed = (self.speed * 2).min(16);
                }
            }
            ShuttleDirection::Backward => {
                if self.speed <= 1 {
                    self.direction = ShuttleDirection::Stopped;
                    self.speed = 0;
                } else {
                    self.speed /= 2;
                }
            }
        }
    }

    /// 按 J 键：反向倍速升级，或正向降速
    pub fn press_j(&mut self) {
        match self.direction {
            ShuttleDirection::Stopped => {
                self.direction = ShuttleDirection::Backward;
                self.speed = 1;
            }
            ShuttleDirection::Backward => {
                if self.speed == 0 {
                    self.speed = 1;
                } else if self.speed < 16 {
                    self.speed = (self.speed * 2).min(16);
                }
            }
            ShuttleDirection::Forward => {
                if self.speed <= 1 {
                    self.direction = ShuttleDirection::Stopped;
                    self.speed = 0;
                } else {
                    self.speed /= 2;
                }
            }
        }
    }

    /// 按 K 键：瞬时急停
    pub fn press_k(&mut self) {
        self.direction = ShuttleDirection::Stopped;
        self.speed = 0;
    }

    /// 空格键：在暂停与 1x 正放之间切换
    pub fn press_space(&mut self) {
        if self.speed == 0 || self.direction == ShuttleDirection::Stopped {
            self.direction = ShuttleDirection::Forward;
            self.speed = 1;
        } else {
            self.direction = ShuttleDirection::Stopped;
            self.speed = 0;
        }
    }

    /// 单帧正进 (K+L)，步进后保持暂停
    pub fn step_forward(&mut self, current: RationalTime, fps: u32) -> RationalTime {
        self.press_k();
        let step = RationalTime::new(1, fps);
        current + step
    }

    /// 单帧后退 (K+J)，步退后保持暂停
    pub fn step_backward(&mut self, current: RationalTime, fps: u32) -> RationalTime {
        self.press_k();
        let step = RationalTime::new(1, fps);
        current - step
    }
}
