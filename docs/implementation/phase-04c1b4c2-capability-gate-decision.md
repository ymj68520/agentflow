# REG-05 — normalization is an additional successor capability gate

**PROPOSED — NOT REGISTERED; NOT IMPLEMENTED.**

## Chosen contract

Choose the **formal prerequisite** model, not independent non-minting postprojection. This resolves accepted D14's ambiguity openly: five-plus-nine continues to characterize the static proof, but is no longer alone sufficient to construct an owner@2 checked capability. Allocation0212/0213 and the D14 amendment are new normative basis requiring separate approval. It is not retroactive v1 behavior.

```text
ProofEligible = root_export
             && all five mandatory core predicates Proven
             && exactly nine unique evaluated dimensions Proven
ViewReady = CompleteAndValid(captured facts,scope,roster,links)
         && bounded_normalize_and_encode(captured facts) succeeds
CanConstructCheckedV2 = ProofEligible && ViewReady
```

The private production checker is the sole constructor. The pure normalizer receives validator-origin facts, cannot set verdicts or construct capabilities. It is called **before** every successor private checked construction path, including the Portable D10 path. The old constructor cannot remain as an unguarded alternative in the current-only build. No public API accepts a caller-supplied CompleteView as proof.

Five mandatory preflights: assertion list/order/target validity; exact pins; context/resolver association; current compiler identity; subflow dependency closure. Current `analyze_root_export_assertion` computes `preflight_ok` at799 and static result at813, appends body830–837 later. Preserve that role distinction. The core context predicate may link detailed resolver facts but does not add an arbitrary sixth core condition.

If core fails: capture failed keys and all six boundaries; nine static facts are non-evaluated blocked records, not nine separate evaluated unknown-evidence facts. If core passes: evaluate exactly nine and retain all required cause facts. Every known X child has separate feasibility/security, plus any distinct source-coverage predicate. Missing mapping emits source-coverage Unproven; it is not evidence to invent evaluated aspects for unknown children.

Six root boundaries: body/implementation RequiresEvidence; Scope authority/budget Unproven; runtime/publisher NotEstablished. Runtime evidence RequiresEvidence is a companion field on the same runtime predicate. They remain in a complete static-positive view and do not become mandatory static source dependencies. Per-call body requirements add separately scoped records.

## Failure and availability contract

Proposed internal interfaces (NOT IMPLEMENTED):
- `capture_analysis_facts_v2` obtains complete facts at producing branches.
- `normalize_complete_g01_v1` returns Result<CompleteView,NormalizationFailure>.
- `check_root_export_v2` privately creates Checked only after ProofEligible and ViewReady.
- Portable `revalidate_root_export_v2` maps stage-tagged failures to analysis failure, never to a raw report that looks checked.

NormalizationFailure closed classes: IncompleteFacts,InvalidFacts,UnsupportedModel,InvalidGraph,ResourceLimit,InvalidCanonicalEncoding. Each carries stage,typed source/path and diagnostic reason; not a fifth proof verdict. Admission errors are separately InvalidIdentity,IncompleteContent,UnsupportedSemantics at D0–D4. Stable external error mapping must be explicitly materialized in the implementation transaction.

For a proof-eligible input whose report exceeds byte/work budget, result is failure(ResourceLimit) with **no capability** and no valid CompleteView. A diagnostic may say the static proof phase completed; it is tagged partial and never has successful projected eligibility/capability fields. IncompleteFacts also returns no Checked. For a complete non-positive proof, a complete view can still be returned but cannot construct Checked. A normalization failure must not be repackaged as nine-dimensional Violation or silently fall back to old strings.

Availability changes: some formerly legal positive proofs may now fail on report limits/unsupported capture; the [size review](phase-04c1b4c2-ace1-registration-review.md) estimates why. T7 must materialize small-positive new-edition fixtures, resource-edge positives and old-positive analogues; measure rejection rates/limits and obtain explicit activation approval. Never weaken Complete Scope or raise limits silently just to keep them green.

## Alternatives and security argument

Separate postprojection would preserve old capability availability, but would require a second incomplete-report API and strict consumers; rejected here because the accepted CompleteAndValid obligation contract is intended to accompany every successor checked report. It remains a technically possible future separately approved edition, not this plan's open implementation choice.

Proof computation -> typed facts -> typed obligations -> canonical view is one-way. Validation/encoding success can **withhold** a capability but cannot turn a non-Proven predicate Proven. O empty, short report, sorted text or reported "complete" never authorizes anything. Serializer output bytes are not fed back as proof evidence. Scope contains only input/declaration/context provenance, not report or capability. Finite DAG ranking has no recursive digest equation.

Required new tests: private constructor bypass fails; each core failure blocks nine; body RequiresEvidence with static-positive still constructs only the static compatibility capability; missing fact/contradiction conflict fails; valid proof +128MiB+1/work+1 yields ResourceLimit/no Checked; fragments/unknown shape/old tuple cannot construct; G02 missing content cannot return ViewReady; no runtime authority or body discharge. D24,D28,M08,M09,M11 are design vectors, all Not Executed.
