# 进程间通信与接口协议 (IPC Protocol)

## 1. 概述与通信信道

- **Rust 主进程 (Client/Coordinator)** $\longleftrightarrow$ **Python 智能子进程 (ASR/NLP Worker)**
  - 底层信道：标准输入输出流（`stdin` / `stdout`）或 本地命名管道（Windows Named Pipe `\\.\pipe\clipflow-py-worker`）。
  - 通信格式：换行符分隔的 JSON（NDJSON）或 JSON-RPC 2.0。
- **Rust 主进程 (Coordinator)** $\longrightarrow$ **HyperFrames 动效渲染器 (Worker)**
  - 底层信道：CLI 子进程调用或 本地 HTTP/WebSocket 协议驱动 Headless 浏览器。

---

## 2. Rust 与 Python 子进程通信接口

### 2.1 基础信封定义

#### 请求 (Rust $\to$ Python)
```json
{
  "id": "req-1001",
  "method": "asr.transcribe",
  "params": {
    "media_path": "D:/Footage/raw_talking_head.mp4",
    "language": "zh",
    "compute_type": "int8",
    "batch_size": 8
  }
}
```

#### 响应 (Python $\to$ Rust)
```json
{
  "id": "req-1001",
  "status": "success",
  "data": {
    "task_id": "task-asr-20260923-01",
    "audio_duration_s": 320.5
  }
}
```

#### 实时流式事件 (Python $\to$ Rust Event)
```json
{
  "event": "asr.segment",
  "task_id": "task-asr-20260923-01",
  "data": {
    "start": 12.35,
    "end": 15.80,
    "text": "今天我们来聊一下如何用 Rust 制作现代桌面软件。",
    "words": [
      {"word": "今天", "start": 12.35, "end": 12.80},
      {"word": "我们", "start": 12.80, "end": 13.10},
      {"word": "来聊一下", "start": 13.10, "end": 13.90}
    ]
  }
}
```

### 2.2 口播剪辑分析接口：`speech.analyze_cuts`
用于让 Python 端分析并输出气口、无声段、错句及语气词建议剪除清单：

```json
// 请求
{
  "id": "req-1002",
  "method": "speech.analyze_cuts",
  "params": {
    "task_id": "task-asr-20260923-01",
    "remove_fillers": true,
    "min_silence_duration_ms": 400
  }
}

// 响应
{
  "id": "req-1002",
  "status": "success",
  "data": {
    "suggested_cuts": [
      {
        "type": "silence",
        "start": 4.20,
        "end": 4.95,
        "reason": "长时间停顿 (750ms)"
      },
      {
        "type": "filler",
        "start": 18.10,
        "end": 18.60,
        "text": "那个……",
        "reason": "无意义语气词"
      },
      {
        "type": "repetition",
        "start": 45.10,
        "end": 48.20,
        "reason": "口误重复，保留后半句"
      }
    ]
  }
}
```

---

## 3. Rust 与 HyperFrames 动效渲染协同规范

### 3.1 动效任务下发与离屏渲染
Rust 调度器生成 HTML/CSS 动画描述文件后，向 HyperFrames 触发渲染：

```json
{
  "template_id": "lower_third_minimal",
  "props": {
    "title": "ClipFlow 导演级剪辑 Agent",
    "subtitle": "自动化口播剪辑与动效渲染",
    "duration_frames": 150,
    "fps": 30,
    "width": 1920,
    "height": 1080
  },
  "output_path": "D:/Cache/Render/anim_001.mov",
  "output_codec": "prores_ks",
  "pix_fmt": "yuva444p10le"
}
```

### 3.2 渲染结果上架时间轴
HyperFrames 渲染出带有透明 Alpha 通道的视频（或 PNG 序列）后，Rust 时间轴引擎将其以 `Clip` 形式自动放置在指定视频轨（如 V2）的时间位置。
