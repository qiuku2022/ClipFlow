//! ClipFlow 基础公共库 (clipflow-common)
//!
//! 提供有理数时间、时间区间、SMPTE 时间码与系统统一错误类型。

pub mod errors;
pub mod time;

pub use errors::{ClipFlowError, Result};
pub use time::{FrameRate, RationalTime, SmpteTimecode, TimeRange};
