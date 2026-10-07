# 时间轴数据模型与工程持久化规范 (Timeline Data Model & Project Format)

> **版本**：v0.3.0  
> **更新时间**：2026-10-07  
> **适用技术栈**：Rust 1.99 (MSVC), serde 1.0, zstd 0.13  
> **核心地位**：ClipFlow 全局时间轴单一事实来源（SSOT）的 Rust 内部数据结构、命令模式事务、亚毫秒时间计算与 `.clipflow` 工程存储标准。

---

## 1. 时间基准与时间码数学 (Time Base & Timecode)

为了彻底消除浮点数累加在非编剪辑中造成的“时间漂移”与“帧错位”，全系统禁止使用 `f32`/`f64` 作为时间轴关键位置与时长的基准单位。系统统一采用**有理数时间标度 (Rational Time)**。

### 1.1 有理数时间核心结构：`RationalTime`

```rust
use serde::{Deserialize, Serialize};

/// 有理数时间戳，表示为: ticks / timebase
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct RationalTime {
    /// 刻度数（可为负值，用于相对位移）
    pub value: i64,
    /// 每秒的刻度分母（Timebase），如 60000、24000 等，严禁为 0
    pub timescale: u32,
}

impl RationalTime {
    pub const ZERO: Self = Self { value: 0, timescale: 1000 };

    pub fn new(value: i64, timescale: u32) -> Self {
        assert!(timescale > 0, "Timescale must be positive");
        Self { value, timescale }
    }

    /// 转换为目标分母的等价有理数时间（保持精度）
    pub fn rescaled_to(&self, new_timescale: u32) -> Self {
        if self.timescale == new_timescale {
            return *self;
        }
        let scaled_val = (self.value as i128 * new_timescale as i128) / self.timescale as i128;
        Self {
            value: scaled_val as i64,
            timescale: new_timescale,
        }
    }

    /// 转换为浮点秒（仅在送入音频驱动或日志输出等非状态机逻辑中使用）
    pub fn to_seconds(&self) -> f64 {
        self.value as f64 / self.timescale as f64
    }

    /// 从秒与目标分母构建
    pub fn from_seconds(seconds: f64, timescale: u32) -> Self {
        Self {
            value: (seconds * timescale as f64).round() as i64,
            timescale,
        }
    }
}
```

### 1.2 时间区间：`TimeRange`

采用**左闭右开 `[start, start + duration)`** 区间，防止相邻片段接缝重叠或漏帧。

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: RationalTime,
    pub duration: RationalTime,
}

impl TimeRange {
    pub fn new(start: RationalTime, duration: RationalTime) -> Self {
        Self { start, duration }
    }

    pub fn end_exclusive(&self) -> RationalTime {
        let dur = self.duration.rescaled_to(self.start.timescale);
        RationalTime::new(self.start.value + dur.value, self.start.timescale)
    }

    pub fn contains(&self, time: RationalTime) -> bool {
        let t = time.rescaled_to(self.start.timescale);
        let end = self.end_exclusive();
        t.value >= self.start.value && t.value < end.value
    }
}
```

### 1.3 SMPTE 时间码表示与丢帧规则

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FrameRate {
    Fps23_976, // 24000/1001
    Fps24,     // 24/1
    Fps25,     // 25/1 (PAL)
    Fps29_97,  // 30000/1001 (NTSC Drop-Frame 或 Non-Drop-Frame)
    Fps30,     // 30/1
    Fps50,     // 50/1
    Fps59_94,  // 60000/1001
    Fps60,     // 60/1
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SmpteTimecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub frames: u8,
    pub is_drop_frame: bool,
}

impl SmpteTimecode {
    /// 格式化为标准专业监视器时间码字符串
    /// 非丢帧: 01:23:45:12
    /// 丢帧格式: 01:23:45;12 (分号分隔分秒与帧)
    pub fn to_string(&self) -> String {
        let delimiter = if self.is_drop_frame { ';' } else { ':' };
        format!(
            "{:02}:{:02}:{:02}{}{:02}",
            self.hours, self.minutes, self.seconds, delimiter, self.frames
        )
    }
}
```

### 1.4 Agent 防腐网关与帧网格硬吸附 (`AgentTimelineAcl`)

大模型 (LLM) 与外部 Agent 工具通过自然语言或 JSON 交互时，产出的时间戳天然为十进制浮点秒数（`f64`）。为杜绝浮点数侵入时间轴内部状态，系统设立显式防腐层：外部浮点秒数在穿透至 `TimelineCommand` 之前，必须由 `AgentTimelineAcl` 强制量化并吸附至最近的物理帧分界点，并消除切片微小缝隙引发的 1 帧黑屏空洞。

> **只读与修改路径隔离原则**：  
> `AgentTimelineAcl` 仅作为写操作与时间轴事务的单向防腐栅栏。Agent 执行只读状态查询（如 `timeline_inspect_range`）时，直接读取内存中各 Clip 的 `RationalTime` 并转换为人类与 LLM 习惯的 `f64` 浮点秒返回，**绝不经过 ACL 的二次整数截断量化**，避免无意义的微秒级精度漂移。

