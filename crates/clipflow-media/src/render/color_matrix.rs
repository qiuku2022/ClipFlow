use crate::decode::provider::{ColorRange, ColorStandard};

#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColorParams {
    pub matrix_row0: [f32; 4],
    pub matrix_row1: [f32; 4],
    pub matrix_row2: [f32; 4],
    pub yuv_offset: [f32; 4],
}

// 确保符合 std140 16 字节对齐
const _: () = assert!(std::mem::size_of::<ColorParams>() == 64);

#[derive(Default)]
pub struct ColorMatrixEngine;

impl ColorMatrixEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn get_color_params(&self, standard: ColorStandard, range: ColorRange) -> ColorParams {
        match (standard, range) {
            (ColorStandard::Bt709, ColorRange::Limited) => ColorParams {
                matrix_row0: [1.164383, 0.0, 1.792741, 0.0],
                matrix_row1: [1.164383, -0.213249, -0.532909, 0.0],
                matrix_row2: [1.164383, 2.112402, 0.0, 0.0],
                yuv_offset: [16.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 0.0],
            },
            (ColorStandard::Bt709, ColorRange::Full) => ColorParams {
                matrix_row0: [1.0, 0.0, 1.574800, 0.0],
                matrix_row1: [1.0, -0.187324, -0.468124, 0.0],
                matrix_row2: [1.0, 1.855600, 0.0, 0.0],
                yuv_offset: [0.0, 128.0 / 255.0, 128.0 / 255.0, 0.0],
            },
            (ColorStandard::Bt601, ColorRange::Limited) => ColorParams {
                matrix_row0: [1.164383, 0.0, 1.596027, 0.0],
                matrix_row1: [1.164383, -0.391762, -0.812968, 0.0],
                matrix_row2: [1.164383, 2.017232, 0.0, 0.0],
                yuv_offset: [16.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, 0.0],
            },
            (ColorStandard::Bt601, ColorRange::Full) => ColorParams {
                matrix_row0: [1.0, 0.0, 1.402000, 0.0],
                matrix_row1: [1.0, -0.344136, -0.714136, 0.0],
                matrix_row2: [1.0, 1.772000, 0.0, 0.0],
                yuv_offset: [0.0, 128.0 / 255.0, 128.0 / 255.0, 0.0],
            },
        }
    }

    /// 在 CPU 上执行精确色彩转换与误差测算
    pub fn yuv_to_rgb(
        &self,
        y: f32,
        u: f32,
        v: f32,
        standard: ColorStandard,
        range: ColorRange,
    ) -> (f32, f32, f32) {
        let params = self.get_color_params(standard, range);
        let dy = y - params.yuv_offset[0];
        let du = u - params.yuv_offset[1];
        let dv = v - params.yuv_offset[2];

        let r = params.matrix_row0[0] * dy + params.matrix_row0[1] * du + params.matrix_row0[2] * dv;
        let g = params.matrix_row1[0] * dy + params.matrix_row1[1] * du + params.matrix_row1[2] * dv;
        let b = params.matrix_row2[0] * dy + params.matrix_row2[1] * du + params.matrix_row2[2] * dv;

        (r.clamp(0.0, 1.0), g.clamp(0.0, 1.0), b.clamp(0.0, 1.0))
    }
}
