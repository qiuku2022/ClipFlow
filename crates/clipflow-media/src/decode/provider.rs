use clipflow_common::{ClipFlowError, RationalTime};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorStandard {
    Bt709,
    Bt601,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorRange {
    Limited,
    Full,
}

pub struct DecodedFrame {
    pub pts: RationalTime,
    pub width: u32,
    pub height: u32,
    pub y_plane: Vec<u8>,
    pub uv_plane: Vec<u8>,
    pub color_standard: ColorStandard,
    pub color_range: ColorRange,
}

pub trait VideoTextureProvider: Send {
    fn width(&self) -> u32;
    fn height(&self) -> u32;
    fn poll_next_frame(&mut self) -> Option<DecodedFrame>;
    fn seek(&mut self, target: RationalTime) -> Result<(), ClipFlowError>;
}
