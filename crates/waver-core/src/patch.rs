//! Compiled patch bundle: schedule + shared parameter cells for GUI ↔ audio.

use std::collections::HashMap;
use std::sync::Arc;

use crate::{NodeId, NodeKind, ParamCell, ParamId, Schedule};

/// Default parameter value for a `(NodeKind, ParamId)` slot.
#[must_use]
pub fn default_param_value(kind: NodeKind, param: ParamId) -> f32 {
    match kind {
        NodeKind::Vco => match param.raw() {
            0 => 440.0, // freq Hz
            1 => 0.5,   // amp
            2 => 0.0,   // wave (sine)
            _ => 0.0,
        },
        _ => 0.0,
    }
}

/// Human-readable parameter labels for the UI (index = ParamId raw).
#[must_use]
pub fn param_label(kind: NodeKind, param: ParamId) -> &'static str {
    match kind {
        NodeKind::Vco => match param.raw() {
            0 => "频率 (Hz)",
            1 => "振幅",
            2 => "波形",
            _ => "参数",
        },
        _ => "参数",
    }
}

/// Lock-free parameter cells keyed by node + param index.
#[derive(Clone, Default)]
pub struct ParamRegistry {
    cells: HashMap<(NodeId, ParamId), Arc<ParamCell>>,
}

impl ParamRegistry {
    /// Empty registry.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create cells with defaults for every param slot on each scheduled node.
    #[must_use]
    pub fn with_defaults(schedule: &Schedule) -> Self {
        let mut registry = Self::new();
        for (node, kind) in schedule.nodes() {
            let count = kind.port_counts().params;
            for raw in 0..count {
                let param = ParamId::new(raw);
                let value = default_param_value(kind, param);
                registry
                    .cells
                    .insert((node, param), Arc::new(ParamCell::new(value)));
            }
        }
        registry
    }

    /// Reuse existing cells for surviving nodes; create defaults for new ones.
    #[must_use]
    pub fn merge(existing: Option<&Self>, schedule: &Schedule) -> Self {
        let mut registry = Self::new();
        for (node, kind) in schedule.nodes() {
            let count = kind.port_counts().params;
            for raw in 0..count {
                let param = ParamId::new(raw);
                let cell = existing
                    .and_then(|prev| prev.get(node, param))
                    .unwrap_or_else(|| {
                        Arc::new(ParamCell::new(default_param_value(kind, param)))
                    });
                registry.cells.insert((node, param), cell);
            }
        }
        registry
    }

    /// Look up a shared parameter cell.
    #[must_use]
    pub fn get(&self, node: NodeId, param: ParamId) -> Option<Arc<ParamCell>> {
        self.cells.get(&(node, param)).cloned()
    }

    /// Iterate all `(NodeId, ParamId, Arc<ParamCell>)` entries.
    pub fn iter(&self) -> impl Iterator<Item = (NodeId, ParamId, Arc<ParamCell>)> + '_ {
        self.cells
            .iter()
            .map(|(&(node, param), cell)| (node, param, Arc::clone(cell)))
    }
}

/// Schedule plus parameter registry, swapped atomically to the audio thread.
#[derive(Clone)]
pub struct CompiledPatch {
    /// Execution order and routing.
    pub schedule: Schedule,
    /// Knobs shared with the GUI.
    pub params: ParamRegistry,
}

impl CompiledPatch {
    /// Empty patch (silence).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            schedule: Schedule::empty(),
            params: ParamRegistry::new(),
        }
    }

    /// Build from a compiled schedule, optionally preserving prior param values.
    #[must_use]
    pub fn from_schedule(schedule: Schedule, existing: Option<&ParamRegistry>) -> Self {
        Self {
            params: ParamRegistry::merge(existing, &schedule),
            schedule,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{CompiledPatch, ParamRegistry, default_param_value};
    use crate::{Graph, NodeKind, ParamId, Schedule};

    #[test]
    fn default_vco_params() {
        assert!((default_param_value(NodeKind::Vco, ParamId::new(0)) - 440.0).abs() < f32::EPSILON);
        assert!((default_param_value(NodeKind::Vco, ParamId::new(1)) - 0.5).abs() < f32::EPSILON);
    }

    #[test]
    fn merge_preserves_existing_node_params() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let schedule = graph.compile().expect("compile");
        let first = CompiledPatch::from_schedule(schedule.clone(), None);
        if let Some(cell) = first.params.get(vco, ParamId::new(0)) {
            cell.set(880.0);
        }

        let merged = CompiledPatch::from_schedule(schedule, Some(&first.params));
        let cell = merged
            .params
            .get(vco, ParamId::new(0))
            .expect("freq cell");
        assert!((cell.value() - 880.0).abs() < f32::EPSILON);
    }

    #[test]
    fn with_defaults_covers_scheduled_nodes() {
        let schedule = Schedule::empty();
        let registry = ParamRegistry::with_defaults(&schedule);
        assert!(registry.iter().next().is_none());
    }
}
