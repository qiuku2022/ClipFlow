use clipflow_ipc::server::IpcServer;
use clipflow_ipc::job::JobGuard;
use std::process::Stdio;
use tokio::io::{AsyncWriteExt, AsyncReadExt};
use tokio::process::Command;

#[tokio::test]
async fn test_python_rust_ipc_e2e() {
    let pipe_name = format!("clipflow-e2e-{}", uuid::Uuid::new_v4());
    
    // Create server
    let mut server = IpcServer::new(&pipe_name).expect("Failed to create IPC server");
    
    // Spawn python worker
    let mut child = Command::new("uv")
        .arg("run")
        .arg("python")
        .arg("../../python/tests/e2e_worker.py")
        .arg(&pipe_name)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Failed to spawn python worker");
        
    let pid = child.id().expect("Failed to get child pid");
    
    let job = JobGuard::new().expect("Failed to create job guard");
    job.assign_process(pid).expect("Failed to assign process to job");
    
    // Wait for connection
    server.connect().await.expect("Failed to accept client connection");
    
    // Send request
    let req = r#"{"jsonrpc": "2.0", "method": "echo", "params": {"hello": "world"}, "id": 1}"#;
    server.write_all(req.as_bytes()).await.expect("Failed to write to pipe");
    server.write_all(b"\n").await.expect("Failed to write newline");
    
    // Read response
    let mut buf = vec![0u8; 1024];
    let n = server.read(&mut buf).await.expect("Failed to read from pipe");
    let resp = std::str::from_utf8(&buf[..n]).expect("Invalid utf8");
    
    assert!(resp.contains(r#"{"hello": "world"}"#));
    assert!(resp.contains(r#""id": 1"#));
    
    // Drop job guard to kill child process
    drop(job);
    
    // Let OS clean it up
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    
    // Wait for child to exit
    let status = child.try_wait().expect("try_wait failed");
    assert!(status.is_some(), "Process should be killed by JobGuard");
}

