# UI

## 布局

[`WaverApp`](../crates/waver-ui/src/app.rs) 工作区：

1. **模块库** — 可用模块按 `ModuleFamily` 分组，搜索过滤 `name` / `code`；未实现模块收纳在“开发中的模块”折叠区并禁用
2. **画布** — 节点、线缆、拖拽与连线（`editor/`）
3. **底部设备面板** — 选中节点参数；VCO 有专用旋钮 / 波形控件，其余有参数模块显示 `inspector_blurb` 和 0..=1 通用滑条；无参模块只显示说明

顶栏显示工作区与音频连接状态，底栏显示设备、音频格式与未保存提示。节点/线缆计数位于画布标题下方，连接提示位于底部设备面板。

点击顶栏右侧音频状态按钮打开 288 px 音频侧栏，沿用灰色面板、紧凑控件和橙色应用按钮。可选择后端、声卡 / 输出设备，刷新列表，并查看实际采样率和声道。UI 通过 core 的 `AudioSettingsRequest` 向主程序请求刷新或应用；主程序枚举 CPAL 设备并创建新流，成功后替换运行时、重新提交当前 patch，失败保留旧流。ASIO 由可选 `asio` 编译功能提供，普通版本显示禁用原因。

配色、尺寸与设计决策见 [ui-design.md](ui-design.md)。运行 `cargo run -p waver-ui --example preview` 可打开静音界面预览。

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
