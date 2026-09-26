# ClipFlow UI 设计规范与视觉体系 (UI Specification)

> **设计基准**：严格遵循 **NVIDIA 官方深色设计体系 (NVIDIA Dark Mode Design System)** 与高算力硬核科技美学（纯黑底蕴 `#000000`、次级表面 `#1A1A1A`、信号级电能绿 `#76B900`、工控级 2px 精密锐角、Bold 700 权威字重），融合专业音视频剪辑 (NLE) 暗房色彩还原标准，提供高对比度、低视觉疲劳与硬件级操作质感。

---

## 1. 核心设计哲学 (Visual Theme & Atmosphere)

NVIDIA 深色模式的核心在于通过**极度克制的设计语言传达极致的计算能量与精密工控感**：

- **信号而非表面 (Signal-Not-Surface)**：
  标志性 NVIDIA Green (`#76B900`) **绝不作为大面积填充背景**，而作为且仅作为系统的“第一交互信号源”——专用于边框高亮 (`2px solid #76B900`)、播放指针 (Playhead)、激活轨道描边、主操作按钮线框、高亮关键帧与状态指示器。
- **纯粹黑暗房底蕴 (True Black Foundation)**：
  界面以纯黑 (`#000000`) 作为应用最底板，面板容器与卡片采用近黑 (`#1A1A1A`)，搭配中性灰细边框 (`#5E5E5E`)。避免低劣的泛蓝灰色调，确保监视器内的视频色彩还原达到广播级纯净度。
- **工控级精密锐角 (2px Sharp Engineered Corners)**：
  摒弃消费级软件圆润松散的弧度，全系统控件、面板与时间轴切片统一收敛为 **2px 极简工控圆角**（微小标签为 **1px**）。如同高精度数控机床切割的硬件表面，严谨、紧凑、冷峻。
- **材质差异驱动的纵深层次 (Contrast-Driven Depth)**：
  杜绝花哨的毛玻璃 (Glassmorphism) 与强漫反射阴影，层次完全由**材质与色阶反差**（纯黑 `#000000`、近黑 `#1A1A1A`、微弱环境光 `rgba(0,0,0,0.3) 0px 0px 5px`）自然拉开，保持界面的极速响应与硬核专业感。
- **权威默认字重 (Bold 700 as Dominant Voice)**：
  除大段正文与描述采用常规字重 (400) 外，所有标题、按钮、标签、时间码与导航项默认采用 **Bold (700)**，结合紧凑行高 (1.25) 与大写导航文本，塑造工业硬件铭牌般的自信与严谨。

---

## 2. 几何曲率规范 (Border Radius Scale)

系统全量控件严格遵照 NVIDIA 工业硬件设计规范的倒角梯度：

```
+---------------------------------------------------------------------------------+
| 桌面窗口宿主外框 (Host Window Frame): 8-12px (由 Windows 11 OS 顶层外壳接管)      |
|  +---------------------------------------------------------------------------+  |
|  | 内部面板 / 视窗容器 (Panel Level): 2px                                       |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  | 按钮 / 控件 / 卡片 (Control): 2px  |  | 时间轴片段 (Clip): 2px        |  |  |
|  |  | [ 主操作 CTA ]                    |  | [===== 视频 V1 =====]        |  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  |  | 微型标签 / 徽章 (Micro Tag): 1px    |  | 圆形指示器 (Circle): 50%       |  |  |
|  |  +----------------------------------+  +-------------------------------+  |  |
|  +---------------------------------------------------------------------------+  |
+---------------------------------------------------------------------------------+
```

| 级别 | 曲率半径 (Radius) | 对应设计定义 | 软件内具体应用场景 |
| :--- | :--- | :--- | :--- |
| **标准容器与面板 (Standard Panel)** | **2px** | NVIDIA 硬件外壳倒角基准 | PR 四区分屏面板外框、模态对话框、参数调节卡片、监视器外壳 |
| **基础交互控件 (Standard Control)** | **2px** | 统一工控精密圆角 | 按钮 (Button)、输入框 (Input)、下拉菜单、滑块滑槽、数值微调框 |
| **时间轴片段 (Timeline Clip)** | **2px** | 密集数据流精密拼接 | 多轨时间轴上的视频片段、音频片段、字幕块、动效块（紧凑拼合不露缝） |
| **微型标签 (Micro Element)** | **1px** | 极小尺寸结构倒角 | 快捷键提示徽章、轨道头微型编号（`V1`, `A1`）、代码行号块 |
| **圆形指示器 (Circle Indicator)** | **50%** | 点状状态信号 | Agent 在线呼吸指示灯、录音红点、调色色轮中心指针 |

