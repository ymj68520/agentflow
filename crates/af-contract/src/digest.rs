//! Checked ComponentContract digest generation.

use crate::canonical::CanonicalComponentContract;
use crate::error::ContractError;
use crate::proof::CheckedComponentContract;
use crate::validate::StaticContractProofContext;
use af_core::{DigestDomain, DigestRef};

/// Computes the semantic body digest using the existing HashV1 domain.
/// Publication versions, implementation identity, price snapshots and proof
/// results are excluded. A changed relation pin invalidates the checked
/// proof and is rejected here.
pub fn component_contract_digest<'universe>(
    checked: &CheckedComponentContract<'_, 'universe>,
    context: &StaticContractProofContext<'universe>,
) -> Result<DigestRef, ContractError> {
    if !std::ptr::eq(checked.context, context)
        || !checked.proof.is_proven()
        || checked.proof.relation_catalogs != context.relations.pins()
    {
        return Err(ContractError::new(
            "AF-CONTRACT-0019",
            "proof_context_mismatch",
            "checked proof is not proven for the exact current relation-catalog pins; revalidation is required",
        ));
    }
    let canonical = CanonicalComponentContract {
        contract: checked.contract.clone(),
    };
    let bytes = canonical.ace1_bytes(context)?;
    Ok(DigestRef::compute(DigestDomain::ComponentContract, &bytes))
}
