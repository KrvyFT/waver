# 文档索引

伞仓文档。各 crate 的入口说明见对应仓库 README（`crates/*/README.md`）。

| 文档 | 说明 |
|------|------|
| [architecture.md](architecture.md) | 数据流、线程分工、编译路径 |
| [crates.md](crates.md) | Workspace 边界与主要导出类型 |
| [modules.md](modules.md) | 静态模块描述符与扩展清单 |
| [audio-thread.md](audio-thread.md) | Engine、RtCommand、实时安全 |
| [ui.md](ui.md) | Patch 编辑器与 `PatchState` |
| [repos.md](repos.md) | 多仓库 / git submodule 工作流 |

源码级 API 以各 crate 的 `//!` / `///` 为准。
