//! Registered Portable Proof Inputs v1, closed D0-D4 validation, exact D5-D8
//! reconstruction, and fresh D9-D10 static RootExport revalidation.
//!
//! The D5-D8 entry points rebuild the current explicit checker context and
//! freshly invoke the formal ComponentContract, Semantic IR, Phase 3C and
//! computation resolver validators. D9-D10 reuse the formal Root/Recovery
//! derivation and RootExport checker under that same scope. No publication or
//! runtime capability is constructed.

mod codec;
mod error;
mod limits;
mod orders;
mod pipeline;
mod reconstruction;
mod root_revalidation;
mod schema;
mod support;
mod validation;

#[cfg(test)]
#[path = "../tests/support/d9_full_conformance.rs"]
mod d9_full_conformance;

pub use codec::{CanonicalPortableBundle, DecodedPortableBundle, PortableProofCodec};
pub use error::{PortableProofError, ValidationStage};
pub use limits::DecoderLimits;
pub use pipeline::{
    CheckedContractProjection, CheckedIrProjection, ContentReferenceProjection,
    PortableStaticProofAnalysis, ReconstructedStaticInputs, ResolverTargetProjection,
    RootRecoveryProjection, RootSummaryProjection, SourceAssociationProjection,
    SourceAssociationTargetProjection, SubflowTargetIdentityProjection,
};
pub use reconstruction::{
    EffectCatalogProjection, ExecutionCapabilityProjection, FlowSinkProjection, ModuleProjection,
    NamedOperationProjection, PlainChecksumProjection, ReconstructedStaticContext,
    RegionCompatibilityProjection, RelationProjection, ResourceDescriptorProjection,
    RuntimeProfileProjection, StaticContextProjection, UsageMeterProjection, UsageUnitProjection,
};
pub use support::SupportedPortableStaticInputs;
pub use validation::{ContentValidatedPortableBundle, ReferenceValidatedPortableBundle};
