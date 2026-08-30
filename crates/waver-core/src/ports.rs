//! Input / output / parameter counts per [`crate::NodeKind`].

use crate::{NodeKind, ParamId, PortId};

/// Port and parameter cardinality for a module kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PortCounts {
    /// Audio / CV input jacks.
    pub inputs: u32,
    /// Audio / CV output jacks.
    pub outputs: u32,
    /// Knobs and CV amounts owned by the node.
    pub params: u32,
}

impl NodeKind {
    /// Static jack and parameter layout for this kind.
    #[must_use]
    pub const fn port_counts(self) -> PortCounts {
        match self {
            Self::Vco => PortCounts {
                inputs: 0,
                outputs: 1,
                params: 3,
            },
            Self::Vcf => PortCounts {
                inputs: 2,
                outputs: 1,
                params: 2,
            },
            Self::Vca => PortCounts {
                inputs: 2,
                outputs: 1,
                params: 1,
            },
            Self::Adsr => PortCounts {
                inputs: 1,
                outputs: 1,
                params: 4,
            },
            Self::Lfo => PortCounts {
                inputs: 0,
                outputs: 1,
                params: 2,
            },
            Self::Mixer => PortCounts {
                inputs: 4,
                outputs: 1,
                params: 1,
            },
            Self::Output => PortCounts {
                inputs: 1,
                outputs: 0,
                params: 0,
            },
            Self::Silence => PortCounts {
                inputs: 0,
                outputs: 1,
                params: 0,
            },
            Self::Delay => PortCounts {
                inputs: 1,
                outputs: 1,
                params: 0,
            },
        }
    }

    /// True when `port` indexes a valid input jack on this kind.
    #[must_use]
    pub fn is_input_port(self, port: PortId) -> bool {
        port.raw() < self.port_counts().inputs
    }

    /// True when `port` indexes a valid output jack on this kind.
    #[must_use]
    pub fn is_output_port(self, port: PortId) -> bool {
        port.raw() < self.port_counts().outputs
    }

    /// True when `param` indexes a valid parameter slot on this kind.
    #[must_use]
    pub fn is_param(self, param: ParamId) -> bool {
        param.raw() < self.port_counts().params
    }
}
