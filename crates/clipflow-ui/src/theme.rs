use egui::{Color32, CornerRadius, Stroke, Visuals};

/// ClipFlow Neutral Modern 深色模式设计规范主题
pub struct ClipFlowTheme;

impl ClipFlowTheme {
    // ------------------------------------------------------------------------
    // 1. 核心表面与背景色 (Surfaces & Canvas)
    // ------------------------------------------------------------------------
    pub const BG_CANVAS: Color32 = Color32::from_rgb(15, 17, 21);        // #0F1115 (主视口底板)
    pub const SURFACE: Color32 = Color32::from_rgb(23, 26, 33);          // #171A21 (主面板卡片)
    pub const SURFACE_WARM: Color32 = Color32::from_rgb(30, 34, 43);     // #1E222B (顶部工具/底部Dock)
    pub const SURFACE_ACTIVE: Color32 = Color32::from_rgb(35, 39, 51);   // #232733 (选中/激活面板)

    // ------------------------------------------------------------------------
    // 2. 文本灰阶 (Foreground Ramp)
    // ------------------------------------------------------------------------
    pub const TEXT_PRIMARY: Color32 = Color32::from_rgb(248, 250, 252);  // #F8FAFC (主文本)
    pub const TEXT_SECONDARY: Color32 = Color32::from_rgb(226, 232, 240);// #E2E8F0 (次级正文)
    pub const TEXT_MUTED: Color32 = Color32::from_rgb(167, 173, 186);    // #A7ADBA (弱化文字/未激活)
    pub const TEXT_META: Color32 = Color32::from_rgb(100, 116, 139);     // #64748B (刻度/元数据)

    // ------------------------------------------------------------------------
    // 3. 边框与分割线 (Borders & Dividers)
    // ------------------------------------------------------------------------
    pub const BORDER: Color32 = Color32::from_rgb(42, 47, 58);            // #2A2F3A (主分割线)
    pub const BORDER_SOFT: Color32 = Color32::from_rgba_premultiplied(255, 255, 255, 20); // 8% 白细线

    // ------------------------------------------------------------------------
    // 4. 钴蓝交互信号与状态色 (Accent & States)
    // ------------------------------------------------------------------------
    pub const COBALT_ACCENT: Color32 = Color32::from_rgb(47, 111, 235);  // #2F6FEB (主信号/播放指针)
    pub const ACCENT_HOVER: Color32 = Color32::from_rgb(68, 126, 237);   // 提亮悬浮
    pub const ACCENT_ACTIVE: Color32 = Color32::from_rgb(42, 100, 212);  // 按下微沉

    pub const STATUS_SUCCESS: Color32 = Color32::from_rgb(23, 163, 74);  // #17A34A (导出完成/音频安全)
    pub const STATUS_WARN: Color32 = Color32::from_rgb(234, 179, 8);     // #EAB308 (警告/待确认)
    pub const STATUS_DANGER: Color32 = Color32::from_rgb(220, 38, 38);   // #DC2626 (剪切/错误)
    pub const STATUS_INFO: Color32 = Color32::from_rgb(2, 132, 199);     // #0284C7 (转写/信息)

    // ------------------------------------------------------------------------
    // 5. 非编多轨时间轴色谱 (Timeline Tracks)
    // ------------------------------------------------------------------------
    pub const TRACK_VIDEO_BG: Color32 = Color32::from_rgb(21, 30, 46);    // #151E2E (视频片段深蓝)
    pub const TRACK_VIDEO_STROKE: Color32 = Color32::from_rgb(43, 69, 112);// #2B4570 (视频片段描边)
    pub const TRACK_AUDIO_BG: Color32 = Color32::from_rgb(18, 33, 25);    // #122119 (音频片段暗绿)
    pub const TRACK_AUDIO_STROKE: Color32 = Color32::from_rgb(30, 74, 50);// #1E4A32 (音频描边)
    pub const TRACK_SUBTITLE_BG: Color32 = Color32::from_rgb(39, 31, 18); // #271F12 (字幕暗金)
    pub const TRACK_SUBTITLE_STROKE: Color32 = Color32::from_rgb(99, 71, 28); // #63471C (字幕描边)
    pub const TRACK_MOTION_BG: Color32 = Color32::from_rgb(34, 20, 45);   // #22142D (动效暗紫)
    pub const TRACK_MOTION_STROKE: Color32 = Color32::from_rgb(82, 41, 116);// #522974 (动效描边)

    // ------------------------------------------------------------------------
    // 6. 几何圆角曲率 (CornerRadius: 12px Panel, 8px Control, 4px Clip)
    // ------------------------------------------------------------------------
    pub const RADIUS_PANEL: CornerRadius = CornerRadius::same(12);  // 面板容器 (12px)
    pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(8); // 按钮/输入框 (8px)
    pub const RADIUS_CLIP: CornerRadius = CornerRadius::same(4);    // 时间轴片段/小标牌 (4px)

    /// 一键配置全局 egui::Visuals 样式，严格对齐 Neutral Modern 深色规范
    pub fn apply_to(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        // 画布底板与主面板底色
        visuals.panel_fill = Self::BG_CANVAS;
        visuals.window_fill = Self::SURFACE;
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.window_corner_radius = Self::RADIUS_PANEL;

        // 默认控件外观 (8px 圆角，深色卡片表面)
        visuals.widgets.inactive.bg_fill = Self::SURFACE;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::BORDER);
        visuals.widgets.inactive.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // 悬浮状态：向亮色微调并高亮细边框
        visuals.widgets.hovered.bg_fill = Self::SURFACE_WARM;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::BORDER_SOFT);
        visuals.widgets.hovered.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // 激活状态：加深底色
        visuals.widgets.active.bg_fill = Self::SURFACE_ACTIVE;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Self::COBALT_ACCENT);
        visuals.widgets.active.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::TEXT_PRIMARY);

        // 选中高亮状态：注入钴蓝信号
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(47, 111, 235, 45);
        visuals.selection.stroke = Stroke::new(1.5, Self::COBALT_ACCENT);

        ctx.set_visuals(visuals);
    }
}

