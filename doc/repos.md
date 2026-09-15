# 多仓库布局（保留本地目录结构）

日常开发仍在本仓库（伞仓）一次改完；各 crate 另有独立 GitHub 远程，用 **git subtree** 同步。本地路径不变，`cargo run` / 跳转定义体验与现在相同。

## 仓库对照

| 本地路径 | 远程 |
|----------|------|
| 伞仓根（本仓库） | https://github.com/KrvyFT/waver |
| `crates/waver-core` | https://github.com/KrvyFT/waver-core |
| `crates/waver-dsp` | https://github.com/KrvyFT/waver-dsp |
| `crates/waver-engine` | https://github.com/KrvyFT/waver-engine |
| `crates/waver-ui` | https://github.com/KrvyFT/waver-ui |

Cargo 仍用 **path 依赖**（见根 `Cargo.toml` 的 `[workspace.dependencies]`）。子仓是源码镜像，不是给「只 clone 单个 crate 再拼 workspace」用的；独立 clone 时 `*.workspace = true` 无法解析。

## 日常改代码

和平时一样：在 `crates/...` 里改 → 在伞仓提交 → `cargo test --workspace`。

推送到各子仓：

```bash
./scripts/repos.sh push          # 推送全部 crate
./scripts/repos.sh push core     # 只推 waver-core
```

从子仓拉回（少用；一般以伞仓为准）：

```bash
./scripts/repos.sh pull core
```

首次配置远程（clone 后若还没有）：

```bash
./scripts/repos.sh remotes
```

## 为什么用 subtree 而不是 submodule

- **目录结构不变**，没有「先进子模块再 commit」的两步流程
- 伞仓历史完整，方便跨 crate 重构（例如 `ModuleDesc`）
- 子仓仍有独立 URL，便于单独权限、CI、或以后 crates.io 发布

## 约定

1. **以伞仓为真相源**：跨 crate 改动只在伞仓开 PR / 提交。
2. 推子仓前先保证伞仓工作区干净（已提交）。
3. 子仓默认分支：`main`。
4. 不要在子仓直接大改再期望自动合并回伞仓；若必须，用 `pull` 后在伞仓解决冲突。
