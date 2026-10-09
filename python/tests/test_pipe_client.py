import json

from clipflow_worker.pipe_client import PipeClient


def test_pipe_client_encode_request():
    client = PipeClient("test-pipe")
    req_bytes = client.encode_request("test_method", {"arg": 1}, msg_id=42)
    
    # Should be newline-delimited JSON
    assert req_bytes.endswith(b"\n")
    
    parsed = json.loads(req_bytes.decode("utf-8").strip())
    assert parsed["jsonrpc"] == "2.0"
    assert parsed["method"] == "test_method"
    assert parsed["params"] == {"arg": 1}
    assert parsed["id"] == 42

def test_pipe_client_decode_response():
    client = PipeClient("test-pipe")
    
    resp_data = {"jsonrpc": "2.0", "result": "ok", "id": 42}
    resp_bytes = json.dumps(resp_data).encode("utf-8") + b"\n"
    
    parsed = client.decode_response(resp_bytes)
    assert parsed.get("result") == "ok"
    assert parsed.get("id") == 42
