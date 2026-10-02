use clipflow_ipc::stderr_drainer::AsyncStderrDrainer;
use std::process::Stdio;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::timeout;

#[tokio::test]
async fn test_stderr_drain_10000_lines_no_deadlock() {
    // 启动一个连续向 stderr 刷入 10000 行文本的子进程 (约 300KB 数据量，远超 MSVCRT 4KB 缓冲区)
    let mut cmd = Command::new("powershell");
    cmd.args([
        "-NoProfile",
        "-Command",
        r#"1..10000 | ForEach-Object { [Console]::Error.WriteLine("ClipFlow Error Line $_") }"#,
    ]);
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::null());

    let mut child = cmd.spawn().expect("启动子进程失败");
    let stderr = child.stderr.take().expect("获取 stderr 失败");

    // 挂接 AsyncStderrDrainer
    let drainer = AsyncStderrDrainer::spawn(stderr, "test_spam_worker".to_string());

    // 设置 5 秒超时保护，若发生死锁超时必触发
    let wait_result = timeout(Duration::from_secs(5), child.wait()).await;
    assert!(wait_result.is_ok(), "子进程触发了 4KB 缓冲死锁！未在 5 秒内完成");

    let status = wait_result.unwrap().expect("child.wait 失败");
    assert!(status.success(), "子进程退出状态应成功");

    let drained_count = drainer.join().await.expect("join 失败");
    println!("成功异步排空 {} 行 stderr 日志，0 阻塞！", drained_count);
    assert_eq!(drained_count, 10000, "应完整排空全部 10000 行日志");
}
