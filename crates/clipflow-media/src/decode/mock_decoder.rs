use crate::decode::provider::{ColorRange, ColorStandard, DecodedFrame, VideoTextureProvider};
use clipflow_common::{ClipFlowError, RationalTime};

pub struct MockVideoDecoder {
    width: u32,
    height: u32,
    fps: u32,
    current_frame: i64,
}

impl MockVideoDecoder {
    pub fn new(width: u32, height: u32, fps: u32) -> Self {
        Self {
            width,
            height,
            fps,
            current_frame: 0,
        }
    }
}

impl VideoTextureProvider for MockVideoDecoder {
    fn width(&self) -> u32 {
        self.width
    }

    fn height(&self) -> u32 {
        self.height
    }

    fn poll_next_frame(&mut self) -> Option<DecodedFrame> {
        let pts = RationalTime::new(self.current_frame, self.fps);
        self.current_frame += 1;

        let y_len = (self.width * self.height) as usize;
        let uv_len = y_len / 2;

        // 生成测试 Y 平面 (灰阶渐变) 与 UV 平面 (中性灰色 UV=128)
        let y_val = ((self.current_frame * 5) % 256) as u8;
        let y_plane = vec![y_val; y_len];
        let uv_plane = vec![128u8; uv_len];

        Some(DecodedFrame {
            pts,
            width: self.width,
            height: self.height,
            y_plane,
            uv_plane,
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