```rust
/// Agent 外部通信使用的原始请求结构 (仅用于 IPC / JSON 序列化)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCutRequest {
    pub start_time_seconds: f64,
    pub end_time_seconds: f64,
    pub reason: String,
}

/// Agent 时间轴防腐网关
pub struct AgentTimelineAcl;

impl AgentTimelineAcl {
    /// 将外部浮点秒数严格量化吸附至序列当前帧网格分界点
    #[inline]
    pub fn seconds_to_snapped_time(
        seconds: f64,
        timebase: u32,
        fps_denominator: u32,
    ) -> RationalTime {
        if seconds <= 0.0 {
            return RationalTime::new(0, timebase);
        }
        let frame_duration_s = fps_denominator as f64 / timebase as f64;
        let frame_index = (seconds / frame_duration_s).round() as i64;
        let snapped_value = frame_index * (fps_denominator as i64);
        RationalTime::new(snapped_value, timebase)
    }

    /// 对 Agent 提交的切除请求执行合法性校验、帧吸附与拓扑缝合
    /// 相邻切片间距 <= 1 帧微差时自动无缝对齐，保证切片空洞坏帧率严格为 0
    pub fn sanitize_and_stitch_cuts(
        raw_cuts: &[AgentCutRequest],
        source_duration: RationalTime,
        timebase: u32,
        fps_denominator: u32,
    ) -> Vec<TimeRange> {
        let mut valid_ranges = Vec::with_capacity(raw_cuts.len());
        let max_ticks = source_duration.rescaled_to(timebase).value;

        for cut in raw_cuts {
            let start = Self::seconds_to_snapped_time(cut.start_time_seconds, timebase, fps_denominator);
            let end = Self::seconds_to_snapped_time(cut.end_time_seconds, timebase, fps_denominator);

            let clamped_start = start.value.clamp(0, max_ticks);
            let clamped_end = end.value.clamp(clamped_start, max_ticks);

            if clamped_end > clamped_start {
                let duration_ticks = clamped_end - clamped_start;
                valid_ranges.push(TimeRange::new(
                    RationalTime::new(clamped_start, timebase),
                    RationalTime::new(duration_ticks, timebase),
                ));
            }
        }

        let frame_ticks = fps_denominator as i64;
        let mut stitched_ranges: Vec<TimeRange> = Vec::with_capacity(valid_ranges.len());

        for range in valid_ranges {
            if let Some(last) = stitched_ranges.last_mut() {
                let gap = range.start.value - last.end_exclusive().value;
                if gap > 0 && gap <= frame_ticks {
                    last.duration = RationalTime::new(last.duration.value + gap, timebase);
                }
            }
            stitched_ranges.push(range);
        }

        stitched_ranges
    }
}
```

---

## 2. 核心数据模型 (Core Domain Models)

多轨数据模型统一归属于 `clipflow-timeline` 逻辑 crate。模型遵循**单向引用与扁平 ID 寻址**，禁止自引用指针。

```mermaid
classDiagram
    class Project {
        +Uuid id
        +String name
        +AssetPool asset_pool
        +Vec~Sequence~ sequences
        +Uuid active_sequence_id
        +Option~AgentProjectSession~ agent_session
    }
    class Sequence {
        +Uuid id
        +String name
        +FrameRate frame_rate
        +CanvasSize resolution
        +RationalTime playhead
        +Option~TimeRange~ work_area
        +Vec~Track~ video_tracks
        +Vec~Track~ audio_tracks
        +Vec~Track~ subtitle_tracks
        +Vec~Track~ effect_tracks
    }
    class Track {
        +Uuid id
        +String name
        +TrackKind kind
        +bool mute
        +bool solo
        +bool locked
        +bool visible
        +Vec~Clip~ clips
    }
    class Clip {
        +Uuid id
        +Uuid asset_id
        +TimeRange source_range
        +TimeRange timeline_range
        +f64 speed
        +ClipEffects effects
    }
    class Asset {
        +Uuid id
        +PathBuf file_path
        +String sha256
        +AssetKind kind
        +RationalTime duration
    }

    Project "1" *-- "1" AssetPool
    Project "1" *-- "many" Sequence
    Sequence "1" *-- "many" Track
    Track "1" *-- "many" Clip
    Clip --> Asset : 引用
```

### 2.1 资产池与媒体源 (`Asset` & `AssetPool`)

```rust
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssetKind {
    Video { width: u32, height: u32, has_audio: bool },
    Audio { sample_rate: u32, channels: u16 },
    Image { width: u32, height: u32 },
    Subtitle { language: String },
    HyperFramesTemplate { template_id: String, schema_version: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: Uuid,
    pub name: String,
    pub absolute_path: PathBuf,
    /// 相对工程根目录路径（用于便携式工程移动）
    pub relative_path: Option<PathBuf>,
    pub file_size: u64,
    pub sha256_hash: String,
    pub kind: AssetKind,
    pub native_duration: RationalTime,
    pub native_timebase: u32,
    /// 代理媒体（低清快速剪辑，可选）
    pub proxy_path: Option<PathBuf>,
}
```

### 2.2 剪辑片段 (`Clip`) 与多态载荷

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transform2D {
    pub position_x: Animatable<f32>, // 偏移像素 (0.0 为中心)
    pub position_y: Animatable<f32>,
    pub scale_x: Animatable<f32>,    // 默认 1.0
    pub scale_y: Animatable<f32>,
    pub rotation: Animatable<f32>,   // 角度 (0.0 - 360.0)
    pub opacity: Animatable<f32>,    // 0.0 - 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioProperties {
    pub volume_db: Animatable<f32>,  // 分贝，0.0 为原生响度，-60.0 为静音
    pub pan: Animatable<f32>,        // 声相平衡 (-1.0 左声道, 1.0 右声道)
}

