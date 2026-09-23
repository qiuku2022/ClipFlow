# ClipFlow UI 设计规范与视觉体系 (UI Specification)

> **设计基准**：继承 **NVIDIA 高算力硬核科技美学**（纯黑底蕴、信号级电能绿 `#76B900`、严谨工控感），融合 **Windows 11 现代原生桌面几何**（深度遵循 Win 11 窗口与容器默认圆角曲率），并针对 **专业音视频剪辑 (NLE)** 进行深色抗疲劳与色彩还原优化。

---

## 1. 核心设计哲学

- **电能绿信号语言 (Signal-Not-Surface)**：
  继承 NVIDIA 规范原则，标志性电能绿 (`#76B900`) **绝不作为大面积背景色**，而作为且仅作为系统“第一信号源”——用于 Agent 导演执行状态、播放指针 (Playhead)、激活轨道高亮、主 CTA 边框与关键帧高亮。
- **Win 11 原生圆角曲率几何 (Fluent Geometry)**：
  摒弃传统工控软件生硬的直角（1-2px），全系统严格对齐 **Windows 11 默认窗口与控件圆角曲率规范**，赋予专业剪辑软件现代、高级的呼吸感与质感。
- **专业暗色工作室视差 (Dark Studio Contrast)**：
  将纯白反差优化为针对视频工作流的灰阶阶梯（`#0D0D0D` 到 `#282828`），避免纯黑白极端反差导致的视网膜疲劳，保证视频监视器中画面的绝对色彩准确度。

---

## 2. 圆角与几何曲率规范 (Border Radius)

全软件各级矩形元素的圆角曲率严格遵照 Windows 11 设计指南：

```
+---------------------------------------------------------------------------------+
| 主窗口外框 (Window Level): 12px                                                 |
|  +---------------------------------------------------------------------------+  |
|  | 浮动面板 / 卡片容器 (Panel Level): 8px                                      |  |
|  |  +-----------------------+  +------------------------------------------+  |  |
|  |  | 按钮/控件: 4px         |  | 状态胶囊: Pill (Full)                    |  |  |
|  |  +-----------------------+  +------------------------------------------+  |  |
|  +---------------------------------------------------------------------------+  |
+---------------------------------------------------------------------------------+
```

| 级别 | 曲率半径 (Radius) | 对应 Windows 11 规范 | 软件内具体应用场景 |
| :--- | :--- | :--- | :--- |
| **主窗口 (Window)** | **12px** | Windows 11 Top-level Window 默认圆角 | 桌面无边框宿主主窗口、独立弹出式主工作视窗 |
| **容器/面板 (Panel)** | **8px** | Win 11 Overlay / Flyout / Card 默认曲率 | PR 四区分屏面板外框、模态对话框、设置弹窗、监视器外壳 |
| **基础控件 (Control)** | **4px** | Win 11 Common Controls 标准圆角 | 按钮 (Button)、输入框 (Input)、下拉菜单、滑块槽、数值调节框 |
| **时间轴片段 (Clip)** | **3px** | 密集数据流精密曲率 | 多轨时间轴上的视频片段、音频片段、字幕块（紧凑拼合不漏缝） |
| **状态胶囊 (Pill)** | **9999px (全圆角)** | Win 11 Tag / Badge 标准 | Agent 状态标签（“思考中”、“剪辑中”）、时间码浮动 Badge |

---

## 3. 色彩体系 (Color Palette & Tokens)

### 3.1 核心品牌与信号色 (Brand & Signal)

| 语义名称 | 色值 (HEX) | 对应 RGB | 界面功能与定义 |
| :--- | :--- | :--- | :--- |
| **NVIDIA Green (主信号色)** | `#76B900` | `rgb(118, 185, 0)` | **最高优先级交互色**：播放指针、Agent 执行指示、当前激活轨道外框、选中高亮 |
| **Electric Lime (浅亮绿)** | `#BFF230` | `rgb(191, 242, 48)` | 悬浮高亮 (Hover)、关键帧节点选中、波形高能量提示 |
| **Deep Forest (暗绿背景)** | `#152B00` | `rgb(21, 43, 0)` | 仅用于主信号激活区域的轻微呼吸发光背景（< 10% 面积） |

### 3.2 桌面剪辑深色阶梯 (Surfaces & Neutral Grays)

| 阶梯层级 | 色值 (HEX) | 用途与映射面板 |
| :--- | :--- | :--- |
| **Canvas Void (应用最底板)** | `#0D0D0D` | 窗口背景、达芬奇 Dock 栏背景底色、监视器外部暗房区 |
| **Panel Surface (面板主背景)** | `#141414` | PR 剪辑区四大面板（项目池、源监视器、节目监视器、时间轴）背景 |
| **Surface Card (卡片与输入容器)**| `#1C1C1C` | 时间轴轨道槽、参数调节卡片、Agent 规划方案列表项 |
| **Surface Hover (悬浮底色)** | `#262626` | 列表项 Hover、按钮按下背景 |
| **Divider & Border (细分割线)** | `#333333` | 面板之间的分割边框（1px 细线）、时间轴刻度分界 |
| **Text Primary (主要正文)** | `#FFFFFF` | 关键标题、时间码、监视器参数、当前选中文案 |
| **Text Secondary (次要信息)** | `#A7A7A7` | 面板标签、时间轴标尺刻度、素材元数据 |
| **Text Muted (弱化文字)** | `#666666` | 占位符、快捷键提示、禁用状态 |

