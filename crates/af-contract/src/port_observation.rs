//! Branch-created raw port observations. Never a checked contract or authority.
use crate::{ComponentContract, ContractError, StaticContractProofContext};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortDirection {
    Input,
    Output,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortOccurrence {
    pub direction: PortDirection,
    pub original_ordinal: usize,
    pub sorted_ordinal: usize,
    pub port_no: u32,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortCondition {
    PositiveFresh,
    Zero,
    Duplicate,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PortEvent {
    pub occurrence: PortOccurrence,
    pub predecessor: Option<(usize, u32)>,
    pub condition: PortCondition,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PortPrefixStop {
    ComponentIdentity,
    PortRejection(PortOccurrence),
    SchemaResolution(PortOccurrence),
}
#[derive(Default)]
pub(crate) struct PortTrace {
    pub events: Vec<PortEvent>,
    pub stop: Option<PortPrefixStop>,
}
/// Private provenance: callers may inspect but cannot fabricate or edit a run.
pub struct RawPortObservation<'r, 'c, 'u> {
    raw: &'r ComponentContract,
    context: &'c StaticContractProofContext<'u>,
    trace: PortTrace,
    diagnostic: Option<ContractError>,
}
impl<'r, 'c, 'u> RawPortObservation<'r, 'c, 'u> {
    pub fn declaration(&self) -> &'r ComponentContract {
        self.raw
    }
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.context
    }
    pub fn events(&self) -> &[PortEvent] {
        &self.trace.events
    }
    pub fn stop(&self) -> Option<PortPrefixStop> {
        self.trace.stop
    }
    pub fn diagnostic(&self) -> Option<&ContractError> {
        self.diagnostic.as_ref()
    }
    pub fn canonical_validation_succeeded(&self) -> bool {
        self.diagnostic.is_none()
    }
}
pub fn observe_raw_ports<'r, 'c, 'u>(
    raw: &'r ComponentContract,
    context: &'c StaticContractProofContext<'u>,
) -> RawPortObservation<'r, 'c, 'u> {
    let mut trace = PortTrace::default();
    let diagnostic = crate::canonical::canonicalize_observed(raw, context, &mut trace).err();
    RawPortObservation {
        raw,
        context,
        trace,
        diagnostic,
    }
}
