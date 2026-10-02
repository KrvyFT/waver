# DSP 模块：通用接口与新增模块指南

本文回答两件事：**DSP 模块对宿主暴露的通用接口是什么**，以及**照着改哪些文件才能加一个模块**。

- 权威契约（缓冲形状、参数语义、线程位置、宿主限制）在 [通用 DSP 模块接口](../crates/waver-dsp/docs/module-interface.md)，那一页作为 `waver-dsp` 的 rustdoc 被执行，示例是真编译的。
- 本文是**落地路径**：接口摘要 + 逐文件改动清单 + 代码模板 + 提交前检查表 + 会踩的坑。
- 模块目录的元数据侧说明见 [模块系统](modules.md)，音频线程说明见 [音频线程](audio-thread.md)。

## 1. 通用接口摘要

一个 DSP 模块只需要满足一件事：实现 `waver_dsp::Process`，并让 `waver_dsp::for_kind` 能把你这个 `NodeKind` 构造出来。

```rust
// crates/waver-dsp/src/process.rs
pub struct ProcessCtx<'a> {
    pub sample_rate: f32,
    pub block: usize,
    pub inputs: &'a [&'a [f32]],
    pub outputs: &'a mut [&'a mut [f32]],
}

pub trait Process: Send {
    fn process(&mut self, ctx: &mut ProcessCtx<'_>);
    fn master_slice(&self) -> Option<&[f32]> { None }
}
```

```rust
// crates/waver-dsp/src/nodes/mod.rs —— 唯一工厂
pub fn for_kind(kind: NodeKind, node: NodeId, params: &ParamRegistry) -> Option<Box<dyn Process>>
```

| 接口 | 位置 | 职责 |
|---|---|---|
| `Process` | `crates/waver-dsp/src/process.rs:20` | 块处理；`Send`，实例被移进音频回调 |
| `ProcessCtx` | `crates/waver-dsp/src/process.rs:4` | 采样率、有效帧数、单声道平面输入/输出借用 |
| `for_kind` | `crates/waver-dsp/src/nodes/mod.rs:19` | `NodeKind` + `NodeId` + `ParamRegistry` → `Box<dyn Process>`；`None` 时引擎用 `Silence` 兜底 |
| `NodeKind` | `crates/waver-core/src/graph.rs:9` | IR 身份（不是 DSP trait，也不含算法） |
| `ModuleDesc` / `MODULE_CATALOG` | `crates/waver-core/src/module.rs:53` / `:104` | 端口数量、默认参数、文案、`addable` |
| `ParamCell` | `crates/waver-core/src/param.rs` | UI ↔ DSP 的共享参数（`AtomicU32` 存 f32 位模式） |

工厂不校验"这个 `NodeId` 是否真的属于这个 `kind`"：调用方必须从同一份 `Schedule` / `ParamRegistry` 取值。

## 2. ProcessCtx 调用约定

| 字段 | 宿主保证 | 你必须做的 |
|---|---|---|
| `sample_rate` | 有限且 > 0，来自设备默认输出配置（`stream.rs:126`），不一定是 48 kHz | 用它换算频率/秒；不要假设 48000 |
| `block` | 本次有效帧数，**≤ 64**（`engine::BLOCK`），设备缓冲更大时被切块，可能是任意短值 | 只写/只读 `[..block]` |
| `inputs` | 长度 == 该 kind 的输入口数，每路至少 `block` 个样本；未连接的口是**全零总线**；多条线进同一口逐样本求和、不自动限幅 | 不要保留借用；不要假设"有输入口就等于接了线" |
| `outputs` | 每路可写、独占，至少 `block` 个样本；**当前引擎只提供并回写输出口 0** | 必须完整写入 `[..block]`，不得依赖上一块残留 |

四条硬规则：

1. `process` 内**禁止**堆分配、锁、阻塞、文件/网络 I/O、日志。
2. 输出必须写满 `ctx.block`，不要访问 `block` 以外的区域。
3. 不要跨调用保留 `ctx` 里的切片借用；状态存在 `self` 上。
4. `inputs.is_empty()` 表示"这个 kind 没有输入总线"，**不是**"有输入口的模块没接线"（后者是长度正确、内容全零）。

