//! ClipFlow 桌面端应用程序主入口

use clipflow_ui::state::{AppState, RepaintState};
use clipflow_ui::theme::ClipFlowTheme;
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

        // 4. 主视口界面呈现
        ui.heading(
            egui::RichText::new("ClipFlow")
                .color(ClipFlowTheme::TEXT_PRIMARY)
                .strong(),
        );
        ui.label(
            egui::RichText::new("AI 导演级剪辑工坊 (Milestone 0)")
                .color(ClipFlowTheme::TEXT_SECONDARY),
        );

        ui.add_space(16.0);

        // 状态卡片
        egui::Frame::new()
            .fill(ClipFlowTheme::SURFACE)
            .stroke(egui::Stroke::new(1.0, ClipFlowTheme::BORDER))
            .corner_radius(ClipFlowTheme::RADIUS_PANEL)
            .inner_margin(16.0)
            .show(ui, |ui| {
                ui.label(
                    egui::RichText::new("系统状态监控")
                        .color(ClipFlowTheme::TEXT_PRIMARY)
                        .strong(),
                );
                ui.add_space(8.0);
                ui.label(format!("播放头位置: {}", self.state.playhead));
                ui.label(format!("门控调度状态: {:?}", gating));
            });
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
