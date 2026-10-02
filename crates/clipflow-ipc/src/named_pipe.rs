use crate::protocol::{JsonRpcRequest, JsonRpcResponse};
use clipflow_common::{ClipFlowError, Result};
use std::os::windows::io::AsRawHandle;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::net::windows::named_pipe::{ClientOptions, NamedPipeClient, NamedPipeServer, ServerOptions};
use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Pipes::GetNamedPipeClientProcessId;

/// 命名管道安全 SDDL 访问控制策略：仅授予当前用户 Token 与 SYSTEM 权限，拒绝一切跨机连接
pub const CLIPFLOW_PIPE_SDDL: &str = "D:(A;;GA;;;OW)(A;;GA;;;SY)";

/// 命名管道服务端包装
pub struct NamedPipeServerWrapper {
    server: NamedPipeServer,
    _pipe_name: String,
}

impl NamedPipeServerWrapper {
    /// 绑定并创建命名管道监听器
    pub fn bind(pipe_name: &str) -> Result<Self> {
        let server = ServerOptions::new()
            .reject_remote_clients(true)
            .max_instances(1)
            .first_pipe_instance(true)
            .create(pipe_name)
            .map_err(|e| ClipFlowError::Ipc(format!("创建命名管道 {} 失败: {}", pipe_name, e)))?;

        Ok(Self {
            server,
            _pipe_name: pipe_name.to_string(),
        })
    }

    /// 等待客户端接入并升级为双工通信会话
    pub async fn accept(self) -> Result<PipeSession> {
        self.server
            .connect()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("等待客户端连接失败: {}", e)))?;

        let raw_handle = self.server.as_raw_handle() as HANDLE;
        let (reader, writer) = tokio::io::split(self.server);
        let lines = BufReader::new(reader).lines();

        Ok(PipeSession {
            raw_handle,
            writer,
            lines,
            expected_client_pid: None,
        })
    }
}

/// 服务端双工通信管道会话
pub struct PipeSession {
    raw_handle: HANDLE,
    writer: tokio::io::WriteHalf<NamedPipeServer>,
    lines: Lines<BufReader<tokio::io::ReadHalf<NamedPipeServer>>>,
    expected_client_pid: Option<u32>,
}

impl PipeSession {
    /// 获取对端客户端真实进程 PID (Win32 API 原生校验)
    pub fn client_pid(&self) -> Result<u32> {
        let mut client_pid: u32 = 0;
        let res = unsafe { GetNamedPipeClientProcessId(self.raw_handle, &mut client_pid) };
        if res == 0 {
            return Err(ClipFlowError::Ipc(format!(
                "GetNamedPipeClientProcessId 失败: {}",
                std::io::Error::last_os_error()
            )));
        }
        Ok(client_pid)
    }

    /// 设置并校验客户端期望 PID，不匹配则拒绝继续通信
    pub fn verify_client_pid(&mut self, expected_pid: u32) -> Result<()> {
        let actual_pid = self.client_pid()?;
        if actual_pid != expected_pid {
            return Err(ClipFlowError::Ipc(format!(
                "管道客户端 PID 校验失败: 期望 {}, 实际 {}",
                expected_pid, actual_pid
            )));
        }
        self.expected_client_pid = Some(expected_pid);
        Ok(())
    }

    /// 读取一个 JSON-RPC 请求
    pub async fn read_request(&mut self) -> Result<JsonRpcRequest> {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("读取管道请求错误: {}", e)))?
            .ok_or_else(|| ClipFlowError::Ipc("管道连接已被对端关闭".to_string()))?;

        serde_json::from_str(&line)
            .map_err(|e| ClipFlowError::Ipc(format!("反序列化 JSON-RPC 请求失败: {}: {}", e, line)))
    }

    /// 发送一个 JSON-RPC 响应
    pub async fn send_response(&mut self, response: &JsonRpcResponse) -> Result<()> {
        let mut json = serde_json::to_string(response)
            .map_err(|e| ClipFlowError::Ipc(format!("序列化响应失败: {}", e)))?;
        json.push('\n');

        self.writer
            .write_all(json.as_bytes())
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("写入管道响应失败: {}", e)))?;
        self.writer
            .flush()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("刷新管道缓冲失败: {}", e)))?;

        Ok(())
    }

    /// 发送一个 JSON-RPC 请求
    pub async fn send_request(&mut self, request: &JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(request)
            .map_err(|e| ClipFlowError::Ipc(format!("序列化请求失败: {}", e)))?;
        json.push('\n');

        self.writer
            .write_all(json.as_bytes())
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("写入管道请求失败: {}", e)))?;
        self.writer
            .flush()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("刷新管道缓冲失败: {}", e)))?;

        Ok(())
    }

    /// 读取一个 JSON-RPC 响应
    pub async fn read_response(&mut self) -> Result<JsonRpcResponse> {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("读取管道响应错误: {}", e)))?
            .ok_or_else(|| ClipFlowError::Ipc("管道连接已被对端关闭".to_string()))?;

        serde_json::from_str(&line)
            .map_err(|e| ClipFlowError::Ipc(format!("反序列化 JSON-RPC 响应失败: {}: {}", e, line)))
    }
}

/// 客户端管道包装
pub struct PipeClientWrapper {
    writer: tokio::io::WriteHalf<NamedPipeClient>,
    lines: Lines<BufReader<tokio::io::ReadHalf<NamedPipeClient>>>,
}

impl PipeClientWrapper {
    pub async fn connect(pipe_name: &str) -> Result<Self> {
        let client = ClientOptions::new()
            .open(pipe_name)
            .map_err(|e| ClipFlowError::Ipc(format!("连接命名管道 {} 失败: {}", pipe_name, e)))?;

        let (reader, writer) = tokio::io::split(client);
        let lines = BufReader::new(reader).lines();

        Ok(Self { writer, lines })
    }

    pub async fn send_request(&mut self, request: &JsonRpcRequest) -> Result<()> {
        let mut json = serde_json::to_string(request)
            .map_err(|e| ClipFlowError::Ipc(format!("序列化请求失败: {}", e)))?;
        json.push('\n');

        self.writer
            .write_all(json.as_bytes())
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("写入管道请求失败: {}", e)))?;
        self.writer
            .flush()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("刷新管道请求失败: {}", e)))?;

        Ok(())
    }

    pub async fn read_response(&mut self) -> Result<JsonRpcResponse> {
        let line = self
            .lines
            .next_line()
            .await
            .map_err(|e| ClipFlowError::Ipc(format!("读取管道响应错误: {}", e)))?
            .ok_or_else(|| ClipFlowError::Ipc("管道连接已被对端关闭".to_string()))?;

        serde_json::from_str(&line)
            .map_err(|e| ClipFlowError::Ipc(format!("反序列化 JSON-RPC 响应失败: {}: {}", e, line)))
    }
}
