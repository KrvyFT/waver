//! Static module descriptors: single source of truth for ports, params, and UI copy.

use crate::{NodeKind, ParamId, PortCounts};

/// Sidebar grouping for the module library.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleSection {
    /// Primary signal sources / sinks.
    Core,
    /// Helpers and utilities.
    Utility,
    /// Enumerated but not yet addable / runnable.
    Planned,
}

impl ModuleSection {
    /// Chinese label shown above each library group.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Core => "核心",
            Self::Utility => "工具",
            Self::Planned => "计划中",
        }
    }
}

/// Compile-time metadata for one [`NodeKind`].
#[derive(Clone, Copy, Debug)]
pub struct ModuleDesc {
    /// IR kind this descriptor belongs to.
    pub kind: NodeKind,
    /// Jack and parameter cardinality.
    pub ports: PortCounts,
    /// Sidebar display name (Chinese).
    pub name: &'static str,
    /// Short English / code token (e.g. `"VCO"`).
    pub code: &'static str,
    /// Title drawn on the canvas node header.
    pub canvas_label: &'static str,
    /// One-line summary inside the node body.
    pub summary: &'static str,
    /// Inspector panel description.
    pub inspector_blurb: &'static str,
    /// Library section.
    pub section: ModuleSection,
    /// When false, the library entry is visible but disabled.
    pub addable: bool,
    /// Default values; length must equal `ports.params`.
    pub param_defaults: &'static [f32],
    /// UI labels; length must equal `ports.params`.
    pub param_labels: &'static [&'static str],
}

impl ModuleDesc {
    /// Default for `param`, or `0.0` if out of range / missing.
    #[must_use]
    pub fn default_param(self, param: ParamId) -> f32 {
        self.param_defaults
            .get(param.raw() as usize)
            .copied()
            .unwrap_or(0.0)
    }

    /// Label for `param`, or a generic fallback.
    #[must_use]
    pub fn param_label(self, param: ParamId) -> &'static str {
        self.param_labels
            .get(param.raw() as usize)
            .copied()
            .unwrap_or("参数")
    }
}

const VCO_PARAM_DEFAULTS: &[f32] = &[440.0, 0.5, 0.0];
const VCO_PARAM_LABELS: &[&str] = &["频率 (Hz)", "振幅", "波形"];

/// Every built-in kind, including planned (non-addable) entries.
pub const MODULE_CATALOG: &[ModuleDesc] = &[
    ModuleDesc {
        kind: NodeKind::Vco,
        ports: PortCounts {
            inputs: 0,
            outputs: 1,
            params: 3,
        },
        name: "振荡器",
        code: "VCO",
        canvas_label: "振荡器 · VCO",
        summary: "",
        inspector_blurb: "",
        section: ModuleSection::Core,
        addable: true,
        param_defaults: VCO_PARAM_DEFAULTS,
        param_labels: VCO_PARAM_LABELS,
    },
    ModuleDesc {
        kind: NodeKind::Output,
        ports: PortCounts {
            inputs: 1,
            outputs: 0,
            params: 0,
        },
        name: "输出",
        code: "Output",
        canvas_label: "输出 · Output",
        summary: "音频设备输出",
        inspector_blurb: "将输入信号发送到音频设备。",
        section: ModuleSection::Core,
        addable: true,
        param_defaults: &[],
        param_labels: &[],
    },
    ModuleDesc {
        kind: NodeKind::Delay,
        ports: PortCounts {
            inputs: 1,
            outputs: 1,
            params: 0,
        },
        name: "块延迟",
        code: "Delay",
        canvas_label: "块延迟 · Delay",
        summary: "1 block · 块延迟",
        inspector_blurb: "将信号延迟一个音频块。",
        section: ModuleSection::Utility,
        addable: true,
        param_defaults: &[],
        param_labels: &[],
    },
    ModuleDesc {
        kind: NodeKind::Silence,
        ports: PortCounts {
            inputs: 0,
            outputs: 1,
            params: 0,
        },
        name: "静音源",
        code: "Silence",
        canvas_label: "静音源 · Silence",
        summary: "静音信号源",
        inspector_blurb: "输出恒为零的静音信号。",
        section: ModuleSection::Utility,
        addable: true,
        param_defaults: &[],
        param_labels: &[],
    },
    ModuleDesc {
        kind: NodeKind::Vcf,
        ports: PortCounts {
            inputs: 2,
            outputs: 1,
            params: 2,
        },
        name: "滤波器",
        code: "VCF",
        canvas_label: "滤波器 · VCF",
        summary: "计划中",
        inspector_blurb: "此模块暂无可编辑参数。",
        section: ModuleSection::Planned,
        addable: false,
        param_defaults: &[0.0, 0.0],
        param_labels: &["参数", "参数"],
    },
    ModuleDesc {
        kind: NodeKind::Vca,
        ports: PortCounts {
            inputs: 2,
            outputs: 1,
            params: 1,
        },
        name: "放大器",
        code: "VCA",
        canvas_label: "放大器 · VCA",
        summary: "计划中",
        inspector_blurb: "此模块暂无可编辑参数。",
        section: ModuleSection::Planned,
        addable: false,
        param_defaults: &[0.0],
        param_labels: &["参数"],
    },
    ModuleDesc {
        kind: NodeKind::Adsr,
        ports: PortCounts {
            inputs: 1,
            outputs: 1,
            params: 4,
        },
        name: "包络",
        code: "ADSR",
        canvas_label: "包络 · ADSR",
        summary: "计划中",
        inspector_blurb: "此模块暂无可编辑参数。",
        section: ModuleSection::Planned,
        addable: false,
        param_defaults: &[0.0, 0.0, 0.0, 0.0],
        param_labels: &["参数", "参数", "参数", "参数"],
    },
    ModuleDesc {
        kind: NodeKind::Lfo,
        ports: PortCounts {
            inputs: 0,
            outputs: 1,
            params: 2,
        },
        name: "LFO",
        code: "LFO",
        canvas_label: "LFO",
        summary: "计划中",
        inspector_blurb: "此模块暂无可编辑参数。",
        section: ModuleSection::Planned,
        addable: false,
        param_defaults: &[0.0, 0.0],
        param_labels: &["参数", "参数"],
    },
    ModuleDesc {
        kind: NodeKind::Mixer,
        ports: PortCounts {
            inputs: 4,
            outputs: 1,
            params: 1,
        },
        name: "混音器",
        code: "Mixer",
        canvas_label: "混音器 · Mixer",
        summary: "计划中",
        inspector_blurb: "此模块暂无可编辑参数。",
        section: ModuleSection::Planned,
        addable: false,
        param_defaults: &[0.0],
        param_labels: &["参数"],
    },
];

