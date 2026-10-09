use clipflow_ui::theme::ClipFlowTheme;
use egui::{Color32, CornerRadius};

#[test]
fn test_clipflow_theme_tokens() {
    // 验证底层画布颜色 #0F1115
    assert_eq!(ClipFlowTheme::BG_CANVAS, Color32::from_rgb(15, 17, 21));

    // 验证主表面卡片颜色 #171A21
    assert_eq!(ClipFlowTheme::SURFACE, Color32::from_rgb(23, 26, 33));

    // 验证主交互色 Cobalt Blue #2F6FEB
    assert_eq!(ClipFlowTheme::COBALT_ACCENT, Color32::from_rgb(47, 111, 235));

    // 验证工业级圆角
    assert_eq!(ClipFlowTheme::RADIUS_PANEL, CornerRadius::same(12));
    assert_eq!(ClipFlowTheme::RADIUS_CONTROL, CornerRadius::same(8));
    assert_eq!(ClipFlowTheme::RADIUS_CLIP, CornerRadius::same(4));
}

