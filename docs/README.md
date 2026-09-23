# ClipFlow 文档索引

本文档目录维护 ClipFlow 桌面应用的工程规范、架构蓝图与交付标准，作为全生命周期开发的单一事实来源（SSOT）。

## 核心定位与技术栈摘要

- **定位**：Windows 端以 **剪辑 Agent（导演级 AI）** 为核心的智能桌面视频制作软件。
- **界面范式**：
  - **底部工作流导航**：严格遵循**达芬奇 (DaVinci Resolve) Dock 栏分页模式**（6 大分页：`Agent`、`剪辑`、`动画`、`声音`、`图片`、`导出`）。
  - **核心骨架**：**全部页面公用一条时间线**（多轨数据、播放指针与硬解播放状态跨页面无缝漫游）。
  - **专业剪辑台**：`剪辑` 界面**完全对齐 Premiere Pro (PR)** 经典四区分屏与多轨时间轴布局。
- **技术拓扑**：
  - **主进程**：Rust 1.98 + wgpu 30.0 + egui 0.36 + FFmpeg 9.0.2
  - **智能子进程**：Python 3.13 (`uv` 管理) + `faster-whisper` 1.2.1 (`large-v2`, INT8)
  - **动效引擎**：继承 `HyperFrames`（HTML/CSS/JS 确定性逐帧渲染）

---

## 文档清单

| 序号 | 文档名称 | 主要内容 | 状态 |
| :--- | :--- | :--- | :--- |
| 01 | [产品与需求规格 (prd.md)](file:///d:/Work/Dev/ClipFlow/docs/prd.md) | 导演级 Agent 定位、口播智能剪辑、达芬奇 Dock 栏 6 大工作流、PR 剪辑界面对齐标准 | 已对齐最新架构 |
| 02 | [系统架构设计 (architecture.md)](file:///d:/Work/Dev/ClipFlow/docs/architecture.md) | Rust(wgpu+egui) 主进程、Python(Whisper) 子进程、HyperFrames 动效引擎三层多进程拓扑 | 已对齐最新架构 |
| 03 | [进程间通信协议 (ipc-protocol.md)](file:///d:/Work/Dev/ClipFlow/docs/ipc-protocol.md) | Rust 与 Python 之间的 ASR 转写、口播切分、HyperFrames 渲染任务下发接口定义 | 已对齐最新架构 |
| 04 | [开发环境与构建指南 (environment-setup.md)](file:///d:/Work/Dev/ClipFlow/docs/environment-setup.md) | Rust 工具链、MSVC、Python 3.13 (`uv`)、FFmpeg 9.0.2、Node 24 本地调试流程 | 已对齐最新架构 |
| 05 | [打包分发与更新方案 (packaging-release.md)](file:///d:/Work/Dev/ClipFlow/docs/packaging-release.md) | 复合多运行时目录布局编排、NSIS 打包、代码签名与更新方案 | 已对齐最新架构 |
| 06 | [UI 设计与视觉规范 (ui-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/ui-spec.md) | 继承 NVIDIA 电能绿与工控美学、对齐 Windows 11 原生圆角曲率、egui 0.36 主题映射 | 新增完成 |
| 07 | [剪辑工作台布局规范 (edit-layout-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/edit-layout-spec.md) | 完全对齐 Premiere Pro 经典四区分屏、六大功能区深度规格与公用时间线深度实现 | 新增完成 |
| 08 | [时间轴数据模型与工程持久化规范 (timeline-data-model.md)](file:///d:/Work/Dev/ClipFlow/docs/timeline-data-model.md) | 亚毫秒 RationalTime、Rust 核心数据结构、Undo/Redo 事务系统与 .clipflow 容器规范 | 新增完成 (P0) |
| 09 | [多媒体管线与音画同步渲染规范 (media-pipeline-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/media-pipeline-spec.md) | FFmpeg 9.0.2 D3D11VA 硬解、wgpu 30.0 纹理流水线、音频时钟 A/V Sync、波形与金字塔缓存 | 新增完成 (P0) |
| 10 | [导演级 Agent 协议与剪辑指令集规范 (agent-director-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/agent-director-spec.md) | 导演大模型感知规划、Tool Calling 时间轴剪辑指令集、DirectorPlan Schema 与 Agent 工作台规格 | 新增完成 (P1) |
| 11 | [HyperFrames 动效引擎集成规范 (hyperframes-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/hyperframes-spec.md) | Web 动效模板标准 (HTML/CSS/GSAP)、虚拟时间逐帧确定性渲染、双通道预览导出与预置资产库 | 新增完成 (P1) |
| 12 | [代码仓库架构与开发路线图 (codebase-and-roadmap.md)](file:///d:/Work/Dev/ClipFlow/docs/codebase-and-roadmap.md) | Cargo Workspace 多 Crate 划分、依赖拓扑、M0~M4 渐进式开发里程碑与核对验收标准 | 新增完成 (P2) |
| 13 | [辅助工作流页面规格 (workflow-pages-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/workflow-pages-spec.md) | 达芬奇 Dock 栏其余页面深度规格：【声音】混音台与 AI 降噪、【图片】封面制作、【导出】母带渲染 | 新增完成 (P2) |
| 14 | [性能基准与质量验收规范 (qa-and-benchmarks.md)](file:///d:/Work/Dev/ClipFlow/docs/qa-and-benchmarks.md) | 音画同步误差阈值 (≤16.6ms)、60FPS渲染、ASR/粗剪准确率红线与自动化测试套件 | 新增完成 (P3) |