`master_slice()` 只有 sink 需要实现（当前只有 `Output`）：引擎在调度序里**逆序找第一个 `NodeKind::Output`**，取它的主总线复制到设备各声道。自建 sink 必须同步改引擎的选择逻辑，否则不会被读。

## 3. 端口与参数模型

- `PortCounts { inputs, outputs, params }` 来自 `ModuleDesc`，是唯一权威；输入口与输出口**各自从 0 编号**，`PortId(0)` 可以同时表示"输入 0"和"输出 0"，方向由连线的端点决定。
- 参数用 `ParamId::new(i)`，`i < ports.params`；默认值来自 catalog，编译时由 `ParamRegistry::with_defaults` 建 `Arc<ParamCell>`。
- 处理器在**构造时**取一次 `Arc<ParamCell>`，`process` 里通常**每块读一次** `value()`（别每样本读原子）。
- `ParamCell` 是 `Relaxed` 位模式读写：不校验范围、不拒 NaN/Inf、不做平滑，多个参数也不是一次原子快照。范围/防御性处理由模块自己做，或由 UI 的控件限制。
- 图重编译时 `ParamRegistry::merge` 按 `(NodeId, ParamId)` **复用存活节点的 cell**（旋钮值不丢），但引擎会重建**所有**处理器 → 相位、PRNG 序列、Delay 历史都会重置。

### 3.1 监视缓冲（只读显示模块）

要写"只显示、不发声"的模块（参照 `Scope`）：

- `ModuleDesc.monitors: true` 让 `ParamRegistry` 为该节点建一块 `Arc<ScopeTap>`（`crates/waver-core/src/tap.rs`，容量 `SCOPE_CAPACITY = 1024`）；`merge` 按 `NodeId` 复用它，重编译不丢历史。
- 工厂侧：`NodeKind::Scope => { let tap = params.tap(node)?; Some(Box::new(Scope::new(tap))) }`——`tap(node)` 返回 `Option<Arc<ScopeTap>>`，缺失就回 `None`。
- 处理器侧：0 输出（不必碰 `ctx.outputs`），只在 `process` 里逐帧 `self.tap.push(if sample.is_finite() { sample } else { 0.0 })`。`push` 是单写者 + `Relaxed`/`Release` 原子，无锁无分配，可以跑在音频回调里。
- UI 侧：`editor/node_view.rs` 的 `draw_scope` 用 `params.and_then(|p| p.tap(node.id)).as_deref()` 拿缓冲，`snapshot` 后再画；卡片是独立的 `SCOPE_SIZE` + `SCOPE_JACK_Y`，端口行不跟随通用 `56 + 32·index` 基准。
- 快照是**最新**的 N 个样本、最旧在前，且不与某次 `process_block` 对齐；显示层要对齐只能自己找过零点（`scope_trace`）。

## 4. 线程与实时边界

| 谁 | 在哪 | 做什么 |
|---|---|---|
| GUI 线程 | `PatchState::recompile` | 编辑 `Graph` → `compile_patch` → 入队 `RtCommand::SwapSchedule`；`ParamCell::set` 写参数（不入队） |
| 音频回调 | `stream.rs` 回调开头 | `while consumer.pop() { apply_rt }` → `Engine::rebuild` → **`for_kind` 在这里被调用** |
| 音频回调 | `Engine::process_block` | 清输出缓冲 → 按 `Schedule::order` 逐节点路由求和 → `process` → 取主总线 → 写设备缓冲 |

两个直接后果：

1. `process` / `master_slice` 必须实时安全。
2. `for_kind` **允许分配（`Box::new`），但它当前跑在音频回调里**——这是宿主尚未解决的实时债，不是你能在模块里绕开的。构造尽量轻，别在工厂里做重活。

宿主硬上限（写模块前必须确认你的端口数在范围内）：

- **最多 4 个输入口**（`engine.rs:12` `MAX_INPUTS = 4`；`scratch_in` 与 `local_out` 都按它定长）。
- **1 个输出口**（引擎只回写输出口 0）。
- **每块最多 64 帧**（`engine.rs:10` `BLOCK = 64`；`Output::MAX_BLOCK` 也必须保持 64）。

## 5. 三种改动路径

