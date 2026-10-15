# Registration decision register

All decisions are **planning decisions**. Proposed rules: **PROPOSED — NOT REGISTERED**. Required code entries: **NOT IMPLEMENTED**. 'Decided' is not implementation or production closure.

## REG-01 — Placement

- Decision: Three separate OS semantics/RM closed-table/PP2 successor sources.
- Alternatives: One giant source or fake AFIDL module rejected.
- Normative basis: Registry mechanics§3;acceptedD01–D16;0214/0223 proposed.
- Security argument: Exact independent source ownership avoids hidden semantics in prose.
- Compatibility impact: +3 sources;old43 remain.
- Required code delta (NOT IMPLEMENTED): Author approved normative files,descriptors,registry generator.
- Required test evidence: collision/anchor/spec-check and exact old-source hash gates.
- Unresolved implementation dependencies: Separate approval;no actual new source bytes yet.

## REG-02 — Rule allocation

- Decision: 32 proposed SEMANTIC0197–0228;one independently testable invariant per ID.
- Alternatives: D01 IDs as rules or one umbrella invariant rejected.
- Normative basis: Registry§4 permanence/granularity;allocation table.
- Security argument: No reused ID/anchor can conceal changed meaning.
- Compatibility impact: Conditional404->436,not current registration.
- Required code delta (NOT IMPLEMENTED): Add real descriptors+case refs+coverage only in T1 branch.
- Required test evidence: 0 current conflicts;positive/rejecting conformance per row.
- Unresolved implementation dependencies: Recheck allocation at authorization time.

## REG-03 — Schema

- Decision: Register non-wire semantic relation plus closed RM descriptors.
- Alternatives: Unregistered Rust reflection and unnecessary public wire schema rejected.
- Normative basis: ACE1 schema-directed;registry closed_table kind;0214.
- Security argument: Every field/variant/width/option closed;unknown rejects.
- Compatibility impact: AFIDL17/ABI775 unchanged;report semantics explicitly new.
- Required code delta (NOT IMPLEMENTED): Materialize exact Shape/descriptors/type closure and typed getter values.
- Required test evidence: Nominal-width/options/unknown fields/both-side omission tests.
- Unresolved implementation dependencies: Full RM bytes and G02 getters NOT IMPLEMENTED.

## REG-04 — Canonicalization

- Decision: Existing ACE1 plus complete typed minimal DAG and typed set order.
- Alternatives: JSON key sorting/Debug/hash/count equality rejected.
- Normative basis: D10–D13;0215–0219.
- Security argument: Injective complete identity;origin not erased by interning.
- Compatibility impact: No HashV1/semantic digest or Bundle grammar change.
- Required code delta (NOT IMPLEMENTED): Bounded typed codec/decoder and exact canonical string view.
- Required test evidence: D01–16,D30,D38–45,M14;resource boundary tests.
- Unresolved implementation dependencies: Independent encoder/decoder actual evidence required.

## REG-05 — Capability relation

- Decision: Normalization is additional formal capability prerequisite before private creation.
- Alternatives: Separate postprojection rejected for this edition.
- Normative basis: Explicit D14 amendment;0212/0213.
- Security argument: Cannot mint from report;text cannot change verdict;all bypasses prohibited.
- Compatibility impact: Availability restriction:proof-positive may ResourceLimit/no Checked.
- Required code delta (NOT IMPLEMENTED): Typed result/error and all private path guards.
- Required test evidence: D24,D28,M08,M09,M11;constructor bypass and no partial output.
- Unresolved implementation dependencies: Positive size/availability characterization and human approval.

## REG-06 — Edition

- Decision: Owner@2 with exact46-source tuple,20 selected specs,138 applied,shape1,obligation1.
- Alternatives: Relabel@1 or version-only identity rejected.
- Normative basis: D15;0221/0223/0228;full edition table.
- Security argument: All source/registry/model/role content bound;138 not closure.
- Compatibility impact: Old tuple necessarily unequal;ABI/hash unchanged.
- Required code delta (NOT IMPLEMENTED): Update owner/support/build constants/complete support getters atomically.
- Required test evidence: Exact tuple mutations,known mixed content,role changes.
- Unresolved implementation dependencies: Future bytes/real case refs not yet materialized.

## REG-07 — Current-only

