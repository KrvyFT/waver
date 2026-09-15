# waver-core

Graph IR、模块目录（`ModuleDesc`）、编译调度与无锁 `ParamCell`。

## 使用方式

- **独立仓库**：可单独 clone / push  
  `https://github.com/KrvyFT/waver-core`
- **伞仓开发（推荐联调）**：在 https://github.com/KrvyFT/waver 里作为 `crates/waver-core` submodule；workspace path 依赖在此解析。

单独 clone 时 `Cargo.toml` 使用 workspace 继承，需在伞仓内 `cargo build`。约定见伞仓 [doc/repos.md](https://github.com/KrvyFT/waver/blob/master/doc/repos.md)。