| 目标 | `graph.rs` | `module.rs` | `waver-dsp` | `waver-ui` |
|---|---|---|---|---|
| **A. 把计划项落地**（`Vca`/`Adsr`/`Lfo`/`Mixer` 已有 kind 与端口数；`Vcf` 已按此路径落地） | 不动 | 改文案 + `addable: true` | 实现 + `for_kind` 分支 | 不动 |
| **B. 已有类型下加新模块**（本文主走查） | +1 变体 | +1 条目 + `desc()` 下标 + 测试 | 实现 + 导出 + `for_kind` | 一般不动 |
| **C. 新增类型**（`ModuleFamily`） | 不动 | +1 family 变体 + `label()` + `ALL` | 新目录 | 不动 |

唯一会**编译报错**提醒你的地方是 `NodeKind` 的穷尽匹配——只有两处：`NodeKind::desc()`（`module.rs:277`）与 `for_kind`（`nodes/mod.rs:19`）。其余（catalog、测试数组）漏改只会**测试失败**或静默错位，所以务必照第 7 节的清单逐条过。

---

## 走查 A：把计划项 `Vca` 落地（零 `waver-core` 逻辑改动）

`Vca` 已经存在于 `NodeKind` 与 `MODULE_CATALOG`（2 入 / 1 出 / 1 参数），只是 `addable: false` 且工厂返回 `None`。三步。

### A1. 实现处理器

新建 `crates/waver-dsp/src/nodes/amp_env/vca.rs`：

```rust
//! Voltage-controlled amplifier: input × control voltage × gain.

use std::sync::Arc;

use waver_core::ParamCell;

use crate::{Process, ProcessCtx};

/// Param 0 is a manual gain applied on top of the control input.
pub struct Vca {
    gain: Arc<ParamCell>,
}

impl Vca {
    /// Shared gain cell (param 0).
    #[must_use]
    pub fn with_params(gain: Arc<ParamCell>) -> Self {
        Self { gain }
    }
}

impl Process for Vca {
    fn process(&mut self, ctx: &mut ProcessCtx<'_>) {
        let n = ctx.block;
        let raw = self.gain.value();
        let gain = if raw.is_finite() { raw.clamp(0.0, 1.0) } else { 0.0 };
        // 输入口 1（control）未接线时是一条全零总线，因此 cv 为 0 时输出静音。
        let signal = ctx.inputs.first().copied();
        let control = ctx.inputs.get(1).copied();
        let out = &mut ctx.outputs[0][..n];
        for (i, dst) in out.iter_mut().enumerate() {
            let s = signal.map_or(0.0, |bus| bus[i]);
            let cv = control.map_or(1.0, |bus| bus[i]);
            *dst = s * cv * gain;
        }
    }
}
```

### A2. 导出并绑定工厂

`crates/waver-dsp/src/nodes/amp_env/mod.rs`（新建）：

```rust
mod vca;

pub use vca::Vca;
```

`crates/waver-dsp/src/nodes/mod.rs`：加模块与再导出，

```rust
mod amp_env;
// ...
pub use amp_env::Vca;
```

并把 `for_kind` 里的 `NodeKind::Vca` 从 `None` 移到分支：

```rust
        NodeKind::Vca => {
            let gain = params.get(node, ParamId::new(0))?;
            Some(Box::new(Vca::with_params(gain)))
        }
        NodeKind::Vcf | NodeKind::Adsr | NodeKind::Lfo | NodeKind::Mixer => None,
```

`crates/waver-dsp/src/lib.rs:8`：把新类型加进 `pub use`。

### A3. 打开 `addable` 并改写文案

`crates/waver-core/src/module.rs` 中 `Vca` 条目（`:207`）：

```rust
        addable: true,
        param_defaults: &[1.0],
        param_labels: &["增益"],
```

同时把 `summary: "计划中"` 换成真实的一句话（如 `"输入 × 控制电压"`）、补 `inspector_blurb`。默认参数从 `0.0` 改成 `1.0` 是有意的：默认 0 会让新加的节点完全静音。

最后按第 7 节更新 `module.rs` 测试里的端口表。

---

## 走查 B：新增一个模块 `Gain`（完整流程，含 core 改动）

`Gain` 是 1 入 / 1 出 / 1 参数（0..=1 线性）的最小样例，与 [接口文档里的 Gain 示例](../crates/waver-dsp/docs/module-interface.md) 不同：**这里的 `Gain` 是一个真正的 `NodeKind`，走完整注册链路。**

