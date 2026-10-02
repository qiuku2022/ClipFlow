from typing import Any, Optional
from pydantic import BaseModel, Field


class JsonRpcRequest(BaseModel):
    jsonrpc: str = "2.0"
    id: Optional[str] = None
    method: str
    params: dict[str, Any] = Field(default_factory=dict)


class JsonRpcError(BaseModel):
    code: int
    message: str
    data: Optional[Any] = None


class JsonRpcResponse(BaseModel):
    jsonrpc: str = "2.0"
    id: Optional[str] = None
    result: Optional[Any] = None
    error: Optional[JsonRpcError] = None

    @classmethod
    def success(cls, req_id: Optional[str], result: Any) -> "JsonRpcResponse":
        return cls(jsonrpc="2.0", id=req_id, result=result, error=None)

    @classmethod
    def failure(cls, req_id: Optional[str], code: int, message: str) -> "JsonRpcResponse":
        return cls(
            jsonrpc="2.0",
            id=req_id,
            result=None,
            error=JsonRpcError(code=code, message=message),
        )