impl NodeKind {
    /// Static descriptor for this kind.
    #[must_use]
    pub const fn desc(self) -> &'static ModuleDesc {
        // Manual index keeps this `const` without scanning MODULE_CATALOG.
        match self {
            Self::Vco => &MODULE_CATALOG[0],
            Self::Output => &MODULE_CATALOG[1],
            Self::Delay => &MODULE_CATALOG[2],
            Self::Silence => &MODULE_CATALOG[3],
            Self::Vcf => &MODULE_CATALOG[4],
            Self::Vca => &MODULE_CATALOG[5],
            Self::Adsr => &MODULE_CATALOG[6],
            Self::Lfo => &MODULE_CATALOG[7],
            Self::Mixer => &MODULE_CATALOG[8],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MODULE_CATALOG, ModuleDesc};
    use crate::NodeKind;

    #[test]
    fn catalog_covers_every_kind_exactly_once() {
        let mut seen = Vec::new();
        for desc in MODULE_CATALOG {
            assert!(
                !seen.contains(&desc.kind),
                "duplicate kind {:?}",
                desc.kind
            );
            seen.push(desc.kind);
            assert_eq!(desc.kind.desc().kind, desc.kind);
        }
        let all = [
            NodeKind::Vco,
            NodeKind::Vcf,
            NodeKind::Vca,
            NodeKind::Adsr,
            NodeKind::Lfo,
            NodeKind::Mixer,
            NodeKind::Output,
            NodeKind::Silence,
            NodeKind::Delay,
        ];
        for kind in all {
            assert!(
                MODULE_CATALOG.iter().any(|d| d.kind == kind),
                "missing {:?}",
                kind
            );
        }
        assert_eq!(seen.len(), all.len());
    }

    #[test]
    fn param_slices_match_port_counts() {
        for desc in MODULE_CATALOG {
            assert_eq!(
                desc.param_defaults.len() as u32,
                desc.ports.params,
                "{:?} defaults",
                desc.kind
            );
            assert_eq!(
                desc.param_labels.len() as u32,
                desc.ports.params,
                "{:?} labels",
                desc.kind
            );
        }
    }

    #[test]
    fn desc_ports_match_legacy_layout() {
        let cases: &[(NodeKind, u32, u32, u32)] = &[
            (NodeKind::Vco, 0, 1, 3),
            (NodeKind::Vcf, 2, 1, 2),
            (NodeKind::Vca, 2, 1, 1),
            (NodeKind::Adsr, 1, 1, 4),
            (NodeKind::Lfo, 0, 1, 2),
            (NodeKind::Mixer, 4, 1, 1),
            (NodeKind::Output, 1, 0, 0),
            (NodeKind::Silence, 0, 1, 0),
            (NodeKind::Delay, 1, 1, 0),
        ];
        for &(kind, inputs, outputs, params) in cases {
            let ModuleDesc { ports, .. } = *kind.desc();
            assert_eq!(ports.inputs, inputs, "{kind:?} inputs");
            assert_eq!(ports.outputs, outputs, "{kind:?} outputs");
            assert_eq!(ports.params, params, "{kind:?} params");
        }
    }
}
