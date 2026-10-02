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

    /// 渲染 PR 经典上半屏 3 区分屏工作台 (素材库 25% + 双监视器 50% + 属性检查器 25%)
    ///
    /// 采用独立绝对矩形（Rect）几何隔离切分，彻底规避流式布局边距累加推挤，
    /// 确保最右侧检查器卡片 100% 完整显示右侧边框与圆角。
    pub fn show(&mut self, ui: &mut egui::Ui, height: f32) {
        let avail_rect = ui.available_rect_before_wrap();
        let total_w = avail_rect.width();

        let gap = 8.0;
        let usable_w = (total_w - 2.0 * gap).max(300.0);

        let w1 = (usable_w * self.zone1_width_ratio).floor();
        let w3 = (usable_w * self.zone3_width_ratio).floor();
        let w2 = usable_w - w1 - w3; // 精确互补，w1 + w2 + w3 + 2*gap 恒等于 total_w

        let y_min = avail_rect.min.y;
        let r1 = egui::Rect::from_min_size(avail_rect.min, egui::vec2(w1, height));
        let r2 = egui::Rect::from_min_size(
            egui::pos2(r1.max.x + gap, y_min),
            egui::vec2(w2, height),
        );
        let r3 = egui::Rect::from_min_size(
            egui::pos2(r2.max.x + gap, y_min),
            egui::vec2(w3, height),
        );

        // 统一绘制 3 个卡片的刚性几何底板与边框，确保卡片外轮廓 100% 贴合数学边界，
        // 彻底根除内部文本宽度不足导致卡片缩水产生的“视觉虚假缝隙”
        for r in [&r1, &r2, &r3] {
            ui.painter().rect_filled(
                *r,
                crate::theme::ClipFlowTheme::RADIUS_PANEL,
                crate::theme::ClipFlowTheme::SURFACE,
            );
            ui.painter().rect_stroke(
                *r,
                crate::theme::ClipFlowTheme::RADIUS_PANEL,
                egui::Stroke::new(1.0, crate::theme::ClipFlowTheme::BORDER),
                egui::StrokeKind::Inside,
            );
        }

        let inner_padding = 8.0;

        // 1. Zone 1: 项目素材面板 (左侧 25%)
        let content_r1 = r1.shrink(inner_padding);
        let mut ui1 = ui.new_child(egui::UiBuilder::new().max_rect(content_r1));
        ui1.vertical(|ui| {
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

        // 2. Zone 2: 双监视器视窗 (中间 50%)
        let content_r2 = r2.shrink(inner_padding);
        let mut ui2 = ui.new_child(egui::UiBuilder::new().max_rect(content_r2));
        {
            let ui = &mut ui2;
            let total_cw = content_r2.width();
            let mon_gap = 8.0;
            let mon_w = ((total_cw - mon_gap) / 2.0).floor().max(40.0);
            let mon_h = (content_r2.height() - 28.0).max(40.0);

            // 左监视器：源监视器
            let left_rect = egui::Rect::from_min_size(
                content_r2.min,
                egui::vec2(mon_w, content_r2.height()),
            );
            let mut left_ui = ui.new_child(egui::UiBuilder::new().max_rect(left_rect));
            left_ui.vertical(|ui| {
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
                    egui::vec2(mon_w, mon_h),
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

            // 右监视器：节目监视器
            let right_rect = egui::Rect::from_min_size(
                egui::pos2(left_rect.max.x + mon_gap, content_r2.min.y),
                egui::vec2(mon_w, content_r2.height()),
            );
            let mut right_ui = ui.new_child(egui::UiBuilder::new().max_rect(right_rect));
            right_ui.vertical(|ui| {
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
                    egui::vec2(mon_w, mon_h),
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
        }

        // 3. Zone 3: 属性检查器面板 (右侧 25%)
        let content_r3 = r3.shrink(inner_padding);
        let mut ui3 = ui.new_child(egui::UiBuilder::new().max_rect(content_r3));
        ui3.vertical(|ui| {
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

        // 在父级 UI 中预先占位整个上半屏区域，将游标推进到下半部
        ui.allocate_rect(
            egui::Rect::from_min_size(avail_rect.min, egui::vec2(total_w, height)),
            egui::Sense::hover(),
        );
    }
}
