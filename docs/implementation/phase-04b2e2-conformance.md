# Phase 4B.2E.2 — Independent Conformance

The independent Python oracle is `tools/gen-computation-root-export-corpus.py`.
It constructs formal checked-IR fixtures and expected verdicts from declarative
inputs and mutation labels; it never invokes Rust or imports production
projection output. The production runner is
`crates/af-computation/tests/root_export_conformance.rs` and always calls the
real checked-contract, checked-IR, Phase 3C, resolver, workflow-root,
RootRecovery, and RootExport analysis/check APIs reachable for the case.

Corpus revision 1 has 35 CaseIds: 22 design-vector mappings (X01–X22) plus
cross-dimension and proof-lifecycle cases. Stage distribution is 34 cases at
RootExport checker and one evidence-gated ComponentContract rejection; no
invalid object is forged into a CheckedComponentContract. The runner asserts
manifest/disk equality, required field presence, exact typed map row counts,
all nine independent verdicts, feasibility/security separation, body/runtime
separation and authority/budget boundary status.

The isolated comparator mutation suite detects 38 corruptions, including
changed exact pins, schemas, mappings, full EffectAtom qualifier, guard,
resource source/alias coordinate, Recovery profile, context content, compiler
semantics, output producer, X feasibility/security, body proof, and runtime
authority. Expected values are not derived from Rust output.

The earlier production suites remain separate and are rerun unchanged:
Control 58, Advanced Control 72, Scope/Closure 55, Root Recovery 44.
