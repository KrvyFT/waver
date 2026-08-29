//! Graph compile / topology errors.

/// Failures produced while compiling a [`crate::Graph`] into a [`crate::Schedule`].
#[derive(Clone, Debug, Eq, PartialEq, thiserror::Error)]
pub enum GraphError {
    /// Remaining nodes after Kahn's algorithm; a cycle was not broken by a delay.
    #[error("graph contains a cycle")]
    Cycle,
}
