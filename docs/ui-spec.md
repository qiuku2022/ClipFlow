# ClipFlow UI 设计规范与视觉体系 (UI Specification)

> **设计基准**：采用来自 OpenDesign 的 **Neutral Modern 深色设计体系 (Neutral Modern Dark Mode)**。  
> **核心定位**：以“沉静内敛、内容优先、精密工控 (Calm, Functional, Quietly Confident)”为核心哲学。以深岩灰底板 (`#0F1115`)、分层中性表面 (`#171A21` / `#1E222B`)、精密发丝边框 (`#2A2F3A`) 与极度克制的钴蓝交互信号 (`#2F6FEB`)，构建兼顾广播级暗房色彩还原、长时间剪辑无视觉疲劳与现代产品质感的专业桌面音视频工作台。

---

## 1. 核心设计哲学 (Visual Theme & Atmosphere)

Neutral Modern 深色模式遵循 OpenDesign 规范与专业数字视音频工作台 (NLE/DAW) 的界面法则：

- **内容为先，界面退后 (Content-First, Chrome-Second)**：
  软件界面的使命是烘托画面与波形内容，而非宣泄界面装饰。杜绝无意义的彩色背景涂抹、发光大投影与毛玻璃折射，所有视觉层次均由严谨的色阶与结构边框自然拉开。
- **非纯黑沉浸暗房基底 (True Dark Slate, Not Pure Black)**：
  严格遵循设计体系的防眩光准则——**避免使用刺眼的纯黑 `#000000` 与 100% 刺眼纯白 `#FFFFFF`**（防止 OLED/高刷显示器的拖影残影与高反差视觉疲劳）。工作区以 `#0F1115` 为主底板，面板以 `#171A21` 形成温润悬浮，确保监视器内视频与调色预览达到广播级纯净度。
- **极度克制的钴蓝信号 (Cobalt Blue Accent `#2F6FEB`)**：
  标志性 Cobalt Blue (`#2F6FEB`) 是系统的**第一交互信号源**。遵循“单屏不超过 1~2 处显著焦点”的严苛原则，专用于：播放指针 (Playhead)、主操作按钮 (Primary CTA)、选中与激活边框、关键帧节点与进度指示。
- **暗色状态反转法则 (Dark Mode State Inversion)**：
  在深色模式下，按钮与控件悬停 (Hover) 状态不采取暗化，而是通过向白色混色提亮 (`color-mix(in oklab, var(--accent), white 10%)`) 形成清晰反馈；按下 (Active) 则微幅下沉。
- **严谨工业级倒角梯度 (Tailored Radius Scale)**：
  为高密度信息流量身定制：微型切片与轨道标签收敛为 **4px**，基础控件（按钮、输入框）为 **8px**，视窗面板容器为 **12px**。既保持工控机具的紧凑精密，又具备当代软件的高级耐看质感。

---

## 2. 几何曲率规范 (Border Radius Scale)

系统全量控件遵循精密、清晰的圆角层级梯队：

```
+---------------------------------------------------------------------------------+
| 桌面窗口宿主外框 (Host Window Frame): 8-12px (由 Windows 11 OS 顶层外壳接管)      |
|  +---------------------------------------------------------------------------+  |
|  | 内部面板 / 视窗容器 (Panel Level): 12px (--radius-md)                        |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  | 基础交互控件 (Control): 8px       |  | 时间轴片段 (Clip): 4px        |  |  |
|  |  | [ 主操作 CTA ]                    |  | [===== 视频 V1 =====]        |  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  | 微型标签 / 徽章 (Micro Tag): 4px   |  | 圆形指示器 (Circle): 50%       |  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  +---------------------------------------------------------------------------+  |
+---------------------------------------------------------------------------------+
```

| 级别 | Token | 曲率半径 (Radius) | 对应设计定义 | 软件内具体应用场景 |
| :--- | :--- | :--- | :--- | :--- |
| **视窗与面板容器** | `--radius-md` | **12px** | 模块化视窗外壳 | PR 四区分屏面板外框、模态对话框、参数调节卡片、监视器外框 |
| **基础交互控件** | `--radius-sm` | **8px** | 标准人机交互件 | 按钮 (Button)、输入框 (Input)、下拉菜单、搜索框、数值微调框 |
| **高密流切片/标签**| `--radius-xs` | **4px** | 密集数据切片 | 多轨时间轴上的视频片段、音频片段、字幕块、动效块、轨道头编号 |
| **微型状态标签** | `--radius-xs` | **4px** | 辅助状态标签 | 快捷键提示徽章、时间码微调徽标、帧率标签 |
| **胶囊与点状指示** | `--radius-pill`| **9999px (50%)** | 状态流动指示 | Agent 在线呼吸指示灯、录音红点、调色色轮中心指针、Chip 标签 |

