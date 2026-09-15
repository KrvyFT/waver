# UI

## 布局

[`WaverApp`](../crates/waver-ui/src/app.rs) 三栏：

1. **模块库** — 按 `MODULE_CATALOG` 的 `ModuleSection` 分组；搜索过滤 `name` / `code`；`addable == false` 禁用
2. **画布** — 节点、线缆、拖拽与连线（`editor/`）
3. **检查器** — 选中节点参数；VCO 有专用旋钮 / 波形控件，其余多显示 `inspector_blurb`

顶栏与底栏：运行状态、设备名、节点/线缆计数等。

字体与主题：`setup_fonts` / `setup_theme`（启动时由 `main` 调用）。

## PatchState

[`patch_state.rs`](../crates/waver-ui/src/patch_state.rs)

| 字段 / 方法 | 作用 |
|-------------|------|
| `graph` | 可编辑 IR |
| `positions` | 画布坐标 |
| `compiled` | 最近一次成功编译的 `Arc<CompiledPatch>`（GUI 读参） |
| `selected` | 当前选中节点 |
| `add_node` / `add_node_at` | `Graph::insert` + 布局 |
| `try_connect` | 连线（失败则记错误） |
| `recompile` | `compile_patch` → `SwapSchedule` |

默认 patch：`VCO` 与 `Output` 已连接。

## 与 core 的契约

- UI **只**依赖 `waver-core`：用 `NodeKind`、`MODULE_CATALOG`、`ParamRegistry` / `ParamCell`。
- 不构造 `Vco` 等具体 DSP；音频侧由引擎根据 schedule 的 kind 工厂化。
- 画布标签：`kind.desc().canvas_label`；库条目直接遍历 catalog。

## 扩展 UI 控件

新模块若只需默认检查器文案：在 `ModuleDesc` 填好即可。

若需自定义旋钮 / 选择器：

1. 在 `editor/mod.rs` 按 `NodeKind` 分支画控件
2. 通过 `compiled.params.get(node, ParamId::…)` 取 `ParamCell` 并 `set` / `value`
3. 如需特殊命中区域，扩展 `node_view.rs`（参考 VCO）
