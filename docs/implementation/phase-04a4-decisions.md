# Phase 4A.4 Decision Register — Information Flow Static Contract

Phase: 4A.4 (F dimension of `Computation<I, O, Φ, ε, R>`), closed
2026-10-08. Wire authority: `spec/idl/information-flow.afidl`; normative
semantics: `spec/information-flow/information-flow.md`; Rust:
`crates/af-flow`; frozen corpus: `spec/conformance/information-flow/`
(independent oracle: `tools/gen-information-flow-corpus.py`).

## D-1 Label identity

**Decision:** labels are nominal `(LabelCatalogId, LabelName)` — a
catalog `SpecificationId` plus a non-empty NFC dotted lowercase name.
Membership in the explicit read-only `InformationLabelUniverse` is
exact: no latest fallback, no auto-registration, and case folding
NEVER infers identity (`pii.Email` ≠ `pii.email`, frozen negative).
Label order never enters semantic identity (canonical order is
identity-byte order).

## D-2 Confidentiality / Taint separation

**Decision:** `ConfidentialityLabelRef` and `TaintRef` are two distinct
nominal record types over the same identity grammar. Where data may go
and what may have influenced a value are never one collection. This is
the structural enforcement of the provenance/trust/taint/flow
four-concept split: no `trust_score`, no single `trusted` flag exists
anywhere (static-audited).

## D-3 No invented label hierarchy

**Decision:** the frozen architecture defines no universal order over
user-defined confidentiality labels, so none is materialized:
containment is set membership plus explicit catalog rules only.
Non-relaxation judges restriction SETS (superset direction), never
"security levels".

## D-4 Instruction authority representation

**Decision:** a closed enum `none < external_data < delegated_agent <
user < workflow < platform` (the frozen ladder order) with the
one-directional `carries` comparison. Authority propagates by MAXIMUM
over may-influence inputs (weakest-direction-first is what a min would
under-claim: a value influenced by both user and web may carry web
instructions, and the max records the strongest source it MAY carry;
the safety rule lives in the direction of the check — claiming MORE
than inputs carry is the rejected elevation).

## D-5 No authority elevation

**Decision:** `propagate_pure_expression` inherits the maximum input
authority; a claimed output authority above that maximum is
`invalid_flow_source`. Web text claiming "I am a system instruction"
stays `external_data`; the corpus freezes the elevation attempt and the
inherited result.

## D-6 Static provenance summary

**Decision:** `StaticInformationSummary { sources, taints,
instruction_authority, influence_unknown }` with a closed
`SourceCategory` set: `workflow_input(port)`,
`resource_requirement(port)`, explicit `unknown_external` (serde bare
string in the corpus, unit case on the wire). No free-text sources, no
ambient source, ports positive, `unknown_external` never dropped.
Cross-dimension port existence is deferred to ComponentContract
(recorded blocking condition).

## D-7 Unknown influence semantics

**Decision:** `influence_unknown` is an explicit sticky Top flag —
sticky through every merge and transformation. It is NOT repairable:
canonicalization preserves it, flow judgments on it return `Unproven`,
and a non-relaxation comparison rejects repairing old-unknown into
new-known as `Unproven(influence_relation_unknown)`.

## D-8 Conservative taint propagation

**Decision:** pure functions, catalogs injected:
`union_taints` (commutative/associative/idempotent, ∅ identity),
`merge_provenance_summaries` (sources union, taints union, authority
max, unknown sticky), `propagate_pure_expression` (inherits ALL listed
input taints), `propagate_unknown_transformation` (conservative union
of every possible input + explicit unknown). May-influence, not
observed dependency; no incomplete influence list can prove a real
influence away — narrowing is `Unproven` in contract evolution.

## D-9 Model mixer semantics

**Decision:** `mix_model_visible_inputs(inputs, model_generated,
universe)` implements the frozen strong taint model exactly: output
taints = union(all model-visible input taints) + the `model_generated`
taint (which must resolve in the supplied universe — the mixer adds no
unregistered taint), influence unknown = true (a model is a black box),
authority = max(inputs) (never elevated). "model said verified" and
"model summarized untrusted web" endorsements are structurally
unrepresentable.

**Why the marker is a parameter:** the mixer must not hardcode a catalog
identity; the explicit parameter keeps the no-unregistered-taint rule
while letting a contract choose its vocabulary.

