# 音频线程

## Engine

[`crates/waver-engine/src/engine.rs`](../crates/waver-engine/src/engine.rs)

- 内部块长 **`BLOCK = 64`** 帧，与 cpal 回调长度无关；长回调会按块步进处理（见 `stream.rs`）。
- 持有：`Arc<CompiledPatch>`、执行序、`HashMap<NodeId, Box<dyn Process>>`、每输出口预分配缓冲。
- `apply_rt(RtCommand)`：在块边界调用。
  - `SwapSchedule` → `rebuild`（可分配：建处理器、清缓冲）
  - `AllNotesOff`：占位，尚无 voice allocator
- `process_block`：按 order 跑节点 → fan-in 求和 → 取最后一个 `Output` 的 `master_slice` → 写交错设备缓冲。

## 命令队列

[`crates/waver-core/src/command.rs`](../crates/waver-core/src/command.rs)

GUI → 音频：`rtrb` SPSC（容量见 `stream.rs`，当前 256）。

| 命令 | 含义 |
|------|------|
| `SwapSchedule(Arc<CompiledPatch>)` | 整图替换（GUI 侧已分配） |
| `AllNotesOff` | 预留 |

高频参数：**不要**塞进 `RtCommand`；用 `ParamCell`。

## Process 实时约束

[`crates/waver-dsp/src/process.rs`](../crates/waver-dsp/src/process.rs)

在 `process` 中：

- 禁止堆分配
- 禁止锁 / 阻塞
- 禁止文件 / 网络 IO

允许：读写传入的缓冲切片、读 `ParamCell`、节点内部固定大小状态。

`for_kind` / `rebuild` 在非严格实时路径上，**可以**分配。

## 流生命周期

`spawn_output()`（[`stream.rs`](../crates/waver-engine/src/stream.rs)）：

1. 选默认输出设备，优先 f32 / 48 kHz
2. 建 `Engine` + 命令环
3. 回调内 drain 命令再渲染
4. 通过原子 `EngineStatus` 与可选错误字符串把状态暴露给 GUI

设备失败时仍返回 runtime，UI 显示错误；不把引擎锁给 GUI。

## 已知缺口

`process_node` 路径上仍有少量临时 `Vec` / `order.clone()`，与「回调零分配」目标不完全一致；改调度逻辑时应注意收紧。
