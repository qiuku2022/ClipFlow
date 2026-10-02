use crate::theme::ClipFlowTheme;
use egui::{vec2, Align, Align2, Color32, FontId, Layout, Response, Sense, Stroke, Ui};

/// 达芬奇式 6 大分页工作流枚举
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WorkflowPage {
    #[default]
    Agent,
    Edit,
    Motion,
    Audio,
    Image,
    Deliver,
}

impl WorkflowPage {
    pub const ALL: [Self; 6] = [
        Self::Agent,
        Self::Edit,
        Self::Motion,
        Self::Audio,
        Self::Image,
        Self::Deliver,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Agent => "AGENT",
            Self::Edit => "EDIT",
            Self::Motion => "MOTION",
            Self::Audio => "AUDIO",
            Self::Image => "IMAGE",
            Self::Deliver => "DELIVER",
        }
    }

    pub fn label_zh(&self) -> &'static str {
        match self {
            Self::Agent => "导演",
            Self::Edit => "剪辑",
            Self::Motion => "动效",
            Self::Audio => "声音",
            Self::Image => "调色",
            Self::Deliver => "导出",
        }
    }
}

/// 达芬奇式底部固定 48px Dock 栏
pub struct WorkflowDock;

impl WorkflowDock {
    pub const HEIGHT: f32 = 48.0;

    /// 渲染底部工作流 Dock
    pub fn show(ui: &mut Ui, current_page: &mut WorkflowPage) {
        let dock_rect = ui.available_rect_before_wrap();
        let total_width = dock_rect.width();

        egui::Frame::new()
            .fill(ClipFlowTheme::SURFACE_WARM)
            .stroke(Stroke::new(1.0, ClipFlowTheme::BORDER))
            .show(ui, |ui| {
                ui.set_height(Self::HEIGHT);
                ui.set_width(total_width);

                ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                    ui.add_space(16.0);

                    for page in WorkflowPage::ALL {
                        let is_active = *current_page == page;
                        let text = format!("{} {}", page.as_str(), page.label_zh());

                        let response = Self::render_dock_item(ui, &text, is_active);
                        if response.clicked() {
                            *current_page = page;
                        }
                        ui.add_space(8.0);
                    }
                });
            });
    }

    fn render_dock_item(ui: &mut Ui, text: &str, is_active: bool) -> Response {
        let (rect, response) = ui.allocate_exact_size(vec2(100.0, 36.0), Sense::click());

        if ui.is_rect_visible(rect) {
            let is_hovered = response.hovered();

            // 背景绘制
            let bg_color = if is_hovered {
                ClipFlowTheme::SURFACE_ACTIVE
            } else {
                Color32::TRANSPARENT
            };
            ui.painter()
                .rect_filled(rect, ClipFlowTheme::RADIUS_CONTROL, bg_color);

            // 文本颜色
            let text_color = if is_active {
                ClipFlowTheme::COBALT_ACCENT
            } else if is_hovered {
                ClipFlowTheme::TEXT_PRIMARY
            } else {
                ClipFlowTheme::TEXT_MUTED
            };

            // 顶部激活指示信号线 (2px Cobalt Blue)
            if is_active {
                let stroke_start = rect.left_top();
                let stroke_end = rect.right_top();
                ui.painter().line_segment(
                    [stroke_start, stroke_end],
                    Stroke::new(2.5, ClipFlowTheme::COBALT_ACCENT),
                );
            }

            // 文本绘制
            ui.painter().text(
                rect.center(),
                Align2::CENTER_CENTER,
                text,
                FontId::proportional(13.0),
                text_color,
            );
        }

        response
    }
}
