# 工程缓存、临时工作区与资产寻址规范 (Cache & Storage Specification)

> **版本**：v1.0.0  
> **更新时间**：2026-09-26  
> **适用技术栈**：Rust 1.98 (MSVC), Windows 10/11, Python 3.13 (`uv`), faster-whisper 1.2.1  
> **核心地位**：规范全系统媒体代理、音频波形、动效帧、AI 模型及工程快照的物理落盘拓扑、生命周期管理与磁盘配额淘汰机制。

---

## 1. 存储架构设计原则：双轨分离制 (Two-tier Storage Architecture)

为了彻底杜绝“所有临时文件塞入 C 盘导致系统盘爆满”，以及“工程迁移时丢失波形与代理导致重复计算”的两难困境，ClipFlow 确立**工程同级本地缓存**与**系统级全局配置/模型**双轨分离的物理拓扑：

```
+----------------------------------------------------------------------------------------------------+
|                                    ClipFlow 双轨存储物理拓扑                                         |
+----------------------------------------------------------------------------------------------------+
| [第一轨：工程同级本地缓存]                                                                           |
| 路径: {ProjectDir}/.clipflow_cache/{ProjectHash}/                                                  |
| 目的：强随工程绑定，移动移动硬盘或项目文件夹即带走缓存；删除后可随时无损重建；零污染系统盘。           |
|                                                                                                    |
| ├── proxies/             # 540p 监视器实时播放快速代理切片                                            |
| ├── waveforms/           # 音频波形多级 LOD 峰值二进制 (.peak)                                       |
| ├── hyperframes/         # 动效离屏确定性渲染临时帧缓冲 (RGBA Raw)                                    |
| ├── wal/                 # 预写日志 (session.wal)                                                  |
| └── autosave/            # 历史滚动快照备份 (保留最新 10 份)                                         |
+----------------------------------------------------------------------------------------------------+
| [第二轨：系统级全局共享数据]                                                                         |
| 路径: %LOCALAPPDATA%\ClipFlow\ (Windows 原生规范)                                                   |
| 目的：多工程共享大体积 AI 模型、全局配置、操作日志与崩溃转储，避免多工程重复下载数 GB 权重。            |
|                                                                                                    |
| ├── models/              # faster-whisper large-v2 模型权重 (~3.1GB, 多工程全局复用)                  |
| ├── config/              # 用户偏好设置 (settings.json, keymaps.json)                              |
| ├── logs/                # 宿主与子进程轮转日志 (保留 7 天)                                          |
| └── templates/           # 全局动效模板资产与预置组件库                                               |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. 第一轨：工程本地缓存规约 (`.clipflow_cache/`)

### 2.1 目录组织与哈希寻址
工程保存时，在工程文件（如 `MyVideo.clipflow`）同级自动创建隐藏目录 `.clipflow_cache/`。  
为避免单目录下同名工程混淆，内部以 `ProjectID` 的前 8 位 Hex 或工程文件绝对路径 SHA-256 哈希命名隔离：

```
D:/Videos/202609_Interview/
├── 202609_Interview.clipflow                  # 核心工程文件 (SSOT 容器)
├── Footage_01.mp4                             # 用户原始媒体
└── .clipflow_cache/                           # 隐藏缓存目录
    └── a1b2c3d4/                              # 工程实例缓存根
        ├── proxies/
        │   └── Footage_01_proxy_540p.mp4      # 1/4 代理视频 (NV12 / Fast H.264)
        ├── waveforms/
        │   └── Footage_01_A1.peak             # 音频峰值波形缓存 (多分辨率 LOD)
        ├── hyperframes/
        │   └── seq_01/
        │       ├── frame_0001.raw             # 离屏动效 RGBA 原始帧缓存
        │       └── frame_0002.raw
        ├── wal/
        │   └── session.wal                    # 增量事务预写日志
        └── autosave/
            ├── 202609_Interview_20260926_143000.clipflow
            └── 202609_Interview_20260926_143300.clipflow
