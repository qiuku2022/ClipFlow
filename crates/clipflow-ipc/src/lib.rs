//! ClipFlow 进程间通信与子进程协调器 (clipflow-ipc)
//!
//! 负责 Windows Job Object 防逃逸管控、命名管道 JSON-RPC 2.0 驱动与 stderr 异步排空。

pub mod job_guard;
pub mod named_pipe;
pub mod protocol;
pub mod stderr_drainer;

pub use clipflow_common::*;
pub use job_guard::JobGuard;
pub use named_pipe::{NamedPipeServerWrapper, PipeClientWrapper, PipeSession};
pub use protocol::{JsonRpcError, JsonRpcRequest, JsonRpcResponse};
pub use stderr_drainer::AsyncStderrDrainer;
