//! Commands sent GUI → audio through a lock-free SPSC queue.

use std::sync::Arc;

use crate::CompiledPatch;

/// Topology / transport events. High-rate knobs use [`crate::ParamCell`] instead.
pub enum RtCommand {
    /// Replace the running patch (schedule + param registry). Allocated on the GUI thread.
    SwapSchedule(Arc<CompiledPatch>),
    /// Silence voices / envelopes. No-op until a voice allocator exists.
    AllNotesOff,
}
