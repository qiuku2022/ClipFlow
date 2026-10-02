//! ClipFlow 桌面端应用程序主入口 (Milestone 1: 媒体硬解播放与 PR 经典剪辑台)

use clipflow_common::{RationalTime, TimeRange};
use clipflow_media::audio::{HysteresisSyncComparator, MonotonicClampedClock};
use clipflow_media::render::{DeviceLostWatchdog, ProxyGovernor};
use clipflow_timeline::models::{
    AudioProperties, CanvasSize, Clip, ClipPayload, Sequence, Track, TrackKind, Transform2D,
};
use clipflow_ui::dock::{WorkflowDock, WorkflowPage};
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
    edit_workspace: EditWorkspaceView,
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
        // 1. 初始化中文字体支持 (Windows 微软雅黑 / 黑体)
        ClipFlowTheme::init_fonts(&cc.egui_ctx);
        // 2. 注入全局 Neutral Modern 深色视觉主题
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

        let mut state = AppState::default();
        state.active_page = WorkflowPage::Edit; // 默认进入 PR 经典剪辑工作台

        Self {
            state,
            edit_workspace: EditWorkspaceView::new(),
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

        // 3. 门控重绘调度
        let gating = self.state.scheduler.update_gating(self.state.is_playing);
        if gating != RepaintState::Dormant {
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }

        // 4. 视口绝对布局划分：Dock 贴底 + 主内容区铺满
        let total_rect = ui.available_rect_before_wrap();
        let dock_h = WorkflowDock::HEIGHT;

        // 底部 Dock 矩形 (紧贴窗口最下端)
        let dock_rect = egui::Rect::from_min_size(
            egui::pos2(total_rect.left(), total_rect.bottom() - dock_h),
            egui::vec2(total_rect.width(), dock_h),
        );

        // 主内容视口矩形 (占满 Dock 以上全部空间)
        let main_rect = egui::Rect::from_min_max(
            total_rect.min,
            egui::pos2(total_rect.right(), total_rect.bottom() - dock_h - 2.0),
        );

        // 渲染主视口
        let mut main_ui = ui.new_child(egui::UiBuilder::new().max_rect(main_rect));
        {
            let ui = &mut main_ui;
            // 顶部状态栏
            ui.horizontal(|ui| {
                ui.heading(
                    egui::RichText::new("ClipFlow")
                        .color(ClipFlowTheme::TEXT_PRIMARY)
                        .strong()
                        .size(15.0),
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
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.label(
                        egui::RichText::new(format!(
                            "门控调度: {:?} | 播放头: {}",
                            gating, self.state.playhead
                        ))
                        .color(ClipFlowTheme::TEXT_MUTED)
                        .size(11.0),
                    );
                });
            });

            ui.add_space(4.0);

            let avail_h = ui.available_height();

            if self.state.active_page == WorkflowPage::Edit {
                // M1 PR 经典剪辑工作台：上半屏 58% + 下半屏 42% 满宽时间线
                let split_ratio = self.edit_workspace.upper_height_ratio();
                let upper_h = (avail_h * split_ratio - 4.0).max(180.0);

                // 上半屏：素材库 25% + 双监视器 50% + 检查器 25%
                self.edit_workspace.show(ui, upper_h);

                ui.add_space(4.0);

                // 下半屏：铺满剩余高度的多轨时间线
                render_timeline_placeholder(ui, &self.state);
            } else {
                // 其他工作流页面视窗 (如 Agent 导演页、动效页)
                let card_h = (avail_h * 0.55).max(160.0);
                egui::Frame::new()
                    .fill(ClipFlowTheme::SURFACE)
                    .stroke(egui::Stroke::new(1.0, ClipFlowTheme::BORDER))
                    .corner_radius(ClipFlowTheme::RADIUS_PANEL)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_height(card_h);
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.heading(
                                egui::RichText::new(format!(
                                    "{} 工作区",
                                    self.state.active_page.label_zh()
                                ))
                                .color(ClipFlowTheme::TEXT_PRIMARY),
                            );
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("该模块将在后续对应里程碑中持续交付")
                                    .color(ClipFlowTheme::TEXT_MUTED),
                            );
                        });
                    });

                ui.add_space(4.0);

                // 时间线常驻保活
                render_timeline_placeholder(ui, &self.state);
            }
        }

        // 渲染底部 Dock 栏 (绝对定位贴底)
        let mut dock_ui = ui.new_child(egui::UiBuilder::new().max_rect(dock_rect));
        WorkflowDock::show(&mut dock_ui, &mut self.state.active_page);
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
