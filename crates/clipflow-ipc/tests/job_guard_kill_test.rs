use clipflow_ipc::job_guard::JobGuard;
use std::process::Command;
use std::thread;
use std::time::Duration;

#[test]
fn test_job_guard_kill_on_close() {
    let job = JobGuard::new().expect("创建 JobGuard 失败");

    // 启动一个长时间休眠的测试子进程
    let mut cmd = Command::new("powershell");
    cmd.args(["-NoProfile", "-Command", "Start-Sleep -Seconds 30"]);

    let mut child = job.spawn_child_in_job(&mut cmd).expect("启动子进程并绑定 Job 失败");
    let pid = child.id();
    assert!(pid > 0, "子进程 PID 应大于 0");

    // 确保子进程启动并运行中
    thread::sleep(Duration::from_millis(100));
    assert!(
        child.try_wait().expect("try_wait 失败").is_none(),
        "子进程在 Job 释放前应处于运行状态"
    );

    // 显式释放 JobGuard (触发 CloseHandle -> 内核级联强杀)
    drop(job);

    // 等待 Windows 内核调度强杀子进程
    thread::sleep(Duration::from_millis(300));

    // 校验子进程已被强杀退出
    let exit_status = child.try_wait().expect("try_wait 检查退出状态失败");
    assert!(
        exit_status.is_some(),
        "JobGuard drop 后子进程应在内核级被强杀退出"
    );
}
