# 多媒体管线与音画同步渲染规范 (Media Pipeline & A/V Sync Specification)

> **版本**：v1.0.0  
> **更新时间**：2026-09-23  
> **适用技术栈**：Rust 1.98, FFmpeg 9.0.2 (MSVC 静态/动态绑定), wgpu 30.0 (DirectX 12 / Vulkan), cpal 0.15  
> **核心地位**：规范从多媒体硬解、像素格式转换、GPU 纹理上传、音频驱动时钟同步到多轨离屏母带导出的完整管线。

---

## 1. 总体多媒体拓扑与并发线程模型

ClipFlow 针对 4K 60fps 场景的低延迟与平滑播放要求，采用**四线程解耦流式架构**：

```mermaid
flowchart LR
    subgraph Audio_Domain ["音频主时钟域 (实时性最高)"]
        AudioDev["音频硬件输出 (cpal)"]
        AudioRing["音频采样环形缓冲 (RingBuffer)"]
        MasterClock["音频主时钟 (Master Clock)\nt = consumed_samples / sample_rate"]
        AudioRing --> AudioDev
        AudioDev --> MasterClock
    end

    subgraph Video_Decode_Domain ["视频解码异步工作线程"]
        FFmpegCore["FFmpeg 9.0.2 解码器\n(D3D11VA 硬解 / 软解回退)"]
        FramePool["L1 解码视频帧缓存池\n(PTS 有序环形队列, 60-120 帧)"]
        MediaFile["原始媒体文件"] --> FFmpegCore
        FFmpegCore --> FramePool
    end

    subgraph GPU_Render_Domain ["UI 与合成主线程 (60+ FPS)"]
        UI_Loop["egui 0.36 界面轮询"]
        SyncController["音画同步协调器 (A/V Sync)"]
        WgpuPipeline["wgpu 30.0 渲染通道\n(NV12->RGBA 色彩转换着色器)"]
        Monitors["源监视器 / 节目监视器 (TextureId)"]

        MasterClock -.->|当前基准时间戳| SyncController
        FramePool -->|匹配时钟的帧| SyncController
        SyncController --> WgpuPipeline --> Monitors --> UI_Loop
    end
```

---

## 2. 视频硬解与 wgpu 30.0 纹理渲染管线

### 2.1 FFmpeg 9.0.2 硬件加速解码策略 (Windows D3D11VA)

1. **硬件加速初始化**：
   - 首选 `AV_HWDEVICE_TYPE_D3D11VA`（Windows 原生 DirectX 11/12 交互友好）。
   - 显卡驱动不可用或格式不受支持时，平滑降级至 `AV_HWDEVICE_TYPE_DXVA2`；最终兜底多线程 CPU 软解（`threads = available_cores`）。
2. **解码输出格式**：
   - 硬件解码直出格式为 `AV_PIX_FMT_NV12`（Y 亮度单平面 + UV 色度交错平面）。
   - **零 CPU 转换原则**：严禁在 CPU 侧调用 `sws_scale` 将 NV12 转为 RGBA（会导致 4K 高分辨率下 CPU 吞吐暴跌）。直接将 NV12 两个原始内存平面上传至 GPU。

### 2.2 锁页内存环形池 (PinnedFramePool) 与 PCIe DMA 极速直传

为彻底杜绝 4K 60fps 场景下每秒处理 746MB 数据引发的 Windows 堆管理器锁竞争与页面换出（Page Fault），`clipflow-media` 严格执行**锁页内存环形池**与 **PCIe DMA 零分配直传**：

1. **预分配与物理页锁定**：
   - 系统初始化时调用 Windows `VirtualAlloc` 一次性预分配固定容量（默认 64 槽位，单槽 12.5MB）的环形池，并通过 `VirtualLock` 钉住物理内存，禁止操作系统将帧缓冲换出到页面文件；
   - 内存对齐严格遵循 4KB 页面边界与 64-byte CPU 缓存行对齐，确保 PCIe 控制器以最大突发（Burst DMA）吞吐直传。
