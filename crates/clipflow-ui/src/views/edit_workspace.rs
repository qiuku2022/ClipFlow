use crate::views::inspector::InspectorPanel;
use crate::views::monitors::DualMonitorView;
use crate::views::project_panel::ProjectPanel;
use crate::views::splitter::ResizableSplitter;

pub struct EditWorkspaceView {
    splitter: ResizableSplitter,
    project_panel: ProjectPanel,
    monitors: DualMonitorView,
    inspector: InspectorPanel,
    zone1_width_ratio: f32,
    zone2_width_ratio: f32,
    zone3_width_ratio: f32,
}

impl Default for EditWorkspaceView {
    fn default() -> Self {
        Self::new()
    }
}

impl EditWorkspaceView {
    pub fn new() -> Self {
        Self {
            splitter: ResizableSplitter::new(0.58, 0.30, 0.70),
            project_panel: ProjectPanel::new(),
            monitors: DualMonitorView::new(),
            inspector: InspectorPanel::new(),
            zone1_width_ratio: 0.25,
            zone2_width_ratio: 0.50,
            zone3_width_ratio: 0.25,
        }
    }

    pub fn upper_height_ratio(&self) -> f32 {
        self.splitter.ratio()
    }

    pub fn zone1_width_ratio(&self) -> f32 {
        self.zone1_width_ratio
    }

    pub fn zone2_width_ratio(&self) -> f32 {
        self.zone2_width_ratio
    }

    pub fn zone3_width_ratio(&self) -> f32 {
        self.zone3_width_ratio
    }

    pub fn project_panel(&self) -> &ProjectPanel {
        &self.project_panel
    }

    pub fn monitors(&self) -> &DualMonitorView {
        &self.monitors
    }

    pub fn inspector(&self) -> &InspectorPanel {
        &self.inspector
    }

    pub fn splitter_mut(&mut self) -> &mut ResizableSplitter {
        &mut self.splitter
    }

