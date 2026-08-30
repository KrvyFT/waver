//! Graph compile / topology errors.

use crate::{NodeId, PortId};

/// Which side of a jack failed validation.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortDirection {
    /// Expected an input jack index.
    Input,
    /// Expected an output jack index.
    Output,
}

/// Failures produced while compiling a [`crate::Graph`] into a [`crate::Schedule`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum GraphError {
    /// Remaining nodes after Kahn's algorithm; automatic delay insertion did not help.
    #[error("graph contains a cycle")]
    Cycle,
    /// Edge or lookup references a node id that is not in the graph.
    #[error("unknown node {node:?}")]
    UnknownNode {
        /// Missing node handle.
        node: NodeId,
    },
    /// Port index is out of range for the node's kind.
    #[error("invalid {direction:?} port {port:?} on node {node:?}")]
    InvalidPort {
        /// Owner node.
        node: NodeId,
        /// Offending port index.
        port: PortId,
        /// Whether an input or output jack was expected.
        direction: PortDirection,
    },
    /// A cable connects a node to itself.
    #[error("self-loop on node {node:?}")]
    SelfLoop {
        /// Node connected to itself.
        node: NodeId,
    },
}
