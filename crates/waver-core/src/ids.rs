//! Stable identifiers for nodes, ports, and parameters.

/// Opaque node handle. Assigned by [`crate::Graph::insert`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct NodeId(u32);

impl NodeId {
    /// Wrap a raw index. Prefer [`crate::Graph::insert`] in application code.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Underlying index used by the compiler and tests.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Opaque port handle on a node (input or output pin).
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct PortId(u32);

impl PortId {
    /// Wrap a raw port index defined by the node kind.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Underlying port index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

/// Opaque parameter handle (knob / CV amount) on a node.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ParamId(u32);

impl ParamId {
    /// Wrap a raw parameter index defined by the node kind.
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Underlying parameter index.
    pub const fn raw(self) -> u32 {
        self.0
    }
}
