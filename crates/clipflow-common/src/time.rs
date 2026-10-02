use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RationalTime {
    pub value: i64,
    pub timescale: u32,
}

impl RationalTime {
    pub const ZERO: Self = Self { value: 0, timescale: 1000 };

    pub fn new(value: i64, timescale: u32) -> Self {
        assert!(timescale > 0, "Timescale must be positive");
        Self { value, timescale }
    }

    pub fn to_seconds(&self) -> f64 {
        self.value as f64 / self.timescale as f64
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    pub fn new(start: RationalTime, duration: RationalTime) -> Self {
        Self { start, duration }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRate {
    Fps23_976,
    Fps24,
    Fps25,
    Fps29_97,
    Fps30,
    Fps50,
    Fps59_94,
    Fps60,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub is_drop_frame: bool,
}
