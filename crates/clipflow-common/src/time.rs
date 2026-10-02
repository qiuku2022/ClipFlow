use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Neg, Sub};

/// 有理数时间戳，表示为: ticks / timescale
#[derive(Debug, Clone, Copy, Eq, Hash, Serialize, Deserialize)]
pub struct RationalTime {
    /// 刻度数（可为负值，用于相对位移）
    pub value: i64,
    /// 每秒的刻度分母（Timebase），如 60000、24000 等，严禁为 0
    pub timescale: u32,
}

impl RationalTime {
    pub const ZERO: Self = Self {
        value: 0,
        timescale: 1000,
    };

    pub fn new(value: i64, timescale: u32) -> Self {
        assert!(timescale > 0, "Timescale must be positive");
        Self { value, timescale }
    }

    /// 转换为目标分母的等价有理数时间（使用 i128 计算防溢出）
    pub fn rescaled_to(&self, new_timescale: u32) -> Self {
        assert!(new_timescale > 0, "New timescale must be positive");
        if self.timescale == new_timescale {
            return *self;
        }
        let scaled_val = (self.value as i128 * new_timescale as i128) / self.timescale as i128;
        Self {
            value: scaled_val as i64,
            timescale: new_timescale,
        }
    }

    /// 转换为浮点秒（仅用于驱动/日志/UI展示）
    pub fn to_seconds(&self) -> f64 {
        self.value as f64 / self.timescale as f64
    }

    /// 从秒与目标分母构建
    pub fn from_seconds(seconds: f64, timescale: u32) -> Self {
        assert!(timescale > 0, "Timescale must be positive");
        Self {
            value: (seconds * timescale as f64).round() as i64,
            timescale,
        }
    }
}

impl Add for RationalTime {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        if self.timescale == rhs.timescale {
            Self::new(self.value + rhs.value, self.timescale)
        } else {
            let common_scale = self.timescale.max(rhs.timescale);
            let s1 = self.rescaled_to(common_scale);
            let s2 = rhs.rescaled_to(common_scale);
            Self::new(s1.value + s2.value, common_scale)
        }
    }
}

impl Sub for RationalTime {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        if self.timescale == rhs.timescale {
            Self::new(self.value - rhs.value, self.timescale)
        } else {
            let common_scale = self.timescale.max(rhs.timescale);
            let s1 = self.rescaled_to(common_scale);
            let s2 = rhs.rescaled_to(common_scale);
            Self::new(s1.value - s2.value, common_scale)
        }
    }
}

impl Neg for RationalTime {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self::new(-self.value, self.timescale)
    }
}

impl PartialEq for RationalTime {
    fn eq(&self, other: &Self) -> bool {
        (self.value as i128 * other.timescale as i128)
            == (other.value as i128 * self.timescale as i128)
    }
}

impl PartialOrd for RationalTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RationalTime {
    fn cmp(&self, other: &Self) -> Ordering {
        let left = self.value as i128 * other.timescale as i128;
        let right = other.value as i128 * self.timescale as i128;
        left.cmp(&right)
    }
}

impl fmt::Display for RationalTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.value, self.timescale)
    }
}

/// 时间区间：左闭右开 [start, start + duration)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    pub fn new(start: RationalTime, duration: RationalTime) -> Self {
        assert!(duration.value >= 0, "Duration cannot be negative");
        Self { start, duration }
    }

    pub fn end_exclusive(&self) -> RationalTime {
        let dur = self.duration.rescaled_to(self.start.timescale);
        RationalTime::new(self.start.value + dur.value, self.start.timescale)
    }

    pub fn contains(&self, time: RationalTime) -> bool {
        let t = time.rescaled_to(self.start.timescale);
        let end = self.end_exclusive();
        t.value >= self.start.value && t.value < end.value
    }
}

/// 标准视频帧率枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRate {
    Fps23_976, // 24000/1001
    Fps24,     // 24/1
    Fps25,     // 25/1 (PAL)
    Fps29_97,  // 30000/1001 (NTSC)
    Fps30,     // 30/1
    Fps50,     // 50/1
    Fps59_94,  // 60000/1001
    Fps60,     // 60/1
}

impl FrameRate {
    pub fn fps_rational(&self) -> (u32, u32) {
        match self {
            FrameRate::Fps23_976 => (24000, 1001),
            FrameRate::Fps24 => (24, 1),
            FrameRate::Fps25 => (25, 1),
            FrameRate::Fps29_97 => (30000, 1001),
            FrameRate::Fps30 => (30, 1),
            FrameRate::Fps50 => (50, 1),
            FrameRate::Fps59_94 => (60000, 1001),
            FrameRate::Fps60 => (60, 1),
        }
    }
}

/// SMPTE 时间码结构
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub is_drop_frame: bool,
}

impl SmpteTimecode {
    pub fn to_string(&self) -> String {
        let delimiter = if self.is_drop_frame { ';' } else { ':' };
        format!(
            "{:02}:{:02}:{:02}{}{:02}",
            self.hours, self.minutes, self.seconds, delimiter, self.frames
        )
    }

    pub fn from_rational_time(time: RationalTime, fps: FrameRate) -> Self {
        let (fps_num, fps_den) = fps.fps_rational();
        // 计算当前时间的总帧数: (time.value * fps_num) / (time.timescale * fps_den)
        let total_frames = ((time.value as i128 * fps_num as i128)
            / (time.timescale as i128 * fps_den as i128))
            .max(0) as u64;

        let fps_int = (fps_num / fps_den) as u64;
        let frames = (total_frames % fps_int) as u8;
        let total_seconds = total_frames / fps_int;

        let seconds = (total_seconds % 60) as u8;
        let total_minutes = total_seconds / 60;
        let minutes = (total_minutes % 60) as u8;
        let hours = (total_minutes / 60) as u8;

        let is_drop_frame = matches!(fps, FrameRate::Fps29_97 | FrameRate::Fps59_94);

        Self {
            hours,
            minutes,
            seconds,
            frames,
            is_drop_frame,
        }
    }
}

impl fmt::Display for SmpteTimecode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
