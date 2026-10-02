import argparse
import sys
from clipflow_worker.ipc.pipe_client import PipeClient
from clipflow_worker.ipc.protocol import JsonRpcRequest, JsonRpcResponse


def handle_request(req: JsonRpcRequest) -> JsonRpcResponse:
    if req.method == "ping":
        return JsonRpcResponse.success(req.id, "pong")
    return JsonRpcResponse.failure(req.id, -32601, f"未支持的方法: {req.method}")


def main():
    parser = argparse.ArgumentParser(description="ClipFlow Python 计算工作子进程")
    parser.add_argument("--pipe", type=str, help="命名管道路径", required=False)
    args = parser.parse_args()

    # 向 stderr 输出启动日志（供 Rust 端 AsyncStderrDrainer 测试）
    sys.stderr.write("ClipFlow Python Worker 正在初始化...\n")
    sys.stderr.flush()

    if not args.pipe:
        print("未指定 --pipe 参数，Worker 正常退出。")
        return

    sys.stderr.write(f"连接命名管道: {args.pipe}\n")
    sys.stderr.flush()

    client = PipeClient(args.pipe)
    client.connect()
    try:
        client.run_loop(handle_request)
    finally:
        client.close()


if __name__ == "__main__":
    main()
