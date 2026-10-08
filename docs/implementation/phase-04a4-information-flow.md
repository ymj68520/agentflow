# Phase 4A.4 — Information Flow / Provenance Static Contract

Status: complete (closed 2026-10-08). The F dimension of
`Computation<I, O, Φ, ε, R>` is materialized as the static declaration
side only. Companion document: `phase-04a4-decisions.md` (D-1..D-18).

## 1. Deliverables

| Artifact | Path |
|---|---|
| AFIDL module (`agentflow.information.flow@1`) | `spec/idl/information-flow.afidl` |
| Rust crate | `crates/af-flow` |
| Normative semantics | `spec/information-flow/information-flow.md` |
| Conformance corpus (revision 1, 45 cases) | `spec/conformance/information-flow/` |
| Independent Python oracle | `tools/gen-information-flow-corpus.py` |
| Decision register | `docs/implementation/phase-04a4-decisions.md` |

## 2. Wire model

10 types / 27 ABI rows (`agentflow.information.flow@1`): aliases
`LabelCatalogId`, `LabelName`; records `ConfidentialityLabelRef`,
`TaintRef`, `FlowSourceRef`, `StaticInformationSummary`,
`FlowConstraint`, `InformationFlowContract`; enums
`InstructionAuthority` (6 values), `FlowKind` (2 values); variants
`SourceCategory` (3 cases), `FlowSink` (2 cases). The external-effect
sink reuses the effect signature's `EffectOperationRef` by exact
cross-module import (`af-flow` → `af-effect` one-directional). AFIDL
corpus revision 9, purely additive.

## 3. Static analysis surface (`crates/af-flow`)

Dependencies exactly `af-core`, `af-idl`, `af-semantic`, `af-effect`,
`serde`, `unicode-normalization`. No `af-ir` / `af-failure` /
`af-resource` / `af-spec`, no I/O, no clock, no network. All analysis
entry points are pure functions with explicit read-only catalog
parameters (`FlowCatalogs` = labels + effect sinks).

- `model.rs` — the wire types, the authority ladder with the
  one-directional `carries` comparison, serde snake_case spellings;
- `universe.rs` — `InformationLabelUniverse` (two closed vocabularies,
  exact NFC/case-sensitive membership), `EffectSinkUniverse`
  (resolvability + explicit decision-capability classification),
  `SinkClassification` (Unknown/Plain/DecisionCapable — never inferred
  from names);
- `canonical.rs` — `canonicalize_information_flow_contract` (resolve,
  NFC, duplicates rejected, deterministic orders) + canonical ACE1
  bytes + `check_flow` (the three-verdict judgment);
- `propagation.rs` — `union_taints`, `merge_provenance_summaries`,
  `propagate_pure_expression` (authority inherited, elevation
  rejected), `propagate_unknown_transformation` (sticky unknown),
  `mix_model_visible_inputs` (strong mixer: union + `model_generated`
  + unknown + max authority), `FlowProof` + `require_proven`;
- `compatibility.rs` — `flow_not_relaxed` (per-dimension proof:
  restrictions superset, per-kind ceiling subsets, no authority
  elevation, unknown not repaired, source capacity not narrowed);
- `error.rs` — 11 stable kinds.

## 4. Frozen security invariants

Unknown provenance ≠ trusted; unknown taint ≠ empty set; missing flow
proof ≠ allowed; authentic source ≠ trusted instruction; sanitized ≠
endorsed; persistence ≠ trust upgrade; declared contract ≠ runtime
authorization. All seven are enforced structurally and audited.

## 5. Conformance corpus

45 frozen cases, revision 1: 9 golden (empty contract, single/multiple
labels, source-taint summary, authority label, egress sink with frozen
declassification obligation, decision-influence sink with frozen
endorsement obligation, conservative sources, unknown influence with
frozen Unproven verdict), 10 negative (unknown label, duplicate label,
invalid source/sink port, unknown sink operation, label-silently-
removed-as-rejection, unknown-taint-treated-as-clean, untrusted
promoted to instruction, flow without proof, missing privileged
transformation), 12 propagation (union identity/commutative/
associative/idempotent, pure inherit, unknown remains unknown, model
inherits every input taint, model adds model_generated, memory
persistence retains, summarization/schema-validation never endorse,
authority never elevated), 11 compatibility (same contract, removes /
adds permitted sink, drops restriction, taint-narrowing Unproven,
cross-catalog labels fail closed, unknown relationship Unproven,
authority ok/elevated, kinds never compared across + new-kind-same-sink
relaxed), 3 equivalence groups (label, taint, constraint ordering).

## 6. Registry delta

- Rule area `INFOFLOW`; rules `AF-INFOFLOW-0001..0018` (18).
- Specifications +2 (`agentflow.information.flow@1` schema,
  `...semantics@1` schema_language) — 22 total.
- ABI +27 (300 active total); suites +9 (`SUITE-INFOFLOW-*`, 53 total).
- spec-check: 22 specifications / 239 rules / 238 covered + 1
  intentional / 0 missing / 300 ABI / 8 AFIDL modules / 0 cross-reference
  errors.

## 7. Zero drift

Old conformance corpora byte-identical (encoding, digest, semantic-core,
semantic-dataflow, semantic-ir, subflow-interface, spec-registry,
effect-signature, failure-row, resource-requirement); effect corpus
revision 2, failure corpus revision 1, resource corpus revision 1
unchanged; AFIDL corpus 8→9 purely additive (no old digest removed);
semantic-ir normative digest stays `9fb3292a…`; hash-domains.json
unchanged (14 domains); no flow digest of any kind exists.

## 8. Gates

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: 0 errors.
- `cargo test --all`: 244 passed / 0 failed (10 in `af-flow`: 5
  conformance + 5 static-surface audits).
- spec-check: exit 0 (numbers above).
- All 12 generators run twice consecutively: zero diff over the 457-file
  spec tree.

## 9. Boundary

Not implemented (structurally absent, static-audited): runtime
provenance DAG / store, ValueEnvelope runtime ABI, ValueSemanticDigest,
policy engine, information-flow PEP, authorization decisions,
EffectGrant, declassifier/endorser execution, endorsement evidence,
credentials, resource handles, runtime binding, ComponentContract,
executable plan, journal, reducer, runtime, worker, scheduler; the 13
kernel opcodes, SemanticIrDocument, semantic type DAG and the
Effect/Failure/Resource schemas are untouched. The reserved
`component-contract` HashV1 domain stays untouched.