---

## 3. 色彩体系与 Tokens (Color Palette & Semantic Roles)

本地设计体系镜像保存于 [`.local/design-system/neutral-modern/`](file:///d:/Work/Dev/ClipFlow/.local/design-system/neutral-modern/)。

### 3.1 核心中性色与表面层级 (Surfaces & Canvas)

| 语义角色 | Token 名称 | 色值 (HEX / RGBA) | 视觉功能与定义 |
| :--- | :--- | :--- | :--- |
| **画布底色** | `--bg` | `#0F1115` | 应用全局总底板、监视器外部暗房、时间轴轨道槽背景 |
| **主面板表面** | `--surface` | `#171A21` | 剪辑四大面板（素材池、监视器、检查器、时间轴）内部卡片 |
| **次级/工具表面** | `--surface-warm` | `#1E222B` | 顶部菜单栏、工具箱条、底部达芬奇式 Dock 栏背景 |
| **激活/焦点表面** | `--surface-active`| `#232733` | 当前处于焦点、被鼠标选中或高亮的面板区域 |

### 3.2 文本阶梯 (Foreground Ramp)

| 阶梯层级 | Token 名称 | 色值 (HEX) | 用途与映射面板 |
| :--- | :--- | :--- | :--- |
| **主阅读文字** | `--fg` | `#F8FAFC` | 关键标题、主时间码数字、当前激活标签、选中图标 |
| **次级正文** | `--fg-2` | `#E2E8F0` | 属性面板参数名、下拉菜单选项、二级标题 |
| **弱化辅助文本** | `--muted` | `#A7ADBA` | 描述文字、功能说明、占位符、未选中小图标 |
| **极弱/元数据** | `--meta` | `#64748B` | 时间轴刻度数值、轨道头编号 (`V1`, `A1`)、媒体编码信息 |

### 3.3 交互信号色与状态色 (Accent & States)

| 语义名称 | Token 名称 | 色值 (HEX / CSS) | 界面功能与定义 |
| :--- | :--- | :--- | :--- |
| **Cobalt 钴蓝 (主信号)**| `--accent` | `#2F6FEB` | **最高优先级交互色**：播放指针、主 CTA、选中外框、活动轨道指示 |
| **交互提亮 (Hover)** | `--accent-hover` | `color-mix(in oklab, #2F6FEB, white 10%)` | 悬浮状态轻度提亮反馈 |
| **交互下沉 (Active)** | `--accent-active`| `color-mix(in oklab, #2F6FEB, black 10%)` | 鼠标按下时的加深反馈 |
| **信号上层文字** | `--accent-on` | `#FFFFFF` | 钴蓝底色上的高反差纯白文本 |
| **聚焦外框 (Focus Ring)**| `--focus-ring` | `0 0 0 3px rgba(47, 111, 235, 0.35)` | 键盘导航与输入框聚焦光圈 |

### 3.4 结构线条与边框 (Borders & Dividers)

| 语义角色 | Token 名称 | 色值 (HEX / RGBA) | 视觉功能说明 |
| :--- | :--- | :--- | :--- |
| **主边框** | `--border` | `#2A2F3A` | 面板外框、分屏可拖拽分割条、标准输入框描边 |
| **柔和边框** | `--border-soft` | `rgba(255, 255, 255, 0.08)` | 列表项行内分割、时间轴辅助刻度线、卡片内部分区 |
| **激活边框** | `--border-active` | `#2F6FEB` | 激活选中的轨道外框、聚焦容器高亮边框 |

### 3.5 状态与语义色谱 (Status & Semantic)

| 语义类型 | Token 名称 | 基础色值 | 暗色辅助搭配 | 视觉功能说明 |
| :--- | :--- | :--- | :--- | :--- |
| **Success (成功/安全)** | `--success` | `#17A34A` | `#122119` | 导出渲染完成、硬解管线就绪、音频标准电平安全区 |
| **Warning (警告/高能量)**| `--warn` | `#EAB308` | `#271F12` | 音频接近过载 (-6dB~0dB)、硬件高负载警告、入出点标记 |
| **Danger (危险/破坏)** | `--danger` | `#DC2626` | `#2B1214` | 剃刀切割点、删除素材确认、音频爆音过载 (>0dB) |
| **Info (提示/转写)** | `--info` | `#0284C7` | `#0E1F2D` | 后台 ASR 进度指示、系统提示条、素材导入提示 |

### 3.6 专业非编多轨时间轴专属色谱 (Timeline Tracks)

结合非编软件操作直观性与 Neutral Modern 深色规范，轨道片段采用低饱和专业暗彩色底，搭配明晰辨识边框：

| 轨道类型 | 片段填充底色 | 描边高亮色 | 内部波形/数据视觉定义 |
| :--- | :--- | :--- | :--- |
| **视频主轨 (V1/V2/V3)** | `#151E2E` (沉稳青蓝) | `#2B4570` | 视频片段，沉稳低干涉冷调，不干扰监视器画质与调色评估 |
| **音频主轨 (A1/A2/A3)** | `#122119` (暗青墨绿) | `#1E4A32` | 音频片段，内嵌 `#17A34A` 实时双色精细波形 |
| **字幕轨 (C1/Subtitle)** | `#271F12` (暗暖金棕) | `#63471C` | ASR 词级转写字幕块，字形高亮显示，与音视频轨道分明 |
| **动效轨 (HyperFrames)** | `#22142D` (深曜曜紫) | `#522974` | Web 逐帧代码动效图层、片头角标与动态花字 |
| **Agent 建议切除标记** | `#2B1214` (警示暗红) | `#DC2626` | 智能标记的停顿气口、重复口误建议切除区域 |

---

## 4. 字体与排版规范 (Typography)

### 4.1 字体族栈 (Font Family)

- **UI 界面通用 (现代化无衬线体系)**：
  `"Inter", -apple-system, system-ui, "Microsoft YaHei UI", "Source Han Sans CN", sans-serif`
- **时间码、标尺刻度与硬件数值 (精密等宽)**：
  `"JetBrains Mono", ui-monospace, "Cascadia Code", monospace`

### 4.2 文字层级梯度表 (Typography Hierarchy)

| 角色层级 (Role) | 字号 (Size) | 字重 (Weight) | 行高 (Line Height) | 大小写 / 字距 | 应用场景说明 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Display Hero** | 36px | 600 (SemiBold) | 1.20 | -0.01em | 欢迎页大标题、Agent 导演方案总览大标题 |
| **Section Heading** | 24px | 600 (SemiBold) | 1.20 | -0.01em | 面板大标签 (如 "素材池"、"效果控件"、"混音台") |
| **Sub-heading** | 20px | 600 (SemiBold) | 1.25 | 默认 | 导出预设卡片标题、属性折叠组标题 |
| **Body Base** | 16px | 400 (Regular) | 1.50 | 默认 | 参数名称、常规设置说明、对话消息正文 |
| **Body Small** | 14px | 400 (Regular) | 1.50 | 默认 | 列表项标题、素材文件信息、次级正文 |
| **Button Standard**| 14px / 16px | 600 (SemiBold) | 1.20 | 默认 | 标准操作按钮标签、对话框确认/取消 |
| **Nav Dock Item** | 13px | 600 (SemiBold) | 1.20 | UPPERCASE | 达芬奇式底部 6 大 Dock 栏导航项 |
| **Timecode Large** | 20px | 600 (SemiBold) | 1.00 (等宽) | 等宽数字 | 节目监视器大时间码 (`00:01:24:12`) |
| **Timecode Ruler** | 11px | 500 (Medium) | 1.00 (等宽) | 等宽数字 | 时间轴刻度标尺时间标记 |
| **Micro Tag** | 12px | 500 (Medium) | 1.00 | 默认 | 轨道头编号 (`V1`, `A1`)、硬件标签 (`D3D11VA`) |

---

## 5. 核心组件设计规范 (Component Stylings)

### 5.1 按钮交互规范 (Buttons)

#### 主要操作按钮 (Primary CTA)
- **默认外观 (Resting)**：
  - 背景：`#2F6FEB` (Cobalt Blue 填充)
  - 边框：`1px solid #2F6FEB`
  - 圆角：`8px` (`--radius-sm`)
  - 文字颜色：`#FFFFFF` (`--accent-on`)，字重 `600`，字号 `14px`
  - 内边距：`8px 16px` (高度约 34px)
- **悬浮外观 (Hover)**：
  - 背景：`color-mix(in oklab, #2F6FEB, white 10%)`
  - 边框：`color-mix(in oklab, #2F6FEB, white 10%)`
  - 光标：`pointer`
- **按压外观 (Active)**：
  - 背景：`color-mix(in oklab, #2F6FEB, black 10%)`
  - 边框：`color-mix(in oklab, #2F6FEB, black 10%)`

#### 次要操作按钮 (Secondary Button)
- **默认外观**：背景 `#171A21` (`--surface`)，边框 `1px solid #2A2F3A`，圆角 `8px`，文字 `#E2E8F0` (`--fg-2`)
- **悬浮外观**：背景提亮为 `#1E222B`，边框转为 `rgba(255, 255, 255, 0.16)`，文字 `#F8FAFC`
- **按压外观**：背景深为 `#13161C`

#### 禁用状态 (Disabled)
- 背景 `#171A21`，边框 `1px solid #2A2F3A`，文字 `#64748B` (`--meta`)，光标 `not-allowed`，透明度 `0.5`。

---

### 5.2 达芬奇式底部 Dock 栏 (Bottom Workflow Dock)

- **位置与尺寸**：置于应用窗口最底部，固定高度 `48px`，背景为次级表面色 `#1E222B` (`--surface-warm`)。
- **分隔线**：顶部以 `1px solid #2A2F3A` 细线划界。
- **6 大导航分页项** (`AGENT` / `EDIT` / `MOTION` / `AUDIO` / `IMAGE` / `DELIVER`)：
  - 默认状态：文字颜色 `#A7ADBA` (`--muted`)，无背景，`8px` 内圆角。
  - Hover 状态：背景为 `#232733`，文字转亮 (`#F8FAFC`)。
  - 激活状态 (Active)：
    - 顶部呈现 `2px solid #2F6FEB`（Cobalt Blue 信号线）。
    - 文字高亮为 `#2F6FEB`，图标带微弱高亮反馈。

---

### 5.3 全局公用时间线组件 (Global Shared Timeline)

- **结构与位置**：占据视口下方约 38%~42% 空间，跨 6 大页面常驻保活。
- **底板与轨道槽**：
  - 时间线工作台底板：`#171A21` (`--surface`)。
  - 轨道槽内部背景：`#0F1115` (`--bg` 深色下沉槽)。
  - 轨道横向分隔线：`1px solid #2A2F3A`。
- **片段 (Clips)**：
  - 圆角统一为 `4px` (`--radius-xs`)，拼接精密无杂缝。
  - 选中片段外框：`1.5px solid #2F6FEB`。
- **播放指针 (Playhead)**：
  - 游标头：反向五边形纯正 `#2F6FEB` 实体。
  - 纵向贯穿线：`1.5px solid #2F6FEB`，60FPS 丝滑移动。
- **磁吸对齐指示线 (Snapping Line)**：
  - 磁吸瞬间以 `1px dashed #2F6FEB` 纵向贯穿时间轴标尺与轨道。

---

### 5.4 面板卡片与容器规范 (Cards & Panels)

- **面板表面**：`#171A21` (`--surface`)。
- **面板外边框**：`1px solid #2A2F3A` (`--border`)。
- **容器圆角**：`12px` (`--radius-md`)。
- **Header 标题条**：高度 `36px`，底部带有 `1px solid #2A2F3A` 细分割线，标题为 `14px SemiBold` (`#F8FAFC`)。
- **悬浮与层级 (Elevation)**：
  - 平面 (Flat)：默认面板平整无阴影，依靠色阶差区分。
  - 浮层 (Raised)：右键上下文菜单、浮动对话框、下拉菜单，应用 `box-shadow: 0 4px 20px rgba(0, 0, 0, 0.45)`。

---

## 6. Rust + egui 0.36 / wgpu 30.0 代码级映射表

在 Rust 端利用 `egui 0.36` 实现 Neutral Modern 深色规范时，直接映射为以下主题与样式结构体：

```rust
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
```

---

## 7. 与各工作流文档的联动对齐标准

- **剪辑工作台布局 ([`edit-layout-spec.md`](file:///d:/Work/Dev/ClipFlow/docs/edit-layout-spec.md))**：
  - PR 四区分屏外框与主容器采用 `12px` 圆角，内部操作按钮为 `8px`。
  - 面板之间以 `1px solid #2A2F3A` 划分，工作区底盘为 `#0F1115`，卡片表面为 `#171A21`。
  - 时间线播放指针与吸附参考线统一采用 Cobalt Blue (`#2F6FEB`)。
- **辅助工作流页面 ([`workflow-pages-spec.md`](file:///d:/Work/Dev/ClipFlow/docs/workflow-pages-spec.md))**：
  - 【声音】混音台推子与电平表色谱：$-60\text{dB} \sim -12\text{dB}$ 为安全绿 (`#17A34A`)，$-12\text{dB} \sim 0\text{dB}$ 为警告黄 (`#EAB308`)，$\ge 0\text{dB}$ 为过载红 (`#DC2626`)。
  - 【图片】封面画布与选中外框统一遵循 Cobalt Blue (`#2F6FEB`) 信号指示。
  - 【导出】主渲染 CTA 按钮采用标准钴蓝填充 (`#2F6FEB`) 搭配纯白文字 (`#FFFFFF`)。
