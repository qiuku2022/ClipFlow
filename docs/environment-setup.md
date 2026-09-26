# 开发环境与构建指南 (Environment Setup)

本文档明确 ClipFlow 项目在 Windows 平台上的完整工具链依赖与标准调试流程。

## 1. 核心依赖基线表

| 工具 / 运行时 | 指定版本 | 校验命令 | 核心配置与注意点 |
| :--- | :--- | :--- | :--- |
| **操作系统** | Windows 10/11 x64 | `[System.Environment]::OSVersion` | 必须在 PowerShell 7 (`pwsh`) 终端中执行构建与运行 |
| **Rust 工具链** | 1.98 | `rustc --version` / `cargo --version` | 必须安装 `x86_64-pc-windows-msvc` 目标与 Visual Studio C++ Build Tools |
| **图形库 (wgpu)** | 30.0 | - | 支持 DirectX 12 与 Vulkan 后端 |
| **GUI 库 (egui)** | 0.36 | - | 即时模式 UI，由 `eframe 0.36` 驱动桌面窗口 |
| **Python** | 3.13 | `uv run python --version` | 必须由 `uv` 统一管理，根目录以 `.python-version` 钉住 |
| **uv** | 最新稳定版 | `uv --version` | 负责 Python 虚拟环境与依赖管理，严禁使用系统 pip |
| **FFmpeg** | 9.0.2 | `ffmpeg -version` | 严格钉住 9.0.2 版本，用于取帧、静音检测、混音与导出 |
| **Node.js** | 24 LTS | `node -v` | 负责 `HyperFrames` 动效渲染运行时 |
| **ASR 引擎** | faster-whisper 1.2.1 | 模型 `large-v2` | INT8 精度，默认优先 GPU (CUDA)，无 CUDA 自动降级回退 CPU，批大小为 8 |

---

## 2. 初始环境配置步骤

### 2.1 安装 Rust 与 C++ 编译环境
1. 安装 [rustup](https://rustup.rs/) 并配置 MSVC 1.98 工具链：
   ```powershell
   rustup default 1.98-x86_64-pc-windows-msvc
   ```
2. 确保已安装 Visual Studio Build Tools（勾选 "C++ 桌面开发"）。

### 2.2 初始化 Python 计算服务环境 (uv)
在项目根目录下通过 PowerShell 7 执行：
```powershell
# 1. 创建 Python 3.13 虚拟环境
uv venv .venv --python 3.13

# 2. 同步安装 Python 子进程核心依赖 (faster-whisper 1.2.1 等)
uv sync
# 或开发模式可编辑安装：uv pip install -e .
```

### 2.3 配置 HyperFrames 动效环境 (Node.js 24)
```powershell
# 检查 Node.js 24 LTS 环境
node -v

# 安装并初始化 HyperFrames 渲染支持
npm install
```

### 2.4 配置 FFmpeg 9.0.2
确保系统 `PATH` 或项目内置 `bin/` 目录中包含 `ffmpeg.exe` 与 `ffprobe.exe`，版本必须严格匹配 9.0.2：
```powershell
ffmpeg -version
# 期望首行输出：ffmpeg version 9.0.2 ...
```

---

## 3. 日常调试与运行指令

- **启动 Rust 主进程 (开发模式)**：
  ```powershell
  cargo run
  ```
- **单独测试 Python 智能计算子进程 (ASR 与剪辑分析)**：
  ```powershell
  uv run python -m clipflow_worker.main
  ```
- **运行 Rust 单元测试**：
  ```powershell
  cargo test
  ```
