from clipflow_worker.ipc.protocol import JsonRpcRequest, JsonRpcResponse
from clipflow_worker.main import handle_request


def test_json_rpc_ping_pong():
    req = JsonRpcRequest(id="req-1", method="ping", params={})
    resp = handle_request(req)
    assert resp.id == "req-1"
    assert resp.result == "pong"
    assert resp.error is None


def test_json_rpc_unknown_method():
    req = JsonRpcRequest(id="req-2", method="unknown.func", params={})
    resp = handle_request(req)
    assert resp.id == "req-2"
    assert resp.result is None
    assert resp.error is not None
    assert resp.error.code == -32601
