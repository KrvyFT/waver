//! Graph IR, compiled schedule, and lock-free parameter cells.
//!
//! This crate is shared by the GUI and the audio engine. It must not depend on
//! concrete DSP nodes or on windowing/audio-host crates.

mod command;
mod error;
mod graph;
mod ids;
mod param;
mod schedule;
mod status;

pub use command::RtCommand;
pub use error::GraphError;
pub use graph::{Edge, Graph, Node, NodeKind, PortRef};
pub use ids::{NodeId, ParamId, PortId};
pub use param::ParamCell;
pub use schedule::Schedule;
pub use status::EngineStatus;
