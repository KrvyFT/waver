//! Editable graph IR. Compiled on the GUI thread, never on the audio callback.

use crate::{CompiledPatch, GraphError, NodeId, ParamRegistry, PortId, Schedule};

pub use crate::ports::PortCounts;

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
    /// One-block delay line; inserted by the compiler to break feedback loops.
    Delay,
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

    /// Look up a node by id.
    pub fn node(&self, id: NodeId) -> Option<&Node> {
        self.nodes.iter().find(|node| node.id == id)
    }

    /// Append a node and return its id.
    pub fn insert(&mut self, kind: NodeKind) -> NodeId {
        let id = NodeId::new(self.next_id);
        self.next_id = self.next_id.saturating_add(1);
        self.nodes.push(Node { id, kind });
        id
    }

    /// Remove a node and any cables touching it.
    pub fn remove(&mut self, id: NodeId) -> bool {
        let before = self.nodes.len();
        self.nodes.retain(|node| node.id != id);
        if self.nodes.len() == before {
            return false;
        }
        self.edges
            .retain(|edge| edge.from.node != id && edge.to.node != id);
        true
    }

    /// Append a cable. Fan-in summing is recorded in the compiled [`Schedule`].
    /// Duplicate `(from, to)` pairs are ignored.
    pub fn connect(&mut self, from: PortRef, to: PortRef) {
        if self
            .edges
            .iter()
            .any(|edge| edge.from == from && edge.to == to)
        {
            return;
        }
        self.edges.push(Edge { from, to });
    }

    /// Remove a cable by index in [`Self::edges`].
    pub fn disconnect_edge(&mut self, index: usize) -> bool {
        if index >= self.edges.len() {
            return false;
        }
        self.edges.remove(index);
        true
    }

    /// Compile graph into a [`CompiledPatch`], optionally preserving parameter values.
    ///
    /// # Errors
    ///
    /// Same as [`Self::compile`].
    pub fn compile_patch(
        &self,
        existing: Option<&ParamRegistry>,
    ) -> Result<CompiledPatch, GraphError> {
        let schedule = self.compile()?;
        Ok(CompiledPatch::from_schedule(schedule, existing))
    }

    /// Next id that would be assigned by [`Self::insert`].
    pub(crate) fn next_node_id(&self) -> u32 {
        self.next_id
    }

    /// Build an audio-thread schedule.
    ///
    /// Validates ports, topologically sorts with Kahn's algorithm, and inserts
    /// [`NodeKind::Delay`] nodes when feedback would otherwise leave a cycle.
    ///
    /// # Errors
    ///
    /// Returns [`GraphError`] when ports are invalid, a self-loop is present,
    /// or a cycle cannot be broken.
    pub fn compile(&self) -> Result<Schedule, GraphError> {
        crate::compile::compile_graph(self)
    }
}

#[cfg(test)]
mod tests {
    use super::{Graph, NodeKind, PortId, PortRef};

    fn port(node: crate::NodeId, index: u32) -> PortRef {
        PortRef {
            node,
            port: PortId::new(index),
        }
    }

    #[test]
    fn empty_graph_compiles_to_empty_schedule() {
        let graph = Graph::new();
        let schedule = graph.compile().expect("empty graph compiles");
        assert!(schedule.is_empty());
        assert!(schedule.order().is_empty());
    }

    #[test]
    fn disconnected_nodes_all_appear_in_order() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let out = graph.insert(NodeKind::Output);
        let schedule = graph.compile().expect("disconnected graph compiles");
        assert_eq!(schedule.order().len(), 2);
        assert!(schedule.order().contains(&vco));
        assert!(schedule.order().contains(&out));
    }

    #[test]
    fn remove_node_drops_incident_edges() {
        let mut graph = Graph::new();
        let vco = graph.insert(NodeKind::Vco);
        let out = graph.insert(NodeKind::Output);
        graph.connect(port(vco, 0), port(out, 0));
        assert!(graph.remove(vco));
        assert_eq!(graph.edges().len(), 0);
        let schedule = graph.compile().expect("single output node");
        assert_eq!(schedule.order(), &[out]);
    }
}
