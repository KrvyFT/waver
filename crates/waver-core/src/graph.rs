//! Editable graph IR. Compiled on the GUI thread, never on the audio callback.

use crate::{GraphError, NodeId, PortId, Schedule};

/// Built-in module kinds. DSP implementations live in `waver-dsp`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NodeKind {
    Vco,
    Vcf,
    Vca,
    Adsr,
    Lfo,
    Mixer,
    Output,
    Silence,
}

/// A module instance in the patch.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Node {
    /// Stable id assigned at insertion.
    pub id: NodeId,
    /// Which DSP type this slot will run after compile.
    pub kind: NodeKind,
}

/// One directed cable: source output → destination input.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Edge {
    /// Upstream port.
    pub from: PortRef,
    /// Downstream port.
    pub to: PortRef,
}

/// Node + pin addressing a single jack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortRef {
    /// Owner node.
    pub node: NodeId,
    /// Pin on that node.
    pub port: PortId,
}

/// Patch being edited in the UI.
///
/// `compile` is a stub: it always returns an empty [`Schedule`]. Kahn sorting
/// and delay insertion come in a later change.
#[derive(Clone, Debug, Default)]
pub struct Graph {
    nodes: Vec<Node>,
    edges: Vec<Edge>,
    next_id: u32,
}

impl Graph {
    /// Empty patch.
    pub fn new() -> Self {
        Self::default()
    }

    /// Nodes in insertion order.
    pub fn nodes(&self) -> &[Node] {
        &self.nodes
    }

    /// Directed cables.
    pub fn edges(&self) -> &[Edge] {
        &self.edges
    }

    /// Append a node and return its id.
    pub fn insert(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.nodes.push(Node { id, kind });
        id
    }

    /// Append a cable. Fan-in summing is applied at compile time (not yet).
    pub fn connect(&mut self, from: PortRef, to: PortRef) {
        self.edges.push(Edge { from, to });
    }

    /// Build an audio-thread schedule.
    ///
    /// # Errors
    ///
    /// Reserved for cycle detection. The stub never fails.
    pub fn compile(&self) -> Result<Schedule, GraphError> {
        Ok(Schedule::empty())
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, NodeKind};

    #[test]
    fn empty_graph_compiles_to_empty_schedule() {
        let graph = Graph::new();
        let schedule = graph
            .compile()
            .expect("empty graph must compile in the stub");
        assert!(schedule.is_empty());
        assert!(schedule.order().is_empty());
    }

    #[test]
    fn insert_does_not_change_stub_schedule() {
        let mut graph = Graph::new();
        graph.insert(NodeKind::Vco);
        graph.insert(NodeKind::Output);
        let schedule = graph.compile().expect("stub compile");
        assert!(schedule.is_empty());
    }
}
