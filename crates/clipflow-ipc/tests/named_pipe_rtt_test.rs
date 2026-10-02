use clipflow_ipc::named_pipe::{NamedPipeServerWrapper, PipeClientWrapper};
use clipflow_ipc::protocol::{JsonRpcRequest, JsonRpcResponse};
use std::time::Instant;

#[tokio::test]
async fn test_named_pipe_ping_pong_rtt() {
    let pipe_name = format!(r"\\.\pipe\clipflow-test-rtt-{}", uuid::Uuid::new_v4());

    // 1. 创建并启动服务端监听
    let server = NamedPipeServerWrapper::bind(&pipe_name).expect("创建服务端管道失败");

    // 2. 异步连接客户端
    let client_task = tokio::spawn({
        let pipe_name = pipe_name.clone();
        async move {
            let mut client = PipeClientWrapper::connect(&pipe_name)
                .await
                .expect("客户端连接管道失败");

            for i in 0..100 {
                let req = JsonRpcRequest::new(
                    Some(i.to_string()),
                    "ping".to_string(),
                    serde_json::json!({ "seq": i }),
                );
                client.send_request(&req).await.expect("发送请求失败");

                let resp: JsonRpcResponse = client.read_response().await.expect("读取响应失败");
                assert_eq!(resp.id, Some(i.to_string()));
                assert_eq!(resp.result, Some(serde_json::json!("pong")));
            }
        }
    });

    // 3. 服务端等待连接并处理 100 次 ping
    let mut session = server.accept().await.expect("服务端接受连接失败");
    let client_pid = session.client_pid().expect("获取客户端 PID 失败");
    assert!(client_pid > 0, "客户端 PID 应大于 0");

    let start = Instant::now();
    for _ in 0..100 {
        let req: JsonRpcRequest = session.read_request().await.expect("服务端读取请求失败");
        assert_eq!(req.method, "ping");

        let resp = JsonRpcResponse::success(req.id, serde_json::json!("pong"));
        session.send_response(&resp).await.expect("服务端发送响应失败");
    }

    client_task.await.expect("客户端任务异常");
    let total_elapsed = start.elapsed();
    let avg_rtt_ms = total_elapsed.as_secs_f64() * 1000.0 / 100.0;

    println!("命名管道 100 次往返完成，平均 RTT: {:.4} ms", avg_rtt_ms);
    // 规范要求: RTT <= 0.35ms (在本地单机一般为 0.05ms ~ 0.20ms)
    assert!(avg_rtt_ms <= 0.35, "往返 RTT 超出阈值: {:.4} ms", avg_rtt_ms);
}
