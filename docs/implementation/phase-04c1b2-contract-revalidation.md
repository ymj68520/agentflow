# Phase 4C.1B.2 — Fresh ComponentContract Revalidation

D6 resolves every component reference used by the resolver and every target
export reference to an exact ContractAsset. It decodes the asset with the D5
module-set shape into a fresh `ComponentContract`, then calls the formal
`check_contract(raw, proof_context)`.

The only resolver-insertable values are the newly returned checked wrappers.
Their private `proof_context()` reference is compared by pointer identity with
the one D5 context object. The proof must be fully Proven. The canonicalized
contract is re-encoded with the existing ComponentContract shape and compared
byte-for-byte with the submitted asset; its existing ComponentContractDigest
is recomputed and compared with the pin. A valid digest is never used as a
substitute for semantic validation.

All references are loaded before producing the D6 collection. Any missing or
invalid contract returns an error, not a partial checked set. A D6 proof
failure diagnostic retains non-Proven dimensions and action-guard verdicts;
the diagnostic cannot be used to construct a Checked wrapper.