2. **解码与上传双端对象复用**：
   - FFmpeg 解码线程从 `PinnedFramePool` 获取空闲槽位，调用 `av_hwframe_transfer_data` 将显存数据直接写入锁页内存；
   - 主渲染线程调用 `queue.write_texture` 直接将锁页数据提交给 GPU，消费完毕后立即将槽位归还空闲队列，实现**连续播放期间堆内存 0 分配、0 抖动**。

### 2.3 NV12 双平面 GPU 上传与 WGSL 全色域自适应色彩空间矩阵 (`ColorMatrixEngine`)

解码器直出的 NV12 数据直接通过 PCIe 传输原始双平面数据至 GPU 显存，wgpu 端创建两个独立纹理：
- **`texture_y`**：格式 `wgpu::TextureFormat::R8Unorm`（分辨率 $W \times H$）
- **`texture_uv`**：格式 `wgpu::TextureFormat::Rg8Unorm`（分辨率 $W/2 \times H/2$）

#### WGSL 色彩空间矩阵转换着色器 (`nv12_to_rgba.wgsl`)
支持 **BT.709 (HD/4K)** 与 **BT.601 (标清)**，并内置**有限范围（Limited Range 16-235）与全范围（Full Range 0-255）自动适应机制**，彻底消除暗部死黑与高光发灰，色彩精度锁定在 $\Delta E_{00} \le 0.5$：

```wgsl
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

@group(0) @binding(0) var sampler_linear: sampler;
@group(0) @binding(1) var texture_y: texture_2d<f32>;
@group(0) @binding(2) var texture_uv: texture_2d<f32>;

struct ColorParams {
    matrix_row0: vec4<f32>,
    matrix_row1: vec4<f32>,
    matrix_row2: vec4<f32>,
    yuv_offset: vec4<f32>,
};
@group(0) @binding(3) var<uniform> params: ColorParams;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    // 全屏大三角形算法 (无需顶点缓冲)
    let x = f32(i32(in_vertex_index & 1u) * 4 - 1);
    let y = f32(i32(in_vertex_index & 2u) * 2 - 1);
    out.position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>((x + 1.0) * 0.5, (1.0 - y) * 0.5);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    // 采样 Y 单通道与 UV 双通道 (硬件双线性插值自动完成 UV 2x 上采样)
    let y = textureSample(texture_y, sampler_linear, in.tex_coords).r;
    let uv = textureSample(texture_uv, sampler_linear, in.tex_coords).rg;

    // 扣除 YUV 动态偏置 (Limited Range 下 Y-0.062745, UV-0.50196)
    let yuv = vec3<f32>(y, uv.x, uv.y) - params.yuv_offset.xyz;

    // 无分支矩阵点乘快速转为标准 RGB
    let r = dot(params.matrix_row0.xyz, yuv);
    let g = dot(params.matrix_row1.xyz, yuv);
    let b = dot(params.matrix_row2.xyz, yuv);

    return vec4<f32>(clamp(vec3<f32>(r, g, b), vec3<f32>(0.0), vec3<f32>(1.0)), 1.0);
}
```

### 2.4 监视器视窗与 `egui::TextureId` 绑定

- 在每帧 egui 渲染阶段前，`wgpu` 将片段合成至离屏帧缓冲区（Off-screen Framebuffer）。
- 通过 `egui_wgpu::Renderer::register_native_texture` 注册为 `egui::TextureId`。
- 源监视器和节目监视器只需在 UI 代码中调用 `ui.image(egui::load::SizedTexture::new(tex_id, size))` 即可实现原生 GPU 硬件零损耗呈现。

---

## 3. 音画同步 (A/V Sync) 与主时钟调度算法

### 3.1 为什么采用音频作为主时钟 (Audio Master Clock)

人耳对声音的卡顿、爆音和断续容忍度极其苛刻（$\ge 5\text{ms}$ 的突变即可察觉），而人眼对视频画面轻微的帧等待或丢帧具有自然融合容忍度。因此，**系统时钟严格以音频消费游标作为 Master Clock**。

### 3.2 采样点基准时钟计算

通过 `cpal` 音频驱动设备回调：