### 3.3 非编时间轴轨道专属色谱 (Timeline Tracks)

| 轨道类型 | 片段填充底色 | 描边高亮色 | 视觉定义 |
| :--- | :--- | :--- | :--- |
| **视频主轨 (V1/V2)** | `#1B334B` (深岩青) | `#3880BE` | 视频片段，冷调内敛，避免干扰监视器色彩 |
| **音频主轨 (A1/A2)** | `#17382B` (暗松绿) | `#439E6D` | 音频片段，内嵌浅绿波形数据 |
| **字幕轨 (Subtitle)** | `#3D3418` (暗金棕) | `#D4A017` | ASR 自动转写生成的字幕块 |
| **HyperFrames 动效轨** | `#321845` (深幽紫) | `#9B51E0` | 代码渲染的动态包装、片头角标与图表层 |
| **口播待剪除标记 (Cut Mark)** | `#4A1515` (警示暗红)| `#E52020` | Agent 标记的停顿、重读与语气词建议切除段 |

---

## 4. 字体与排版规范 (Typography)

### 4.1 字体栈族 (Font Family)
- **UI 界面通用**：`"Segoe UI Variable", "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif`
- **时间码与数值**（等宽）：`"Consolas", "Cascadia Code", "Segoe UI Mono", monospace`
- **HyperFrames 动效代码编辑区**：`"Cascadia Code", "JetBrains Mono", monospace`

### 4.2 文字层级梯度 (Hierarchy)

| 层级 | 字号 (Size) | 字重 (Weight) | 行高 (Line Height) | 场景说明 |
| :--- | :--- | :--- | :--- | :--- |
| **Display Heading** | 24px | Bold (700) | 1.25 | Agent 页面方案总标题、导出大标题 |
| **Section Heading** | 16px | Bold (700) | 1.25 | 面板标签名 (如 "项目素材"、"效果控件") |
| **UI Normal** | 13px | Regular (400) | 1.40 | 通用列表项、轨道名、菜单选项 |
| **Timecode Large** | 18px | Bold (700) | 1.00 (等宽) | 节目监视器大时间码显示 (`00:01:24:12`) |
| **Timecode Ruler** | 10px | Medium (500) | 1.00 (等宽) | 时间轴标尺刻度文字 |
| **Badge / Label** | 10px | Bold (700) | 1.20 | 状态徽章（如 "GPU-INT8"、"CUDA"） |

---

## 5. 核心界面组件设计规范

### 5.1 达芬奇式底部 Dock 栏 (Bottom Workflow Dock)
- **位置**：应用底部绝对居中或平铺，高度固定为 `48px`，背景为纯黑透光 (`#0D0D0D`)。
- **Dock 栏外框**：内嵌式细分割线 `1px solid #262626`。
- **6 大分页项**（`Agent` / `剪辑` / `动画` / `声音` / `图片` / `导出`）：
  - 默认状态：文字颜色 `#898989`，图标居中，无背景。
  - Hover 状态：背景 `#1C1C1C`，圆角 `4px`，文字变白 (`#FFFFFF`)。
  - 激活状态（Active）：
    - 顶部或底部呈现 `2px solid #76B900`（NVIDIA Green 标志性信号线）。
    - 文字高亮为 `#76B900`，图标带微弱绿色辉光。

### 5.2 全局公用时间线组件 (Global Shared Timeline)
- **常驻布局**：位于底部 Dock 栏（48px）正上方，默认占据下部约 35%~42% 视口高度，中间以 `2px solid #262626` 带有 `8px` 圆角感的可拖拽分割条 (Splitter) 与上方各页面专属视窗隔离。
- **跨页面生命周期**：在用户切换 Dock 栏时，时间线组件保持常驻运行，不触发重新卸载与挂载，保证播放指针位置（Playhead）、播放状态、音画硬解缓存与多轨缩放比例无感连续。
- **页面感知交互特性 (Context-Aware UI)**：
  - **在 `Agent` 页**：时间线上以带有绿色脉冲描边（`#76B900`）实时绘制 Agent 规划生成的镜头切片，以暗红色半透明覆盖条（`#4A1515`）标示口播气口和重读剪除建议。
  - **在 `剪辑` 页**：完全展开对齐 PR 的工具箱（选择/剃刀/波纹剪辑/滑移/吸附）及轨道头控制器（静音 M、独奏 S、锁轨、时间码显示）。
  - **在 `动画` 页**：自动高亮显示挂载 HyperFrames 渲染产物的动效轨道，片段上方浮动显示 HTML/CSS 组件名与渲染状态 Badge。
  - **在 `声音` 页**：音频轨道（A1/A2/A3）垂直撑开高度，渲染精密分贝刻度网格与音量包络线关键帧手柄。
  - **在 `图片` 页**：静止图像轨道生成连续缩略胶卷图 (Filmstrip)，支持从上方素材池一键拖入替换。
  - **在 `导出` 页**：时间轴标尺上方激活渲染入出点区间手柄（In/Out Markers），以黄色带指示导出渲染工作区范围。