/// 词级时间戳 (用于卡拉OK点亮与文本驱动剪辑)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WordTiming {
    pub word: String,
    pub start_time: RationalTime,
    pub end_time: RationalTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SubtitleLayout {
    /// 仅显示原文
    OnlyOriginal,
    /// 仅显示译文
    OnlyTranslate,
    /// 原文在上，译文在下
    OriginalOnTop,
    /// 译文在上，原文在下
    TranslateOnTop,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackgroundBoxStyle {
    /// 背景填充色 (HEX RGBA，如半透明纯黑 [0.0, 0.0, 0.0, 0.65])
    pub fill_color: [f32; 4],
    /// 圆角半径 (px，基准 1080p 下默认 8.0px)
    pub corner_radius: f32,
    /// 水平内边距 (px，默认 16.0px)
    pub padding_h: f32,
    /// 垂直内边距 (px，默认 8.0px)
    pub padding_v: f32,
    /// 边框描边宽度 (0.0 表示无边框，默认 0.0px)
    pub border_width: f32,
    /// 边框颜色 (默认完全透明)
    pub border_color: [f32; 4],
}

/// 字幕样式与排版模型 (完全对齐 subtitle-render-spec.md)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubtitleStyle {
    pub font_family: String,
    pub font_size: f32,
    pub font_weight: u16,
    pub fill_color: [f32; 4],
    pub stroke_width: f32,
    pub stroke_color: [f32; 4],
    pub shadow_offset: [f32; 2],
    pub shadow_blur: f32,
    pub shadow_color: [f32; 4],
    pub position_y_percent: f32,
    pub karaoke_highlight_color: Option<[f32; 4]>,
    pub layout: SubtitleLayout,
    pub secondary_font_size: f32,
    pub secondary_fill_color: [f32; 4],
    pub vertical_gap: f32,
    pub background_box: Option<BackgroundBoxStyle>,
}

/// 片段多态专属载荷 (承载不同轨道类型的领域数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipPayload {
    /// 普通音视频与静态图片片段
    Media,
    /// 口播字幕片段 (C1 轨，含词级时间戳、可选译文与文字样式)
    Subtitle {
        text: String,
        translated_text: Option<String>,
        words: Vec<WordTiming>,
        style: SubtitleStyle,
    },
    /// HyperFrames 动态代码包装片段 (FX 轨，包含模板参数 JSON)
    HyperFrames {
        template_id: String,
        props: serde_json::Value,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clip {
    pub id: Uuid,
    pub name: String,
    /// 关联的素材库资产 ID
    pub asset_id: Uuid,
    /// 在原始媒体素材中的入出点范围
    pub source_range: TimeRange,
    /// 在时间轴全局轨道上的摆放范围
    pub timeline_range: TimeRange,
    /// 播放速度比例 (默认 1.0，-1.0 为倒放)
    pub speed: f64,
    /// 视频变换与合成属性
    pub transform: Transform2D,
    /// 音频增益与声相
    pub audio_props: AudioProperties,
    /// 片段专属滤镜链
    pub filters: Vec<ClipFilter>,
    /// 片段多态专属数据载荷 (音视频 / 字幕 / 动效参数)
    pub payload: ClipPayload,
    /// 是否被禁用（按 D 键静音/隐藏单片段）
    pub disabled: bool,
}

> **片段变速与有理数时间映射公式 (Time Warping & Rational Mapping)**：  
> 1. **有效时长换算**：时间轴跨度与素材源跨度之间满足严格有理数等式：  
>    $$\text{timeline\_range.duration} = \frac{\text{source\_range.duration}}{|speed|}$$  
>    在 Rust 实现中，通过将浮点 `speed` 转换为有理数比例（如 $1.5 = 3/2$），以纯整数乘除运算更新 `timeline_range`，杜绝累积微秒级帧舍入误差。  
> 2. **取样时间戳映射 ($t_{\text{timeline}} \to t_{\text{source}}$)**：  
>    - **正放 ($speed > 0$)**：取样点随时间轴正向推进：  
>      $$t_{\text{source}} = \text{source\_range.start} + (t_{\text{timeline}} - \text{timeline\_range.start}) \times speed$$  
>    - **倒放 ($speed < 0$)**：取样点以素材出点向入点反向溯源：  
>      $$t_{\text{source}} = \text{source\_range.end} - (t_{\text{timeline}} - \text{timeline\_range.start}) \times |speed|$$  
> 3. **三变量调和准则与权威单一源约定 (Three-Variable Reconciliation Rule)**：  
>    `source_range.duration`、`timeline_range.duration` 与 `speed` 三者紧密关联。为杜绝状态冗余产生的不一致，确立以下权威调和规则：  
>    - **常规修剪 (Trim / Slip / Ripple)**：`speed` 视为恒定参数。用户拖拽切片边缘改变 `timeline_range.duration` 时，系统依据固定 $speed$ 自动折算并更新 `source_range.duration`；  
>    - **显式变速 (Speed Dialog)**：用户通过变速对话框主动调整时，显式指定锁定“源范围”或“时间轴范围”之一，重新求值第三项；  
>    - **工程自检校验**：加载工程或外部交换文件时，以 `source_range` 与 `speed` 为物理基准校验 `timeline_range.duration`，若微差处于 1 帧以内则自动向帧分界点强制吸附对齐。  

/// 片段滤镜与特效实例
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClipFilter {
    pub id: Uuid,
    pub name: String,
    /// 是否激活生效（UI 旁路开关）
    pub enabled: bool,
    /// 滤镜类型与专属属性
    pub kind: ClipFilterKind,
}

/// 核心视频滤镜分类与动画参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipFilterKind {
    /// 画面裁剪（上下左右内缩比例 0.0 ~ 1.0）
    Crop {
        top: Animatable<f32>,
        bottom: Animatable<f32>,
        left: Animatable<f32>,
        right: Animatable<f32>,
    },
    /// 基础色彩与曝光调节
    ColorAdjustment {
        exposure: Animatable<f32>,   // 曝光补偿 (-5.0 ~ +5.0 EV)
        contrast: Animatable<f32>,   // 对比度 (-1.0 ~ +1.0)
        saturation: Animatable<f32>, // 饱和度 (0.0 ~ 2.0, 1.0 为原始)
        temperature: Animatable<f32>,// 色温 (-1.0 ~ +1.0)
    },
    /// 3D LUT 色彩查找表
    Lut {
        lut_asset_id: Uuid,
        intensity: Animatable<f32>,  // 混合强度 0.0 ~ 1.0
    },
    /// 高斯模糊
    GaussianBlur {
        radius: Animatable<f32>,     // 模糊半径 (像素)
    },
    /// 自定义着色器/扩展滤镜 (M3+ 扩展点)
    Custom {
        shader_id: String,
        params: serde_json::Value,
    },
}
```

### 2.3 关键帧动画系统 (`Animatable<T>`)

支持常数值与贝塞尔关键帧列表自由切换：

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Interpolation {
    Linear,
    Hold,
    Bezier {
        handle_in_x: f32,
        handle_in_y: f32,
        handle_out_x: f32,
        handle_out_y: f32,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keyframe<T> {
    pub time: RationalTime,
    pub value: T,
    pub interpolation: Interpolation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Animatable<T> {
    Static(T),
    Animated(Vec<Keyframe<T>>),
}

impl<T: Copy + Interpolate> Animatable<T> {
    /// 根据当前时间点求值
    pub fn evaluate_at(&self, time: RationalTime) -> T {
        match self {
            Self::Static(val) => *val,
            Self::Animated(keyframes) => {
                // 执行二分查找并进行贝塞尔或线性插值计算
                interpolate_keyframes(keyframes, time)
            }
        }
    }
}
```

