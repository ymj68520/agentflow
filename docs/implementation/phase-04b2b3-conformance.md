# Phase 4B.2B.3 — Conformance

`tools/gen-computation-scope-closure-corpus.py` is an independent Python-only
oracle. Its 55 vectors are generated from frozen IR/K inputs and are executed
through checked Semantic IR, Phase 3C, production leaf resolution, the
production control analyzer, and `derive_workflow_contract_summary`.

The suite contains 18 direct Scope/resource-boundary cases, 18 exact
compensation/activation/attempt/Recovery-Usage cases, 18 nested cases, and one
negative vector whose K-derived action upper exceeds the declared action
Effect row. The oracle checks ordered tree and leaf closure, source accounting flags, root
export boundary, scope resource attenuation, compensation action identity and
upper bounds, receipt/order statuses, and forward-history retention. It does
not ask Rust to write its own Expected values.

Four isolated comparator mutations cover resource proof, action upper bound,
DFS order, and Recovery-source accounting. All four are rejected. The separate
advanced 72-case test retains its manifest and CaseIds; only the formerly
unsupported Scope expectation changes, as recorded in the decisions document.

**Static Proven Facts:** 55/55 new cases stage-executed and every declared
oracle field compared with zero mismatch.

**Conservative Upper Bounds:** tested over direct K sources and nested control
trees; action multiplication uses checked inputs.

**RequiresEvidence:** receipt-dependent eligibility, multi-event order, and
Scope settlement are asserted as statuses, not simulated.

**Unproven:** authority/budget attenuation and root-export compatibility are
explicitly expected rather than omitted.

**Runtime-Only Obligations:** no runtime effects, grants, receipts, or recovery
actions are executed by these vectors.
