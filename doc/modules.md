# 模块系统

模块身份由 **`NodeKind` 枚举** 表达；元数据集中在静态 **`ModuleDesc`**。不要再为端口数 / 默认参数 / 侧栏文案维护平行 `match`。

## ModuleDesc

定义见 [`crates/waver-core/src/module.rs`](../crates/waver-core/src/module.rs)。

| 字段 | 用途 |
|------|------|
| `kind` | 对应 `NodeKind` |
| `ports` | 输入 / 输出 / 参数个数 |
| `name` / `code` | 侧栏显示（如「振荡器」「VCO」） |
| `canvas_label` | 画布节点标题 |
| `summary` | 节点体内短文案 |
| `inspector_blurb` | 检查器说明（无自定义 UI 时） |
| `section` | `Core` / `Utility` / `Planned` |
| `addable` | 侧栏是否可点击添加 |
| `param_defaults` / `param_labels` | 长度必须等于 `ports.params` |

`MODULE_CATALOG` 收录全部 kind（含计划中）。`NodeKind::desc()` 用手动下标映射，保持 `const`——**改 catalog 顺序时必须同步 `desc()`**。

派生 API：

- `NodeKind::port_counts()` → `desc().ports`
- `default_param_value` / `param_label` → desc 参数表

## 当前内置

| Kind | 可添加 | DSP |
|------|--------|-----|
| Vco / Output | 是 | 有 |
| Delay / Silence | 是 | 有（Delay 也会被编译器插环） |
| Vcf / Vca / Adsr / Lfo / Mixer | 否（Planned） | 无（`for_kind` → `None` → 引擎 Silence） |

## 添加新模块清单

1. 在 `graph.rs` 给 `NodeKind` 加变体。
2. 在 `MODULE_CATALOG` 追加一条 `ModuleDesc`（端口、文案、`section`、`addable`、参数切片）。
3. 更新 `NodeKind::desc()` 的 match 下标，使之指向新条目。
4. 在 `waver-dsp/src/nodes/` 实现 `Process`（`process` 内禁止分配 / 锁 / IO）。
5. 在 `nodes/mod.rs` 的 `for_kind` 里构造实例；有参时从 `ParamRegistry` 取 `Arc<ParamCell>`。
6. 若需专用控件（旋钮布局、波形选择），扩展 `waver-ui` 的 `editor/`（参考 VCO）；否则侧栏 / 标签会自动出现。
7. 跑 `cargo test --workspace`；`module` 测试会检查 catalog 覆盖与参数切片长度。

## 工厂约定

- **唯一工厂**：`waver_dsp::for_kind`。引擎 `rebuild` 调用它，失败则 `Silence`。
- Sink（如 Output）通过 `Process::master_slice` 暴露主总线，供设备写出。
