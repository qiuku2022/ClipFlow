use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipFlowError {
    #[error("I/O 错误: {0}")]
    Io(#[from] std::io::Error),
    #[error("IPC 错误: {0}")]
    Ipc(String),
    #[error("时间轴错误: {0}")]
    Timeline(String),
    #[error("多媒体错误: {0}")]
    Media(String),
    #[error("无效时间参数: {0}")]
    InvalidTime(String),
}

pub type Result<T> = std::result::Result<T, ClipFlowError>;