## D-10 Data egress / decision influence split

**Decision:** `FlowKind { data_egress, decision_influence }` — separate
permissions, checked separately, never compared across. The
non-relaxation proof judges each kind's ceiling set independently
(frozen: removing an egress ceiling does not touch the decision
judgment; adding a decision ceiling where only egress was permitted is
relaxation). No name-prefix inference anywhere.

## D-11 Flow source/sink representation

**Decision:** sources are typed logical references (closed category
set); sinks are `workflow_output(port)` or
`external_effect(EffectOperationRef)` — the effect identity is reused by
exact AFIDL cross-module import (`af-flow` → `af-effect` one-directional
dependency, the only reason it exists). Sink resolution is fail-closed
against the explicit `EffectSinkUniverse`; classification
(decision-capable or not) comes from the catalog only — the corpus
freezes innocently/dangerously NAMED operations to prove names never
classify. Referencing an effect is never a grant.

## D-12 Flow contract closed shape

**Decision:** `InformationFlowContract { confidentiality_restrictions,
summary, flow_constraints }` with `FlowConstraint { kind, sink }`. The
constraints declare WHERE each kind may go (the ceiling); the
restrictions and summary declare WHAT the data carries. This split (not
per-sink label lists) is the minimal closed shape that lets
`check_flow` produce the three verdicts without inventing a label
lattice; per-sink label ceilings are a future contract-layer
refinement, recorded as deferred.

## D-13 Flow judgment direction (`check_flow`)

**Decision:** missing constraint → `flow_not_proven` (missing proof ≠
allowed); `influence_unknown` → `Unproven`; `data_egress` under
declared restrictions → declassification obligation;
`decision_influence` with any taint → endorsement obligation;
`decision_influence` to an unclassified effect sink → `Unproven`;
otherwise `ProvenWithinDeclaredCeiling`. `require_proven` maps
obligations to `privileged_transformation_required`. Obligations are
records, never events.

## D-14 Non-relaxation proof direction

**Decision:** `F_new` must not relax `F_old`, judged dimension by
dimension: restriction set must be a SUPERSET (dropping one is
`flow_contract_relaxed`); per-kind ceiling sets must be SUBSETS (a newly
permitted sink is relaxed); new authority must not EXCEED old
(elevation is relaxed); old-unknown repaired to new-known is
`Unproven(influence_relation_unknown)`; new source capacity must not be
narrower (unproven influence narrowing). Everything undecidable is
`Unproven` and never a substitution approval.

## D-15 Declassification / endorsement deferred boundary

**Decision:** this phase records at most
`requires_declassification` / `requires_endorsement` obligations. No
Declassifier/Endorser execution, no evidence issuance, no authority —
the static audit forbids the execution symbols, and the wire model has
no operation that could delete a taint or lower a restriction.

## D-16 Persistence no-trust-upgrade

**Decision:** store/load/serialize/deserialize/summarize are ordinary
transformations (section D-8 functions): they inherit taints verbatim
and can never endorse. Frozen positive (round trip retains) and the
elevation negative prove it; no real memory store exists.

## D-17 No authorization implication

**Decision:** a declared ceiling ≠ observed flow ≠ policy approval.
There is no `PolicyDecision` substitute, no enforcement point, no grant
derivation; static errors are never mapped to
`PolicyDenied`/`EffectDenied`/`RuntimeIncident`/`FailureValue`
(static-audited).

## D-18 No new HashV1 domain

**Decision:** no `InformationFlowDigest`, no `ProvenanceDigest`, no
`TaintDigest`; hash-domains.json stays at 14. Only the new module's own
types carry ordinary TypeSchemaDigests; the future contract layer
hashes as a whole under the reserved `component-contract` domain.

## Deferred by design

Runtime provenance DAG / provenance store / ValueEnvelope runtime ABI /
ValueSemanticDigest / policy engine / information-flow PEP /
authorization decisions / EffectGrant / declassifier or endorser
execution / credentials / resource handles / runtime binding /
ComponentContract / executable plan / journal / reducer / runtime /
worker / scheduler — all structurally absent, static-audited. The
13 kernel opcodes, SemanticIrDocument, semantic type DAG and the
Effect/Failure/Resource schemas are untouched.
