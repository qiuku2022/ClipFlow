//! ClipFlow 桌面端应用程序主入口

use clipflow_ui::dock::WorkflowDock;
use clipflow_ui::state::{AppState, RepaintState};
use clipflow_ui::theme::ClipFlowTheme;
use clipflow_ui::timeline_placeholder::render_timeline_placeholder;
use eframe::egui;

struct ClipFlowApp {
    state: AppState,
}

impl ClipFlowApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // 在初始化时应用 Neutral Modern 设计系统全局样式
        ClipFlowTheme::apply_to(&cc.egui_ctx);
        Self {
            state: AppState::default(),
        }
    }
}

impl eframe::App for ClipFlowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();

        // 1. 每帧确保样式与主题对齐
        ClipFlowTheme::apply_to(ctx);

        // 2. 检查输入事件更新调度器
        let has_input = ctx.input(|i| {
            !i.raw.events.is_empty() || i.pointer.is_moving() || i.smooth_scroll_delta != egui::Vec2::ZERO
        });
        if has_input {
            self.state.scheduler.on_user_interaction();
        }

        // 3. 门控调度
        let gating = self.state.scheduler.update_gating(self.state.is_playing);
        if gating != RepaintState::Dormant {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // 4. 界面布局：上半屏（工作流页面卡片）、下半屏（时间线底座）、最底部（Dock 栏）
        let total_rect = ui.available_rect_before_wrap();
        let timeline_height = 200.0;
        let upper_height = (total_rect.height() - timeline_height - WorkflowDock::HEIGHT - 16.0).max(150.0);

        // 上半屏：当前工作流页面视窗
        ui.allocate_ui_with_layout(
            egui::vec2(total_rect.width(), upper_height),
            egui::Layout::top_down(egui::Align::Min),
            |ui| {
                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    ui.heading(
                        egui::RichText::new("ClipFlow")
                            .color(ClipFlowTheme::TEXT_PRIMARY)
                            .strong(),
                    );
                    ui.label(
                        egui::RichText::new(format!(
                            "当前工作区: {} ({})",
                            self.state.active_page.as_str(),
                            self.state.active_page.label_zh()
                        ))
                        .color(ClipFlowTheme::COBALT_ACCENT),
                    );
                });

                ui.add_space(8.0);

                egui::Frame::new()
                    .fill(ClipFlowTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, ClipFlowTheme::BORDER))
                    .corner_radius(ClipFlowTheme::RADIUS_PANEL)
                    .inner_margin(12.0)
                    .show(ui, |ui| {
                        ui.label(
                            egui::RichText::new(format!(
                                "{} 视窗内容区 (M0 骨架演示)",
                                self.state.active_page.label_zh()
                            ))
                            .color(ClipFlowTheme::TEXT_PRIMARY)
                            .strong(),
                        );
                        ui.add_space(4.0);
                        ui.label(
                            egui::RichText::new(format!(
                                "门控状态: {:?} | 播放头: {}",
                                gating, self.state.playhead
                            ))
                            .color(ClipFlowTheme::TEXT_MUTED),
                        );
                    });
            },
        );

        ui.add_space(8.0);

        // 下半屏：全局公用时间线占位底座（常驻保活）
        render_timeline_placeholder(ui, &self.state);

        ui.add_space(4.0);

        // 底部达芬奇 48px Dock 栏
        WorkflowDock::show(ui, &mut self.state.active_page);
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("ClipFlow 正在启动主窗口...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 800.0])
            .with_min_inner_size([960.0, 600.0])
            .with_title("ClipFlow - AI 导演级剪辑工坊")
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "ClipFlow",
        native_options,
        Box::new(|cc| Ok(Box::new(ClipFlowApp::new(cc)))),
    )
}
