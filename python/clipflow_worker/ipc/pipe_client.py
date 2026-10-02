import json
import sys
from typing import Callable, Optional
from clipflow_worker.ipc.protocol import JsonRpcRequest, JsonRpcResponse


class PipeClient:
    """Windows 命名管道客户端驱动"""

    def __init__(self, pipe_path: str):
        self.pipe_path = pipe_path
        self._file = None

    def connect(self) -> None:
        """打开命名管道（Windows 原生支持）"""
        self._file = open(self.pipe_path, "r+b", buffering=0)

    def close(self) -> None:
        if self._file and not self._file.closed:
            self._file.close()

    def send_response(self, response: JsonRpcResponse) -> None:
        payload = response.model_dump_json() + "\n"
        self._file.write(payload.encode("utf-8"))
        self._file.flush()

    def run_loop(self, handler: Callable[[JsonRpcRequest], JsonRpcResponse]) -> None:
        """消费管道请求主循环"""
        while True:
            line_bytes = self._file.readline()
            if not line_bytes:
                # 管道断开连接
                break

            line_str = line_bytes.decode("utf-8").strip()
            if not line_str:
                continue

            try:
                data = json.loads(line_str)
                request = JsonRpcRequest.model_validate(data)
                response = handler(request)
            except Exception as e:
                response = JsonRpcResponse.failure(None, -32603, f"内部处理错误: {e}")

            self.send_response(response)
