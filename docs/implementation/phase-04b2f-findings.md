# Phase 4B.2F — Findings Register

## AUDIT-F01 — P0: Effect containment splices bounds and guard across rows

- **Dimension/API:** epsilon; `analyze_root_export_assertion`,
  `check_root_export_assertion`.
- **Location:** `crates/af-computation/src/root_export.rs:2670-2680`.
- **Normative basis:** `AF-SEMANTIC-0166`,
  `spec/computation/root-export-contract-proof.md#effect-containment`, and
  `spec/effect/effect-signature.md#effect-containment`.
- **Minimal reproduction:** P30 in the probe register: root write scope
  `{a=1,b=1}`, unconditional `[0,10]`; export E1 scope `{a=1}`, p1=0 `[0,10]`;
  export E2 scope `{b=1}`, unconditional `[0,5]`; valid Int64 input p1=1.
- **Expected:** Violation (typed input witness) or, if reachability is
  insufficient, Unproven. Never Checked.
- **Observed:** all nine dimensions Proven; `check_root_export_assertion`
  returned Ok. `af_effect::contains` fails, then separate existential bound and
  guard checks continue as Proven.
- **Recommended narrow fix:** require one same export entry to cover scope,
  full atom, bounds and guard. Only emit Violation from a countermodel against
  that same entry. Add P30 as a permanent independent production regression
  in a separately authorized fix phase.
- **Blocking stage:** static RootExport compatibility; current capability is
  not release-control safe.

## AUDIT-F02 — P0: root summary is not tied to the supplied resolver snapshot

- **Dimension/API:** resolver/proof-context provenance affecting I/O/R/E/F/U/K/X;
  `derive_workflow_contract_summary` → `check_root_export_assertion`.
- **Location:** preflight `root_export.rs:939-961`; direct snapshot pin
  validation `root_export.rs:1120-1158`; subflow export capture
  `resolution.rs:585`.
- **Normative basis:** `AF-SEMANTIC-0118..0121`, `AF-SEMANTIC-0159..0161`,
  `AF-SEMANTIC-0171`, and `phase-04b2d-proof-context-lifecycle.md`.
- **Minimal reproduction:** P31: same checked IR and context, derive root
  against snapshot A with subflow export E=1; pass that root to the checker
  with snapshot B attaching the same target IR digest to export E=10; top
  exported contract remains E=1. Build assertion/manifest pins from B.
- **Expected:** stale root rejected/Unproven or a fresh derivation under B
  yields epsilon Violation. A different resolver association must invalidate
  all source summaries derived through it.
- **Observed:** assertion target pin B differs from root source/export pin A;
  all nine dimensions Proven and checked capability issued. Body remains
  RequiresEvidence, which does not repair the static declaration mismatch.
- **Recommended narrow fix:** bind resolver dependency identity to the root
  summary and compare it with the passed snapshot, including every direct
  target export pin; alternatively derive the root inside the checker from
  the exact supplied snapshot.
- **Blocking stage:** proof-context invalidation; current capability is not
  release-control safe.

## AUDIT-F03 — P2: applied RuleId closure is not complete

- **Dimension/API:** CompilerSemanticsIdentity.
- **Location:** `root_export.rs:639-676`.
- **Observation:** the enumerated set contains 22 AF-CONTRACT and 62
  AF-SEMANTIC rules. It omits AF-SEMANTIC-0001..0110, including directly used
  TypeSchemaRef, SSA producer, WorkflowOutput, ownership, Phase 3C, Scope and
  subflow-interface requirements.
- **Expected:** a justified exact applicability closure, or an explicit
  declaration that RuleId provenance is incomplete.
- **Observed:** whole source/specification/rule registry bytes are also
  equality-bound, and all 16 normative spec sources are embedded; no stale
  content bypass was reproduced. The gap is provenance/completeness, not a
  confirmed false-Proven by itself.
- **Recommended narrow fix:** enumerate and justify the actual dependency
  RuleIds; retain the full content-byte binding. Do not claim the present 84
  IDs are a complete applied-rule closure.
- **Blocking stage:** audit evidence completeness (P2).

## AUDIT-F04 — P2: guard countermodel suppression is root-global

- **Dimension/API:** epsilon guard diagnostic precision.
- **Location:** `root_export.rs:2459, 2700-2707`.
- **Observation:** a SELECT/RACE/HANDLE anywhere in the root disables typed
  guard countermodels for every effect comparison, including unrelated paths.
- **Expected:** path-local reachability gating, or conservatively Unproven.
- **Observed:** conservative Unproven; no false-Proven or false bound was
  observed. The f011eef change is safe but may lose a Violation witness.
- **Recommended narrow fix:** carry the current occurrence's exact path and
  gate countermodels per source, preserving all unknown path obligations.
- **Blocking stage:** proof precision/coverage (P2), not a release acceptance
  basis while F01/F02 remain open.

## Severity inventory

- **P0:** AUDIT-F01 and AUDIT-F02 confirmed.
- **P1:** None separately identified; the current static release-control
  conclusion is nevertheless rejected because of the P0s.
- **P2:** AUDIT-F03 and AUDIT-F04.
- **P3:** Trusted body/implementation evidence, publisher authentication,
  scope authority/budget, Worker admission, runtime receipts, dispatch and
  actual recovery remain deferred by the model.
