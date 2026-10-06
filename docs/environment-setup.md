# 开发环境与构建指南 (Environment Setup)

> **版本**：v0.2.0  
> **更新时间**：2026-10-05  
> **适用技术栈**：Rust 1.99 (MSVC), Python 3.13 (`uv`), FFmpeg 9.0.2, Node.js 24 LTS, pwsh 7  
> **核心地位**：规范 Windows 平台开发环境初始化、工具链版本锁定、构建调试命令与环境诊断基线。

---

## 1. 核心依赖基线表

| 工具 / 运行时 | 指定版本 | 校验命令 | 核心配置与注意点 |
| :--- | :--- | :--- | :--- |
| **操作系统** | Windows 10/11 x64 | `[System.Environment]::OSVersion` | 必须在 PowerShell 7 (`pwsh`) 终端中执行构建与运行 |
| **Rust 工具链** | 1.99 | `rustc --version` / `cargo --version` | 必须安装 `x86_64-pc-windows-msvc` 目标与 Visual Studio C++ Build Tools |
| **图形库 (wgpu)** | 30.0 | - | 支持 DirectX 12 与 Vulkan 后端 |
| **GUI 库 (egui)** | 0.36 | - | 即时模式 UI，由 `eframe 0.36` 驱动桌面窗口 |
| **Python** | 3.13 | `uv run python --version` | 必须由 `uv` 统一管理，根目录以 `.python-version` 钉住 |
| **uv** | 最新稳定版 | `uv --version` | 负责 Python 虚拟环境与依赖管理，严禁使用系统 pip |
| **FFmpeg** | 9.0.2 | `ffmpeg -version` | 严格钉住 9.0.2 版本，用于取帧、静音检测、混音与导出 |
| **Node.js** | 24 LTS | `node -v` | **[M4 阶段启用]** 负责 `HyperFrames` 动效渲染运行时；v1.0 MVP (M0~M2) 阶段无需安装 |
| **ASR 引擎** | faster-whisper 1.2.1 | 模型 `large-v2` | INT8 精度，默认优先 GPU (CUDA)，无 CUDA 自动降级回退 CPU，批大小为 8 |

---

## 2. 初始环境配置步骤

### 2.1 安装 Rust 与 C++ 编译环境
1. 安装 [rustup](https://rustup.rs/) 并配置 MSVC 1.99 工具链：
   ```powershell
   rustup default 1.99-x86_64-pc-windows-msvc
   ```
2. 确保已安装 Visual Studio Build Tools（勾选 "C++ 桌面开发"）。

### 2.2 初始化 Python 计算服务环境 (uv) 与 CUDA 加速
在项目根目录下通过 PowerShell 7 执行：
```powershell
# 1. 创建 Python 3.13 虚拟环境
uv venv .venv --python 3.13

# 2. 同步安装 Python 子进程核心依赖 (faster-whisper 1.2.1 等)
uv sync
# 或开发模式可编辑安装：uv pip install -e .
```

> **GPU 加速运行依赖 (CUDA / cuDNN)**：  
> 若需启用 GPU 极速转写（$\ge 12\times$ 实时倍速），Windows 主机须安装 **CUDA Toolkit 12.x** 与 **cuDNN 9.x for CUDA 12**，并确保 `cudnn64_*.dll` 和 `cublas64_*.dll` 所在目录加入系统 `PATH` 或置于 `resources/cuda_runtime/`；无 CUDA 环境时系统自动降级回退至多线程 CPU 模式。

### 2.3 [远期规划 - Milestone 4 阶段启用] 配置 HyperFrames 动效环境 (Node.js 24)
> **阶段说明**：依据 `roadmap.md` 红线，v1.0 MVP (M0~M2) 周期内**严禁引入 npm 与外部动效包**。本步骤仅在进入 Milestone 4 动效包装研发阶段时按需执行。

```powershell
# 检查 Node.js 24 LTS 环境
node -v

# 安装并初始化 HyperFrames 渲染支持
npm install
```

### 2.4 配置 FFmpeg 9.0.2 运行与编译环境
Rust 媒体管线 crate（`clipflow-media`）通过 `ffmpeg-sys-next` 绑定底层 C API，既需要命令行二进制，也需要 C 头文件与导入库：

1. **运行时二进制 (`ffmpeg.exe` / `ffprobe.exe`)**：  
   确保系统 `PATH` 或项目内置 `resources/bin/` 目录中包含 FFmpeg 9.0.2 可执行程序：
   ```powershell
   ffmpeg -version
   # 期望首行输出：ffmpeg version 9.0.2 ...
   ```
2. **Rust 编译期 C 开发库与环境变量**：  
   下载 FFmpeg 9.0.2 Dev/Shared 开发包（包含 `include/` 与 `lib/` 目录），并配置环境变量指向其根目录：
   ```powershell
   # 设置 FFmpeg SDK 根目录（替换为本地实际解压路径）
   [System.Environment]::SetEnvironmentVariable("FFMPEG_DIR", "C:\ffmpeg-9.0.2-full_build-shared", "User")
   # 确保 bin 目录包含相关 avcodec-*.dll, avformat-*.dll 在 PATH 中
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
