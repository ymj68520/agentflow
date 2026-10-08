//! `af-execution` — static Execution Requirements X.

pub mod canonical;
pub mod compatibility;
pub mod error;
pub mod matching;
pub mod model;
pub mod profile;
pub mod universe;

pub use canonical::canonicalize_execution_requirements;
pub use compatibility::{CompatibilityVerdict, ExecutionCompatibility, execution_compatibility};
pub use error::{ExecutionError, kind};
pub use matching::{
    MatchRejected, MatchUnproven, StaticPlacementProof, match_execution_requirements,
};
pub use model::*;
pub use profile::{StaticCapability, StaticExecutorProfile, StaticQuantity, StaticSandbox};
pub use universe::*;