$$\text{Time}_{\text{audio}} = \frac{\text{TotalSamplesConsumed}}{\text{SampleRate}} + \text{DeviceOutputLatency}$$

- `SampleRate` 统一在主混音器重采样为 **48000 Hz 32-bit Float**。
- `DeviceOutputLatency` 动态查询 Windows WASAPI 驱动缓冲时延（一般为 5ms ~ 15ms）。

### 3.3 视频渲染决策状态机

在主线程每次执行画面重绘时，比对当前待显示视频帧的显示时间戳 $PTS_{\text{video}}$ 与 $\text{Time}_{\text{audio}}$ 的差值 $\Delta t = PTS_{\text{video}} - \text{Time}_{\text{audio}}$：

```
                    ┌────────────────────────┐
                    │      计算偏差 Δt       │
                    │  PTS_video - Time_clk  │
                    └───────────┬────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
  Δt > +10ms              -10ms ≤ Δt ≤ +10ms       Δt < -10ms
 视频跑得太快                 【理想同步区】            视频严重滞后
        │                       │                       │
        ▼                       ▼                       ▼
【等待策略 (Hold)】        【渲染本帧 (Render)】     【追赶/丢帧策略】
本帧暂不上屏，保持上一帧，   上传贴图并推进帧队列     │
等待音频时钟追上                                      ├─ -40ms ≤ Δt < -10ms:
                                                      │  立即渲染，不延时
                                                      └─ Δt < -40ms:
                                                         直接丢弃当前帧，连续
                                                         读取下一帧直到对齐
```

### 3.4 变速播放与快速拖拽 (Scrubbing) 处理

- **快速拖拽播放头时**：
  1. 瞬间停止音频播放并清空音频 RingBuffer。
  2. 解码管线切换为“快速寻帧（Fast Seek）”模式：仅解码临近的 I/IDR 关键帧与目标帧，其余 B/P 帧跳过计算。
  3. 停止拖拽（Mouse Release）后，以释放点所在帧为基准重新启动音频硬件流。
- **变速播放 (0.5x ~ 2.0x)**：
  - 音频流经由 WSOLA（Waveform Similarity Overlap-Add）变调不变速算法实时重采样处理，确保音频语调正常。

### 3.5 监视器自适应下采样流控调度器 (`ProxyGovernor`) 与背压控制

为避免在常规小面积监视器视口中盲目上传 4K 60fps 原图造成总线竞争，并保证高速拖拽时毫秒级跟手响应，集成三态自适应流控调度器：

1. **三态分辨率调度矩阵**：
   - **`Full` (1/1, 3840x2160, ~746 MB/s)**：视频暂停（`Paused`）或单帧精修时激活，保障调色与画面细节核验达到广播级所见即所得；
   - **`Half` (1/2, 1920x1080, ~186 MB/s)**：常规播放（`PlayingNormal`）且物理视口宽度 $\le 1440\text{px}$ 时自适应切入，总线带宽节省 $75\%$；
   - **`Quarter` (1/4, 960x540, ~46 MB/s)**：快速拖拽（`ScrubbingFast`）或多机位/多轨并发 $\ge 3$ 时强行激活，拖拽响应锁定在 $\le 25\text{ms}$，PCIe 吞吐死锁在 $\le 60\text{ MB/s}$。
2. **深度 $\le 3$ 严格背压流控（Backpressure Flow Control）**：
   - 解码线程与 GPU 上传队列之间通过 `sync::Condvar` 实现流控门禁。未消费就绪帧满 3 帧时，解码工作线程自动挂起；GPU 消费一帧后立刻通知唤醒，杜绝音画漂移累积与内存无序膨胀。

---

## 4. 高性能多级缓存系统设计

为了支撑剪辑台高频拖拽、百轨并发与实时声波渲染，设立三级专用缓存机制：

### 4.1 L1 视频帧解码环形缓冲池 (Decoded Frame Ring Buffer)

