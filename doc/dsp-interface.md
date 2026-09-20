# 通用 DSP 模块接口

完整规范随 `waver-dsp` 子仓库维护：

**[通用 DSP 模块接口文档](../crates/waver-dsp/docs/module-interface.md)**

内容包括：

- `ModuleFamily` / `ModuleDesc` / `NodeKind` 的分工与注册流程。
- `Process` / `ProcessCtx` 的缓冲、参数、状态和主总线契约。
- `for_kind` 的失败行为、实际线程位置和实时约束。
- 可编译的工厂调用、Gain 模块示例，以及 Noise 扩展示范。
- 当前宿主的端口、块长、参数范围、反馈和实时分配限制。

相关文档：[模块目录](modules.md) · [音频线程](audio-thread.md) · [多仓工作流](repos.md)。