- **播放指针 (Playhead)**：
  - 游标头为反向五边形，颜色为纯正 `#76B900`（NVIDIA Green）。
  - 穿透多轨的时间轴纵向指示线为 `1.5px solid #76B900`，支持全系统空格键即时播放/暂停。

### 5.3 对齐 Premiere Pro (PR) 的剪辑工作台组件
> 详细拆解规范请参阅专项文档：[剪辑工作台布局规范 (edit-layout-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/edit-layout-spec.md) 及参考原图 [pr-reference.png](file:///d:/Work/Dev/ClipFlow/docs/assets/pr-reference.png)。

- **面板卡片 (Panel Canvas)**：
  - 外框带有 `8px` 圆角（Win 11 容器曲率），各面板间以 `2px` 的 `#0D0D0D` 缝隙自然分离。
  - 面板顶部 Header 高度 `32px`，标签页切换采用紧凑圆角卡片。
- **波形渲染 (Audio Waveform)**：
  - 在音频轨矩形内部，以 `#76B900`（正电平）和 `#439E6D`（平稳区）双色柱线实时绘制。

### 5.4 按钮与控制控件规范
- **主要操作按钮 (Primary CTA)**：
  - 继承 NVIDIA 特色：透明底 + `2px solid #76B900` 绿色线框，圆角 `4px`（Win 11 规范），文字白色加粗。
  - 悬浮 (Hover)：背景填充为 `#1EAEDB` (NVIDIA 规范 Teal) 或高亮绿，文字保持清晰反白。
- **次要操作按钮 (Secondary)**：
  - 背景 `#1E1E1E`，边框 `1px solid #333333`，圆角 `4px`。

---

## 6. Rust + egui 0.36 / wgpu 30.0 代码级映射表

在 Rust 端利用 `egui 0.36` 实现本套规范时，直接映射为以下样式与 Token 结构体：

```rust
use egui::{Color32, CornerRadius, Stroke, Visuals};

pub struct ClipFlowTheme;

impl ClipFlowTheme {
    // 品牌信号色
    pub const NVIDIA_GREEN: Color32 = Color32::from_rgb(118, 185, 0);     // #76B900
    pub const ELECTRIC_LIME: Color32 = Color32::from_rgb(191, 242, 48);   // #BFF230
    
    // 背景阶梯
    pub const BG_VOID: Color32 = Color32::from_rgb(13, 13, 13);            // #0D0D0D
    pub const BG_PANEL: Color32 = Color32::from_rgb(20, 20, 20);           // #141414
    pub const BG_CARD: Color32 = Color32::from_rgb(28, 28, 28);            // #1C1C1C
    pub const BORDER_COLOR: Color32 = Color32::from_rgb(51, 51, 51);       // #333333

    // Win 11 圆角曲率映射 (egui 0.36 CornerRadius)
    pub const RADIUS_WINDOW: CornerRadius = CornerRadius::same(12); // 主窗口
    pub const RADIUS_PANEL: CornerRadius = CornerRadius::same(8);   // 面板/对话框
    pub const RADIUS_CONTROL: CornerRadius = CornerRadius::same(4); // 按钮/输入框
    pub const RADIUS_CLIP: CornerRadius = CornerRadius::same(3);    // 时间轴片段

    /// 一键配置全局 egui::Visuals 样式
    pub fn apply_to(ctx: &egui::Context) {
        let mut visuals = Visuals::dark();
        
        // 窗口与面板底色
        visuals.panel_fill = Self::BG_VOID;
        visuals.window_fill = Self::BG_PANEL;
        visuals.window_stroke = Stroke::new(1.0, Self::BORDER_COLOR);
        visuals.window_corner_radius = Self::RADIUS_PANEL;

        // 默认控件外观 (Win 11 4px 圆角)
        visuals.widgets.inactive.bg_fill = Self::BG_CARD;
        visuals.widgets.inactive.corner_radius = Self::RADIUS_CONTROL;
        visuals.widgets.hovered.bg_fill = Color32::from_rgb(38, 38, 38);
        visuals.widgets.hovered.corner_radius = Self::RADIUS_CONTROL;
        
        // 激活与聚焦状态：注入 NVIDIA 绿色信号边框
        visuals.widgets.active.bg_fill = Color32::from_rgb(21, 43, 0);
        visuals.widgets.active.bg_stroke = Stroke::new(1.5, Self::NVIDIA_GREEN);
        visuals.widgets.active.corner_radius = Self::RADIUS_CONTROL;
        
        visuals.selection.bg_fill = Color32::from_rgba_premultiplied(118, 185, 0, 60);
        visuals.selection.stroke = Stroke::new(1.5, Self::NVIDIA_GREEN);

        ctx.set_visuals(visuals);
    }
}
```
