# waver

模块化合成器 / patch 编辑器：在 egui 里搭节点图，编译成调度表后由 cpal 音频回调实时渲染。

## 快速开始

需要 **Rust 1.85+**，以及可用的音频输出设备。

```bash
git clone --recurse-submodules https://github.com/KrvyFT/waver.git
cd waver
cargo run -p waver
cargo test --workspace
```

若已 clone 但子模块为空：

```bash
git submodule update --init --recursive
```

启动后默认 patch 为 **VCO → Output**。左侧模块库添加节点，画布连线，右侧检查器调参。

### Windows 音频

Windows 默认使用 CPAL 的 **WASAPI** 后端，无需安装 ASIO SDK。点击右上角“音频已连接 / 音频未连接”打开右侧音频设置，选择音频后端及声卡 / 输出设备，点击“应用音频设置”切换。可选系统默认输出或指定设备；插拔设备后点击“刷新设备”。切换失败时保留原输出，成功时保留当前 patch 和参数。

使用 Rust MSVC 工具链及 Visual Studio Build Tools 的“使用 C++ 的桌面开发”组件（含 Windows SDK），在 PowerShell 中运行 `cargo run -p waver`。

ASIO 是可选编译功能：安装声卡厂商的 ASIO 驱动，并按 [CPAL 的 ASIO 构建说明](https://github.com/RustAudio/cpal/blob/master/README.md#asio)准备 ASIO SDK 和 LLVM/Clang（`CPAL_ASIO_DIR`、`LIBCLANG_PATH`），运行 `cargo run -p waver --features asio`。该版本会在侧栏中枚举 ASIO 声卡。未启用该功能的普通版本会禁用 ASIO 选项并显示原因。

音频流跟随设备默认采样率、声道数与 PCM 格式，不强制 48 kHz；支持浮点和整数 PCM（包括 16/24/32 位）。内部 DSP 使用 f32，设备输出前完成格式转换与满幅限幅。未找到设备或开流失败时，窗口仍可打开，右侧显示错误。

可运行不发声的 Windows 设备测试：

```powershell
cargo test -p waver-engine windows_default_device_opens_and_runs_silently -- --ignored --nocapture
```

该测试需要可用的默认播放设备。若设备未就绪，请在 Windows“设置 → 系统 → 声音”中确认播放设备，再从侧栏刷新并应用。`windows_explicit_device_opens_while_previous_stream_is_running` 测试验证指定设备和保留旧流的切换路径，同样需要通过 `--ignored` 显式运行。

## Workspace（git submodule）

| Crate | 仓库 | 职责 |
|-------|------|------|
| [waver-core](https://github.com/KrvyFT/waver-core) | 独立 | Graph IR、`ModuleDesc`、编译、`ParamCell`、`RtCommand` |
| [waver-dsp](https://github.com/KrvyFT/waver-dsp) | 独立 | `Process` 与内置节点；`for_kind` 工厂 |
| [waver-engine](https://github.com/KrvyFT/waver-engine) | 独立 | 音频线程 `Engine` + cpal 输出流 |
| [waver-ui](https://github.com/KrvyFT/waver-ui) | 独立 | egui shell（只依赖 core，不依赖 dsp） |
| `waver`（本仓） | 伞仓 | 二进制入口；挂载上述 submodule |

本地路径仍是 `crates/waver-*`。每个 crate 可独立 clone / 在其目录内 `git push`；联调在伞仓用 path 依赖。

依赖边界：UI 不碰 DSP；core 不碰 egui / cpal。

### 改某个 crate 并推送

```bash
cd crates/waver-core
git add -A && git commit -m "…" && git push
cd ../..
./scripts/repos.sh sync
git commit -m "chore: bump waver-core" && git push
```

详见 [doc/repos.md](doc/repos.md)。

## 文档

| 文档 | 内容 |
|------|------|
| [doc/architecture.md](doc/architecture.md) | 整体数据流与线程模型 |
| [doc/crates.md](doc/crates.md) | 各 crate 边界与公共 API |
| [doc/dsp-interface.md](doc/dsp-interface.md) | 通用 DSP 模块接口、实现示例与宿主限制 |
| [doc/modules.md](doc/modules.md) | `ModuleDesc` 与如何添加模块 |
| [doc/audio-thread.md](doc/audio-thread.md) | 实时约束、引擎、命令队列 |
| [doc/ui.md](doc/ui.md) | 三栏工作区与编辑状态 |
| [doc/repos.md](doc/repos.md) | submodule 多仓约定 |

## License

MIT OR Apache-2.0
