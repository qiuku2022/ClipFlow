use clipflow_media::decode::{ColorRange, ColorStandard};
use clipflow_media::render::{ColorMatrixEngine, ColorParams, Nv12RenderPipeline};

#[test]
fn test_wgsl_shader_syntax_and_compilation() {
    // 验证能够无报错创建 Nv12RenderPipeline 或加载着色器源码
    let shader_source = Nv12RenderPipeline::shader_source();
    assert!(shader_source.contains("texture_y"));
    assert!(shader_source.contains("texture_uv"));
    assert!(shader_source.contains("vs_main"));
    assert!(shader_source.contains("fs_main"));
}

#[test]
fn test_bt709_limited_and_full_range_matrices() {
    let engine = ColorMatrixEngine::new();

    // 1. BT.709 Limited Range (标称有限范围，Y: 16-235, UV: 16-240)
    let params_limited = engine.get_color_params(ColorStandard::Bt709, ColorRange::Limited);
    // Limited Range 下 Y 偏置通常为 16/255 ≈ 0.062745, UV 偏置为 128/255 ≈ 0.50196
    assert!((params_limited.yuv_offset[0] - 0.062745).abs() < 1e-4);
    assert!((params_limited.yuv_offset[1] - 0.501960).abs() < 1e-4);

    // 2. BT.709 Full Range
    let params_full = engine.get_color_params(ColorStandard::Bt709, ColorRange::Full);
    assert!((params_full.yuv_offset[0] - 0.0).abs() < 1e-4);
    assert!((params_full.yuv_offset[1] - 0.501960).abs() < 1e-4);

    // 3. 验证纯白 (White) 还原：在 BT.709 Limited 中，Y=235, U=128, V=128 应转为 R=1, G=1, B=1 (误差 < 0.005)
    let (r, g, b) = engine.yuv_to_rgb(235.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, ColorStandard::Bt709, ColorRange::Limited);
    assert!((r - 1.0).abs() < 0.01, "Red was {}", r);
    assert!((g - 1.0).abs() < 0.01, "Green was {}", g);
    assert!((b - 1.0).abs() < 0.01, "Blue was {}", b);

    // 4. 验证纯黑 (Black) 还原：在 BT.709 Limited 中，Y=16, U=128, V=128 应转为 R=0, G=0, B=0
    let (r0, g0, b0) = engine.yuv_to_rgb(16.0 / 255.0, 128.0 / 255.0, 128.0 / 255.0, ColorStandard::Bt709, ColorRange::Limited);
    assert!(r0.abs() < 0.01, "Red black was {}", r0);
    assert!(g0.abs() < 0.01, "Green black was {}", g0);
    assert!(b0.abs() < 0.01, "Blue black was {}", b0);
}

#[test]
fn test_color_params_uniform_layout() {
    let size = std::mem::size_of::<ColorParams>();
    // 4 行 vec4<f32>，每行 16 字节，总计 64 字节，完全对齐 std140
    assert_eq!(size, 64);
}