- Decision: Only exact new current tuple;no historical dispatcher.
- Alternatives: Implicit upgrade and incidental old positive support rejected.
- Normative basis: Accepted migration;PP2 supersedes only0188/0190 owner@2 policy.
- Security argument: Old/mixed/unsupported models fail before fresh proof.
- Compatibility impact: Old valid bundle UnsupportedSemantics;known mixed tuple InvalidIdentity.
- Required code delta (NOT IMPLEMENTED): Stage-tagged taxonomy and singleton support table.
- Required test evidence: Full admission matrix M01–04,M10,M15–16.
- Unresolved implementation dependencies: Separate policy implementation/approval.

## REG-08 — Legacy isolation

- Decision: Freeze old source/environment/512 inputs+Expected;separate new namespaces/jobs.
- Alternatives: Overwrite goldens or call intended Unsupported a regression rejected.
- Normative basis: 0226;accepted512 preservation.
- Security argument: Old soundness evidence stays auditable while new semantics are tested.
- Compatibility impact: 229 neutral unchanged;82 report-sensitive+201 Portable old-domain.
- Required code delta (NOT IMPLEMENTED): Archive recipe,case routing,read-only corpus/write guard,CI split.
- Required test evidence: Legacy512,45 early codec negatives,new old-tuple admission/new positives.
- Unresolved implementation dependencies: Future reproducible runner/toolchain capture atT0.

## REG-09 — Producers

- Decision: Capture complete original facts BEFORE reduction;copy origins,not text.
- Alternatives: U1/U2/U5 text parsing or source fabrication rejected.
- Normative basis: 0197/0200/0210;inventory and frozen predicate rules.
- Security argument: No many-to-one inverse;all source/ordinal/rule data retained.
- Compatibility impact: Versioned report change;old values untouched.
- Required code delta (NOT IMPLEMENTED): All6+3+11/20+65+27 sites and references/ROLE mapping.
- Required test evidence: Per-site declared branch/status/origin/copy/transform vectors.
- Unresolved implementation dependencies: Typed producer implementation and all condition-specific closure evidence.

## REG-10 — G02/G03

- Decision: G02 lossless contents before G01 fullfield;G03 independently before full nonempty subflow closure.
- Alternatives: Counts/checksums or empty-only interface oracle rejected.
- Normative basis: Complete Scope D02;D16;G02/G03 findings.
- Security argument: Missing source evidence cannot become Proof Proven or complete identity.
- Compatibility impact: No present findings closure/field demotion.
- Required code delta (NOT IMPLEMENTED): Read-only full-content views+inventory;independent Phase3D oracle.
- Required test evidence: Same-count member mutation,missing bytes,nonempty interfaces;full78 applicable fields.
- Unresolved implementation dependencies: G02 OPEN/P1;G03 OPEN/P2.

## REG-11 — Conformance

- Decision: Independent new input/Expected namespace and complete field comparator;64 laws remain Not Executed.
- Alternatives: Actual-to-Expected/string-only comparators and pass inflation rejected.
- Normative basis: 0220/0226;D16.
- Security argument: Independent normative derivation and mutation tests catch correlated omissions.
- Compatibility impact: No change to original48/512/oldExpected.
- Required code delta (NOT IMPLEMENTED): New Python oracle/byte fixtures,typed actual getters,crossprocess tests.
- Required test evidence: Every D/M row plus source-wide producer/model/status negatives and78 closure.
- Unresolved implementation dependencies: G01/G02/G03 and new exact normative bytes.

## REG-12 — Activation

- Decision: One reviewed T0–T8 transaction;T1–T7 staging never current;T8 final human-approved activation.
- Alternatives: Incremental released registry/new label+old checker rejected.
- Normative basis: 0227;accepted freeze and current-only policy.
- Security argument: No mismatched semantics can issue checked results during rollout.
- Compatibility impact: Rollback whole immutable release,not selective registry edits.
- Required code delta (NOT IMPLEMENTED): Atomic implementation branch/artifact,CI gates/readiness and no fallback.
- Required test evidence: Both domains,generator idempotence,constructor gates,exact tuple/freeze.
- Unresolved implementation dependencies: All T gates plus separate explicit implementation/activation authority.

All twelve decisions are explicit; none authorizes immediate registry changes. Pending concrete bytes/code/test execution are implementation prerequisites, not a deferred choice of identity model,capability relationship,current-only policy or rollback boundary.