### 2.4 轨道 (`Track`)、通道条与序列总线容器

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TrackKind {
    Video,
    Audio,
    Subtitle,
    HyperFrames,
}

/// 4 段参量均衡器单频段配置 (4-Band Parametric EQ Band)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParametricEqBand {
    /// 中心频率 (20.0 Hz ~ 20000.0 Hz)
    pub freq_hz: f32,
    /// 增益 (-24.0 dB ~ +24.0 dB, 0.0 为平直)
    pub gain_db: f32,
    /// 品质因数 Q (0.1 ~ 10.0, 默认 0.707 对应 Butterworth 响应)
    pub q: f32,
    /// 是否激活该频段
    pub enabled: bool,
}

/// 轨道级音频通道条属性 (Track-Level Audio Strip)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackAudioProperties {
    /// 轨道主推子音量 (-60.0 dB ~ +12.0 dB, 默认 0.0 dB)
    pub fader_volume_db: f32,
    /// 轨道立体声声相 (-1.0 极左, 0.0 居中, 1.0 极右)
    pub pan: f32,
    /// AI 口播人声降噪量 (0.0 ~ 1.0)
    pub ai_denoise_amount: f32,
    /// 动态压缩器阈值 (单位 dB, 默认 0.0 为不触发压缩)
    pub compressor_threshold_db: f32,
    /// 4 段专业参量均衡器: [0: 低切, 1: 低中频, 2: 高中频, 3: 高架空气感]
    pub eq_bands: [ParametricEqBand; 4],
}

impl Default for TrackAudioProperties {
    fn default() -> Self {
        Self {
            fader_volume_db: 0.0,
            pan: 0.0,
            ai_denoise_amount: 0.0,
            compressor_threshold_db: 0.0,
            eq_bands: [
                ParametricEqBand { freq_hz: 80.0, gain_db: 0.0, q: 0.707, enabled: true },
                ParametricEqBand { freq_hz: 500.0, gain_db: 0.0, q: 1.0, enabled: false },
                ParametricEqBand { freq_hz: 3000.0, gain_db: 0.0, q: 1.0, enabled: false },
                ParametricEqBand { freq_hz: 10000.0, gain_db: 0.0, q: 0.707, enabled: false },
            ],
        }
    }
}

/// 序列级主输出总线 (Master Bus)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MasterBusProperties {
    /// 主输出总推子 (默认 0.0 dB)
    pub master_fader_db: f32,
    /// 广播标称响度目标 (默认 -14.0 LUFS)
    pub target_lufs: f32,
    /// 砖墙母带限制器门限 (默认 -1.0 dBFS True Peak)
    pub limiter_ceiling_db: f32,
}