### B1. `crates/waver-core/src/graph.rs`：加变体

```rust
pub enum NodeKind {
    // ... 现有 10 个变体
    /// Linear gain stage (extension demo).
    Gain,
}
```

追加在末尾最省事；`NodeKind` 的声明顺序**不影响** `desc()`（那是手写下标），但会影响你自己的阅读顺序。

### B2. `crates/waver-core/src/module.rs`：加常量与目录条目

```rust
const GAIN_PARAM_DEFAULTS: &[f32] = &[1.0];
const GAIN_PARAM_LABELS: &[&str] = &["增益"];
```

在 `MODULE_CATALOG` **末尾**追加（新下标 10）：

```rust
    ModuleDesc {
        kind: NodeKind::Gain,
        family: ModuleFamily::AmpEnv,
        ports: PortCounts {
            inputs: 1,
            outputs: 1,
            params: 1,
        },
        name: "增益",
        code: "Gain",
        canvas_label: "增益 · Gain",
        summary: "输入 × 增益",
        inspector_blurb: "把输入信号按 0–100% 线性缩放。",
        addable: true,
        param_defaults: GAIN_PARAM_DEFAULTS,
        param_labels: GAIN_PARAM_LABELS,
    },
```

### B3. 同步 `NodeKind::desc()`

`crates/waver-core/src/module.rs:282` 的手写映射加一行：

```rust
            Self::Gain => &MODULE_CATALOG[10],
```

**如果你把条目插在目录中间而不是末尾，必须重排后面所有下标**——`desc()` 是 `const fn`，故意不扫目录（`module.rs:281` 的注释说明了原因）。下标写错不会编译失败，只会让节点用错元数据/端口表。

### B4. 更新 `module.rs` 的测试

- `catalog_covers_every_kind_exactly_once`（`:303`）：把 `NodeKind::Gain` 加进 `all` 数组。这个测试用 `seen.len() == all.len()` 兜底，漏加会失败。
- `desc_ports_match_legacy_layout`（`:363`）：加 `(NodeKind::Gain, 1, 1, 1)`。
- `param_slices_match_port_counts`（`:344`）自动生效：`param_defaults` / `param_labels` 长度必须等于 `ports.params`。
- `oscillator_family_has_vco_and_noise`（`:332`）只在往 `Oscillator` 加模块时需要改（它断言该 family 恰好 2 个）。

### B5. `waver-dsp`：实现处理器

新建 `crates/waver-dsp/src/nodes/amp_env/gain.rs`：

```rust
//! Linear gain stage: `out = in × gain`.

use std::sync::Arc;

use waver_core::ParamCell;

use crate::{Process, ProcessCtx};

pub struct Gain {
    gain: Arc<ParamCell>,
}

impl Gain {
    /// Shared gain cell (param 0).
    #[must_use]
    pub fn with_params(gain: Arc<ParamCell>) -> Self {
        Self { gain }
    }
}

impl Process for Gain {
    fn process(&mut self, ctx: &mut ProcessCtx<'_>) {
        let n = ctx.block;
        let raw = self.gain.value();
        let gain = if raw.is_finite() { raw.clamp(0.0, 1.0) } else { 0.0 };
        let out = &mut ctx.outputs[0][..n];
        match ctx.inputs.first() {
            Some(input) => {
                for (dst, src) in out.iter_mut().zip(&input[..n]) {
                    *dst = *src * gain;
                }
            }
            // 没有输入总线（kind 声明了 0 个输入口）时输出零，而不是保留上一块的残留。
            None => out.fill(0.0),
        }
    }
}
```

模块级测试放在同文件 `#[cfg(test)]`（照 `noise.rs:54` 的写法）：构造两路输出缓冲、跑一块、断言数值。

### B6. `waver-dsp`：导出 + 绑定工厂

`crates/waver-dsp/src/nodes/mod.rs`：

```rust
mod amp_env;
pub use amp_env::Gain;
```

`for_kind` 加分支：

```rust
        NodeKind::Gain => {
            let gain = params.get(node, ParamId::new(0))?;
            Some(Box::new(Gain::with_params(gain)))
        }
```

`crates/waver-dsp/src/lib.rs:8` 加进 `pub use nodes::{...}`。

