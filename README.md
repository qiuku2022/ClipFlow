# ClipFlow

> **AI 智能口播视频剪辑与动态包装工坊**  
> 面向 Windows 原生平台的高性能视频非线性编辑（NLE）工作台。

---

## 💡 核心设计与交互范式

ClipFlow 融合了专业剪辑工作台的性能与生成式 AI 智能助手的自动化能力，采用三层核心交互拓扑：

1. **达芬奇 (DaVinci Resolve) 式底部工作流 Dock 栏**：
   - 底部 48px 常驻 Dock，提供 6 大业务流直达切页：`Agent 导演`、`剪辑 (Edit)`、`动画 (Motion)`、`声音 (Audio)`、`图片 (Image)`、`导出 (Deliver)`。
2. **全局公用多轨时间线 (Global Shared Timeline)**：
   - 全局公用同一条时间轴核心，多轨数据、播放指针与硬件解码播放状态跨页面无缝漫游，彻底消除切页布局颠簸。
3. **Premiere Pro (PR) 经典双层分屏剪辑工作台**：
   - 上半屏（58%）：**项目素材库 (25%)** + **双联监视器 (50%，源/节目监视器)** + **效果属性检查器 (25%)**，采用刚性几何底板渲染，确保 8px 绝对等宽间隙；
   - 下半屏（42%）：铺满剩余高度的多轨时间线，具备二维视口正交裁剪与 SMPTE 时间码标尺。
4. **硬核媒体管线与硬件单调时钟**：
   - FFmpeg 9.0.2 D3D11VA 硬解 NV12 零拷贝直传至 `wgpu 30.0` WGSL 着色器；
   - WASAPI 原生硬件单调主时钟驱动，配有施密特双阈值迟滞比较器，音画同步误差严格控制在 $\le 16.6\text{ms}$。

---

## 🏗️ 架构拓扑 (Polyglot Monorepo)

- **桌面宿主与渲染主进程 (`Rust 1.98`)**：
  - `clipflow-app`：桌面应用程序主入口，生命周期与窗口事件分发；
  - `clipflow-ui`：基于 `egui 0.36` + `wgpu 30.0` 的即时模式 Neutral Modern 深色界面与时间线渲染；
  - `clipflow-timeline`：纯逻辑多轨数据模型、事务栈与 WAL 预写日志；
  - `clipflow-media`：D3D11VA 硬解流水线、WASAPI 单调时钟、.peak 音频波形 LOD 金字塔；
  - `clipflow-ipc`：Windows Job Object 内核级安全防护与命名管道 JSON-RPC 2.0 通信链路；
  - `clipflow-common`：亚毫秒级 RationalTime 时间数学、区间与 SMPTE 时间码。
- **智能计算子进程 (`Python 3.13` by `uv`)**：
  - `faster-whisper 1.2.1` (`large-v2`, INT8) 本地高精度转写与字级时间戳切分。
- **动态包装引擎 (`Node.js 24 LTS`)**：
  - `HyperFrames` 逐帧确定性渲染，实现现代网页动效角标与包装母带合成。

---

## 🚀 研发进展 (Roadmap Status)

| 里程碑 | 核心主题 | 当前状态 | 核心交付成果 |
| :--- | :--- | :---: | :--- |
| **Milestone 0 (M0)** | 工程骨架与基础通信链路 | ✅ 已交付 | Polyglot Monorepo、Job Object 安全防护、命名管道 IPC、主题与 Dock 栏 |
| **Milestone 1 (M1)** | 媒体硬解播放与 PR 剪辑台 | ✅ 已交付 (v0.1.0-alpha) | PR 经典分屏、D3D11VA 硬解抽象、WASAPI 单调时钟、J-K-L 飞梭走带、中文字体适配 |
| **Milestone 2 (M2)** | 本地 ASR 转写与口播粗剪 | ⏳ 下一阶段 | Whisper INT8 本地转写、口播语气词切除、FCP7 XML/EDL 导出 |
| **Milestone 3 (M3)** | 动效包装与硬件加速导出 | 📋 规划中 | HyperFrames 模板集成、共享内存、NVENC 硬件母带导出 |
| **Milestone 4 (M4)** | 导演级 Agent 编排闭环 | 📋 规划中 | DirectorPlan 大模型剪辑指令集、Tool Calling 双轨持久化 |

---

## ⚡ 快速开始

### 1. 环境准备
确保 Windows 机器已安装：
- **Rust** 1.98 (`x86_64-pc-windows-msvc`)
- **Python** 3.13 (由 `uv` 管理)
- **Node.js** 24 LTS
- **FFmpeg** 9.0.2

### 2. VS Code 一键调试 (推荐)
已在仓库内置完整调试与任务矩阵：
- 打开本项目目录，按 **`F5`** 即可自动触发增量编译并启动 `ClipFlow (Native App)` 断点调试；
- 按 **`Ctrl+Shift+B`** 可一键执行 `cargo build -p clipflow-app`。

### 3. 命令行手动构建与运行
```powershell
# 1. 运行工作区自动化测试 (41 项测试全部通过)
cargo test --workspace

# 2. 启动桌面端应用程序
cargo run -p clipflow-app
```

---

## 📖 技术文档

详细的产品规格、架构设计与子系统规范请参阅 [docs 目录](docs/README.md)。