impl Default for MasterBusProperties {
    fn default() -> Self {
        Self {
            master_fader_db: 0.0,
            target_lufs: -14.0,
            limiter_ceiling_db: -1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Track {
    pub id: Uuid,
    pub name: String, // 如 "V1", "A1 (口播)", "FX1"
    pub kind: TrackKind,
    pub mute: bool,
    pub solo: bool,
    pub locked: bool,
    pub visible: bool,
    /// 当 kind == TrackKind::Audio 时强持有的通道条属性
    pub audio_props: Option<TrackAudioProperties>,
    /// 按在时间轴上的 timeline_range.start 升序排列的片段列表
    pub clips: Vec<Clip>,
}

/// 画布分辨率规格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanvasSize {
    pub width: u32,
    pub height: u32,
}

impl CanvasSize {
    pub const P1080_16_9: Self = Self { width: 1920, height: 1080 };
    pub const P1080_9_16: Self = Self { width: 1080, height: 1920 };
    pub const P4K_16_9: Self = Self { width: 3840, height: 2160 };
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub id: Uuid,
    pub name: String,
    /// 序列目标画幅分辨率
    pub resolution: CanvasSize,
    pub timebase: u32,
    pub fps_denominator: u32,
    pub playhead: RationalTime,
    pub work_area: Option<TimeRange>,
    pub tracks: Vec<Track>,
    /// 序列级主输出总线控制
    pub master_bus: MasterBusProperties,
}
```

> **设计决策与类图映射说明 (Design Rationale & Model Mapping)**：  
> 1. **画幅分辨率与时间基**：概念模型类图中的 `resolution` 由 `CanvasSize` 明确承载（预置 1080P/4K 横竖屏常量）；类图中的概念帧率（`FrameRate`）在 Rust 中具体化为有理数基数 `timebase` 与 `fps_denominator`（如 60000 / 1001 对应 59.94 FPS），确保时间轴全局帧计算绝对无累积浮点漂移。  
> 2. **轨道单列表物理存储**：类图中概念性区分了 `video_tracks`、`audio_tracks`、`subtitle_tracks` 与 `effect_tracks`，但在 Rust 物理实现中统一合并为单一有序列表 `tracks: Vec<Track>`，由 `Track.kind` 枚举区分。这种设计保持了图层上下层叠覆盖次序与渲染管线遍历的单一真实源（SSOT），上层业务可通过 `sequence.tracks.iter().filter(|t| t.kind == TrackKind::Video)` 投影过滤。

### 2.5 工程根实体与 Agent 导演会话持久化 (`Project` & `AgentProjectSession`)

为了解决“工程关闭再打开后，Agent 分幕大纲、被拒绝候选切点与决策偏好全盘丢失”的问题，工程根实体显式挂载可选的 `AgentProjectSession`：

```rust
use std::collections::HashSet;
use uuid::Uuid;

/// ClipFlow 工程根实体 (SSOT 根节点)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: Uuid,
    pub name: String,
    pub asset_pool: AssetPool,
    pub sequences: Vec<Sequence>,
    pub active_sequence_id: Uuid,
    /// 导演级 Agent 会话与决策记忆持久化
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_session: Option<AgentProjectSession>,
}

/// Agent 导演工作台持久化会话与决策记忆模型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentProjectSession {
    pub session_id: Uuid,
    pub last_updated_at: i64,
    /// 当前活跃或待确认的完整剪辑方案（DirectorPlan，详见 agent-director-spec.md）
    pub active_plan: Option<serde_json::Value>,
    /// 缓存的宏观分幕大纲（避免工程重开后重复消耗 Token 重新分幕）
    pub cached_outline: Vec<CachedChapterSummary>,
    /// 用户决策偏好与负样本记忆（防二次分析反复推荐已被用户否决的切点）
    pub user_preferences: AgentUserPreferences,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedChapterSummary {
    pub chapter_index: u32,
    pub headline: String,
    pub time_range: (f64, f64),
    pub key_takeaways: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentUserPreferences {
    /// 偏好的粗剪节奏（FastPaced, Steady, Climax）
    pub preferred_rhythm: String,
    /// 用户曾明确取消勾选/拒绝切除的时间区间签名散列集合（避免重新分析时逆向推荐）
    pub rejected_cut_hashes: HashSet<String>,
    /// 自定义配乐避让深度与偏好音量
    pub bgm_ducking_preference_db: Option<f32>,
}
```
*注：内存常驻的 `TimelineMemoryIndex`（包含时序声学物理锚点、分幕缓存与负样本指纹，详见 agent-director-spec.md 第 2.5 节）在落盘时将其语义分段摘要与负样本切点签名沉淀映射至 `AgentProjectSession`（`cached_outline` 与 `rejected_cut_hashes`），确保工程重开时零 Token 损耗还原记忆上下文。该字段随 `Project` 一并序列化至 `.clipflow` 容器内部，经 Zstandard 压缩存储，体积增加小于 15KB，但在异机迁移和重新打开时能够实现 100% 完整的 Agent 会话与剪辑方案还原。*

### 2.6 不可变时间线快照与草稿沙箱 (`TimelineSnapshot` & `DraftTimeline`)

为了彻底杜绝外部 Agent、内置协调器或复杂交互中途由于报错、网络中断或 Token 超限导致的“时间轴半残破损与脏读脏写”，系统确立**“提案制沙箱（Propose-Apply）”核心契约**：

```mermaid
flowchart LR
    LiveSequence["原生时间线 Sequence (SSOT)"] -->|只读快照 clone / Arc| Snapshot["TimelineSnapshot (不可变)"]
    Snapshot -->|作为底模基准| Draft["DraftTimeline 沙箱 (内存推演)"]
    Agent["Agent / 交互操作"] -->|执行工具并记录命令| Draft
    Draft -->|累积产出| Cmds["Vec<TimelineCommand> (原子事务)"]
    UserApprove["用户审查批准 / auto-apply"] -->|apply_transaction| LiveSequence
    Cmds --> UserApprove
```

```rust
use std::sync::Arc;

/// 全局只读时间线快照（轻量、线程安全，供播放器、波形与 UI 零拷贝并发消费）
#[derive(Debug, Clone)]
pub struct TimelineSnapshot {
    pub sequence_id: Uuid,
    pub revision: u64,
    pub inner: Arc<Sequence>,
}

impl TimelineSnapshot {
    pub fn new(sequence: &Sequence, revision: u64) -> Self {
        Self {
            sequence_id: sequence.id,
            revision,
            inner: Arc::new(sequence.clone()),
        }
    }
}

/// 专供 Agent 推演、多步复合操作与模拟剪辑的内存草稿沙箱
pub struct DraftTimeline<'a> {
    /// 派生时的基准快照，用于检测版本漂移 (Stale Check)
    base_snapshot: &'a TimelineSnapshot,
    /// 暂存的推演状态副本（对 Sequence 局部进行修改，不碰全局 SSOT）
    staged_sequence: Sequence,
    /// 该沙箱生命周期内执行的所有原子命令序列（Replay 账本）
    recorded_commands: Vec<Box<dyn TimelineCommand>>,
}

impl<'a> DraftTimeline<'a> {
    pub fn new(base: &'a TimelineSnapshot) -> Self {
        Self {
            base_snapshot: base,
            staged_sequence: (*base.inner).clone(),
            recorded_commands: Vec::new(),
        }
    }