- **容量**：常驻内存 60 ~ 120 帧（1080p 约占用 200MB ~ 400MB 内存）。
- **策略**：
  - 双向预读：以播放头为中心，向前（Future）预读 70% 帧，向后（Past）保留 30% 帧（便于即时单帧后退或慢速倒放）。
  - LRU 淘汰机制：一旦内存压力超过设定的水位线（例如 1.5GB），优先淘汰离当前时间码最远的非关键帧。

### 4.2 L2 缩略图金字塔 (Thumbnail Pyramid)

- **生成时机**：素材导入工程时，后台低优先级线程池启动抽帧。
- **物理布局**：存储于工程临时的 `.thumbs/` 目录下，命名为 `{asset_sha256}_{step_ms}.bin`。
- **规格**：
  - 缩略图固定规格：高度 72px，宽度按素材画幅自适应（如 128px）。
  - 存储格式：轻量 WebP 或原始 RGB565 压缩块。
  - **缩放级联**：
    - Level 0 (大缩放)：每 1 秒一张图；
    - Level 1 (中缩放)：每 5 秒一张图；
    - Level 2 (全局视角)：每 30 秒一张图。时间轴缩放时刻直接根据当前每像素代表时长读取对应 Level，极速流畅。

### 4.3 音频波形峰值缓存文件 (`.peak`)

为防止在多轨时间轴展开时频繁解码长音频导致界面卡死，ClipFlow 采用专业音频 DAW 级的峰值缓存协议：

```
+-------------------------------------------------------------------+
| Header: "CFPK" (4B) | Version (2B) | Channels (2B) | SampleRate (4B) |
+-------------------------------------------------------------------+
| WindowSize: u32 (默认 256 采样点一个数据包)                          |
+-------------------------------------------------------------------+
| Peak Records Array:                                               |
| [                                                                 |
|   { min_sample: i16, max_sample: i16, rms_energy: u16 }, ...       |
| ]                                                                 |
+-------------------------------------------------------------------+
```

- **极速渲染**：40 分钟口播音频的 `.peak` 文件大小仅约为 **1.2 MB**。
- **UI 绘制映射**：egui 在时间轴上绘制波形时，只需读取可视区域对应区间的 Min/Max 序列，一次性构造为 GPU 三角形网格（`egui::Mesh`），毫秒级 60fps 拖拽无卡顿。

### 4.4 波形 LOD 动态级联与防抖规范 (Waveform LOD & Hysteresis)

为杜绝时间轴连续缩放（Zoom In/Out）期间波形频繁重构造成的掉帧与锯齿跳变闪烁（Aliasing Popping），建立 4 级 LOD 与滞后防抖机制：

1. **4 级 LOD 金字塔划分**：
   - **LOD 0 (宏观全景)**：$pps < 10.0\text{ px/s}$，单像素代表 2048 采样点；
   - **LOD 1 (中观概览)**：$10.0 \le pps < 60.0\text{ px/s}$，单像素代表 512 采样点；
   - **LOD 2 (精细剪辑)**：$60.0 \le pps < 300.0\text{ px/s}$，单像素代表 128 采样点；
   - **LOD 3 (单帧精修)**：$pps \ge 300.0\text{ px/s}$，单像素代表 32 采样点或直读原始样本。
2. **$\pm 15\%$ 切换滞后门限（Hysteresis Thresholding）**：
   - 上升门限与下降门限分离（如 LOD 1 升至 LOD 2 需突破 $65.0\text{ pps}$，降回 LOD 1 需跌破 $55.0\text{ pps}$）。在死区区间内严格保持已有网格，仅做 GPU 矩阵线性拉伸，消除临界震荡。
3. **`WaveformMeshHolder` 局部保留与复用**：
   - 时间轴主线程常驻持有当前音轨的 `egui::Mesh` 句柄；在视口未跨级且未超出覆盖时间窗口时，直接复用已有网格，重绘耗时锁定为 $0.0\text{ms}$；仅在跨级时异步构建新网格，单帧构建限额 $\le 0.9\text{ms}$。

---

## 5. 母带导出与硬件编码管线

导出页面触发母带导出时，主进程启动专用的离屏批处理渲染上下文：

