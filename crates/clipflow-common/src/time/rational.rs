use serde::{Deserialize, Serialize};

/// 有理数时间戳，表示为: ticks / timescale
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RationalTime {
    /// 刻度数（可为负值，用于相对位移）
    pub value: i64,
    /// 每秒的刻度分母（Timescale），如 60000、24000 等，严禁为 0
    pub timescale: u32,
}

impl RationalTime {
    pub const ZERO: Self = Self { value: 0, timescale: 1000 };

    pub fn new(value: i64, timescale: u32) -> Self {
        assert!(timescale > 0, "Timescale must be positive");
        Self { value, timescale }
    }

    /// 转换为目标分母的等价有理数时间（保持精度）
    pub fn rescaled_to(&self, new_timescale: u32) -> Self {
        if self.timescale == new_timescale {
            return *self;
        }
        
        // Use i128 to prevent overflow during intermediate multiplication
        let scaled_val = (self.value as i128 * new_timescale as i128) / self.timescale as i128;
        
        Self {
            value: scaled_val as i64,
            timescale: new_timescale,
        }
    }

    /// 转换为浮点秒（仅在送入音频驱动或日志输出等非状态机逻辑中使用）
    pub fn to_seconds(&self) -> f64 {
        self.value as f64 / self.timescale as f64
    }

    /// 从秒与目标分母构建
    pub fn from_seconds(seconds: f64, timescale: u32) -> Self {
        Self {
            value: (seconds * timescale as f64).round() as i64,
            timescale,
        }
    }
}

