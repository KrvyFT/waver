# Crate 边界

## 依赖图

```
waver (bin)
  ├── waver-engine ──► waver-dsp ──► waver-core
  └── waver-ui ──────────────────► waver-core
```

硬规则：

- **`waver-core`**：不依赖 egui、cpal、具体 DSP。
- **`waver-ui`**：只依赖 `waver-core`（+ eframe / rtrb），**不**依赖 `waver-dsp`。
- **`waver-dsp`**：只依赖 `waver-core`；`process` 内零分配。
- **`waver-engine`**：把 DSP 接到设备；持有 `Box<dyn Process>`。

## waver-core

路径：`crates/waver-core`

| 模块 | 职责 |
|------|------|
| `graph` | `Graph` / `Node` / `NodeKind` / `Edge` / `PortRef` |
| `module` | `ModuleDesc` / `MODULE_CATALOG` / `ModuleSection` |
| `ports` | `PortCounts`；`port_counts` 委托 desc |
| `compile` / `schedule` | 拓扑编译 → 只读 `Schedule` |
| `patch` | `CompiledPatch`、`ParamRegistry`、默认参数 |
| `param` | `ParamCell` |
| `command` | `RtCommand` |
| `status` | `EngineStatus`（原子快照） |
| `ids` / `error` | `NodeId` / `PortId` / `ParamId`、`GraphError` |

## waver-dsp

路径：`crates/waver-dsp`

- `Process` / `ProcessCtx`
- 内置节点：`Vco`、`Output`、`Delay`、`Silence`
- **`for_kind`**：唯一实例化入口（可在 rebuild 时分配）

未实现 kind（VCF 等）返回 `None`；引擎回退为 `Silence`。

## waver-engine

路径：`crates/waver-engine`

- `Engine` / `BLOCK`（64 帧，与设备回调长度解耦）
- `spawn_output` / `AudioRuntime` / `EngineError`

## waver-ui

路径：`crates/waver-ui`

- `WaverApp`、`setup_fonts` / `setup_theme`
- 内部：`PatchState`、`editor/`（画布、线缆、旋钮）

## 二进制

`src/main.rs`：启动失败时向 stderr 打错误；音频设备失败时窗口仍可开，错误文案由 UI 读 `AudioRuntime` 共享状态展示。