### B7. UI：通常不需要改

画布与检查器读的是 `ModuleDesc`：

- 侧栏会自动出现「增益 · Gain」（`addable: true` 才可点）。
- 检查器对 `ports.params > 0` 的模块渲染 0..=1 线性滑条，标签取 `param_labels`。
- 画布卡片按 `node_size()` 画端口（非 VCO 是 180×164），1 入 1 出放得下。

**只有**当你的参数不是 0..=1 线性、或需要专用控件（枚举、对数旋钮、按钮组）时，才必须动 `waver-ui`，见 8.4。

### B8. 提交

三个子仓分别提交、再 bump 伞仓指针：

```sh
cd crates/waver-core && git add -A && git commit -m "feat: add Gain kind" && git push
cd ../waver-dsp && git add -A && git commit -m "feat: implement Gain" && git push
cd ../..
./scripts/repos.sh sync
git commit -m "chore: bump waver-core, waver-dsp"
```

---

## 6. 新增一个类型（`ModuleFamily`）

```rust
// crates/waver-core/src/module.rs:7
pub enum ModuleFamily { /* ... */ Dynamics }          // 1. 加变体

pub const ALL: &'static [Self] = &[ /* ... */ Self::Dynamics ];   // 2. 加进侧栏顺序

pub const fn label(self) -> &'static str {
    match self {
        // ...
        Self::Dynamics => "动态",                     // 3. 加中文标题
    }
}
```

`label()` 是穷尽 `match`，加变体后编译器会提醒。可选：在 `crates/waver-dsp/src/nodes/` 下建同名目录（`dynamics/`）——目录只是源码组织方式，与 `ModuleFamily` 不是强绑定，`for_kind` 也不按 family 分派。之后按走查 B 往这个类型里挂具体 `NodeKind`。

## 7. 提交前检查表

1. `NodeKind` 变体已加，且 `desc()` 下标指向**正确的那一条** catalog。
2. `MODULE_CATALOG` 有条目，`param_defaults.len() == param_labels.len() == ports.params`。
3. `ports` 与真实端口一致（`desc_ports_match_legacy_layout`）。
4. `for_kind` 已绑定；`addable: true` 只在绑定完成之后——`waver-dsp` 的 `catalog_availability_matches_factory`（`nodes/mod.rs:68`）断言 `for_kind(...).is_some() == desc.addable`，标错会失败。
5. `process` 内无分配/锁/IO；只写 `[..ctx.block]`；不越界读输入；未连接输入按零处理。
6. 端口数在宿主上限内（≤4 入 / 1 出），且卡片几何放得下（8.3）。
7. 参数语义落在 0..=1（否则见 8.4）。
8. 默认参数不会让新节点"看起来坏了"（默认 0 增益 = 静音，通常该给 1.0 或中间值）。
9. `crates/waver-dsp/src/lib.rs` 导出了新类型（照现有约定）。
10. 监视模块（`monitors: true`）已在 `for_kind` 绑定 `ParamRegistry::tap`，`process` 只 `push` 有限值、不写输出。

```sh
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps

# PowerShell 里第三行写成：
$env:RUSTDOCFLAGS = '-D warnings'; cargo doc --workspace --no-deps

# 需要真实声卡的静音开流测试（默认 #[ignore]）：
cargo test -p waver-engine windows_default_device_opens_and_runs_silently -- --ignored --nocapture
```

## 8. 写模块前必读的坑

**8.1 只有输出口 0 会被回写。** `Engine::process_node` 把 `local_out[0]` 拷进 `(node, PortId(0))`；声明 2 个输出口的模块会执行但第 2 路永远不会被路由。多输出需要先改引擎。

**8.2 最多 4 个输入口。** `scratch_in` / `local_out` 都是 `[[f32; BLOCK]; 4]`（`engine.rs:38`、`:141`），第 5 个输入口会索引越界 panic。`Mixer` 的 4 个输入已经到顶。

**8.3 非 VCO 卡片高 164，端口 y = 82 + 32·index。** `node_view.rs:16` 与 `:184-189`：第 4 个输入口落在 y = 178，**已经画到卡片外面**，命中区域也跟着错位。所以落地 `Mixer`（4 入）之前必须先调 `node_size()` 或端口间距——这是 `Mixer` 目前只能停在"计划中"的原因之一。

