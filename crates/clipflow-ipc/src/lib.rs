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

use std::path::{Path, PathBuf};

/// 解析项目根目录下 Python 运行时解释器路径与环境变量
///
/// 优先穿透 .venv/pyvenv.cfg 解析底层无 shim 的真实 python.exe，
/// 规避 Windows virtualenv trampoline shim 导致的 PID 错位与 stderr 缓冲丢失。
pub fn resolve_python_runtime(root_dir: &Path) -> (PathBuf, Vec<(String, String)>) {
    let mut envs = vec![
        ("PYTHONUNBUFFERED".to_string(), "1".to_string()),
        ("PYTHONIOENCODING".to_string(), "utf-8".to_string()),
        ("PYTHONPATH".to_string(), root_dir.join("python").to_string_lossy().to_string()),
    ];

    let venv_dir = root_dir.join(".venv");
    if venv_dir.exists() {
        envs.push(("VIRTUAL_ENV".to_string(), venv_dir.to_string_lossy().to_string()));

        let cfg_path = venv_dir.join("pyvenv.cfg");
        if let Ok(content) = std::fs::read_to_string(cfg_path) {
            for line in content.lines() {
                if let Some(home) = line.strip_prefix("home = ") {
                    let real_python = Path::new(home.trim()).join("python.exe");
                    if real_python.exists() {
                        return (real_python, envs);
                    }
                }
            }
        }

        let venv_python = venv_dir.join("Scripts").join("python.exe");
        if venv_python.exists() {
            return (venv_python, envs);
        }
    }

    (PathBuf::from("python"), envs)
}
