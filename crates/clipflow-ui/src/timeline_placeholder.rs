use crate::state::AppState;
use crate::theme::ClipFlowTheme;
use egui::{vec2, Pos2, Rect, Stroke, StrokeKind, Ui};

/// 渲染全局公用时间线占位底座（在 M1 全量时间线接入前保活并提供统一视觉骨架）
pub fn render_timeline_placeholder(ui: &mut Ui, state: &AppState) {
    let available_rect = ui.available_rect_before_wrap();

    egui::Frame::new()
        .fill(ClipFlowTheme::SURFACE)
        .stroke(Stroke::new(1.0, ClipFlowTheme::BORDER))
        .corner_radius(ClipFlowTheme::RADIUS_PANEL)
        .inner_margin(8.0)
        .show(ui, |ui| {
            let width = available_rect.width() - 16.0;

            // 1. 顶部时间标尺占位条
            let (ruler_rect, _) = ui.allocate_exact_size(vec2(width, 24.0), egui::Sense::hover());
            ui.painter().rect_filled(
                ruler_rect,
                ClipFlowTheme::RADIUS_CLIP,
                ClipFlowTheme::SURFACE_WARM,
            );
            ui.painter().text(
                ruler_rect.left_center() + vec2(12.0, 0.0),
                egui::Align2::LEFT_CENTER,
                format!("时间线标尺 (播放头: {} | 缩放: {:.1}x)", state.playhead, state.zoom_level),
                egui::FontId::monospace(11.0),
                ClipFlowTheme::TEXT_META,
            );

            ui.add_space(4.0);

            // 2. 下沉轨道槽区 (V1, A1, C1)
            let track_height = 36.0;
            for (idx, (name, bg, stroke)) in [
                ("V1 视频主轨", ClipFlowTheme::TRACK_VIDEO_BG, ClipFlowTheme::TRACK_VIDEO_STROKE),
                ("A1 音频主轨", ClipFlowTheme::TRACK_AUDIO_BG, ClipFlowTheme::TRACK_AUDIO_STROKE),
                ("C1 语音字幕", ClipFlowTheme::TRACK_SUBTITLE_BG, ClipFlowTheme::TRACK_SUBTITLE_STROKE),
            ]
            .iter()
            .enumerate()
            {
                let (track_rect, _) = ui.allocate_exact_size(vec2(width, track_height), egui::Sense::hover());
                // 轨道深色下沉槽
                ui.painter().rect_filled(track_rect, ClipFlowTheme::RADIUS_CLIP, *bg);
                ui.painter().rect_stroke(
                    track_rect,
                    ClipFlowTheme::RADIUS_CLIP,
                    Stroke::new(1.0, *stroke),
                    StrokeKind::Inside,
                );

                // 轨道头标牌
                let header_rect = Rect::from_min_size(track_rect.min, vec2(90.0, track_height));
                ui.painter().rect_filled(header_rect, ClipFlowTheme::RADIUS_CLIP, ClipFlowTheme::SURFACE_WARM);
                ui.painter().text(
                    header_rect.center(),
                    egui::Align2::CENTER_CENTER,
                    *name,
                    egui::FontId::proportional(12.0),
                    ClipFlowTheme::TEXT_SECONDARY,
                );

                // 占位切片块演示
                if idx == 0 {
                    let clip_rect = Rect::from_min_max(
                        Pos2::new(track_rect.min.x + 120.0, track_rect.min.y + 4.0),
                        Pos2::new(track_rect.min.x + 360.0, track_rect.max.y - 4.0),
                    );
                    ui.painter().rect_filled(clip_rect, ClipFlowTheme::RADIUS_CLIP, ClipFlowTheme::TRACK_VIDEO_BG);
                    ui.painter().rect_stroke(
                        clip_rect,
                        ClipFlowTheme::RADIUS_CLIP,
                        Stroke::new(1.5, ClipFlowTheme::COBALT_ACCENT),
                        StrokeKind::Inside,
                    );
                    ui.painter().text(
                        clip_rect.center(),
                        egui::Align2::CENTER_CENTER,
                        "talking_head_01.mp4",
                        egui::FontId::proportional(12.0),
                        ClipFlowTheme::TEXT_PRIMARY,
                    );
                }

                ui.add_space(4.0);
            }
        });
}
