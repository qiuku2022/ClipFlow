//! ClipFlow 进程间通信与子进程协调器 (clipflow-ipc)
//!
//! 负责 Windows Job Object 防逃逸管控、命名管道 JSON-RPC 2.0 驱动与 stderr 异步排空。

pub use clipflow_common::*;
