use clipflow_common::RationalTime;
use clipflow_ui::state::{AppState, RepaintScheduler, RepaintState};
use clipflow_ui::theme::ClipFlowTheme;
use egui::Color32;
use std::time::Duration;

#[test]
fn test_theme_tokens_match_specification() {
    // 验证 Neutral Modern 深色规范核心色阶
    assert_eq!(ClipFlowTheme::BG_CANVAS, Color32::from_rgb(15, 17, 21)); // #0F1115
    assert_eq!(ClipFlowTheme::SURFACE, Color32::from_rgb(23, 26, 33)); // #171A21
    assert_eq!(ClipFlowTheme::SURFACE_WARM, Color32::from_rgb(30, 34, 43)); // #1E222B
    assert_eq!(ClipFlowTheme::COBALT_ACCENT, Color32::from_rgb(47, 111, 235)); // #2F6FEB

    // 验证圆角几何梯队 (12px, 8px, 4px)
    assert_eq!(ClipFlowTheme::RADIUS_PANEL.nw, 12);
    assert_eq!(ClipFlowTheme::RADIUS_CONTROL.nw, 8);
    assert_eq!(ClipFlowTheme::RADIUS_CLIP.nw, 4);
}

#[test]
fn test_app_state_defaults() {
    let state = AppState::default();
    assert_eq!(state.playhead, RationalTime::ZERO);
    assert_eq!(state.zoom_level, 1.0);
    assert!(!state.is_playing);
}

#[test]
fn test_repaint_scheduler_dormant_transition() {
    let mut scheduler = RepaintScheduler::new();
    assert_eq!(scheduler.state(), RepaintState::Interacting);

    // 模拟经过 160ms 无交互
    scheduler.advance_time_for_test(Duration::from_millis(160));
    assert_eq!(scheduler.update_gating(false), RepaintState::Dormant);

    // 模拟用户交互输入
    scheduler.on_user_interaction();
    assert_eq!(scheduler.state(), RepaintState::Interacting);

    // 播放状态下强制维持 Playing
    assert_eq!(scheduler.update_gating(true), RepaintState::Playing);
}
