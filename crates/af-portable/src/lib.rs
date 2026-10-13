//! Registered Portable Proof Inputs v1, closed D0-D4 validation, and exact
//! D5-D8 static input reconstruction.
//!
//! The D5-D8 entry points rebuild the current explicit checker context and
//! freshly invoke the formal ComponentContract, Semantic IR, Phase 3C and
//! computation resolver validators. Checked references remain scoped to the
//! owned D5 arena; only non-authoritative projections leave that scope. This
//! crate does not construct Root, RootRecovery, RootExport, publication or
//! runtime capabilities.

mod codec;
mod error;
mod limits;
mod orders;
mod pipeline;
mod reconstruction;
mod schema;
mod support;
mod validation;

pub use codec::{CanonicalPortableBundle, DecodedPortableBundle, PortableProofCodec};
pub use error::{PortableProofError, ValidationStage};
pub use limits::DecoderLimits;
pub use pipeline::{
    CheckedContractProjection, CheckedIrProjection, ContentReferenceProjection,
    ReconstructedStaticInputs, ResolverTargetProjection, SourceAssociationProjection,
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
