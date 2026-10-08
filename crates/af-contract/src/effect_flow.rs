//! Public exact EffectFlowPartition proof projection.

use crate::model::ComponentContract;
use crate::proof::DimensionProof;
use crate::validate::{StaticContractProofContext, analyze_dimension};

#[must_use]
pub fn analyze(
    contract: &ComponentContract,
    context: &StaticContractProofContext<'_>,
) -> DimensionProof {
    analyze_dimension(contract, context, "information-flow")
}
