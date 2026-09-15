# 架构

## 目标

GUI 线程编辑 patch（节点、线缆、参数），音频线程只读已编译调度并跑 `Process`。拓扑变更整图替换；旋钮等高频控制走共享 `ParamCell`，不经命令队列。

## 数据流

```
┌───────────── GUI 线程 ─────────────┐     rtrb SPSC      ┌────── 音频回调 ──────┐
│  PatchState.graph                  │                    │                      │
│       │ compile_patch              │  SwapSchedule      │  Engine.rebuild      │
│       ▼                            │ ─────────────────► │       │              │
│  CompiledPatch (Schedule+Params)   │                    │  for_kind → Process  │
│                                    │                    │       │              │
│  ParamCell.set (旋钮) ─────────────┼── Arc 共享 ────────┼───────┘ process_block│
│                                    │                    │       ▼              │
│  EngineStatus / 设备错误 (原子读) ◄─┼────────────────────┤  cpal interleaved    │
└────────────────────────────────────┘                    └──────────────────────┘
```

入口在 [`src/main.rs`](../src/main.rs)：`spawn_output()` 拿到 `Producer<RtCommand>`，交给 `WaverApp`；`AudioRuntime` 与窗口同生命周期。

## 线程分工

| 工作 | 线程 | 说明 |
|------|------|------|
| 图编辑、编译、`SwapSchedule` 分配 | GUI | `Graph` 注释约定：从不在回调里 compile |
| drain 命令、按 `Schedule` 处理块 | 音频 | `Engine` 不得包进 `Mutex` |
| 旋钮读写 | 双方 | `ParamCell` = `AtomicU32` bit pattern，`Relaxed` |

## 编译路径

1. UI 改 `Graph`（`insert` / `connect` / …）
2. `Graph::compile_patch`（[`compile.rs`](../crates/waver-core/src/compile.rs)）
   - 校验端口与节点
   - Kahn 拓扑排序
   - 若有环，插入最多 64 个 `Delay` 拆反馈
3. 得到 `CompiledPatch`：`Schedule` + `ParamRegistry`（新节点用默认值，存活节点 `merge` 保留旋钮）
4. `commands.push(RtCommand::SwapSchedule(Arc::…))`
5. 回调块边界 `apply_rt` → `rebuild`：按 order 调用 `for_kind`，重建端口缓冲

没有增量 `AddNode` RT 命令；加模块 = 改 IR + 整表替换。

## 相关文档

- 模块注册：[modules.md](modules.md)
- 音频侧细节：[audio-thread.md](audio-thread.md)
- UI 状态：[ui.md](ui.md)
