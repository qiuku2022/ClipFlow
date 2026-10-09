use eframe::egui;
use clipflow_ui::theme::ClipFlowTheme;

#[derive(Default)]
pub struct ClipFlowApp {
    // 稍后将在此处添加应用状态
}

impl eframe::App for ClipFlowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // 全局设置主题
        ClipFlowTheme::apply_to(ui.ctx());

        egui::CentralPanel::default().show(ui, |ui| {
            ui.heading("ClipFlow Studio M0");
            ui.label("Window Host with Neutral Modern Dark Theme is ready.");
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1280.0, 720.0])
            .with_min_inner_size([800.0, 600.0])
            .with_title("ClipFlow"),
        ..Default::default()
    };

    eframe::run_native(
        "ClipFlow",
        options,
        Box::new(|_cc| Ok(Box::new(ClipFlowApp::default()))),
    )
}
