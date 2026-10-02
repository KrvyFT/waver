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

`for_kind` / `rebuild` 当前由音频回调中的 `apply_rt` 调用，仍会分配和释放内存。它们不是后台编译线程路径；整个引擎尚未满足严格实时要求。完整约定见 [通用 DSP 模块接口](dsp-interface.md)。

## 流生命周期

`spawn_output()`（[`stream.rs`](../crates/waver-engine/src/stream.rs)）：

1. 选默认输出设备及其默认 PCM 格式、采样率和声道数；Windows 使用 WASAPI 的系统混音格式
2. 建 `Engine` + 命令环
3. 回调内 drain 命令，再按最多 64 帧分块渲染为 f32，转换到设备 PCM 格式
4. 通过原子 `EngineStatus` 与可选错误字符串把状态暴露给 GUI

设备失败时仍返回 runtime，UI 显示错误；不把引擎锁给 GUI。

设备边界支持有符号/无符号 8、16、24、32、64 位整数，以及 f32/f64；不支持 DSD。转换缓冲在建流时分配，回调复用。输出超过满幅时限幅到 [-1, 1]，非有限值输出静音，无符号 PCM 静音使用其中点。该转换层不增加回调内分配，但下述引擎原有分配仍存在。

## 已知缺口

`process_node` 路径上仍有少量临时 `Vec` / `order.clone()`，与「回调零分配」目标不完全一致；改调度逻辑时应注意收紧。