**8.4 通用检查器只有 0..=1 线性滑条。** `ModuleDesc` 里没有 min/max、单位、步长、枚举或缩放元数据（`module.rs:46` 明确承认），VCO 的频率/振幅/波形控件是 UI 里**按 kind 硬编码**的（`node_view.rs:108-162` 的旋钮与波形按钮，`editor/mod.rs` 的旋钮拖动分支）。新增 Hz / 秒 / 枚举参数，要么先把参数定义成 0..=1 的归一化量，要么改 `waver-ui` 并补元数据。`Vcf` 的截止频率走的就是后一条路：`ParamCell` 直接存 Hz，`waver-ui/src/editor/mod.rs` 的 `frequency_range()` 提供 20–20 000 的对数滑条与 Hz 读数（catalog 默认 2 500 Hz）。

**8.5 重编译会重置算法状态。** 参数值经 `ParamRegistry::merge` 保留，但 `Engine::rebuild` 会重建**所有**处理器，相位 / 噪声序列 / Delay 历史从头开始。不要在模块里依赖"跨重编译连续"。

**8.6 反馈靠编译器插 `Delay`。** 你可以直接画环，编译器会拆一条边并插入一块延迟；被拆的边读上一块结果。若消费者在调度序里先于 Delay 执行，会读到零——目前的反馈音频不保证正确。

**8.7 未连接输入 = 全零总线。** `inputs.len()` 始终等于输入口数量，没接线的那一路是全零。所以"cv 乘法器"型模块（`Vca`/`Vcf`）在 cv 没接线时会静音——这是真实语义，但需要文案提示或 UI 侧的常量源配合。反之 `inputs.is_empty()` 只在 kind 声明 0 个输入口时成立。

**8.8 `master_slice` 只对 `NodeKind::Output` 生效。** 引擎按 `kind == NodeKind::Output` 判断并取调度序里最后一个；自定义 sink 光实现 `master_slice` 不会被读。

**8.9 `param_labels` 是 UI 文案，不会约束数值。** 写成 "频率 (Hz)" 就必须在 UI 侧真的按 Hz 处理（VCO 就是这么做的），否则用户会看到一个标着 Hz 的 0..=1 滑条。

**8.10 监视缓冲只能从已编译的 patch 里拿。** 卡片画波形用的参数来自 `patch.compiled.as_ref().map(|c| &c.params)`（`crates/waver-ui/src/editor/mod.rs:145-151`）；给尚未编译的图或编译失败后保留的旧 patch 画 tap 只会得到空图。`ScopeTap` 也不校验样本范围，画之前自行 `clamp(-1.0, 1.0)`。

## 9. 参考文件

| 位置 | 看什么 |
|---|---|
| `crates/waver-dsp/docs/module-interface.md` | 权威契约：缓冲、参数、主总线、线程、宿主限制 |
| `crates/waver-dsp/src/process.rs` | `Process` / `ProcessCtx` 定义 |
| `crates/waver-dsp/src/nodes/mod.rs` | `for_kind` 与两条契约测试 |
| `crates/waver-dsp/src/nodes/oscillator/noise.rs` | 带参数的源节点参照（含测试写法） |
| `crates/waver-dsp/src/nodes/io/output.rs` | sink / `master_slice` 参照 |
| `crates/waver-dsp/src/nodes/utility/delay.rs` | 带块内状态的参照 |
| `crates/waver-dsp/src/nodes/utility/scope.rs` | 只读显示节点（监视缓冲）参照 |
| `crates/waver-core/src/tap.rs` | `ScopeTap`：`push` / `snapshot` 环形监视缓冲 |
| `crates/waver-core/src/module.rs` | `ModuleDesc`、`MODULE_CATALOG`、`desc()`、元数据测试 |
| `crates/waver-core/src/graph.rs` | `NodeKind`、`Graph` 编辑 API |
| `crates/waver-engine/src/engine.rs` | 路由求和、`BLOCK` / `MAX_INPUTS`、主总线选择 |
| `crates/waver-ui/src/editor/node_view.rs` | 卡片几何、命中测试、VCO 专用控件 |
| `doc/modules.md` | 模块目录与 `ModuleFamily` 总览 |