    /// 在沙箱中尝试执行单条命令（立即反映到 staged_sequence，供后续工具自检观测）
    pub fn execute_staged<C: TimelineCommand + 'static>(&mut self, mut command: C) -> Result<(), CommandError> {
        command.execute(&mut self.staged_sequence)?;
        self.recorded_commands.push(Box::new(command));
        Ok(())
    }

    /// 提取出全部记录的命令账本（移出所有权，供主时间线一次性原子提交）
    pub fn take_commands(&mut self) -> Vec<Box<dyn TimelineCommand>> {
        std::mem::take(&mut self.recorded_commands)
    }

    /// 获取当前的暂存序列视图（用于即时度量评估、幽灵图层预览或抽帧观测）
    pub fn staged_sequence(&self) -> &Sequence {
        &self.staged_sequence
    }

    /// 校验沙箱是否已过期（若原生序列的版本号已被手动编辑推进，则拒绝盲目合入）
    pub fn is_stale(&self, current_live_revision: u64) -> bool {
        self.base_snapshot.revision != current_live_revision
    }
}
```

---

## 3. 命令模式与撤销/重做 (Command & Undo/Redo)

所有改变时间轴状态的写操作，必须封装为不可分割的 `TimelineCommand`，统一通过 `TimelineHistory` 执行与回退。

### 3.1 命令特质定义

```rust
pub trait TimelineCommand: Send + Sync {
    /// 执行命令并改变序列状态
    fn execute(&mut self, sequence: &mut Sequence) -> Result<(), CommandError>;
    /// 回退命令，恢复先前状态
    fn undo(&mut self, sequence: &mut Sequence) -> Result<(), CommandError>;
    /// 诊断与界面展示用文案（如：“分割片段”、“波纹删除”）
    fn description(&self) -> &'static str;
}
```

### 3.2 典型核心原子命令实现范例

#### ① 片段分割命令：`SplitClipCommand`
- **执行**：在给定播放头切片，原片段截短至 `cut_point`，新增后半段片段并插入当前轨道。
- **回退**：移除新插入的后半段，将原片段的 `timeline_range.duration` 与 `source_range.duration` 恢复原始长度。

#### ② 波纹删除命令：`RippleDeleteCommand`
- **执行**：删除指定片段，并在受影响的联动轨道上，将所有处于该片段之后的 Clip 向左平移 `deleted_duration`。
- **回退**：将删除的片段恢复至原位，并将后续所有片段向右回移等长区间。
- **多轨波纹保护机制**：支持传入联动轨道范围白名单 `affected_track_ids`；若未显式指定，默认仅联动该片段所在轨道及未锁定的主音画/字幕轨，独立背景音乐轨（A2）及显式锁定轨道绝不发生平移，杜绝音乐被剪碎错位。

```rust
pub struct RippleDeleteCommand {
    pub target_clip_id: Uuid,
    /// 联动波纹平移的轨道白名单（None 表示默认策略：仅联动当前轨及未锁定的主画/主音/字幕轨）
    pub affected_track_ids: Option<Vec<Uuid>>,
    /// 内部执行暂存的状态数据（用于精确回退）
    deleted_clip: Option<Clip>,
    shifted_clips: Vec<(Uuid, RationalTime)>,
}
```

#### ③ 复合事务命令：`CompoundCommand`
- 用于将 Agent 的一次多步决策（如：“剔除 10 处口播停顿并自动对齐”）打包为单一事务，用户按一次 `Ctrl + Z` 即可一次性整体回退。

```rust
pub struct CompoundCommand {
    commands: Vec<Box<dyn TimelineCommand>>,
    desc: &'static str,
}
```

### 3.3 撤销栈架构设计

```rust
pub struct TimelineHistory {
    undo_stack: Vec<Box<dyn TimelineCommand>>,
    redo_stack: Vec<Box<dyn TimelineCommand>>,
    max_depth: usize, // 默认 100 步
}

