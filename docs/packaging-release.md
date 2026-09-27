# 打包分发与更新方案 (Packaging & Release)

> **版本**：v1.0.0  
> **更新时间**：2026-09-26  
> **适用技术栈**：Rust 1.98, Python 3.13, Node.js 24 LTS, FFmpeg 9.0.2, NSIS, signtool  
> **核心地位**：规范 Windows 端复合多运行时目录布局、分级安装包（Lite/Full）编排、NSIS 打包脚本、代码签名与静默热更新机制。

---

## 1. 交付形态与发布目标 (Tiered Packaging)

为了彻底解决“Python + CUDA + Node + FFmpeg”复合架构导致的 3GB+ 体积黑洞，系统提供两种发布形态：

1. **轻量核心安装包 (`ClipFlow-Setup-Lite.exe`，推荐，体积 $\le 250\text{ MB}$)**：
   - 内置：Rust 主进程、轻量嵌入式 Python 3.13 (纯 CPU 推理)、精简版 FFmpeg 9.0.2、Node.js 24 LTS 与无头 Chromium 动效运行时；
   - 动效预览依托 Node.js 24 无头 Chromium 渲染池，通过 Win32 命名共享内存向 `wgpu 30.0` 直传 Raw RGBA（零磁盘 IO、单帧延迟 $\le 1.5\text{ms}$）；
   - 任何无显卡轻薄本或普通 PC 可极速下载，开箱即用；首次转写时按需拉取 `whisper-large-v2` INT8 模型或外置导入。
2. **全量离线专业版 (`ClipFlow-Setup-Full.exe`，供局域网与内网工作室)**：
   - 预捆绑 CUDA 12.x / cuDNN 运行时与 `whisper-large-v2` INT8 权重（体积约 2.2GB，解压即部署至 `%LOCALAPPDATA%\ClipFlow\models\`），解压即享满血 GPU 加速。

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
│   │   ├── package.json
│   │   └── chromium/               # 精简无头 Chromium (~70MB，仅保留无头渲染核心与必备 dll)
│   ├── bin/
│   │   ├── ffmpeg.exe              # FFmpeg 9.0.2
│   │   └── ffprobe.exe             # FFprobe 9.0.2
│   └── templates/                  # 预置动效模板库 (HTML/CSS/JS)
```

> **存储分离说明**：依据 [cache-and-storage-spec.md](cache-and-storage-spec.md)“双轨分离制”，大体积 AI 模型严禁随软件安装包本地打包或随工程目录重复复制，统一集中寻址于 `%LOCALAPPDATA%\ClipFlow\models\faster-whisper-large-v2\`。

### 2.1 硬件感知按需扩展机制 (On-Demand Acceleration Packs)
- **GPU 加速包自动识别**：
  软件启动时探测本地 GPU 设备。若发现 NVIDIA 独显（RTX 20/30/40 系列及更高），且 `resources/cuda_runtime/` 为空，在设置面板弹出轻量提示：
  *“检测到您的设备支持 NVIDIA GPU 极速转写与硬件加速，是否一键下载 GPU 增强包 (约 600MB)？”*
- **国内高速镜像源与断点续传**：
  `whisper-large-v2` 模型权重（INT8 约 1.5GB）与 CUDA 加速包统一接入国内高速 CDN 节点与阿里 ModelScope 开源镜像，支持断点续传与后台静默校验（SHA-256），下载中途退出可随时恢复。
- **HyperFrames 动效运行时治理**：
  动效层由 `resources/hyperframes_runtime/chromium/` 裁切版无头集群双 Worker（`PingPongPoolManager`）驱动（剔除所有不必要的音视频解码器与语言包，体积压减至 ~70MB），启动配置硬限 `--force-gpu-mem-available-mb=512` 与 120 帧周期内存主动清洗，单帧 Raw RGBA 直灌 wgpu 延迟 $\le 1.5\text{ms}$，兼顾超轻预览与母带级逐帧确定性。

---

## 3. 代码签名与分发安全 (Windows)
- 使用微软官方签名工具 `signtool.exe`，针对 `ClipFlow.exe` 及所有捆绑的 `.exe` 和 `.dll` 进行 SHA-256 签名：
  ```powershell
  signtool sign /fd SHA256 /tr http://timestamp.digicert.com /td SHA256 /a "dist/ClipFlow.exe"
  ```
- 生产环境建议使用受信任的代码签名证书（OV/EV），避免 Windows Defender SmartScreen 拦截。

---

## 4. 自动更新与热补丁策略 (Auto-Update)

### 4.1 更新清单协议 (`latest.json`)
主进程启动时以非阻塞后台任务向版本 CDN 请求 `latest.json`：

```json
{
  "version": "1.1.0",
  "min_compatible_version": "1.0.0",
  "release_date": "2026-10-15T00:00:00Z",
  "mandatory": false,
  "changelog": "1. 提升 4K 时间轴多轨缩放流畅度\n2. 新增 3 款科技风动态角标模板",
  "packages": {
    "full": {
      "url": "https://cdn.clipflow.dev/releases/ClipFlow-Setup-1.1.0.exe",
      "sha256": "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
      "size_bytes": 245367800
    },
    "differential": {
      "url": "https://cdn.clipflow.dev/releases/diff/diff-1.0.0-to-1.1.0.pck",
      "sha256": "ca978112ca1bbdcafac231b39a23dc4da786eff8147c4e72b9807785afee48bb",
      "size_bytes": 18452000
    }
  }
}
```

### 4.2 提权替换与失败自动回滚
1. **下载与校验**：优先下载增量差量包（若版本跨度过大则回退全量包），下载完毕在独立临时工作区完成 SHA-256 哈希完整性校验。
2. **原子无锁热替换**：
   - 用户点击“立即重启更新”后，主进程拉起独立的轻量级提权更新辅助器 `resources/bin/updater.exe` 并正常退出自身；
   - `updater.exe` 等待主进程完全释放文件句柄后，将原主程序备份重命名为 `ClipFlow.exe.bak`，再将新二进制覆写替换；
3. **回滚容灾策略**：
   - 替换完成后自动启动新版 `ClipFlow.exe --health-check`；
   - 若在启动后 3 秒内发生崩溃、缺失依赖或 panic，`updater.exe` 立即恢复 `ClipFlow.exe.bak` 备份并告警提示，确保用户生产环境 100% 可用。

