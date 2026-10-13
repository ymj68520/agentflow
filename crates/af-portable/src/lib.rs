//! Registered Portable Proof Inputs v1 and closed D0-D4 validation.
//!
//! This crate returns raw/checked-by-stage input capabilities. It contains no
//! functions that create Checked contracts/IR, a resolver snapshot, Root,
//! RootRecovery or a RootExport result.

mod codec;
mod error;
mod limits;
mod orders;
mod schema;
mod support;
mod validation;

pub use codec::{CanonicalPortableBundle, DecodedPortableBundle, PortableProofCodec};
pub use error::{PortableProofError, ValidationStage};
pub use limits::DecoderLimits;
pub use support::SupportedPortableStaticInputs;
pub use validation::{ContentValidatedPortableBundle, ReferenceValidatedPortableBundle};
