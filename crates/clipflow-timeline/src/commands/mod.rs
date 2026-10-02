pub mod compound;
pub mod delete;
pub mod insert;
pub mod keyframe;
pub mod ripple_delete;
pub mod split;

pub use compound::CompoundCommand;
pub use delete::DeleteClipCommand;
pub use insert::InsertClipCommand;
pub use keyframe::ModifyKeyframeCommand;
pub use ripple_delete::RippleDeleteCommand;
pub use split::SplitClipCommand;

use crate::models::sequence::Sequence;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommandError {
    #[error("Clip not found: {0}")]
    ClipNotFound(uuid::Uuid),
    #[error("Track not found: {0}")]
    TrackNotFound(uuid::Uuid),
    #[error("Invalid cut point: {0}")]
    InvalidCutPoint(String),
    #[error("Execution error: {0}")]
    ExecutionFailed(String),
}

pub trait TimelineCommand: Send + Sync {
    /// 执行命令并改变序列状态
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError>;
    /// 回退命令，恢复先前状态
    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError>;
    /// 诊断与界面展示用文案（如：“分割片段”、“波纹删除”）
    fn description(&self) -> &'static str;
}
