//! Graph IR, compiled schedule, and lock-free parameter cells.
//!
//! This crate is shared by the GUI and the audio engine. It must not depend on
//! concrete DSP nodes or on windowing/audio-host crates.

mod command;
mod compile;
mod error;
mod graph;
mod ids;
mod param;
mod ports;
mod schedule;
mod status;

pub use command::RtCommand;
pub use error::{GraphError, PortDirection};
pub use graph::{Edge, Graph, Node, NodeKind, PortCounts, PortRef};
pub use ids::{NodeId, ParamId, PortId};
pub use param::ParamCell;
pub use schedule::{Link, Schedule};
pub use status::EngineStatus;
