use tokio::net::windows::named_pipe::{NamedPipeServer, ServerOptions};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use std::pin::Pin;
use std::task::{Context, Poll};

pub struct IpcServer {
    pipe_name: String,
    server: NamedPipeServer,
}

impl IpcServer {
    pub fn new(name: &str) -> std::io::Result<Self> {
        let pipe_name = format!(r#"\\.\pipe\{}"#, name);
        let server = ServerOptions::new()
            .first_pipe_instance(true)
            .create(&pipe_name)?;
            
        Ok(Self { pipe_name, server })
    }

    pub fn pipe_name(&self) -> &str {
        &self.pipe_name
    }

    pub async fn connect(&self) -> std::io::Result<()> {
        self.server.connect().await
    }
}

// Forward AsyncRead and AsyncWrite to the inner NamedPipeServer
impl AsyncRead for IpcServer {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().server).poll_read(cx, buf)
    }
}

impl AsyncWrite for IpcServer {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<std::io::Result<usize>> {
        Pin::new(&mut self.get_mut().server).poll_write(cx, buf)
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().server).poll_flush(cx)
    }

    fn poll_shutdown(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<std::io::Result<()>> {
        Pin::new(&mut self.get_mut().server).poll_shutdown(cx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::windows::named_pipe::ClientOptions;
    use tokio::io::{AsyncWriteExt, AsyncReadExt};

    #[tokio::test]
    async fn test_ipc_server_connection() {
        let pipe_name = format!("clipflow-test-{}", uuid::Uuid::new_v4());
        let mut server = IpcServer::new(&pipe_name).unwrap();
        
        let full_pipe_name = server.pipe_name().to_string();
        
        let client_task = tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            let mut client = ClientOptions::new().open(&full_pipe_name).unwrap();
            client.write_all(b"Hello from client").await.unwrap();
            
            let mut buf = [0u8; 128];
            let n = client.read(&mut buf).await.unwrap();
            assert_eq!(&buf[..n], b"Hello from server");
        });
        
        server.connect().await.expect("Failed to connect client");
        
        let mut buf = [0u8; 128];
        let n = server.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"Hello from client");
        
        server.write_all(b"Hello from server").await.unwrap();
        
        client_task.await.unwrap();
    }
}