---

## 3. 色彩体系与 Tokens (Color Palette & Semantic Roles)

### 3.1 核心品牌与信号色 (Primary Brand & Signal)

| 语义名称 | 色值 (HEX) | RGB 值 | 界面功能与定义 |
| :--- | :--- | :--- | :--- |
| **NVIDIA Green (主信号色)** | `#76B900` | `rgb(118, 185, 0)` | **最高优先级交互色**：播放指针、主 CTA 边框、选中高亮、激活轨道外框 |
| **NVIDIA Green Light (亮绿高亮)** | `#BFF230` | `rgb(191, 242, 48)` | 悬浮高亮 (Hover)、关键帧节点选中高亮、音频波形峰值瞬变 |
| **True Black (纯黑画布)** | `#000000` | `rgb(0, 0, 0)` | 全局主视口底板、监视器外部暗房、时间轴轨道槽背景 |
| **Near Black (表面深灰)** | `#1A1A1A` | `rgb(26, 26, 26)` | 面板卡片底色、列表项背景、浮动弹窗与对话框背景 |
| **Pure White (纯白文字)** | `#FFFFFF` | `rgb(255, 255, 255)` | 深色背景上的主要文字、主标题、激活时间码、选中图标 |

### 3.2 交互状态与动态色彩转移 (Interactive States)

NVIDIA 规范中独特的交互色彩转移法则（非单纯调亮透明度，而是向专属色相偏移）：

| 交互状态 | 映射色值 (HEX) | RGB 值 | 行为定义与应用组件 |
| :--- | :--- | :--- | :--- |
| **Button Hover (按钮悬浮)** | `#1EAEDB` (NVIDIA Teal) | `rgb(30, 174, 219)` | 鼠标悬浮于主要按钮时，背景由透明切换为填充 Teal，文字保持反白 |
| **Button Active (按钮按下)** | `#007FFF` (Electric Blue) | `rgb(0, 127, 255)` | 按钮被鼠标按下激活，背景切换为亮蓝，边框切换为 `1px solid #003EFF` |
| **Link Hover (链接悬浮)** | `#3860BE` (Interactive Blue)| `rgb(56, 96, 190)` | 所有可点击文本链接从白色或浅灰平滑切换为交互蓝 |
| **Focus Ring (键盘聚焦)** | `#76B900` / `#000000` | - | 控件聚焦外框，暗色模式下以 `2px solid #76B900` 发光线呈现 |

### 3.3 中性灰阶阶梯 (Neutral Scale)

| 阶梯层级 | 色值 (HEX) | 用途与映射面板 |
| :--- | :--- | :--- |
| **Canvas Void (画布底层)** | `#000000` | 窗口总底板、Dock 栏底色、视频监视器外部深色沉浸暗房 |
| **Panel Surface (面板背景)** | `#1A1A1A` | 剪辑四大面板（项目池、源监视器、节目监视器、时间轴）内部卡片 |
| **Gray Border (分割与边框)** | `#5E5E5E` | 面板间的精密分割线 (`1px solid #5E5E5E`)、次级容器边框 |
| **Gray 500 (微弱提示/页脚)** | `#757575` | 占位符文字 (Placeholder)、禁用控件文字、底部版权信息 |
| **Gray 400 (次要元数据)** | `#898989` | 标尺刻度辅助线、媒体元数据（帧率、编码格式、采样率） |
| **Gray 300 (次级正文)** | `#A7A7A7` | 面板副标题、未选中状态标签、功能说明描述文本 |
| **Pure White (主阅读文字)** | `#FFFFFF` | 关键标题、时间码、监视器参数、当前选中文案 |

### 3.4 状态与语义色谱 (Status & Semantic)

