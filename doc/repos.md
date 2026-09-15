# 多仓库布局（git submodule）

每个 crate 是**独立 Git 仓库**，在伞仓里以 submodule 挂在 `crates/` 下。  
本地目录结构不变，可继续 `cargo run`；也可单独 clone / 在 crate 目录里 `git push`。

## 仓库

| 本地路径 | 独立仓库 |
|----------|----------|
| 伞仓根 | https://github.com/KrvyFT/waver |
| `crates/waver-core` | https://github.com/KrvyFT/waver-core |
| `crates/waver-dsp` | https://github.com/KrvyFT/waver-dsp |
| `crates/waver-engine` | https://github.com/KrvyFT/waver-engine |
| `crates/waver-ui` | https://github.com/KrvyFT/waver-ui |

Cargo 仍用伞仓 **path 依赖**。单独 clone 某个 crate 时，因 `*.workspace = true`，**不能**直接 `cargo build`；开发与联调以伞仓为准。

## 首次拿到伞仓

```bash
git clone --recurse-submodules https://github.com/KrvyFT/waver.git
# 若已 clone 未拉子模块：
git submodule update --init --recursive
```

从 subtree 迁到 submodule（仅需一次）：

```bash
./scripts/migrate-to-submodules.sh
git commit -m "chore: track workspace crates as git submodules"
git push
```

## 在某个 crate 里独立提交 / 推送

```bash
cd crates/waver-core
git checkout -b my-change
# 改代码…
git add -A && git commit -m "feat: …"
git push -u origin HEAD
cd ../..
./scripts/repos.sh sync          # 把伞仓里的 submodule 指针 staged
git commit -m "chore: bump waver-core"
git push
```

跨多个 crate 改完时：分别在各自目录 commit + push，再 `./scripts/repos.sh sync` 一次，伞仓记全部新指针。

## 助手命令

```bash
./scripts/repos.sh status        # 各子模块分支 / 是否脏
./scripts/repos.sh push          # 各 crate 目录 git push
./scripts/repos.sh push core     # 只推 waver-core
./scripts/repos.sh pull          # 按远程更新子模块
./scripts/repos.sh sync          # git add crates/* 指针
```

## 约定

1. **crate 源码的 commit 发生在子仓库**；伞仓 commit 主要更新 submodule 指针、二进制、`doc/`、`scripts/`。
2. 子模块默认跟踪远程 `main`。
3. 不要在未初始化 submodule 的空目录里直接写文件。
4. 以前的 subtree 远程名 `crate-waver-*` 可删：`git remote remove crate-waver-core` 等（可选清理）。
