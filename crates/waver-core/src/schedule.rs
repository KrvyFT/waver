//! Compiled, audio-thread-only execution order.

use crate::NodeId;

/// Linear node order consumed by the audio callback.
///
/// Built on the GUI thread and swapped in via [`crate::RtCommand::SwapSchedule`].
/// The audio thread must treat this as read-only.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Schedule {
    order: Vec<NodeId>,
}

impl Schedule {
    /// No nodes. The engine fills output with silence.
    pub fn empty() -> Self {
        Self { order: Vec::new() }
    }

    /// Topological order. Empty in the skeleton (Kahn is not implemented yet).
    pub fn order(&self) -> &[NodeId] {
        &self.order
    }

    /// True when the callback has nothing to process besides silence.
    pub fn is_empty(&self) -> bool {
        self.order.is_empty()
    }
}
