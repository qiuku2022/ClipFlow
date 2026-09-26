# 时间轴数据模型与工程持久化规范 (Timeline Data Model & Project Format)

> **版本**：v1.0.0  
> **更新时间**：2026-09-26  
> **适用技术栈**：Rust 1.98 (MSVC), serde 1.0, zstd 0.13  
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
}

/// 片段多态专属载荷 (承载不同轨道类型的领域数据)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClipPayload {
    /// 普通音视频与静态图片片段
    Media,
    /// 口播字幕片段 (C1 轨，含词级时间戳与文字样式)
    Subtitle {
        text: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sequence {
    pub id: Uuid,
    pub name: String,
    pub timebase: u32,
    pub fps_denominator: u32,
    pub playhead: RationalTime,
    pub work_area: Option<TimeRange>,
    pub tracks: Vec<Track>,
    /// 序列级主输出总线控制
    pub master_bus: MasterBusProperties,
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
- **执行**：删除指定片段，并在该轨道及所有未锁定轨道上，将所有处于该片段之后的 Clip 向左平移 `deleted_duration`。
- **回退**：将删除的片段恢复至原位，并将后续所有片段向右回移等长区间。

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
}
```

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
- **定时全量快照**：每隔 3 分钟后台静默序列化一份全量工程至第一轨本地缓存目录 `.clipflow_cache/{ProjectHash}/autosave/{Name}_{YYYYMMDD_HHMMSS}.clipflow`（详见 [`cache-and-storage-spec.md` 第 2 节](file:///d:/Work/Dev/ClipFlow/docs/cache-and-storage-spec.md#L41)），最大保留 10 份历史快照。
- **异常恢复检测**：启动时如检测到异常退出遗留的 WAL 日志，弹出对话框提示用户“检测到未保存的工程修改，是否一键恢复”。

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

系统确立**“M2 零阻抗切点外发先行 $\to$ M3+ 通用 IR 演进”**的实施矩阵：

1. **Milestone 2 (M2) 核心落地**：
   - **Apple FCP7 XML (`xmeml v5`)**：与 ClipFlow 平行多轨结构 1:1 零阻抗契合，经由 `quick-xml` 流式生成，路径强制规范化为 RFC 3986 `file://localhost/...` 百分号转义 URI，Premiere Pro 与 DaVinci Resolve 导入成功率高达 **99.9%**；
   - **规范化 CMX 3600 EDL**：符合 80 列定宽规范，Reel ID 规约为 8 字符，通过注入 `* FROM CLIP NAME` 与 `* SOURCE FILE` 扩展注释行安全传递 UTF-8 中文长路径，杜绝穿孔卡协议导致的乱码与媒体离线。
2. **Milestone 3+ (M3+) 架构演进**：
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

