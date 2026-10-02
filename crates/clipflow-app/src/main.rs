//! ClipFlow 桌面端应用程序主入口 (Milestone 1: 媒体硬解播放与 PR 经典剪辑台)

use clipflow_common::{RationalTime, TimeRange};
use clipflow_media::audio::{HysteresisSyncComparator, MonotonicClampedClock};
use clipflow_media::render::{DeviceLostWatchdog, ProxyGovernor};
use clipflow_timeline::models::{
    AudioProperties, CanvasSize, Clip, ClipPayload, Sequence, Track, TrackKind, Transform2D,
};
use clipflow_ui::dock::WorkflowDock;
use clipflow_ui::interaction::ShuttleController;
use clipflow_ui::state::{AppState, RepaintState};
use clipflow_ui::theme::ClipFlowTheme;
use clipflow_ui::timeline::SharedTimelineCanvas;
use clipflow_ui::timeline_placeholder::render_timeline_placeholder;
use clipflow_ui::views::EditWorkspaceView;
use eframe::egui;
use std::sync::Arc;
use uuid::Uuid;

fn create_demo_clip(name: &str, start_ticks: i64, dur_ticks: i64, timescale: u32) -> Clip {
    Clip {
        id: Uuid::new_v4(),
        name: name.to_string(),
        asset_id: Uuid::new_v4(),
        source_range: TimeRange::new(
            RationalTime::new(0, timescale),
            RationalTime::new(dur_ticks, timescale),
        ),
        timeline_range: TimeRange::new(
            RationalTime::new(start_ticks, timescale),
            RationalTime::new(dur_ticks, timescale),
        ),
        speed: 1.0,
        transform: Transform2D::default(),
        audio_props: AudioProperties::default(),
        filters: Vec::new(),
        payload: ClipPayload::Media,
        disabled: false,
    }
}

struct ClipFlowApp {
    state: AppState,
    _edit_workspace: EditWorkspaceView,
    _timeline_canvas: SharedTimelineCanvas,
    _sequence: Sequence,
    shuttle: ShuttleController,
    _clock: Arc<MonotonicClampedClock>,
    _comparator: HysteresisSyncComparator,
    _governor: ProxyGovernor,
    _watchdog: DeviceLostWatchdog,
}

impl ClipFlowApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        ClipFlowTheme::apply_to(&cc.egui_ctx);

        // 初始化示范工程与多轨序列
        let mut seq = Sequence::new("ClipFlow Demo Project", CanvasSize::P1080_16_9, 60, 1);
        let clip_v = create_demo_clip("camera_a_4k.mp4", 0, 1200, 60);
        let clip_a = create_demo_clip("camera_a_audio.wav", 0, 1200, 60);

        let v1 = Track {
            id: Uuid::new_v4(),
            name: "V1".to_string(),
            kind: TrackKind::Video,
            mute: false,
            solo: false,
            locked: false,
            visible: true,
            audio_props: None,
            clips: vec![clip_v],
        };
        let a1 = Track {
            id: Uuid::new_v4(),
            name: "A1".to_string(),
            kind: TrackKind::Audio,
            mute: false,
            solo: false,
            locked: false,
            visible: true,
            audio_props: None,
            clips: vec![clip_a],
        };

        seq.tracks.push(v1);
        seq.tracks.push(a1);

        Self {
            state: AppState::default(),
            _edit_workspace: EditWorkspaceView::new(),
            _timeline_canvas: SharedTimelineCanvas::new(),
            _sequence: seq,
            shuttle: ShuttleController::new(),
            _clock: Arc::new(MonotonicClampedClock::new(16_666_667)),
            _comparator: HysteresisSyncComparator::new(),
            _governor: ProxyGovernor::new(),
            _watchdog: DeviceLostWatchdog::new(),
        }
    }
}

impl eframe::App for ClipFlowApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        let ctx = ui.ctx();
        ClipFlowTheme::apply_to(ctx);

        // 1. 快捷键与飞梭走带交互 (J-K-L 与空格)
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Space) {
                self.shuttle.press_space();
                self.state.is_playing = self.shuttle.is_playing();
            } else if i.key_pressed(egui::Key::L) {
                self.shuttle.press_l();
                self.state.is_playing = self.shuttle.is_playing();
            } else if i.key_pressed(egui::Key::J) {
                self.shuttle.press_j();
                self.state.is_playing = self.shuttle.is_playing();
            } else if i.key_pressed(egui::Key::K) {
                self.shuttle.press_k();
                self.state.is_playing = false;
            }
        });

        // 2. 交互状态更新
        let has_input = ctx.input(|i| {
            !i.raw.events.is_empty()
                || i.pointer.is_moving()
                || i.smooth_scroll_delta != egui::Vec2::ZERO
        });
        if has_input {
            self.state.scheduler.on_user_interaction();
        }

        // 3. 门控调度
        let gating = self.state.scheduler.update_gating(self.state.is_playing);
        if gating != RepaintState::Dormant {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // 4. PR 经典分屏工作台布局
        let total_rect = ui.available_rect_before_wrap();
        let timeline_height = 240.0;
        let upper_height =
            (total_rect.height() - timeline_height - WorkflowDock::HEIGHT - 16.0).max(150.0);

        // 上半屏：当前工作流页面视窗 (58% 经典分屏)
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
                            "当前工作区: {} ({}) | 走带速度: {}x ({:?})",
                            self.state.active_page.as_str(),
                            self.state.active_page.label_zh(),
                            self.shuttle.speed_multiplier(),
                            self.shuttle.direction()
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
                                "{} 视窗内容区 (M1 PR 经典剪辑工作台)",
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

        // 下半屏：全局公用时间线底座
        render_timeline_placeholder(ui, &self.state);

        ui.add_space(4.0);

        // 底部达芬奇 48px Dock 栏
        WorkflowDock::show(ui, &mut self.state.active_page);
    }
}

fn main() -> eframe::Result<()> {
    tracing_subscriber::fmt::init();
    tracing::info!("ClipFlow M1 正在启动主窗口...");

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1024.0, 640.0])
            .with_title("ClipFlow - AI 导演级剪辑工坊 (M1)")
            .with_active(true),
        ..Default::default()
    };

    eframe::run_native(
        "ClipFlow",
        native_options,
        Box::new(|cc| Ok(Box::new(ClipFlowApp::new(cc)))),
    )
}
