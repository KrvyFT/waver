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
| [doc/modules.md](doc/modules.md) | `ModuleDesc` 与如何添加模块 |
| [doc/audio-thread.md](doc/audio-thread.md) | 实时约束、引擎、命令队列 |
| [doc/ui.md](doc/ui.md) | 三栏工作区与编辑状态 |
| [doc/repos.md](doc/repos.md) | submodule 多仓约定 |

## License

MIT OR Apache-2.0