| 语义类型 | 标志色 (HEX) | 暗色搭配 (HEX) | 视觉功能说明 |
| :--- | :--- | :--- | :--- |
| **Error / Destructive (错误/危险)** | `#E52020` (Red 500) | `#650B0B` (Red 800) | 渲染失败、素材丢失、文件删除警告、重读停顿建议切除条 |
| **Warning / Energy (警告/高能量)** | `#DF6500` (Orange 400)| `#EF9100` (Yellow 300) | 音频电平接近过载 (>-6dB)、硬件占用警告、时间轴入出点标记 |
| **Success / Positive (成功/正常)** | `#3F8500` (Green 500) | `#152B00` (Deep Forest) | 渲染导出完成、硬解管线正常就绪、音频标准电平安全区 |
| **Informational (信息指示)** | `#0046A4` (Blue 700) | `#0B1B33` | 软件更新提示、后台 ASR 转写进度、系统状态广播 |
| **AI / Premium (Agent/动效)** | `#4D1368` (Purple 800)| `#8C1C55` (Fuchsia 700)| 导演 Agent 思考规划高亮、HyperFrames 动态图层专属品牌色 |

### 3.5 非编多轨时间轴专属色谱 (Timeline Tracks)

结合专业剪辑直观性与 NVIDIA 深色工程标准：

| 轨道类型 | 片段填充底色 | 描边高亮色 | 内部波形/数据视觉定义 |
| :--- | :--- | :--- | :--- |
| **视频主轨 (V1/V2/V3)** | `#1B2A38` (深岩蓝) | `#3860BE` | 视频片段，沉稳冷调，严防界面原色干扰画质监看 |
| **音频主轨 (A1/A2/A3)** | `#152B1E` (暗松绿) | `#3F8500` | 音频片段，内嵌 `#76B900`（正电平）动态双色精细波形 |
| **字幕轨 (Subtitle)** | `#352810` (暗金棕) | `#EF9100` | ASR 词级转写字幕块，高辨识度金黄边框指示 |
| **HyperFrames 动效轨** | `#28103A` (深曜紫) | `#9B51E0` | 代码逐帧渲染动效层、片头角标与动态图表图层 |
| **Agent 建议切除标记** | `#3D0F0F` (警示暗红)| `#E52020` | 智能标记的停顿无声、重复啰嗦与错词建议切除切片 |

### 3.6 阴影与微光系统 (Shadows & Elevation)

- **环境卡片阴影 (Ambient Shadow)**：`rgba(0, 0, 0, 0.3) 0px 0px 5px 0px`。用于浮动卡片、菜单和模态框，极其微弱克制。
- **无模糊光效原则**：拒绝大面积的 CSS/GPU Gaussian Blur，深度由纯色边框与灰度梯度构建；仅当 Agent 处于活动规划状态时，允许在输入框周围产生轻量绿色呼吸脉冲边框 (`box-shadow: 0 0 6px rgba(118, 185, 0, 0.4)`)。

---

## 4. 字体与排版规范 (Typography)

### 4.1 字体栈族 (Font Family)

- **UI 界面通用 (欧洲工业工程质感)**：
  `"NVIDIA-EMEA", "Segoe UI Variable", "Segoe UI", -apple-system, "PingFang SC", "Microsoft YaHei", sans-serif`
- **时间码与数值参数 (精密等宽)**：
  `"Cascadia Code", "Consolas", "Segoe UI Mono", monospace`
- **HyperFrames 动效代码编辑区**：
  `"Cascadia Code", "JetBrains Mono", monospace`

### 4.2 文字层级梯度表 (Typography Hierarchy)

