use egui::{Color32, Stroke, Ui};
use crate::theme::ClipFlowTheme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowPage {
    Agent,
    Edit,
    Motion,
    Audio,
    Image,
    Deliver,
}

impl WorkflowPage {
    pub fn name(&self) -> &'static str {
        match self {
            WorkflowPage::Agent => "AGENT",
            WorkflowPage::Edit => "EDIT",
            WorkflowPage::Motion => "MOTION",
            WorkflowPage::Audio => "AUDIO",
            WorkflowPage::Image => "IMAGE",
            WorkflowPage::Deliver => "DELIVER",
        }
    }
    
    pub const ALL: [WorkflowPage; 6] = [
        WorkflowPage::Agent,
        WorkflowPage::Edit,
        WorkflowPage::Motion,
        WorkflowPage::Audio,
        WorkflowPage::Image,
        WorkflowPage::Deliver,
    ];
}

pub struct DockState {
    current_page: WorkflowPage,
}

impl DockState {
    pub fn new(initial_page: WorkflowPage) -> Self {
        Self {
            current_page: initial_page,
        }
    }

    pub fn current_page(&self) -> WorkflowPage {
        self.current_page
    }

    pub fn navigate_to(&mut self, page: WorkflowPage) {
        self.current_page = page;
    }

    /// 渲染底部 48px 导航栏
    pub fn ui(&mut self, ui: &mut Ui) {
        // 固定在底部的 Panel
        egui::Panel::bottom("clipflow_dock")
            .exact_size(48.0)
            .frame(
                egui::Frame::new()
                    .fill(ClipFlowTheme::SURFACE_WARM)
                    .inner_margin(egui::Margin::symmetric(16, 0))
            )
            .show_separator_line(false) // 我们自己画顶部分割线
            .show(ui, |ui| {
                // 绘制顶部分割线 1px solid #2A2F3A
                let rect = ui.max_rect();
                ui.painter().hline(
                    rect.x_range(),
                    rect.top(),
                    Stroke::new(1.0, ClipFlowTheme::BORDER),
                );

                ui.horizontal_centered(|ui| {
                    ui.spacing_mut().item_spacing.x = 24.0;
                    
                    for page in WorkflowPage::ALL {
                        let is_active = self.current_page == page;
                        
                        let text_color = if is_active {
                            ClipFlowTheme::COBALT_ACCENT
                        } else {
                            ClipFlowTheme::TEXT_MUTED
                        };

                        let response = ui.add(
                            egui::Button::new(
                                egui::RichText::new(page.name())
                                    .size(13.0)
                                    .color(text_color)
                            )
                            .fill(if is_active { ClipFlowTheme::SURFACE_ACTIVE } else { Color32::TRANSPARENT })
                            .frame(is_active)
                        );

                        if response.clicked() {
                            self.navigate_to(page);
                        }

                        // 如果激活，绘制顶部的 2px 蓝线 (Cobalt Blue 信号线)
                        if is_active {
                            let top_line_y = rect.top();
                            ui.painter().hline(
                                response.rect.x_range(),
                                top_line_y + 1.0, // 稍微向下偏移避免被裁剪
                                Stroke::new(2.0, ClipFlowTheme::COBALT_ACCENT),
                            );
                        }
                    }
                });
            });
    }
}