impl TimelineHistory {
    pub fn new(max_depth: usize) -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
            max_depth,
        }
    }

    pub fn execute(&mut self, mut cmd: Box<dyn TimelineCommand>, seq: &mut Sequence) -> Result<(), CommandError> {
        cmd.execute(seq)?;
        self.undo_stack.push(cmd);
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear(); // 执行新操作后清空重做栈
        Ok(())
    }

    pub fn undo(&mut self, seq: &mut Sequence) -> Result<bool, CommandError> {
        if let Some(mut cmd) = self.undo_stack.pop() {
            cmd.undo(seq)?;
            self.redo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub fn redo(&mut self, seq: &mut Sequence) -> Result<bool, CommandError> {
        if let Some(mut cmd) = self.redo_stack.pop() {
            cmd.execute(seq)?;
            self.undo_stack.push(cmd);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// 原子应用整批命令事务，并将其压入单一 Undo 栈节点
    pub fn apply_transaction(
        &mut self,
        commands: Vec<Box<dyn TimelineCommand>>,
        desc: &'static str,
        sequence: &mut Sequence,
    ) -> Result<(), CommandError> {
        if commands.is_empty() {
            return Ok(());
        }

        let mut compound = CompoundCommand {
            commands,
            desc,
        };

        // 一次性顺序执行批次内的所有子命令
        compound.execute(sequence)?;

        // 作为一个整体节点推入撤销栈
        self.undo_stack.push(Box::new(compound));
        if self.undo_stack.len() > self.max_depth {
            self.undo_stack.remove(0);
        }
        self.redo_stack.clear();
        Ok(())
    }
}
```

### 3.4 批量事务执行与单步撤销保证

当 Agent 导演通过 `DraftTimeline` 提交一整组切分、调色、字幕重排等复合操作时，若按传统模式逐一入栈，撤销栈将产生数十个碎屑步骤，用户按 `Ctrl + Z` 将陷入漫长且中间状态破损的单步回滚。  
通过 `apply_transaction`，系统把全部子命令整体装配进单个 `CompoundCommand` 中执行并压栈。**用户仅需按一次 `Ctrl + Z`，即可无缝完全撤销一整组复合剪辑修改**，实现最高等级的心理安全感与操作可控性。

---

## 4. 工程持久化文件规范 (`.clipflow`)

### 4.1 二进制容器布局 (Container Layout)

`.clipflow` 文件结构兼顾**防止篡改、极速加载与轻量压缩**：

```
+-------------------------------------------------------------------------+
| Magic Bytes: [0x43, 0x46, 0x50, 0x46]  ("CFPF" = ClipFlow Project File) | 4 字节
+-------------------------------------------------------------------------+
| Format Version: 0x0001                                                  | 2 字节
+-------------------------------------------------------------------------+
| Flags: (0x01 = ZSTD Compressed, 0x02 = Has Checksum)                    | 2 字节
+-------------------------------------------------------------------------+
| Uncompressed Payload Size: u64                                          | 8 字节
+-------------------------------------------------------------------------+
| Payload SHA-256 Checksum: [u8; 32]                                      | 32 字节
+-------------------------------------------------------------------------+
| Compressed Data Stream (Zstandard 压缩的 JSON 序列化体, zstd 0.13)         | 可变长
+-------------------------------------------------------------------------+
```

### 4.2 媒体重连策略 (Media Relinking)

工程在异机迁移或目录重命名时，采用三级渐进匹配恢复媒体关联：

1. **绝对路径直连**：检查 `absolute_path` 是否存在且文件大小匹配。
2. **相对路径回退**：将 `relative_path` 与当前 `.clipflow` 所在物理目录拼合检查。
3. **哈希与签名快速扫描**：若前两级失效，触发“素材脱机 (Media Offline)”状态；在用户指定搜索目录内，通过匹配 `file_size` + 头部 1MB SHA-256 哈希实现毫秒级自动重连。

### 4.3 自动保存与预写日志 (Auto-Save & WAL)

- **防崩溃日志 (WAL)**：每执行一次 `TimelineCommand`，主进程以追加写入（Append-only）形式向临时工作目录记录 `session.wal`。
  由于 Rust `Box<dyn TimelineCommand>` 动态分发特质对象无法直接被 `serde` 序列化，系统定义强类型的可序列化载荷枚举 `TimelineCommandPayload`，确保每次提交写操作时能够生成确定性的二进制/JSON 增量帧：
  ```rust
  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub enum TimelineCommandPayload {
      SplitClip {
          clip_id: Uuid,
          cut_point: RationalTime,
          new_clip_id: Uuid,
      },
      RippleDelete {
          target_clip_id: Uuid,
          affected_track_ids: Option<Vec<Uuid>>,
      },
      MoveClip {
          clip_id: Uuid,
          target_track_id: Uuid,
          target_start: RationalTime,
      },
      TrimClipEdge {
          clip_id: Uuid,
          new_timeline_range: TimeRange,
      },
      Compound {
          sub_commands: Vec<TimelineCommandPayload>,
          description: String,
      },
  }

  #[derive(Debug, Clone, Serialize, Deserialize)]
  pub struct WalFrame {
      pub sequence_number: u64,
      pub timestamp_ms: i64,
      pub command: TimelineCommandPayload,
  }
  ```
- **定时全量快照**：每隔 3 分钟后台静默序列化一份全量工程至第一轨本地缓存目录 `.clipflow_cache/{ProjectHash}/autosave/{Name}_{YYYYMMDD_HHMMSS}.clipflow`（详见 [`cache-and-storage-spec.md` 第 2 节](cache-and-storage-spec.md#2-第一轨工程本地缓存规约-clipflow_cache)），最大保留 10 份历史快照。
- **异常恢复检测**：启动时如检测到异常退出遗留的 WAL 日志，弹出对话框提示用户“检测到未保存的工程修改，是否一键恢复”。恢复时基于最近的自动快照重放未提交的 `WalFrame` 流。

---

## 5. 外部工程交换与切点导出规范 (NLE Project Interchange & Cut-List Export)

为打破生态孤岛、打通与 Premiere Pro、DaVinci Resolve 及 Pro Tools 等工业级后期工作流的无缝接续，时间轴引擎在保持自身内部纯 Rust `RationalTime` SSOT 模型的同时，提供标准化的外部工程交换与切点导出协议。

### 5.1 导出器抽象契约 (`TimelineExporter`)

所有外部交换格式均遵循单一抽象 Trait，严禁在 UI 或媒体管线内散落手写格式转换逻辑：

```rust
use std::path::Path;
use anyhow::Result;

/// 外部时间轴工程导出器标准契约
pub trait TimelineExporter: Send + Sync {
    /// 导出器标识，如 "fcp7_xml", "cmx3600_edl", "otio"
    fn format_id(&self) -> &'static str;

    /// 规范文件扩展名，如 "xml", "edl", "otio"
    fn file_extension(&self) -> &'static str;

    /// 执行序列化导出
    fn export_sequence(&self, sequence: &Sequence, output_path: &Path) -> Result<()>;
}
```

### 5.2 一维切点抽取与有理数帧精度对齐 (`CutList`)

在口播粗剪外发场景中，系统先将平行多轨序列抽离正规化为纯净的一维切点列表（`CutList`），消除多轨交错阻抗：

```rust
/// 标准化切点事件
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutEvent {
    pub event_index: usize,
    pub record_in_frame: i64,
    pub record_out_frame: i64,
    pub duration_frames: i64,
    pub kind: CutEventKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CutEventKind {
    Clip {
        clip_id: Uuid,
        asset_id: Uuid,
        file_path: PathBuf,
        file_name: String,
        source_in_frame: i64,
        source_out_frame: i64,
    },
    Gap,
}

/// 标准化切点列表
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CutList {
    pub sequence_name: String,
    pub target_fps: FrameRate,
    pub total_duration_frames: i64,
    pub events: Vec<CutEvent>,
}
```

#### 有理数整除帧对齐铁律（杜绝浮点数累加漂移）
所有时间戳与帧序号转换统一走 `i128` 向零截断整数整除，严禁浮点数参与中间计算：
$$\text{Frame}(T, FPS) = \left\lfloor \frac{V \cdot N_{fps}}{S \cdot D_{fps}} \right\rfloor$$
严格保障相邻连续片段的无缝守恒：
$$F_{end}^{(n)} \equiv F_{start}^{(n+1)}$$

### 5.3 梯次格式导出矩阵

系统确立**“M3 工业级切点外发先行 $\to$ M4+ 通用 IR 演进”**的实施矩阵（v1.0 MVP M0~M2 阶段聚焦于内部时间轴引擎与本地剪辑闭环）：

1. **Milestone 3 (M3 远期规划) 核心落地**：
   - **剪映 / CapCut 标准草稿 (`draft_content.json`)**：
     - **业务价值**：彻底打破开源 NLE 早期缺少海量大众贴纸、花字与爆款特效的冷启动短板。让 ClipFlow 的高性能 Rust 引擎与 Agent 负责听声音、删停顿、对节奏与排粗剪，随后一键直接导出剪映本地工程；
     - **时间戳投影铁律**：剪映草稿内部时间轴基准固定为**微秒刻度 ($1\text{s} = 1,000,000\mu\text{s}$)**。转换时调用有理数精确折算：
       $$T_{\mu s} = \frac{\text{RationalTime.value} \times 1,000,000}{\text{RationalTime.timescale}}$$
     - **多轨数据对齐**：视频轨道映射至 `tracks[type="video"]`，多轨音频映射至 `tracks[type="audio"]`，口播字幕（`C1` 轨）映射至剪映的独立文本轨 `tracks[type="text"]`（含词级样式、描边与底板颜色），片段入出点 `source_timerange` 与 `target_timerange` 严格对齐微秒刻度，用户在剪映中双击工程即可零缝隙无缝接续；
   - **Apple FCPXML 1.10 DTD 规范导出**：
     - 严格依照 Apple FCPXML 1.10 DTD 规范输出，消除旧版 FCP7 XML 在现代 Final Cut Pro 与 DaVinci Resolve 中的兼容警报；
     - 原地引用导入媒体必须指向真实文件 URI（严禁手写非法 `<pathurl>`），连带故事线（Connected Storylines）与变速片段严格依照有理数帧率采样，确保 Resolve 能 100% 自动关联摄影机原始素材；
   - **Apple FCP7 XML (`xmeml v5`)**：与 ClipFlow 平行多轨结构 1:1 零阻抗契合，经由 `quick-xml` 流式生成，路径强制规范化为 RFC 3986 `file://localhost/...` 百分号转义 URI，Premiere Pro 与 DaVinci Resolve 导入成功率高达 **99.9%**；
   - **规范化 CMX 3600 EDL**：符合 80 列定宽规范，Reel ID 规约为 8 字符，通过注入 `* FROM CLIP NAME` 与 `* SOURCE FILE` 扩展注释行安全传递 UTF-8 中文长路径，杜绝穿孔卡协议导致的乱码与媒体离线。
2. **Milestone 4+ (M4+ 远期规划) 架构演进**：
   - **OpenTimelineIO (OTIO) 通用 IR**：引入好莱坞工业开源内存标准充当中枢适配层，解耦内部数据结构与多格式转换；
   - **FCPX Spine 树投影算法**：实现多轨时间轴向 Apple FCPXML 磁性故事板（`<spine>` + 相对 `offset` + `<gap>` 填充）的降维转换。

### 5.4 静态合规预检与降级诊断看板 (`ConformInspector`)

为杜绝“外部导出静默丢失专属图层”引发的信任崩塌，在写盘前自动执行静态合规扫描：

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum IssueSeverity {
    Information,        // 纯净切点，无损映射
    Warning,            // 属性微调（如音量曲线变为平直增益）
    UnsupportedDropped, // 无法承载（如 HyperFrames Web 动效图层、复杂变速曲线）
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosticReport {
    pub target_format: String,
    pub total_clips_scanned: usize,
    pub warning_count: usize,
    pub dropped_count: usize,
    pub items: Vec<DiagnosticItem>,
}
```

若时间轴包含 HyperFrames 动态 Web 角标（FX 轨），扫描器将其判定为 `UnsupportedDropped` 并在导出弹窗中提示替代建议：“建议在 ClipFlow 中先将此段动效渲染为 Apple ProRes 4444 独立透明图层，再送入 PR 叠加”，消灭黑盒静默丢特性的焦虑。