| 角色层级 (Role) | 字号 (Size) | 字重 (Weight) | 行高 (Line Height) | 大小写规范 | 应用场景说明 |
| :--- | :--- | :--- | :--- | :--- | :--- |
| **Display Hero** | 36px (2.25rem) | **700 (Bold)** | 1.25 (紧凑) | 默认 | 欢迎引导大标题、Agent 导演核心总方案命名 |
| **Section Heading** | 24px (1.50rem) | **700 (Bold)** | 1.25 (紧凑) | 默认 | 面板大标签 (如 "项目池"、"效果控件"、"混音台") |
| **Sub-heading** | 20px (1.25rem) | **700 (Bold)** | 1.25 (紧凑) | 默认 | 导出预设卡片标题、Agent 规划子步骤标题 |
| **Body Bold** | 16px (1.00rem) | **700 (Bold)** | 1.50 | 默认 | 关键状态指示、主操作参数项、选中文本 |
| **Body Regular** | 16px (1.00rem) | 400 (Regular) | 1.50 | 默认 | 视频长描述、AI 对话详细说明正文、使用指南 |
| **Button Large** | 18px (1.13rem) | **700 (Bold)** | 1.25 (紧凑) | 默认 | 首页或全局重要操作主 CTA 按钮 |
| **Button Standard**| 16px (1.00rem) | **700 (Bold)** | 1.25 (紧凑) | 默认 | 标准工具栏按钮、模态框确认/取消按钮 |
| **Button Compact** | 14.4px (0.9rem) | **700 (Bold)** | 1.00 (等宽) | 字母间距 0.14px | 轨道头小按钮 (Mute/Solo/Lock)、时间码微调按钮 |
| **Nav Link Upper** | 14px (0.88rem) | **700 (Bold)** | 1.43 | **UPPERCASE** | 达芬奇式底部 Dock 栏导航项、硬件参数标牌 |
| **Timecode Large** | 18px (1.13rem) | **700 (Bold)** | 1.00 (等宽) | 等宽数字 | 节目监视器大时间码显示 (`00:01:24:12`) |
| **Timecode Ruler** | 10px (0.63rem) | 500 (Medium) | 1.00 (等宽) | 等宽数字 | 时间轴标尺刻度时间文字 |
| **Micro Label** | 11px (0.69rem) | **700 (Bold)** | 1.00 (紧凑) | **UPPERCASE** | 硬件级状态徽章 (如 "GPU-INT8", "CUDA-D3D11VA") |

---

## 5. 核心组件设计规范 (Component Stylings)

### 5.1 按钮交互规范 (Buttons)

#### 主要操作按钮 (Primary CTA)
- **默认外观 (Resting)**：
  - 背景：`transparent` (全透明)
  - 边框：`2px solid #76B900` (NVIDIA Green 纯正信号边框)
  - 圆角：`2px` (工业级锐角)
  - 文字颜色：`#FFFFFF`，字重 `700 (Bold)`，字号 `16px`
  - 内边距：`11px 13px`
- **悬浮外观 (Hover)**：
  - 背景：填充 `#1EAEDB` (NVIDIA Teal)
  - 边框：`2px solid #1EAEDB`
  - 文字颜色：`#FFFFFF` (清晰反白)
- **按压外观 (Active / Pressed)**：
  - 背景：填充 `#007FFF` (Electric Blue)
  - 边框：`1px solid #003EFF`
  - 文字颜色：`#FFFFFF`

#### 次要操作按钮 (Secondary Button)
- **默认外观**：背景 `transparent`，边框 `1px solid #76B900`，圆角 `2px`，文字 `#A7A7A7`
- **悬浮外观**：边框颜色变亮为 `#BFF230`，文字变白 (`#FFFFFF`)

#### 禁用状态 (Disabled)
- 边框 `1px solid #5E5E5E`，文字 `#757575`，背景透明，光标呈现 `not-allowed`。

---

### 5.2 达芬奇式底部 Dock 栏 (Bottom Workflow Dock)

- **位置与尺寸**：置于应用窗口最底部，高度固定为 `48px`，背景为纯黑底板 (`#000000`)。
- **边框与分隔**：顶部设置 `1px solid #5E5E5E` 分割细线，面板与 Dock 栏边界严密分明。
- **6 大导航分页项** (`AGENT` / `EDIT` / `MOTION` / `AUDIO` / `IMAGE` / `DELIVER`)：
  - 排版模式：文字强制采用英汉双显或全大写英文硬件标签风格（字号 `14px`，字重 `700`）。
  - 默认状态：文字颜色 `#898989`，无背景，2px 边缘微倒角。
  - Hover 状态：背景微亮为 `#1A1A1A`，文字转白 (`#FFFFFF`)。
  - 激活状态 (Active)：
    - 顶部呈现 `2px solid #76B900`（NVIDIA Green 标志性信号线）。
    - 文字高亮为 `#76B900`，图标带微弱高亮反馈。

---

### 5.3 全局公用时间线组件 (Global Shared Timeline)

- **结构与位置**：紧贴底部 Dock 栏上方，占据下部约 35%~42% 视口高度，跨 6 大页面常驻保活。
- **底板与轨道槽**：
  - 时间线底板：`#1A1A1A`。
  - 轨道槽内部背景：`#000000` (纯黑下沉槽)。
  - 轨道分隔线：`1px solid #5E5E5E`。
