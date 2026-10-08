# Phase 4B.0 — Cross-dimension Counterexamples

These are design-level vectors, not production conformance fixtures. They
record API behavior where it can be reproduced and the conservative result a
future complete checker must return. `Violation` means the closed rule proves
the contract invalid; `Unproven` means no positive compatibility/security
claim may be made.

| # | Construction | Existing API observation | Required Component-level result | Class |
| --- | --- | --- | --- | --- |
| 1 | ε declares `data.write` with `requirement_port_no=7`; R contains only port 1 | `EffectSignature::validate` and `ResourceRequirementRow::validate` are independent; both can succeed | Violation: ε selector port 7 is absent from R | P2 |
| 2 | `data.write` is catalog-valid, R port 1 is a recognized `GitRepository`, but no relation says the operation can act on that resource type | `EffectCatalogUniverse` and `ResourceTypeUniverse` are independent; no cross product API exists | Unproven until an explicit operation/resource rule proves compatibility | P1 |
| 3 | ε has `data.write` on R1 and R2; F allows `external_effect(data.write)` | Both EffectAtoms resolve to the same `FlowSink::ExternalEffect(EffectOperationRef)` | Unproven for selector-specific allowance; current F cannot allow R1 while forbidding R2 | P1 |
| 4 | F source is `workflow_input(4)` but Component I has only port 1 | `canonicalize_information_flow_contract` validates positive syntax, not interface membership | Violation for missing I port | P2 |
| 5 | F sink is `workflow_output(3)` but Component O has no port 3 | F canonicalization does not resolve O | Violation for missing O port | P2 |
| 6 | Summary contains `unknown_external`, no taints, `influence_unknown=false`, and claims a clean flow | `check_flow` returns Unproven for unknown external provenance; the revision-two corpus pins this | Unproven; never repair to an empty/trusted source set | P0 invariant already enforced |
| 7 | Closed ε has atom A but K has no clause for A | `analyze_recovery_safety` returns `Unproven(MissingClause)` | Unproven; do not default to replayable or recoverable | P2 |
| 8 | K has a clause for atom B absent from closed ε | `analyze_recovery_safety` returns `RECOVERY_EFFECT_NOT_DECLARED` | Violation | P2 |
| 9 | K requires compensation operation `data.refund`, while ε declares `data.refund` only on another resource selector | The K compensation field stores only `EffectOperationRef`; validation checks operation name membership in the operation catalog | Unproven; operation-only identity cannot prove the declared selector is available | P1 |
| 10 | K clause uses `idempotency=natural`; ε has same operation/selector with `idempotency=none`; evidence says Applied | `AtomKey::of(natural) == AtomKey::of(none)`; the direct public-API probe made `analyze_recovery_safety` return `ProvenStaticConstraint` | Violation: the K clause's full EffectAtom is absent from ε | **P0** |
| 11 | K says reconciliation is required, but does not identify the reconciliation operation/effect | Wire field is only `ReconciliationRequirement::{None,Required}`; `RecoveryOperationUniverse` recognition is not connected to the clause | Unproven; no declared effect identity can be checked against ε or R | P1 |
| 12 | K requires reconciliation; U omits all recovery meters or contains only explicit `finite(0)` for a required recovery meter | `usage_not_increased` treats missing coordinates as Unproven; no K→usage derivation exists | Unproven unless an explicit recovery-usage relation proves the envelope; missing is never zero | P1 |
| 13 | Old/new projections are compared under different fixed `PriceTableSnapshot` values | `cost_not_increased` returns `Unproven(DifferentPriceSnapshot)` | Unproven; lower numeric projection under a new price table is not evidence of cheaper implementation | Correct existing behavior |
| 14 | Old X requires `sandbox >= strong`; new X requires `basic` | `execution_compatibility(new,old)` can report feasibility Proven and security Violation | Combined Violation; a larger eligible set does not preserve security | Correct existing behavior |
| 15 | F confidentiality policy allows only region A; X allows only region B | Both dimension-specific canonicalizers can succeed; no label↔region relation exists | Unproven until an explicit catalog rule says whether the combination is permitted | P1 |
| 16 | Old/new documents use `SemanticTypeId=1`, but one expands to `string` and the other to `int64` | `StructuralType` expansion yields unequal values; raw local ID equality is not a comparison | Unproven for substitution; a concrete exact call binding with incompatible structure is rejected | P2 |
| 17 | Old/new documents use different SemanticTypeIds that expand to the same `option<list<string>>` | `StructuralType` derives structural equality; `SubflowInterface` compares expanded structures | Proven for the exact structural relation; local IDs remain irrelevant | Current API proves subflow case |
| 18 | R old requires `{repo=alpha}`; R new requires `{repo=alpha, branch=main}`; ε selector adds the same `branch=main` scope binding | R non-strengthening treats the new R pair as strengthening; ε scope containment treats additional exact bindings as a narrower effect selector | R Violation even though ε scope is narrower. Never reuse the ε subset direction for R | Existing R/ε rules differ |
| 19 | ε is Open/Unknown and K has clauses for all visible known atoms only | `analyze_recovery_safety` returns `Unproven(UnknownEffectCoverage)` for non-closed rows | Unproven; known clauses do not prove residual/unknown coverage | Correct existing behavior |
| 20 | INVOKE or CALL_SUBFLOW has no available Φ declaration and is tentatively treated as an empty row | Failure analysis represents incomplete call summaries as `StaticFailureEnvelope::Unproven`; Phase 3D derives only I/O/R | Unproven; never substitute `FailureRow::empty()` | Correct existing boundary |
| 21 | U omits the model-token meter while a cost projection is requested | `project_cost` treats missing price/usage coverage as Unproven; `within_ceiling` does not treat missing as zero | Unproven, not free and not finite zero | Correct existing behavior |
| 22 | Component I port 1 and R port 1 both exist; F says `workflow_input(1)` and ε selector says requirement port 1 | Their numeric payloads are equal but the namespace owners differ | F reference resolves I; ε reference resolves R. Do not merge or cross-resolve them | P2 |
| 23 | `PARALLEL(A,B)` needs 2 CPU and 4 CPU, with different X regions/runtime profiles | X explicitly forbids blind control-tree aggregation; `StaticExecutorProfile` describes one candidate | No single-worker sum or group placement conclusion without a formal topology/affinity contract | Correct existing boundary |
| 24 | K declares KEYED but key evidence/provider honoring is absent | `analyze_recovery_safety` returns `RequiresEvidence` with key/idempotency obligations | RequiresEvidence; never a static retry permit | Correct existing behavior |
| 25 | A public port uses `UserId = string`; another contract uses primitive `string` | Phase 3A type expansion collapses the alias; Phase 3D Subflow projection retains the exact alias TypeSchemaRef and rejects alias-vs-target | Unproven until the ABI comparison policy is reconciled | P1 |

## Reproducing the P0 probe

The probe used only current public model/API shapes:

```rust
let natural = atom(EffectIdempotencyQualifier::Natural);
let none = atom(EffectIdempotencyQualifier::None);
assert_eq!(AtomKey::of(&natural), AtomKey::of(&none));

let contract = RecoveryContract {
    replay_semantics: ReplaySemantics::Recompute,
    effect_recovery_clauses: vec![clause_for(natural, IdempotencySemantics::Natural)],
};
let epsilon = EffectSignature { row: EffectRow::Closed(vec![entry_for(none)]) };
let evidence = [(AtomKey::of(&none), RecoveryEvidenceState::Applied)].into();
assert_eq!(
    analyze_recovery_safety(&contract, &epsilon, &universe, &evidence)?,
    RecoveryProof::ProvenStaticConstraint,
);
```

The source of the collision is `AtomKey::of`, which maps each qualifier to
`family_no()` and therefore discards the `EffectIdempotencyQualifier` member.
The same key is used by ε containment/algebra and K coverage. The test probe
was temporary and is not retained as a production test or wire artifact.

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Field matrix](phase-04b0-field-matrix.md) ·
[Port namespaces](phase-04b0-port-namespace-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[Decisions](phase-04b0-decisions.md)