    /// 渲染 PR 经典上半屏 3 区分屏工作台 (素材库 25% + 双监视器 50% + 属性面板 25%)
    pub fn show(&mut self, ui: &mut egui::Ui, height: f32) {
        let total_rect = ui.available_rect_before_wrap();
        let total_w = total_rect.width();

        let w1 = (total_w * self.zone1_width_ratio - 4.0).max(120.0);
        let w2 = (total_w * self.zone2_width_ratio - 8.0).max(240.0);
        let w3 = (total_w * self.zone3_width_ratio - 4.0).max(120.0);

        ui.allocate_ui_with_layout(
            egui::vec2(total_w, height),
            egui::Layout::left_to_right(egui::Align::Min),
            |ui| {
                // Zone 1: 项目素材面板
                egui::Frame::new()
                    .fill(crate::theme::ClipFlowTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, crate::theme::ClipFlowTheme::BORDER))
                    .corner_radius(crate::theme::ClipFlowTheme::RADIUS_PANEL)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(w1);
                        ui.set_height(height - 16.0);
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("项目素材库")
                                    .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY)
                                    .size(13.0)
                                    .strong(),
                            );
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("📁 媒体片段 (A-Roll)")
                                    .color(crate::theme::ClipFlowTheme::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("  ▶ camera_a_4k.mp4")
                                    .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY),
                            );
                            ui.label(
                                egui::RichText::new("  ▶ camera_b_1080p.mp4")
                                    .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY),
                            );
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("📁 音频资产")
                                    .color(crate::theme::ClipFlowTheme::TEXT_SECONDARY),
                            );
                            ui.label(
                                egui::RichText::new("  🔊 master_mic.wav")
                                    .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY),
                            );
                        });
                    });

                ui.add_space(4.0);

                // Zone 2: 双监视器视窗 (源监视器 + 节目监视器)
                egui::Frame::new()
                    .fill(crate::theme::ClipFlowTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, crate::theme::ClipFlowTheme::BORDER))
                    .corner_radius(crate::theme::ClipFlowTheme::RADIUS_PANEL)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(w2);
                        ui.set_height(height - 16.0);
                        ui.horizontal(|ui| {
                            let mon_w = (w2 - 24.0) / 2.0;

                            // 左：源监视器
                            ui.vertical(|ui| {
                                ui.set_width(mon_w);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("源监视器")
                                            .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(self.monitors.source_timecode())
                                                    .color(crate::theme::ClipFlowTheme::COBALT_ACCENT)
                                                    .monospace(),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(4.0);
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(mon_w, (height - 64.0).max(40.0)),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    crate::theme::ClipFlowTheme::RADIUS_CONTROL,
                                    crate::theme::ClipFlowTheme::BG_CANVAS,
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "源素材预览 (就绪)",
                                    egui::FontId::proportional(12.0),
                                    crate::theme::ClipFlowTheme::TEXT_MUTED,
                                );
                            });

                            ui.separator();

                            // 右：节目监视器
                            ui.vertical(|ui| {
                                ui.set_width(mon_w);
                                ui.horizontal(|ui| {
                                    ui.label(
                                        egui::RichText::new("节目监视器")
                                            .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY)
                                            .strong(),
                                    );
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Center),
                                        |ui| {
                                            ui.label(
                                                egui::RichText::new(self.monitors.program_timecode())
                                                    .color(crate::theme::ClipFlowTheme::COBALT_ACCENT)
                                                    .monospace(),
                                            );
                                        },
                                    );
                                });
                                ui.add_space(4.0);
                                let (rect, _) = ui.allocate_exact_size(
                                    egui::vec2(mon_w, (height - 64.0).max(40.0)),
                                    egui::Sense::hover(),
                                );
                                ui.painter().rect_filled(
                                    rect,
                                    crate::theme::ClipFlowTheme::RADIUS_CONTROL,
                                    crate::theme::ClipFlowTheme::BG_CANVAS,
                                );
                                ui.painter().text(
                                    rect.center(),
                                    egui::Align2::CENTER_CENTER,
                                    "时间线节目回放 (D3D11VA 60 FPS)",
                                    egui::FontId::proportional(12.0),
                                    crate::theme::ClipFlowTheme::TEXT_MUTED,
                                );
                            });
                        });
                    });

                ui.add_space(4.0);

                // Zone 3: 属性检查器面板
                egui::Frame::new()
                    .fill(crate::theme::ClipFlowTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, crate::theme::ClipFlowTheme::BORDER))
                    .corner_radius(crate::theme::ClipFlowTheme::RADIUS_PANEL)
                    .inner_margin(8.0)
                    .show(ui, |ui| {
                        ui.set_width(w3);
                        ui.set_height(height - 16.0);
                        ui.vertical(|ui| {
                            ui.heading(
                                egui::RichText::new("效果检查器")
                                    .color(crate::theme::ClipFlowTheme::TEXT_PRIMARY)
                                    .size(13.0)
                                    .strong(),
                            );
                            ui.add_space(6.0);
                            ui.separator();
                            ui.add_space(4.0);
                            ui.label(
                                egui::RichText::new("二维变换 (Transform 2D)")
                                    .color(crate::theme::ClipFlowTheme::TEXT_SECONDARY)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("  缩放: 100% | 旋转: 0.0°")
                                    .color(crate::theme::ClipFlowTheme::TEXT_MUTED),
                            );
                            ui.label(
                                egui::RichText::new("  位置: (0.0, 0.0) | 不透明度: 100%")
                                    .color(crate::theme::ClipFlowTheme::TEXT_MUTED),
                            );
                            ui.add_space(6.0);
                            ui.label(
                                egui::RichText::new("音频混音 (Track Audio)")
                                    .color(crate::theme::ClipFlowTheme::TEXT_SECONDARY)
                                    .strong(),
                            );
                            ui.label(
                                egui::RichText::new("  推子: 0.0 dB | 声像: 居中")
                                    .color(crate::theme::ClipFlowTheme::TEXT_MUTED),
                            );
                        });
                    });
            },
        );
    }
}
