# 打包分发与更新方案 (Packaging & Release)

## 1. 交付形态与发布目标 (Tiered Packaging)

为了彻底解决“Python + CUDA + Node + FFmpeg”复合架构导致的 3GB+ 体积黑洞，系统提供两种发布形态：

1. **轻量核心安装包 (`ClipFlow-Setup-Lite.exe`，推荐，体积 $\le 250\text{ MB}$)**：
   - 内置：Rust 主进程、轻量嵌入式 Python 3.13 (纯 CPU 推理)、精简版 FFmpeg 9.0.2、Node.js 24 LTS 运行时；
   - 动效预览直接复用 Windows 10/11 原生常驻的 **Microsoft Edge WebView2**（开发者打包体积为 0）；
   - 任何无显卡轻薄本或普通 PC 可极速下载，开箱即用。
2. **全量离线专业版 (`ClipFlow-Setup-Full.exe`，供局域网与内网工作室)**：
   - 预捆绑 CUDA 12.x / cuDNN 运行时与 `whisper-large-v2` INT8 权重（体积约 2.2GB），解压即享满血 GPU 加速。

---

## 2. 外部资产与分级运行时目录编排

```
ClipFlow_Release/
├── ClipFlow.exe                    # Rust 主进程编译产物 (Release 模式，已剥离符号)
├── resources/
│   ├── python_runtime/             # 嵌入式 Python 3.13 (默认提供 CPU 支持)
│   │   ├── python.exe
│   │   └── Lib/site-packages/
│   ├── cuda_runtime/               # [按需下载] CUDA 12 / cuDNN 动态链接库 (.dll)
│   ├── hyperframes_runtime/        # HyperFrames 离屏渲染轻量级环境
│   │   ├── node.exe                # Node.js 24 LTS 运行时
│   │   └── package.json
│   ├── bin/
│   │   ├── ffmpeg.exe              # FFmpeg 9.0.2
│   │   └── ffprobe.exe             # FFprobe 9.0.2
│   └── models/
│       ├── whisper-base/           # [内置默认] 极轻量转写模型 (~140MB)，用于基础试用
│       └── whisper-large-v2/       # [按需下载] 满血精度 INT8 权重 (~1.5GB)
```

### 2.1 硬件感知按需扩展机制 (On-Demand Acceleration Packs)
- **GPU 加速包自动识别**：
  软件启动时探测本地 GPU 设备。若发现 NVIDIA 独显（RTX 20/30/40 系列及更高），且 `resources/cuda_runtime/` 为空，在设置面板弹出轻量提示：
  *“检测到您的设备支持 NVIDIA GPU 极速转写与硬件加速，是否一键下载 GPU 增强包 (约 600MB)？”*
- **国内高速镜像源与断点续传**：
  模型权重与 CUDA 加速包统一接入国内高速 CDN 节点与阿里 ModelScope 开源镜像，支持断点续传与后台静默校验（SHA-256），下载中途退出可随时恢复。
- **动效预览零体积方案**：
  交互编辑视窗直接绑定操作系统已有的 `WebView2`，彻底省去捆绑 250MB 独立 Chromium 的开销；仅在最终离屏导出时按需调用轻量无头驱动。

---

## 3. 代码签名与分发安全 (Windows)
- 使用微软官方签名工具 `signtool.exe`，针对 `ClipFlow.exe` 及所有捆绑的 `.exe` 和 `.dll` 进行 SHA-256 签名：
  ```powershell
  signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a "dist/ClipFlow.exe"
  ```
- 生产环境建议使用受信任的代码签名证书（OV/EV），避免 Windows Defender SmartScreen 拦截。

---

## 4. 自动更新与热补丁策略 (Auto-Update)
- 主进程启动时静默向更新服务检查 `latest.json`。
- 支持差量补丁（仅更新 `ClipFlow.exe` 或动效模板库）与全量覆盖升级。

