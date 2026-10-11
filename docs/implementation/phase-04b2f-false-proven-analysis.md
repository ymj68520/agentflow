# Phase 4B.2F — False-Proven Analysis

## Checked capability construction

`check_root_export_assertion` (`crates/af-computation/src/root_export.rs:884`)
calls `analyze_root_export_assertion` and returns
`CheckedRootExportCompatibility` whenever `report.static_compatibility` is
`Proven`. Analysis first requires every preflight proof to be Proven; otherwise
it creates nine Unproven dimensions. With a passing preflight, all nine
dimension functions run independently. A preflight/dimension Violation wins;
otherwise every dimension must be Proven for static compatibility to be
Proven. RequiresEvidence and Unproven are not converted to Proven. The
body/implementation/runtime/publisher statuses are separate and do not gate
static envelope compatibility by design.

That aggregate gate has no observed “eight Proven plus one Unproven” shortcut.
The two P0s below occur *inside inputs/coordinate provenance that the report
incorrectly treats as Proven*, so the aggregate check cannot catch them.

## AUDIT-F01 — bounds and guard are satisfied by different export rows

**Severity:** P0 — confirmed static soundness defect.<br>
**API:** `analyze_root_export_assertion` / `check_root_export_assertion`.<br>
**Dimension:** epsilon; the checked static capability is affected.

**Source:** `crates/af-computation/src/root_export.rs:2635-2692`, especially
the separate `bounds_covered` and `guard_covered` searches at 2670-2680. The
normative per-entry relation is implemented by
`af_effect::contains` (`crates/af-effect/src/algebra.rs:120-235`), which claims
one unused covering export slot only when operation, qualifier, selector
scope, bound, and guard all fit in that same slot.

**Minimal legal input.** Start from the checked X06
`spec/conformance/computation-root-export/positive/exact-input-output.json`
fixture. The source component has one Int64 RunParameter at port 1 and a
checked resource slot for port 1. Add a checked write Effect with natural
idempotency qualifier, selector scope `{a=1,b=1}`, unconditional guard, and
bounds `[0,10]`. The exact context recognizes resource attributes `a` and `b`
and its EffectResource relation maps those selector keys to those attributes.
Declare this exported closed epsilon row (same operation, qualifier, and
resource port):

```text
row E1: scope {a=1}, guard input_equals(port 1, Int64(0)), bounds [0,10]
row E2: scope {b=1}, guard unconditional,                  bounds [0,5]
```

The two rows have distinct full AtomKeys because their scope bindings differ;
the checked ComponentContract validator accepts the row and its complete
relation snapshot. The root scope `{a=1,b=1}` is narrower than both selectors.
At the valid root input valuation `port 1 = 1`, E1 is unavailable. E2 is the
only matching declared ceiling and permits at most 5, while the root may cause
10. No single export row covers the root entry.

**Observed production result:** the full checked-contract → checked-IR →
Phase 3C → resolver → root derivation → RootRecovery → RootExport pipeline
completed. I/O, O, Phi, epsilon, R, F, U, K, and X all reported Proven;
`check_root_export_assertion` returned `Ok(CheckedRootExportCompatibility)`.
The probe IR pin was
`bdb7c94eac9e076442208e1dc59c7e848d3c3f2897b2e2d1e3d0db2c52ccf32c` and the
top export pin was
`c6fa1f415e81e30a718e06503c337ec06cef1ccd15e6384087e6c9df50610823`.

**Contradiction:** `af_effect::contains` correctly fails to find one export
slot covering the root entry. The fallback then finds the bound in E1 and the
guard in E2 independently; because any export guard covers the root guard, it
continues as though the combined relation were Proven. `guard_counterexample`
is never reached. This contradicts `spec/effect/effect-signature.md#effect-containment`
and `AF-SEMANTIC-0166`: one declaration's guard and another declaration's
bound cannot be spliced into one ceiling.