- **片段 (Clips)**：
  - 圆角一律为 `2px`，四周紧贴无空隙拼接。
  - 选中片段外框：`1.5px solid #76B900`。
- **播放指针 (Playhead)**：
  - 游标头：反向五边形纯正 `#76B900` 实体。
  - 贯穿多轨纵向线：`1.5px solid #76B900`，播放时以 60FPS 垂直同步平滑移动。
- **磁吸对齐指示线 (Snapping Line)**：
  - 当边缘发生磁吸时，以 `1px dashed #BFF230`（亮绿）纵向贯穿标尺，毫秒级即时对齐反馈。

---

### 5.4 面板卡片与容器规范 (Cards & Containers)

- **面板背景 (Panel Surface)**：`#1A1A1A`。
- **面板外框 (Border)**：`1px solid #5E5E5E`。
- **曲率半径 (Radius)**：`2px`。
- **Header 标题条**：高度 `32px`，底边为 `1px solid #5E5E5E`，标题字号 `14px Bold`，纯白反白。
- **阴影**：浮动悬浮卡片使用克制的环境阴影 `rgba(0, 0, 0, 0.3) 0px 0px 5px 0px`。

---

## 6. Rust + egui 0.36 / wgpu 30.0 代码级映射表

在 Rust 端利用 `egui 0.36` 实现本套 NVIDIA 深色模式规范时，直接映射为以下主题与样式结构体：

