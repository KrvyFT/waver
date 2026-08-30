//! Compiled, audio-thread-only execution order and routing.

use std::collections::HashMap;

use crate::{NodeId, NodeKind, PortRef};

/// One directed cable in the compiled patch: upstream output → downstream input.
///
/// Multiple [`Link`] rows may target the same input jack; the engine sums them
/// (fan-in) at run time.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Link {
    /// Source jack.
    pub from: PortRef,
    /// Destination jack.
    pub to: PortRef,
}

/// Linear node order and wiring consumed by the audio callback.
///
/// Built on the GUI thread and swapped in via [`crate::RtCommand::SwapSchedule`].
/// The audio thread must treat this as read-only.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    order: Vec<NodeId>,
    kinds: HashMap<NodeId, NodeKind>,
    links: Vec<Link>,
}

impl Schedule {
    /// No nodes. The engine fills output with silence.
    #[must_use]
    pub fn empty() -> Self {
        Self::default()
    }

    /// Build a schedule from a successful compile.
    pub(crate) fn new(
        order: Vec<NodeId>,
        kinds: HashMap<NodeId, NodeKind>,
        links: Vec<Link>,
    ) -> Self {
        Self { order, kinds, links }
    }

    /// Topological execution order (includes compiler-inserted delay nodes).
    #[must_use]
    pub fn order(&self) -> &[NodeId] {
        &self.order
    }

    /// All directed cables after compile (including through delay nodes).
    #[must_use]
    pub fn links(&self) -> &[Link] {
        &self.links
    }

    /// Look up the module kind for a scheduled node.
    #[must_use]
    pub fn kind_of(&self, id: NodeId) -> Option<NodeKind> {
        self.kinds.get(&id).copied()
    }

    /// Iterator over `(NodeId, NodeKind)` for every scheduled node.
    pub fn nodes(&self) -> impl Iterator<Item = (NodeId, NodeKind)> + '_ {
        self.kinds.iter().map(|(&id, &kind)| (id, kind))
    }

    /// Incoming links for one input jack.
    #[must_use]
    pub fn sources_to(&self, to: PortRef) -> Vec<PortRef> {
        self.links
            .iter()
            .filter(|link| link.to == to)
            .map(|link| link.from)
            .collect()
    }

    /// True when the callback has nothing to process besides silence.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}
