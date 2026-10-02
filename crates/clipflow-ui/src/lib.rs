//! ClipFlow 全局用户界面库 (clipflow-ui)
//!
//! 负责 Neutral Modern 深色主题、达芬奇式底部 Dock 栏、PR 分屏与时间线视图。

pub mod state;
pub mod theme;

pub use clipflow_common::*;
pub use state::{AppState, RepaintScheduler, RepaintState};
pub use theme::ClipFlowTheme;