**Expected:** Violation (the `port 1 = 1` valuation witnesses the undercovered
upper bound), or at minimum Unproven; never a checked capability.<br>
**Observed:** epsilon Proven; all dimensions Proven; checked capability issued.<br>
**Narrow fix proposal:** keep each candidate export row as one proof witness;
require a single row to cover both guard and bounds before continuing. A
countermodel must be evaluated against that same row and the exact reachable
valuation. Preserve this test as a production-entry regression.<br>
**Blocking stage:** RootExport static compatibility; not suitable as release
control.

## AUDIT-F02 — root derivation is not bound to the passed resolver snapshot

**Severity:** P0 — confirmed stale-provenance soundness defect.<br>
**API:** `derive_workflow_contract_summary` followed by
`check_root_export_assertion`.<br>
**Dimension:** epsilon and resolver/proof-context lifecycle.

**Source:** `resolve_subflow` stores the snapshot's exact exported contract pin
in `ResolvedTarget::Subflow` (`crates/af-computation/src/resolution.rs:585`).
The RootExport preflight at `crates/af-computation/src/root_export.rs:939-961`
checks that the root, checked export, passed snapshot, and caller use the same
`StaticContractProofContext` pointer, then verifies the assertion dependency
pins against the *passed* snapshot. It does not verify that the supplied root
summary's resolver-derived source pins match that snapshot or store/check an
identity for the resolver snapshot used during derivation.

**Reproduction:** use the formal
`computation-root-export/subflow/body-evidence-absent` fixture. Keep one
SemanticIrDigest and the same proof-context object. Derive the root against
snapshot A, whose exact target exported ComponentContract has write upper 1.
Create a second valid snapshot B for the same target SemanticIrDigest and same
context, with the target export write upper 10. Keep the root's public exported
ComponentContract at write upper 1. Build the assertion/manifest/dependency
pins from B, then call RootExport analysis/check with root(A), snapshot(B), the
same IR, top export, and context. Both snapshots contain valid checked
contracts; the only mutation is the resolver's subflow export association.

**Observed:** `check_root_export_assertion` returned a checked capability;
all nine dimensions were Proven and body conformance was
RequiresEvidence. The root effect source pin was
`2019eeb505c3d1e14fa086a89bd3868a6f1692224fbad44f3651fe493943afc5`, while the
assertion/current resolver target export pin was
`1851278a5faa9a8b239abe89d4191d3f3735b83004124894b791304ba07fc661`. The top
root-export contract pin equals the root-derived export-A pin. Under snapshot
B the CALL_SUBFLOW declaration admits upper 10, which exceeds the top export's
upper 1; a fresh derivation under B must not be Proven.

`validate_subflow_dependencies` (`root_export.rs:1120-1158`) verifies the
assertion's direct/transitive list against snapshot B, but it does not inspect
the subflow `ResolvedTarget` already embedded in the supplied root summary.
The exact same context pointer therefore does not establish exact snapshot
identity or dependency content.

**Expected:** reject the mismatched summary as stale/Unproven, or derive the
root again under snapshot B and return Violation for epsilon.<br>
**Observed:** all dimensions Proven; checked capability issued.<br>
**Narrow fix proposal:** bind a content-complete resolver dependency identity
to `DerivedWorkflowContractSummary` and compare it to the passed snapshot, or
make the checker derive from the supplied snapshot itself. Compare every
resolved target export pin, not just the assertion's copy of the pins.<br>
**Blocking stage:** proof-context invalidation; release control is not
accepted.

## Exhaustive gate review

The aggregate decision code at `root_export.rs:808-829` has the intended
fail-closed order: preflight Violation or any dimension Violation yields
Violation; otherwise all nine must be Proven; RequiresEvidence and Unproven
remain non-Proven. The found defects are inside the epsilon relation and root
snapshot provenance, before that final gate. The body, implementation,
publisher, authority, budget, and runtime fields remain separately typed and
were not promoted by either probe.
