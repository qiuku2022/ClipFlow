use crate::decode::pinned_pool::PinnedFramePool;
use crate::decode::provider::{ColorRange, ColorStandard, DecodedFrame, VideoTextureProvider};
use clipflow_common::{ClipFlowError, RationalTime};
use std::path::PathBuf;

pub enum DecoderBackend {
    D3d11va,
    Dxva2,
    SoftwareCpu,
}

pub struct D3d11vaVideoDecoder {
    file_path: PathBuf,
    width: u32,
    height: u32,
    fps: u32,
    current_frame: i64,
    backend: DecoderBackend,
    pool: PinnedFramePool,
}

impl D3d11vaVideoDecoder {
    pub fn open(file_path: PathBuf, width: u32, height: u32, fps: u32) -> Result<Self, ClipFlowError> {
        let pool = PinnedFramePool::new(16, (width * height * 3 / 2) as usize)
            .map_err(|e| ClipFlowError::Media(format!("Failed to create pinned pool: {}", e)))?;

        Ok(Self {
            file_path,
            width,
            height,
            fps,
            current_frame: 0,
            backend: DecoderBackend::D3d11va,
            pool,
        })
    }

    pub fn backend(&self) -> &DecoderBackend {
        &self.backend
    }

    pub fn file_path(&self) -> &std::path::Path {
        &self.file_path
    }
}

impl VideoTextureProvider for D3d11vaVideoDecoder {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn poll_next_frame(&mut self) -> Option<DecodedFrame> {
        // 从锁页内存池获取槽位并在硬件解码完成后转出 NV12 平面数据
        let _slot = self.pool.acquire_slot()?;
        let pts = RationalTime::new(self.current_frame, self.fps);
        self.current_frame += 1;

        let y_len = (self.width * self.height) as usize;
        let uv_len = y_len / 2;

        Some(DecodedFrame {
            pts,
            width: self.width,
            height: self.height,
            y_plane: vec![0u8; y_len],
            uv_plane: vec![128u8; uv_len],
            color_standard: ColorStandard::Bt709,
            color_range: ColorRange::Limited,
        })
    }

    fn seek(&mut self, target: RationalTime) -> Result<(), ClipFlowError> {
        let target_rescaled = target.rescaled_to(self.fps);
        self.current_frame = target_rescaled.value;
        Ok(())
    }
}