```rust
use egui::{Color32, CornerRadius, Stroke, Visuals};

/// ClipFlow NVIDIA 官方深色模式设计规范主题
pub struct ClipFlowTheme;

impl ClipFlowTheme {
    // ------------------------------------------------------------------------
    // 1. 核心品牌与信号色 (Brand & Signal)
    // ------------------------------------------------------------------------
    pub const NVIDIA_GREEN: Color32 = Color32::from_rgb(118, 185, 0);     // #76B900 (主信号)
    pub const ELECTRIC_LIME: Color32 = Color32::from_rgb(191, 242, 48);   // #BFF230 (悬浮与高光)
    pub const TRUE_BLACK: Color32 = Color32::from_rgb(0, 0, 0);           // #000000 (画布底板)
    pub const NEAR_BLACK: Color32 = Color32::from_rgb(26, 26, 26);        // #1A1A1A (卡片/面板)
    pub const PURE_WHITE: Color32 = Color32::from_rgb(255, 255, 255);     // #FFFFFF (纯白字)

    // ------------------------------------------------------------------------
    // 2. 交互状态转移色 (Interactive Transitions)
    // ------------------------------------------------------------------------
    pub const BUTTON_HOVER_TEAL: Color32 = Color32::from_rgb(30, 174, 219);  // #1EAEDB (Hover)
    pub const BUTTON_ACTIVE_BLUE: Color32 = Color32::from_rgb(0, 127, 255);  // #007FFF (Active)
    pub const LINK_HOVER_BLUE: Color32 = Color32::from_rgb(56, 96, 190);     // #3860BE (Link)

    // ------------------------------------------------------------------------
    // 3. 中性灰阶阶梯 (Neutral Grays)
    // ------------------------------------------------------------------------
    pub const GRAY_BORDER: Color32 = Color32::from_rgb(94, 94, 94);       // #5E5E5E (分割细线)
    pub const GRAY_500: Color32 = Color32::from_rgb(117, 117, 117);       // #757575 (占位符/禁用)
    pub const GRAY_400: Color32 = Color32::from_rgb(137, 137, 137);       // #898989 (次级标尺)
    pub const GRAY_300: Color32 = Color32::from_rgb(167, 167, 167);       // #A7A7A7 (次级正文)

    // ------------------------------------------------------------------------
    // 4. 状态与非编轨道色谱 (Status & Tracks)
    // ------------------------------------------------------------------------
    pub const STATUS_RED: Color32 = Color32::from_rgb(229, 32, 32);       // #E52020 (错误/切除标记)
    pub const STATUS_RED_BG: Color32 = Color32::from_rgb(61, 15, 15);      // #3D0F0F (切除底色)
    pub const STATUS_ORANGE: Color32 = Color32::from_rgb(223, 101, 0);     // #DF6500 (预警)
    pub const STATUS_GREEN_500: Color32 = Color32::from_rgb(63, 133, 0);   // #3F8500 (成功/音频边框)

    pub const TRACK_VIDEO_BG: Color32 = Color32::from_rgb(27, 42, 56);     // #1B2A38 (视频轨深蓝)
    pub const TRACK_VIDEO_STROKE: Color32 = Color32::from_rgb(56, 96, 190);// #3860BE (视频描边)
    pub const TRACK_AUDIO_BG: Color32 = Color32::from_rgb(21, 43, 30);     // #152B1E (音频轨暗绿)
    pub const TRACK_SUBTITLE_BG: Color32 = Color32::from_rgb(53, 40, 16);  // #352810 (字幕暗金)
    pub const TRACK_HYPERFRAMES_BG: Color32 = Color32::from_rgb(40, 16, 58); // #28103A (动效深紫)

    // ------------------------------------------------------------------------
    // 5. NVIDIA 工业精密圆角曲率 (CornerRadius: 2px Standard, 1px Micro)
    // ------------------------------------------------------------------------
    pub const RADIUS_PANEL: CornerRadius = CornerRadius::same(2);   // 容器/面板 (2px)
    pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(2); // 按钮/输入框 (2px)
    pub const RADIUS_CLIP: CornerRadius = CornerRadius::same(2);    // 时间轴片段 (2px)
    pub const RADIUS_MICRO: CornerRadius = CornerRadius::same(1);   // 微型标签徽章 (1px)

    /// 一键配置全局 egui::Visuals 样式，严格对齐 NVIDIA 深色规范
    pub fn apply_to(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();

        // 画布与面板底色
        visuals.panel_fill = Self::TRUE_BLACK;
        visuals.window_fill = Self::NEAR_BLACK;
        visuals.window_stroke = Stroke::new(1.0, Self::GRAY_BORDER);
        visuals.window_corner_radius = Self::RADIUS_PANEL;

        // 默认控件外观 (2px 工业工控锐角，默认透明/深黑)
        visuals.widgets.inactive.bg_fill = Self::NEAR_BLACK;
        visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, Self::GRAY_BORDER);
        visuals.widgets.inactive.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, Self::PURE_WHITE);

        // 悬浮状态：向 NVIDIA Teal 转移
        visuals.widgets.hovered.bg_fill = Self::BUTTON_HOVER_TEAL;
        visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, Self::BUTTON_HOVER_TEAL);
        visuals.widgets.hovered.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, Self::PURE_WHITE);

        // 激活状态：向 Electric Blue 转移
        visuals.widgets.active.bg_fill = Self::BUTTON_ACTIVE_BLUE;
        visuals.widgets.active.bg_stroke = Stroke::new(1.0, Color32::from_rgb(0, 62, 255));
        visuals.widgets.active.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.active.fg_stroke = Stroke::new(1.0, Self::PURE_WHITE);

        // 选择与高亮状态：注入 NVIDIA 绿色信号
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(118, 185, 0, 40);
        visuals.selection.stroke = Stroke::new(1.5, Self::NVIDIA_GREEN);

        ctx.set_visuals(visuals);
    }
}
```

---

## 7. 与各工作流文档的联动对齐标准

- **剪辑工作台布局 ([`edit-layout-spec.md`](file:///d:/Work/Dev/ClipFlow/docs/edit-layout-spec.md))**：
  - PR 四区分屏外框与工具栏按钮采用统一的 `2px` 工业倒角。
  - 面板之间以 `1px solid #5E5E5E` 划分，内部底色为 `#1A1A1A`。
- **辅助工作流页面 ([`workflow-pages-spec.md`](file:///d:/Work/Dev/ClipFlow/docs/workflow-pages-spec.md))**：
  - 【声音】混音台推子与电平表色谱：$-60\text{dB} \sim -12\text{dB}$ 为电能绿 (`#76B900`)，$-12\text{dB} \sim -2\text{dB}$ 为警告橙黄 (`#DF6500`)，$\ge -2\text{dB}$ 为过载红 (`#E52020`)。
  - 【图片】封面画布与安全框指示线统一遵循 NVIDIA 亮绿高亮 (`#BFF230`)。
  - 【导出】主渲染 CTA 按钮采用 NVIDIA 纯正绿色边框 (`2px solid #76B900`)。
