use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::task::JoinHandle;

/// 子进程 stderr 异步流式排空器
///
/// 通过独立 Tokio 异步任务持续流式读取子进程标准错误输出，
/// 彻底消除 Windows MSVCRT 4KB 管道缓冲满载导致的子进程阻塞与通信死锁。
/// 支持容错字符编码转换 (lossy UTF-8)，并将子进程日志结构化注入至 Rust tracing 框架。
pub struct AsyncStderrDrainer {
    lines_count: Arc<AtomicUsize>,
    handle: JoinHandle<usize>,
}

impl AsyncStderrDrainer {
    /// 挂接一个异步读取流并启动后台排空任务 (适用于 tokio::process::ChildStderr)
    pub fn spawn<R: AsyncRead + Unpin + Send + 'static>(reader: R, process_name: String) -> Self {
        let lines_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&lines_count);

        let handle = tokio::spawn(async move {
            let mut reader = BufReader::new(reader);
            let mut total = 0;
            let mut buf = Vec::new();

            while let Ok(n) = reader.read_until(b'\n', &mut buf).await {
                if n == 0 {
                    break;
                }
                total += 1;
                count_clone.store(total, Ordering::Relaxed);

                let line = String::from_utf8_lossy(&buf);
                let trimmed = line.trim_end_matches(['\r', '\n']);
                tracing::warn!(
                    target: "subprocess",
                    process = %process_name,
                    "{}",
                    trimmed
                );
                buf.clear();
            }

            total
        });

        Self {
            lines_count,
            handle,
        }
    }

    /// 从同步的 std::process::ChildStderr 构造排空任务 (容错 lossy 编码读取)
    pub fn spawn_std(stderr: std::process::ChildStderr, process_name: String) -> Self {
        let lines_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&lines_count);

        let handle = tokio::task::spawn_blocking(move || {
            use std::io::{BufRead, BufReader};
            let mut reader = BufReader::new(stderr);
            let mut total = 0;
            let mut buf = Vec::new();

            while let Ok(n) = reader.read_until(b'\n', &mut buf) {
                if n == 0 {
                    break;
                }
                total += 1;
                count_clone.store(total, Ordering::Relaxed);

                let line = String::from_utf8_lossy(&buf);
                let trimmed = line.trim_end_matches(['\r', '\n']);
                tracing::warn!(
                    target: "subprocess",
                    process = %process_name,
                    "{}",
                    trimmed
                );
                buf.clear();
            }

            total
        });

        Self {
            lines_count,
            handle,
        }
    }

    /// 获取当前已排空的日志总行数
    pub fn lines_drained(&self) -> usize {
        self.lines_count.load(Ordering::Relaxed)
    }

    /// 等待排空任务结束并返回消费的日志总行数
    pub async fn join(self) -> Result<usize, tokio::task::JoinError> {
        self.handle.await
    }
}
