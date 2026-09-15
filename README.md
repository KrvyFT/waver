# waver

模块化合成器 / patch 编辑器：在 egui 里搭节点图，编译成调度表后由 cpal 音频回调实时渲染。

## 快速开始

需要 **Rust 1.85+**，以及可用的音频输出设备。

```bash
cargo run -p waver
# 测试
cargo test --workspace
```

启动后默认 patch 为 **VCO → Output**。左侧模块库添加节点，画布连线，右侧检查器调参。

## Workspace

| Crate | 职责 |
|-------|------|
| [`waver-core`](crates/waver-core) | Graph IR、模块目录、编译、`ParamCell`、`RtCommand` |
| [`waver-dsp`](crates/waver-dsp) | `Process` 与内置节点；`for_kind` 工厂 |
| [`waver-engine`](crates/waver-engine) | 音频线程 `Engine` + cpal 输出流 |
| [`waver-ui`](crates/waver-ui) | egui shell（只依赖 core，不依赖 dsp） |
| `waver`（本包） | 把音频运行时接到窗口 |

依赖边界：UI 不碰 DSP；core 不碰 egui / cpal。

## 文档

| 文档 | 内容 |
|------|------|
| [doc/architecture.md](doc/architecture.md) | 整体数据流与线程模型 |
| [doc/crates.md](doc/crates.md) | 各 crate 边界与公共 API |
| [doc/modules.md](doc/modules.md) | `ModuleDesc` 与如何添加模块 |
| [doc/audio-thread.md](doc/audio-thread.md) | 实时约束、引擎、命令队列 |
| [doc/ui.md](doc/ui.md) | 三栏工作区与编辑状态 |
| [doc/repos.md](doc/repos.md) | 伞仓 + 各 crate 独立仓库（git submodule） |

本地仍是 `crates/waver-*`。每个 crate 可独立 clone / 在其目录内 `git push`；联调在伞仓用 path 依赖。详见 [doc/repos.md](doc/repos.md)。

## License

MIT OR Apache-2.0
