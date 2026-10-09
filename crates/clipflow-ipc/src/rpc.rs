use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcRequest {
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<RpcError>,
    pub id: Value,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl RpcResponse {
    pub fn new_result(id: Value, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: Some(result),
            error: None,
            id,
        }
    }

    pub fn new_error(id: Value, code: i32, message: impl Into<String>, data: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(RpcError {
                code,
                message: message.into(),
                data,
            }),
            id,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_rpc_request_decode() {
        let json_str = r#"{"jsonrpc": "2.0", "method": "subtract", "params": [42, 23], "id": 1}"#;
        let req: RpcRequest = serde_json::from_str(json_str).expect("Failed to decode");
        
        assert_eq!(req.jsonrpc, "2.0");
        assert_eq!(req.method, "subtract");
        assert_eq!(req.params.unwrap()[0], json!(42));
        assert_eq!(req.id.unwrap().as_i64(), Some(1));
    }

    #[test]
    fn test_rpc_request_notification() {
        let json_str = r#"{"jsonrpc": "2.0", "method": "update", "params": [1,2,3]}"#;
        let req: RpcRequest = serde_json::from_str(json_str).expect("Failed to decode");
        
        assert_eq!(req.id, None);
        assert_eq!(req.method, "update");
    }

    #[test]
    fn test_rpc_response_success() {
        let res = RpcResponse::new_result(json!(1), json!(19));
        let json_str = serde_json::to_string(&res).unwrap();
        assert_eq!(json_str, r#"{"jsonrpc":"2.0","result":19,"id":1}"#);
    }

    #[test]
    fn test_rpc_response_error() {
        let res = RpcResponse::new_error(json!(1), -32600, "Invalid Request", None);
        let json_str = serde_json::to_string(&res).unwrap();
        assert_eq!(json_str, r#"{"jsonrpc":"2.0","error":{"code":-32600,"message":"Invalid Request"},"id":1}"#);
    }
}

