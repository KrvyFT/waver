# 模块系统

处理器实现和宿主契约详见 [通用 DSP 模块接口](dsp-interface.md)。

- **类型** = [`ModuleFamily`](../crates/waver-core/src/module.rs)（侧栏分组，可继续加变体）
- **模块** = [`NodeKind`](../crates/waver-core/src/graph.rs)（IR 身份）；同一 family 下可有多条 [`ModuleDesc`](../crates/waver-core/src/module.rs)
- 元数据集中在 `MODULE_CATALOG`；`addable: false` 表示计划中（仍可在侧栏看到）

## ModuleFamily（类型）

| 变体 | 侧栏标题 | 当前模块 |
|------|----------|----------|
| `Oscillator` | 振荡器 | VCO、**Noise（示范）** |
| `Filter` | 滤波器 | VCF（计划中） |
| `AmpEnv` | 放大 / 包络 | VCA、ADSR（计划中） |
| `Modulation` | 调制 | LFO（计划中） |
| `Mixer` | 混音 | Mixer（计划中） |
| `Utility` | 工具 | Delay、Silence |
| `Io` | 输入 / 输出 | Output |

`ModuleFamily::ALL` 决定侧栏顺序。

## ModuleDesc

| 字段 | 用途 |
|------|------|
| `kind` | 对应 `NodeKind` |
| `family` | 所属类型 |
| `ports` | 输入 / 输出 / 参数个数 |
| `name` / `code` | 侧栏显示 |
| `canvas_label` / `summary` / `inspector_blurb` | 画布与检查器文案 |
| `addable` | 是否可点击添加 |
| `param_defaults` / `param_labels` | 长度必须等于 `ports.params` |

`NodeKind::desc()` 用手动下标映射，保持 `const`——**改 catalog 顺序时必须同步 `desc()`**。

派生：`port_counts` / `default_param_value` / `param_label` → desc。

## DSP 目录（按类型）

```
crates/waver-dsp/src/nodes/
  mod.rs                 # for_kind
  oscillator/{vco,noise}.rs
  utility/{delay,silence}.rs
  io/output.rs
```

## 添加新**类型**

1. 在 `ModuleFamily` 增加变体，并写入 `label()` 与 `ALL`。
2. （可选）在 `waver-dsp/src/nodes/` 下新建同名目录（如 `filter/`）。
3. 该类型下再按「添加新模块」挂载具体 `NodeKind`。

## 添加新**模块**（已有类型下）

标准示范：**Noise**（`Oscillator` 下第二个模块）。

1. `graph.rs`：`NodeKind` 加变体。
2. `MODULE_CATALOG` 追加 `ModuleDesc`（填好 `family`、端口、参数）。
3. 更新 `NodeKind::desc()` 下标。
4. 在 `nodes/<family>/` 实现 `Process`（`process` 内禁止分配 / 锁 / IO）。
5. `for_kind` 增加分支。
6. 无专用画布控件时：侧栏自动出现；检查器对非 VCO 参数用通用滑条（读 `param_labels`）。
7. `cargo test --workspace`。

对照文件：

- catalog：[`module.rs`](../crates/waver-core/src/module.rs) 中 `Noise` 条目
- DSP：[`noise.rs`](../crates/waver-dsp/src/nodes/oscillator/noise.rs)
- 工厂：[`nodes/mod.rs`](../crates/waver-dsp/src/nodes/mod.rs) `for_kind`

## 工厂约定

- **唯一工厂**：`waver_dsp::for_kind`；失败 → 引擎 `Silence`。
- Sink（Output）通过 `Process::master_slice` 暴露主总线。
