//! Commands sent GUI → audio through a lock-free SPSC queue.

use std::sync::Arc;

use crate::Schedule;

/// Topology / transport events. High-rate knobs use [`crate::ParamCell`] instead.
pub enum RtCommand {
    /// Replace the running schedule. Allocated on the GUI thread.
    SwapSchedule(Arc<Schedule>),
    /// Silence voices / envelopes. No-op until a voice allocator exists.
    AllNotesOff,
}
