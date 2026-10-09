use std::collections::VecDeque;
use std::sync::Arc;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::sync::Mutex;

const MAX_CAPACITY: usize = 4096;

#[derive(Clone)]
pub struct AsyncStderrDrainer {
    buffer: Arc<Mutex<VecDeque<u8>>>,
}

impl AsyncStderrDrainer {
    pub fn new<R: AsyncRead + Unpin + Send + 'static>(mut reader: R) -> Self {
        let buffer = Arc::new(Mutex::new(VecDeque::with_capacity(MAX_CAPACITY)));
        let buffer_clone = Arc::clone(&buffer);
        
        tokio::spawn(async move {
            let mut chunk = [0u8; 1024];
            loop {
                match reader.read(&mut chunk).await {
                    Ok(0) => break, // EOF
                    Ok(n) => {
                        let mut buf = buffer_clone.lock().await;
                        for &b in &chunk[..n] {
                            if buf.len() == MAX_CAPACITY {
                                buf.pop_front();
                            }
                            buf.push_back(b);
                        }
                    }
                    Err(_) => break, // Drop quietly on error (e.g. process died)
                }
            }
        });

        Self { buffer }
    }

    pub async fn get_contents(&self) -> String {
        let buf = self.buffer.lock().await;
        let bytes: Vec<u8> = buf.iter().copied().collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_drainer_captures_up_to_4kb() {
        let (mut tx, rx) = tokio::io::duplex(8192);
        
        let drainer = AsyncStderrDrainer::new(rx);
        
        // Write 5KB of data
        let chunk1 = vec![b'A'; 3000];
        let chunk2 = vec![b'B'; 2000]; // total 5000 bytes
        
        tx.write_all(&chunk1).await.unwrap();
        tx.write_all(&chunk2).await.unwrap();
        drop(tx); // close stream
        
        // Let drainer finish
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        
        let content = drainer.get_contents().await;
        
        // Should keep the LAST 4096 bytes.
        // That means 4096 bytes total: (5000 - 4096) = 904 bytes of 'A', and 2000 bytes of 'B'
        assert_eq!(content.len(), 4096);
        assert!(content.ends_with(&String::from_utf8(vec![b'B'; 2000]).unwrap()));
        assert!(content.starts_with(&String::from_utf8(vec![b'A'; 904]).unwrap()));
    }
}