```

### 2.2 各类型缓存文件技术规格

| 缓存类型 | 存储格式 / 编码 | 命名规范 | 生成时机 | 重建与恢复策略 |
| :--- | :--- | :--- | :--- | :--- |
| **540p 代理** | H.264 / NV12 / Ultrafast / 无音频 | `{AssetHash}_proxy_540p.mp4` | 媒体首次导入或后台静默生成 | 若丢失，监视器降级软解或按需重新转码 |
| **音频波形** | 自研定宽二进制 (含 Min/Max 浮点) | `{AssetHash}_{TrackId}.peak` | 导入媒体时快速扫描音频流 | 若丢失，在 500ms 内重新提取并流式重绘 |
| **动效帧缓冲**| 裸 Raw RGBA (紧凑字节流) | `hf_{LayerId}_f{FrameIdx}.raw`| HyperFrames Worker 离屏渲染 | 若丢失，依据 HTML/CSS/JS 确定性再次求值 |
| **WAL 日志** | 二进制追加流 (Append-only) | `session.wal` | 每次 `TimelineCommand` 提交 | 启动异常检测时读取重放，正常保存后清空 |
| **历史快照** | `.clipflow` 标准格式 (Zstandard) | `{Name}_{YYYYMMDD_HHMMSS}.clipflow` | 后台定时器（每 3 分钟触发） | 循环覆盖，磁盘严格锁定最新 10 份 |

---

## 3. 第二轨：全局共享数据规约 (`%LOCALAPPDATA%\ClipFlow\`)

### 3.1 路径拓扑与权限
使用 Windows 标准 API `SHGetKnownFolderPath` 获取 `FOLDERID_LocalAppData`，确保在多用户或域环境下隔离安全：
- 物理路径：`C:\Users\<Username>\AppData\Local\ClipFlow\`
- 读写模式：仅当前用户具备完全控制权限。

### 3.2 AI 模型资产统一寻址 (`models/`)
针对 `faster-whisper 1.2.1` 运行所需的 `large-v2` 模型（体积约 3.1GB），系统禁止在每个工程目录下重复拉取：
- **模型存放路径**：`%LOCALAPPDATA%\ClipFlow\models\faster-whisper-large-v2\`
- **校验与离线加载**：
  - 启动 Python Worker 时，通过环境变量 `HF_HUB_OFFLINE=1` 与 `HF_HOME=%LOCALAPPDATA%\ClipFlow\models` 强制绑定寻址；
  - 首次运行由应用前置检测模型完整性（校验 `model.bin` 与 `vocabulary.json` 的 SHA-256）；
  - 若模型不存在，主界面提示“检测到缺少离线语音模型”，支持内置下载器或引导用户手动放置。

### 3.3 集中日志与轮转策略 (`logs/`)
- **日志切分**：按天滚动命名，如 `clipflow_2026-09-26.log`；
- **保留周期**：系统自动清理超过 **7 天** 的历史日志；
- **最大容量硬限**：单日志文件 $\le 50\text{MB}$，超出后自动切分为 `.1`, `.2`。

---

## 4. 磁盘配额管理与缓存清理机制

### 4.1 配额警戒线与自动淘汰规则
- **全局软上限配额**：默认设定为 **50 GB**（用户可在偏好设置中调至 20GB ~ 200GB 或无限制）。
- **淘汰优先级（LRU 最少使用原则）**：
  1. **第一优先级清退**：`hyperframes/` 离屏渲染切片（成本低，可秒级重新求值）；
  2. **第二优先级清退**：`proxies/` 代理视频文件；
  3. **受保护区（严禁自动清退）**：`waveforms/`（波形）、`wal/`（未保存日志）、`autosave/`（历史工程）。

### 4.2 用户交互与主动清理入口
在 UI 顶栏菜单 `Edit` $\to$ `Preferences` $\to$ `Media Cache` 提供以下控制看板：
- **当前工程缓存占用**：直观展示当前工程的代理、波形和动效占用空间（支持一键 `Clean Current Project Cache`）；
- **全局缓存管理**：扫描列出所有脱机工程的无主孤儿缓存（支持一键 `Purge Unused Cache`）；
- **位置迁移**：支持将第一轨或第二轨的默认生成根目录重定向至高速 NVMe SSD 数据盘。
