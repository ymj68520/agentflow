# Phase 4C.1B.4C.4R1 — standalone substitution amendment design

**PROPOSED — NOT REGISTERED. DESIGN ONLY.** Start HEAD `2497b302c9ff72d1275868b96b50f409499e79cd`, clean;1794 pre-existing nonignored files physically fingerprinted before authoring. No existing file may change. No production/Registry/AFIDL/ABI/HashV1/Expected/Capture/normalization/activation change is authorized.

Recommendation: **Option B**, a closed `standalone_substitution` PredicateFamily and corresponding standalone subject in a successor non-wire model. Option A was examined first: `contract_validation` exists as a Scope, **not** as a PredicateFamily. Creating that family merely to nest substitution is also an additive model amendment, not reuse of an existing binding. Option C is a separately authorized reduction, not selected.

Read [registered-binding audit](phase-04c1b4c4r1-registered-binding-audit.md), [actual semantics](phase-04c1b4c4r1-substitution-semantics.md), [twelve rows](phase-04c1b4c4r1-twelve-dimension-roster.md), [draft](phase-04c1b4c4r1-normative-amendment-draft.md), [migration](phase-04c1b4c4r1-edition-migration-impact.md), [48 design vectors](phase-04c1b4c4r1-decision-vectors.md), and [readiness](phase-04c1b4c4r1-readiness.md).

The draft separates **local relation facts** from **final dimension evidence judgments**. Same-lineage local equality is Proven, but the existing final identity row can remain Unproven when other contents differ. Context mismatch returns12 Unproven carriers without comparison execution; it is not an error-valued Rust rejection or an invented D10 report. R1 preserves both facts, rather than silently repairing either behavior.

T3-REG-01 remains a registered-production blocker until separately approved R2 registration/migration and R3 producer adaptation. R1 design acceptance, if its gates pass, is not formal registration or human authorization of R2. All new type/case/clause names in this document set are PROPOSED — NOT REGISTERED, including where factored by cross-reference.