```mermaid
flowchart TD
    TimelineSeq["遍历时间轴序列帧 (Frame by Frame)"] --> Compositor["GPU 合成器 (wgpu)"]
    
    subgraph Multi_Layer_Blend ["多层图层叠加"]
        V_Track["视频轨 (V1-V3) 缩放与位置变换"]
        FX_Track["HyperFrames 透明动效层 (Alpha 通道)"]
        Sub_Track["C1 字幕轨 (文字栅格化)"]
        V_Track --> Compositor
        FX_Track --> Compositor
        Sub_Track --> Compositor
    end

    Compositor -->|离屏渲染无损帧| OffscreenTexture["离屏纹理 (RGBA)"]
    OffscreenTexture --> PixelBuffer["Direct3D11 / 内存缓冲共享"]
    
    subgraph FFmpeg_Muxer ["FFmpeg 9.0.2 硬件编码输出"]
        PixelBuffer --> VideoEncoder["NVENC (H.264 / HEVC) 硬件编码器"]
        AudioMixer["多轨全局混音器 (Master Bus)"] --> AudioEncoder["AAC 编码器 (320kbps)"]
        VideoEncoder --> Muxer["MP4 / MOV 容器封装"]
        AudioEncoder --> Muxer
        Muxer --> TargetFile["最终输出母带视频 (.mp4)"]
    end
```

### 5.1 导出特性与质量规约
- **背压控制 (Backpressure)**：GPU 合成帧率与 NVENC 编码吞吐量严格保持流控队列深度 $\le 3$，防止内存无限制溢出。
- **色彩空间一致性**：导出渲染通道强制使用 Rec.709 全范围（Full Range）或限制范围（Limited Range，TV标准），与节目监视器预览所见即所得。

---

## 6. 多媒体管线三级容灾降级与 DeviceLost 自愈状态机

为保障桌面客户端在 Windows 异构环境下的极致稳定性，多媒体管线集成自动化容灾看门狗：

### 6.1 三级解码平滑降级梯队
1. **第一梯队（默认首选）**：`AV_HWDEVICE_TYPE_D3D11VA` 专用硬件加速芯片解码；
2. **第二梯队（驱动异常回退）**：`AV_HWDEVICE_TYPE_DXVA2` 兼容模式；
3. **第三梯队（终极兜底）**：多线程 CPU 软解（`threads = available_cores`）。若显卡硬件驱动彻底崩溃或遇畸变损坏流，50ms 内无感切换至软解，**100% 杜绝软件闪退**。

### 6.2 DirectX 12 `DeviceLost` 毫秒级无感自愈
当遭遇显示器休眠唤醒、HDR 切换或显卡驱动 TDR 超时重置（`DXGI_ERROR_DEVICE_RESET`）时：
- 主进程捕获错误并触发轻量级设备重建；
- 冻结当前时间码（PTS 保持不变），重新初始化 `wgpu::Device` 与着色器管道；
- 直接从 `PinnedFramePool` 提取当前帧重新上屏，恢复耗时 $\le 100\text{ms}$，撤销事务栈与用户编辑数据零丢失。

---

## 7. 远期演进：FFmpeg 9.0.2 原生 D3D12VA 零拷贝与接口解耦

针对远期百轨 8K RAW 等极端重载工况，ClipFlow 建立清晰的技术路线分水岭：

1. **废止基于 `wgpu-hal` 跨 API 共享（路线 B）**：严禁在生产代码中使用脆弱的 D3D11-to-D3D12 DXGI NT Handle 穿透；
2. **确立路线 C 终局演进方向**：FFmpeg 9.0.2 原生支持 `D3D12VA`。远期通过向 FFmpeg 传递 `wgpu` 底层的同一个 `ID3D12Device` 实体，实现同设备显存直接复用与队列内 Fence 同步，达成真正的同 API 物理零拷贝；
3. **`VideoTextureProvider` 依赖反转契约**：上层时间轴与监视器仅依赖统一 trait 接口，底层可无缝在路线 A（锁页内存上传）、路线 C（D3D12VA 零拷贝）与 CPU 软解间自由切换，上层业务代码零改动。

