# ClipFlow

ClipFlow 是一款以**剪辑 Agent（导演级 AI）**为核心的智能桌面视频制作软件。采用 Rust 宿主与 Python 智能辅助进程的双语言架构，提供对齐专业工业级剪辑台（Premiere Pro / DaVinci Resolve）的底层性能与操作范式。

## 核心技术栈

- **主进程 (GUI & 媒体渲染)**：Rust 1.99, wgpu 30.0, egui 0.36, FFmpeg 9.0.2
- **子进程 (AI & 算法支撑)**：Python 3.13 (`uv`), faster-whisper 1.2.1

## 核心架构与文档规范

ClipFlow 全局技术规格、架构蓝图与交付标准收敛于单一事实来源（SSOT）。
详情请参阅工程核心导航索引：[docs/README.md](docs/README.md)

## 快速启动建议

```powershell
# 1. 验证 Rust 基础工作区编译状态
cargo check --workspace

# 2. 验证 Python 子进程环境 (依赖 uv)
uv run python --version
```

