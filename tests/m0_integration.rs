use clipflow_common::{FrameRate, RationalTime, SmpteTimecode, TimeRange};
use clipflow_ipc::job_guard::JobGuard;
use clipflow_ipc::named_pipe::NamedPipeServerWrapper;
use clipflow_ipc::protocol::JsonRpcRequest;
use clipflow_ipc::resolve_python_runtime;
use clipflow_ipc::stderr_drainer::AsyncStderrDrainer;
use clipflow_ui::dock::WorkflowPage;
use clipflow_ui::state::{AppState, RepaintScheduler, RepaintState};
use clipflow_ui::theme::ClipFlowTheme;
use eframe::egui::Color32;
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_m0_common_and_ui_integration() {
    println!(">>> 验证 M0 Common 与 UI 主题集成...");

    // 1. 时间数学库与 SMPTE 帧率换算
    let t = RationalTime::new(72000, 24); // 3000 秒
    assert_eq!(t.to_seconds(), 3000.0);
    let tc = SmpteTimecode::from_rational_time(t, FrameRate::Fps24);
    assert_eq!(tc.to_string(), "00:50:00:00");

    let range = TimeRange::new(RationalTime::ZERO, t);
    assert!(range.contains(RationalTime::new(1000, 24)));
    assert!(!range.contains(RationalTime::new(72000, 24)));

    // 2. OpenDesign Neutral Modern 主题 Token 对齐
    assert_eq!(ClipFlowTheme::BG_CANVAS, Color32::from_rgb(15, 17, 21));
    assert_eq!(ClipFlowTheme::SURFACE, Color32::from_rgb(23, 26, 33));
    assert_eq!(ClipFlowTheme::COBALT_ACCENT, Color32::from_rgb(47, 111, 235));

    // 3. 全局状态根与工作流分页切换
    let mut state = AppState::default();
    assert_eq!(state.active_page, WorkflowPage::Agent);

    for page in WorkflowPage::ALL {
        state.active_page = page;
        assert_eq!(state.active_page, page);
        assert!(!page.as_str().is_empty());
        assert!(!page.label_zh().is_empty());
    }

    // 4. 门控调度器
    let mut scheduler = RepaintScheduler::new();
    scheduler.advance_time_for_test(Duration::from_millis(160));
    assert_eq!(scheduler.update_gating(false), RepaintState::Dormant);
    scheduler.on_user_interaction();
    assert_eq!(scheduler.state(), RepaintState::Interacting);

    println!(">>> M0 Common 与 UI 集成验证成功！");
}

#[tokio::test]
async fn test_m0_ipc_and_python_worker_end_to_end() {
    println!(">>> 验证 M0 IPC 命名管道与真实 Python Worker 联调及防逃逸...");

    // 1. 创建 Win32 JobGuard
    let job = JobGuard::new().expect("创建 JobGuard 失败");

    // 2. 创建命名管道服务端
    let pipe_name = format!(r"\\.\pipe\clipflow-m0-int-{}", Uuid::new_v4());
    let server = NamedPipeServerWrapper::bind(&pipe_name).expect("绑定命名管道失败");

    // 3. 解析 Python 真实运行时并拉起子进程
    let root_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();

    let (python_bin, env_vars) = resolve_python_runtime(root_dir);
    println!("定位到 Python 运行时: {:?}", python_bin);

    let mut cmd = Command::new(&python_bin);
    cmd.current_dir(root_dir);
    for (k, v) in env_vars {
        cmd.env(k, v);
    }
    cmd.args([
        "-m",
        "clipflow_worker.main",
        "--pipe",
        &pipe_name,
    ]);
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::null());

    let mut child = job
        .spawn_child_in_job(&mut cmd)
        .expect("启动 Python Worker 并绑定 Job 失败");
    let worker_pid = child.id();
    assert!(worker_pid > 0, "Python Worker PID 必须合法");

    // 4. 挂接 AsyncStderrDrainer 消费 Python Worker stderr 日志
    let stderr = child.stderr.take().expect("获取 child stderr 失败");
    let drainer = AsyncStderrDrainer::spawn_std(stderr, "python_worker_m0".to_string());

    // 5. 等待 Python Worker 连接命名管道
    let mut session = tokio::time::timeout(Duration::from_secs(5), server.accept())
        .await
        .expect("等待 Python Worker 连接超时 (5s)")
        .expect("接受管道会话失败");

    // 6. 校验对端客户端 PID (必须与 Worker PID 精确相等)
    let client_pid = session.client_pid().expect("获取客户端 PID 失败");
    println!("管道已连接，客户端 PID: {} | 启动 Worker PID: {}", client_pid, worker_pid);
    assert_eq!(client_pid, worker_pid, "命名管道客户端 PID 必须与直属 Worker PID 一致");

    // 7. 发送 JSON-RPC ping 并等待 pong
    let req = JsonRpcRequest::new(
        Some("req-m0-smoke".to_string()),
        "ping".to_string(),
        serde_json::json!({ "timestamp": 123456 }),
    );
    session.send_request(&req).await.expect("发送 ping 请求失败");

    let resp = session.read_response().await.expect("读取 pong 响应失败");
    println!("收到 Python Worker 响应: {:?}", resp);
    assert_eq!(resp.id, Some("req-m0-smoke".to_string()));
    assert_eq!(resp.result, Some(serde_json::json!("pong")));

    // 8. 确保 stderr 日志被排空
    thread::sleep(Duration::from_millis(200));
    let drained_lines = drainer.lines_drained();
    println!("已排空子进程 stderr 日志行数: {}", drained_lines);
    assert!(drained_lines > 0, "应捕获到 Python 启动输出日志");

    // 9. 验证 JobGuard 内核级联强杀（零孤儿逃逸）
    drop(session);
    drop(job); // 显式释放 Job

    thread::sleep(Duration::from_millis(400));
    let exit_status = child.try_wait().expect("try_wait 失败");
    println!("JobGuard 释放后子进程退出状态: {:?}", exit_status);
    assert!(exit_status.is_some(), "Python Worker 在 JobGuard 释放后必须被内核强杀退出！");

    println!(">>> M0 IPC 全链路冒烟测试成功！零孤儿逃逸验证通过！");
}
