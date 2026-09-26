# ClipFlow 文档索引 (Documentation SSOT)

> **版本**：v1.0.0  
> **更新时间**：2026-09-26  
> **适用技术栈**：Rust 1.98, wgpu 30.0, egui 0.36, FFmpeg 9.0.2, Python 3.13 (`uv`), Node.js 24 LTS  
> **核心地位**：ClipFlow 全局技术规格单一事实来源（SSOT）导航索引，收录 18 篇技术子规范与研发路线图。

---

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
| 06 | [UI 设计与视觉规范 (ui-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/ui-spec.md) | 采用 OpenDesign Neutral Modern 深色设计体系、深岩灰底板、分层中性表面、钴蓝交互信号、egui 0.36 主题映射 | 已升级对齐 |
| 07 | [剪辑工作台布局规范 (edit-layout-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/edit-layout-spec.md) | 完全对齐 Premiere Pro 经典四区分屏、六大功能区深度规格与公用时间线深度实现 | 新增完成 |
| 08 | [时间轴数据模型与工程持久化规范 (timeline-data-model.md)](file:///d:/Work/Dev/ClipFlow/docs/timeline-data-model.md) | 亚毫秒 RationalTime、Rust 核心数据结构、Undo/Redo 事务系统与 .clipflow 容器规范 | 新增完成 (P0) |
| 09 | [多媒体管线与音画同步渲染规范 (media-pipeline-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/media-pipeline-spec.md) | FFmpeg 9.0.2 D3D11VA 硬解、wgpu 30.0 纹理流水线、音频时钟 A/V Sync、波形与金字塔缓存 | 新增完成 (P0) |
| 10 | [导演级 Agent 协议与剪辑指令集规范 (agent-director-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/agent-director-spec.md) | 导演大模型感知规划、Tool Calling 时间轴剪辑指令集、DirectorPlan Schema 与 Agent 工作台规格 | 新增完成 (P1) |
| 11 | [HyperFrames 动效引擎集成规范 (hyperframes-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/hyperframes-spec.md) | Web 动效模板标准 (HTML/CSS/GSAP)、虚拟时间逐帧确定性渲染、双通道预览导出与预置资产库 | 新增完成 (P1) |
| 12 | [代码仓库架构与模块职责规范 (codebase-structure.md)](file:///d:/Work/Dev/ClipFlow/docs/codebase-structure.md) | Cargo Workspace 多 Crate 划分、单向无环依赖拓扑与模块职责边界 | 新增完成 (P2) |
| 13 | [辅助工作流页面规格 (workflow-pages-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/workflow-pages-spec.md) | 达芬奇 Dock 栏其余页面深度规格：【声音】混音台与 AI 降噪、【图片】封面制作、【导出】母带渲染 | 新增完成 (P2) |
| 14 | [性能基准与质量验收规范 (qa-and-benchmarks.md)](file:///d:/Work/Dev/ClipFlow/docs/qa-and-benchmarks.md) | 音画同步误差阈值 (≤16.6ms)、60FPS渲染、ASR/粗剪准确率红线与自动化测试套件 | 新增完成 (P3) |
| 15 | [工程缓存、临时工作区与资产寻址规范 (cache-and-storage-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/cache-and-storage-spec.md) | 双轨分离制拓扑、工程同级 .clipflow_cache、%LOCALAPPDATA% 模型共享与 LRU 淘汰 | 新增完成 (P0) |
| 16 | [字幕样式排版与 GPU 监视器文本渲染规范 (subtitle-render-spec.md)](file:///d:/Work/Dev/ClipFlow/docs/subtitle-render-spec.md) | wgpu 离屏文本着色管线、cosmic-text/glyphon、词级卡拉OK高亮与双模剪辑联动 | 新增完成 (P1) |
| 17 | [工程研发路线图与原子任务看板 (roadmap.md)](file:///d:/Work/Dev/ClipFlow/docs/roadmap.md) | M0~M4 全周期 48 个原子任务、时序依赖、验收命令 (DoD) 与任务追踪看板 | 新增完成 (P0) |
| 18 | [安全威胁模型、沙箱隔离与隐私合规规范 (security-and-privacy.md)](file:///d:/Work/Dev/ClipFlow/docs/security-and-privacy.md) | Chromium 沙箱、命名管道 DACL 权限、XML 防注入与 DPAPI 凭据加密 | 新增完成 (P0) |
