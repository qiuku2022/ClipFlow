import json
import time
from typing import Any


class PipeClient:
    def __init__(self, pipe_name: str):
        self.pipe_name = pipe_name
        self.pipe_path = rf"\\.\pipe\{pipe_name}"
        self.f = None

    def connect(self, timeout: float = 5.0):
        start = time.time()
        while time.time() - start < timeout:
            try:
                self.f = open(self.pipe_path, "r+b")  # noqa: SIM115
                return
            except FileNotFoundError:
                time.sleep(0.1)
        raise TimeoutError(f"Failed to connect to {self.pipe_path}")

    def send_request(self, method: str, params: Any | None = None, msg_id: Any | None = None):
        if not self.f:
            raise RuntimeError("Pipe not connected")
        req = self.encode_request(method, params, msg_id)
        self.f.write(req)
        self.f.flush()

    def read_response(self) -> dict[str, Any] | None:
        if not self.f:
            raise RuntimeError("Pipe not connected")
        line = self.f.readline()
        if not line:
            return None
        return self.decode_response(line)

    def encode_request(self, method: str, params: Any | None = None, msg_id: Any | None = None) -> bytes:
        req: dict[str, Any] = {"jsonrpc": "2.0", "method": method}
        if params is not None:
            req["params"] = params
        if msg_id is not None:
            req["id"] = msg_id
        return json.dumps(req).encode("utf-8") + b"\n"

    def decode_response(self, data: bytes) -> dict[str, Any]:
        return json.loads(data.decode("utf-8").strip())
