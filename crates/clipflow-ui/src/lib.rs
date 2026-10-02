//! ClipFlow 全局用户界面库 (clipflow-ui)
//!
//! 负责 Neutral Modern 深色主题、达芬奇式底部 Dock 栏、PR 分屏与时间线视图。

pub mod dock;
pub mod state;
pub mod theme;
pub mod timeline;
pub mod timeline_placeholder;
pub mod views;

pub use clipflow_common::*;
pub use dock::{WorkflowDock, WorkflowPage};
pub use state::{AppState, RepaintScheduler, RepaintState};
pub use theme::ClipFlowTheme;
pub use timeline::*;
pub use timeline_placeholder::render_timeline_placeholder;
pub use views::*;
