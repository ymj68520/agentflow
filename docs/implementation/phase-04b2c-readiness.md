# Phase 4B.2C — Readiness and next-stage decision

## Audit scope and baseline

Audit baseline HEAD: `d96dac047dfca8ae8cc129e0861ff6b66e284219` (Phase
4B.2B.3 final). Audit scope is read-only analysis of current production
semantics plus these reports; the temporary arithmetic probe was removed.
No Rust production code, AFIDL, ABI, digest, frozen Expected, rule registry,
runtime object, or capability was changed.

Repository baseline verified in this checkout:

```text
40 Specifications
360 Normative Rules
477 Active ABI Assignments
16 AFIDL Modules
14 HashV1 Domains

Computation Control      58 / 58
Advanced Control         72 / 72
Scope / Closure          55 / 55
```

The 72-case Scope transition remains revision 2 with the preserved CaseId;
only its one formerly unsupported Expected differs from the previous phase.
The other 71 Expected objects and the 58-case revision-5 corpus are unchanged.

## Readiness by level

| Level | Result | Release statement allowed |
| --- | --- | --- |
| 1. Structural Closure | **Accepted** | Checked IR rejects cycles/multiple parents/detached nodes; workflow walk accounts every declaration and every leaf/source or fails explicitly. |
| 2. Conservative Envelope Soundness | **Accepted with explicit separation** | Root E/U envelopes are conservative; K-derived potential actions are separate conditional facts. A narrower declared action-E guard is not treated as proof that the K action cannot occur; the wider sidecar remains RequiresEvidence. |
| 3. Cross-Dimension Consistency | **Partial / not export-ready** | Leaf/source E↔R/F/K↔U/F↔X checks remain. Action-E guard compatibility is not proven, and root-total conditional K aggregation needs an explicit coordinate rule. |
| 4. Root Export Compatibility | **Not established** | `NotExpressedInSemanticIr` / `Unproven`; no root pin and typed I/O/R map. |
| 5. Implementation / Runtime Assurance | **Not established, deferred** | No body attestation, grant, Worker admission, budget reservation, receipt, or recovery execution. |

## Findings summary

* P0: **none confirmed** in the root E/U envelopes or structural walk.
* P1: formal Root Export proof/mapping design is absent; action-E guard versus
  K-derived sidecar semantics and nested CompScope local-versus-root K totals
  are not fully frozen.
* P2: the existing oracle does not freeze K-bearing nested Retry/Iterate,
  same-atom Forward/action, or K-bearing RACE/TIMEOUT/HANDLE combinations;
  exact root export I/O/R/E/Phi/F/U/K/X comparisons are not present.
* P3: authority, budget, resource handles, receipt classification,
  reconciliation provider, cancellation/settlement, Worker admission and
  implementation/body conformance are intentionally runtime or release
  evidence, not current static proof.

The 40 independent counterexamples and current coverage/severity are in
`phase-04b2c-counterexamples.md`. The exact per-opcode and cross-dimension
matrices are in `phase-04b2c-opcode-matrix.md` and
`phase-04b2c-cross-dimension-matrix.md`.

## Recommendation

Choose **Outcome C — Design Incompleteness**. No P0 false-negative total was
confirmed: the K-derived action sidecar is wider than the narrow action-E row
and remains RequiresEvidence. But E↔K guard compatibility is not proven, and
root-total conditional K aggregation is unspecified; do not claim Level 3 or
Level 4 export compatibility. The next authorized stage should decide these
semantics and design **Phase 4B.2D — Root Export Contract Proof Model**. Option
A is recommended for evaluation; no assertion type/checker is created here.

CALL_SUBFLOW body evidence remains a separate requirement. A root export
assertion may prove static envelope containment only and must not claim all
subflow/component implementations conform. Runtime authority or execution
work must not be bundled into that phase.

## Gate record

Pre-commit audit documentation checks:

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `cargo test --all` — pass, including 58/58 Control, 72/72 Advanced Control,
  55/55 Scope/Closure, leaf, ComponentContract, IR/dataflow, and static-surface
  suites.
- `cargo run -p af-spec --bin spec-check -- check` — pass: 40 specifications,
  360 rules, 359 covered, one intentional exception, 0 missing coverage,
  477 active ABI rows, 16 AFIDL modules, 0 cross-reference errors.
- All 21 `tools/gen-*.py` generators ran twice. SHA-256 over
  `spec/conformance/` plus `spec/registry/` matched both rounds:
  `5398CAAF031420ED38D6999845B8B927F448092DFFC96282802E314ED41E1B23`.
- Starting-HEAD comparison found no changes to AFIDL, ABI rows, HashV1 domains,
  digest goldens, or existing Conformance Expected. Only these audit documents
  are added; the isolated guard probe was deleted.

The post-commit rerun and final audit-document commit are reported in the
completion message. Passing tests do not establish root export compatibility
or runtime assurance.
