# Prerequisites, references and diagnostic classification

Design disposition: **both remain RequiredSemantic**, with explicit typed identities and canonical string projections in a future version. The current executable contract is unchanged. See [draft D08–D09](phase-04c1b4c1-g01-normative-contract-draft.md), [inventory P/R](phase-04c1b4c1-producer-inventory.md).

## Prerequisites

`PrerequisiteKey=(scope,stage,predicate,source,target,coordinate,rule,requirement,origin)`, distinguished by its prerequisite tag/role from an obligation. Stage is the originating premise check; consumer dimensions/stages remain explicit edges. Its status is established/missing/requires_evidence/contradicted, linked to the underlying typed predicate fact. A prerequisite is not a synonym for missing evidence: a Proven proof may and normally does have established prerequisites. Unknown applicability/status cannot become established or empty.

P1–P10's nineteen literal occurrences translate into these named formal predicates, with exact source/rule/context: closed contract schema; imported module closure; explicit schema universe; positive unique input/output namespaces; exact failure pins; effect operation membership; explicit resource universe; conditional effect/resource relation coverage; flow label/sink universe; I/O/R namespace validity; additive meter/unit descriptors; exhaustive per-action recovery usage coverage; complete effect-row action coverage; exact K clause/action identities; execution universe validity; conditional F/X relation coverage; all required contract dimensions Proven; exact relation pins. Repeated schema-universe references share the exact same prerequisite key only in the same scope/content; both consumer edges remain. The fallback twentieth description is not a new positive proof rule.

Conditional relation prerequisites have explicit applicability predicates (`epsilon_nonempty`, `flow_constraints_require_relation`) in the fact roster, not English "when" parsing. If false, no such prerequisite instance; if unknown, an unresolved applicability fact. For missing schema members, operation coverage or pins, the typed catalog/member/contract port locates the source. Proven with missing/contradicted prerequisite is InvalidFacts. Lists have Set semantics/canonical typed order, not evaluation order; SEQUENCE control dependencies stay separately ordered.

Subfinding **G01-P / P1**: current prerequisites have no per-token formal identity, applicability or satisfaction status; ComponentContract constructors attach them to all verdict classes, while Control/Leaf/RootExport constructors leave their private lists empty. Design accepted; registration/production adaptation pending. No old empty list is retroactively filled, and no current token is declared a normative fact merely because Rust emitted it.

## References

Disposition = **typed Evidence Coordinate**, not a free-form proof ID and not RuleId. The full closed union and None semantics are in D09. `computation:2` requires caller IR and nominal ComputationId; raw hex requires full DigestRef/domain/target role; port2 requires direction/contract; SchemaRef requires all nominal fields; debug AtomKey requires the complete typed atom; resolver-difference text requires exact old/current contents and tagged difference.

`rule_id` answers which normative rule governs the fact. `reference` answers which evidence object/coordinate is relevant. Neither text nor RuleId alone establishes a complete dependency closure; F03 remains OPEN/P2. A primary None is valid only if no singled-out evidence reference is required; it cannot erase source identity, missing prerequisite or unknown coordinate. Every Some reference must resolve in the exact scope.

Subfinding **G01-R / P1**: current field mixes semantic coordinates, human resolver-difference messages and Debug/Display strings. The future model keeps required evidence facts while moving only derived English presentation into reason; this needs explicit versioned adaptation and cannot be done as a silent DiagnosticOnly demotion. The v1 field remains required in current tests.

Future primary-reference selection is not left open: derive the full typed subject reference set from source/target/coordinate, deduplicate full equal references, and select its typed minimum, except the six root independent-boundary predicates have no singled-out primary. Their source/identity links remain complete. Missing required references fail; arbitrary producer first-difference order is not normative. Some reference strings include edition and complete scope; None follows that fixed predicate policy. See draft D09 for all cases/variant order.

## Uniform classification principle

| Field | Previous | Proposed / this phase | Basis and preservation requirement |
| --- | --- | --- | --- |
| unresolved_obligations | RequiredSemantic | RequiredSemantic; unchanged now | All undischarged predicates/source occurrences preserved; future canonical projection replaces lossy prose |
| prerequisites | RequiredSemantic | RequiredSemantic; unchanged now | Formal predicate identity, applicability, satisfaction and consumer links |
| reference | RequiredSemantic | RequiredSemantic evidence coordinate; unchanged now | Exact typed subject and association retained, not a sentence token |
| rule_id | RequiredSemantic | Unchanged | Exact governing rule/content association; not a closure certificate |
| ActionGuardCompatibility.reason / proof reason | DiagnosticOnly | Unchanged | Verdict/basis/full source/action/guards/bounds/typed countermodel carry decisions; reason only explains them |

The classification test is semantic substitutability, not Rust String versus enum and not Python convenience: if replacing the final displayed prose while preserving every typed fact/link cannot change identity/status/eligibility, it is diagnostic. If any required datum is only in prose, obtain a lossless typed fact; do not ignore that datum. Presence/string type still validate for diagnostics. `validate.rs::set_dimension_at` internally compares reason text to recognize its retained update; this is an implementation selection mechanism, not an approved public identity grammar. Future cause IDs remove that dependence without silently altering retained reference facts.

There is **no classification migration in this phase**. Any future choice of Option C would require a separate Previous/New/Basis/Preservation/Comparator/Migration approval, not this recommendation. Option A requires future closed typed prerequisite/reference schemas, independent derivation/encoding tests, producer adapters, dual old/new version expectations and adversarial checks. G01-P/R are not production-closed by the design.
