# Phase 4B.2C — Proof completeness

## Proof levels

| Level | Audited claim | Result |
| --- | --- | --- |
| 1. Structural Closure | Every checked computation/owned child is reachable exactly once; leaves resolve or fail explicitly; no cycle or detached node | **Accepted** for `CheckedIrDocument` |
| 2. Conservative Envelope Soundness | Operator-specific E/Phi/U upper summaries are conservative; known source identities and unknowns remain | **Accepted** for the formal per-dimension envelopes |
| 3. Cross-Dimension Consistency | Local leaf K↔E/U, E↔R/F, and F↔X facts remain source-scoped through control composition | **Accepted locally; root K-action total needs an explicit coordinate rule** |
| 4. Root Export Compatibility | Derived root is compatible with a declared exported ComponentContract | **Unproven / not modeled** |
| 5. Implementation / Runtime Assurance | Body, grant, Worker, reservation, observed effect, recovery execution | **Not established and out of scope** |

## Reports and verdicts

`ControlProofReport` stores every per-dimension proof. Its `overall()` selects
the priority `Violation > RequiresEvidence > Unproven > Proven`; it does not
discard the other entries. `InheritedControlObligation` carries source node,
dimension, verdict, RuleId, reference, and missing-premise reason. The derived
summary also holds the exact in-process `ProofContextReference` used for the
analysis.

Examples checked against the implementation:

* A known complete E envelope with unknown Flow remains known in E and
  Unproven in Flow; overall is not promoted to Proven.
* A child RequiresEvidence inherited under a parent with a computable envelope
  remains RequiresEvidence in the parent's proof list.
* If one child is Violation and another RequiresEvidence, overall is Violation
  **and both source diagnostics remain in the stored proof dimensions**.
* Structural closure can be Proven while root-export compatibility is
  Unproven; `DerivedWorkflowContractSummary::overall()` merges them and does
  not call the tree/export pair Proven.
* Missing/unknown children use Unknown/Partial/PerChild values with unresolved
  source IDs. Empty is not synthesized from absent input.

## Context and digest identity

ComponentContractDigest commits stable contract bytes; it does not commit the
static proof context. The current `ProofContextReference` uses exact object
identity in the current process. Changing a relation/catalog context prevents
reuse of a checked leaf, while the same contract bytes can retain the same
contract digest. This is an in-process capability check, not a signature,
serialized identity, or cross-process proof.

That boundary is correct for this phase. A future persistent export assertion
must identify all proof assumptions through a separately specified stable
manifest or content identities; it cannot serialize the current pointer
reference or claim its persistence semantics.

## Scope and action proof states

Scope resource attenuation is Proven only for exact Phase 3C parent/declaration/
child sets. Authority/Budget are Unproven because the frozen IR has no ceilings;
settlement is RequiresEvidence. Compensation action eligibility is
RequiresEvidence when its potential upper is nonzero; the `RecoveryBlocked`
status remains separate from typed Failure. A mismatching/underdeclared action
Effect row is Violation. The local action bound does not assert which receipt
occurred or that an action ran.

## Gaps, not verdict compression

* `RootExportCompatibilityStatus::NotExpressedInSemanticIr` is a structural
  limitation, not a successful compatibility proof.
* AWAIT has an exact signal pin but unknown external provenance and no
  interaction-adapter result contract.
* CALL_SUBFLOW declaration/export resolution does not prove its implementation
  body; missing body evidence remains RequiresEvidence.
* The nested CompScope action summary has no explicit root-total coordinate
  for control families introduced by ancestors. The root E/U envelopes still
  carry those factors. The gap is classified as a design/aggregation issue in
  the audit, not silently hidden by an overall status.
