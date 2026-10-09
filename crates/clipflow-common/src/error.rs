use thiserror::Error;

#[derive(Error, Debug)]
pub enum ClipFlowError {
    #[error("Invalid arguments: {0}")]
    InvalidArgument(String),

    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    
    #[error("Internal logic error: {0}")]
    Internal(String),
}

