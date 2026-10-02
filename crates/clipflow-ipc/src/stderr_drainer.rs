use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncRead, BufReader};
use tokio::task::JoinHandle;

/// 子进程 stderr 异步流式排空器
///
/// 通过独立 Tokio 异步任务持续流式读取子进程标准错误输出，
/// 彻底消除 Windows MSVCRT 4KB 管道缓冲满载导致的子进程阻塞与通信死锁。
/// 同时将子进程日志结构化注入至 Rust tracing 框架。
pub struct AsyncStderrDrainer {
    lines_count: Arc<AtomicUsize>,
    handle: JoinHandle<usize>,
}

impl AsyncStderrDrainer {
    /// 挂接一个异步读取流并启动后台排空任务
    pub fn spawn<R: AsyncRead + Unpin + Send + 'static>(reader: R, process_name: String) -> Self {
        let lines_count = Arc::new(AtomicUsize::new(0));
        let count_clone = Arc::clone(&lines_count);

        let handle = tokio::spawn(async move {
            let mut lines = BufReader::new(reader).lines();
            let mut total = 0;

            while let Ok(Some(line)) = lines.next_line().await {
                total += 1;
                count_clone.store(total, Ordering::Relaxed);
                tracing::warn!(
                    target: "subprocess",
                    process = %process_name,
                    "{}",
                    line
                );
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
