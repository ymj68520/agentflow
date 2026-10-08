# Phase 4B.1 — Conformance and Gate Report

## Frozen surfaces and additive ABI

The implementation starts at `36fc9dd6f2ce1b137fe5f963b1eb2d925364923a`.
The only new AFIDL modules are:

1. `agentflow.component.contract@1`
2. `agentflow.effect.resource.compatibility@1`
3. `agentflow.recovery.usage.coverage@1`
4. `agentflow.flow.execution.relation@1`

The new modules contribute 26 TypeSchemaDigests and 77 stable ABI
assignments. The ComponentContract digest uses the existing
`component-contract` HashV1 domain; the domain registry remains at 14.
Registry totals are 36 Specifications, 317 Rules, 477 active ABI
assignments, 16 AFIDL modules, and 72 conformance suites. Rule coverage has
zero missing references.

Baseline comparison verified all 12 prior AFIDL files byte-identical, all
400 prior ABI rows unchanged, all 296 prior Rule entries unchanged, and all
67 prior suite entries unchanged. The single changed prior Specification
descriptor is `agentflow.registry.rule-areas@1`, whose source digest updates
to include the additive `CONTRACT` Rule Area. Its new schema IDs, rules,
assignments, suites and corpus are additive. All 618 pre-existing
conformance files other than the intentionally extended `suites.json` remain
byte-identical; this includes Effect corpus revision 3, Recovery revision 2,
the prior TypeSchemaDigest corpus and SemanticIrDigest corpus.

## New conformance corpus

The independent Python oracle generates four-module schema goldens (26 named
type schemas), nine contract goldens, 24 negative vectors, four canonical
vectors, 14 cross-dimension vectors, 16 substitution vectors, and eight
equivalence vectors. Each relation catalog has two valid snapshots, two
invalid vectors, one canonical-order vector, and two snapshot-equivalence
vectors. Rust tests decode every contract golden against the compiled
ComponentContract shape and independently verify its HashV1 digest; the
non-empty relation catalog bytes/checksums are also checked against the
oracle.

## Gate results

The full gate set was rerun after the implementation and report commits and
passed:

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo test --all`
- `cargo run -p af-spec --bin spec-check -- check`
- all 17 `tools/gen-*.py` generators, two consecutive rounds with identical
  `spec/` tree SHA-256 `7760843a3cb6110d3bf7c8d70239b8c79cc9581d757d5af8de702fd453fe9355`

The Phase 4B.1 Rust crate contains unit tests for recovery-cycle detection and
checked usage arithmetic, conformance tests for four-schema and catalog
goldens, digest construction, exact I/O pins, epsilon/resource matching,
full qualifier identity, F/X relations, recovery usage aggregation,
proof-context invalidation, substitution and static-surface boundaries.
The final repository handoff includes the post-commit run, final commit IDs,
and a clean Git-state check.

## Decisions and deferrals

No accepted Phase 4B.0B ABI or identity decision was amended. Missing catalog,
meter, or exact atom evidence remains `Unproven`; open/unknown rows are not
treated as empty. Publication version, implementation digest, pricing,
diagnostics and relation snapshots remain outside ComponentContract
semantic identity. The SHA-256 relation checksum is not a signature or new
HashV1 domain.

Whole Semantic IR traversal, control-tree contract closure, runtime,
scheduler, worker attestation, execution authorization, effect grants,
resource handles, recovery execution, receipts, reservations, ledgers, plans,
journals and reducers remain deferred. This phase stops at a single
ComponentContract and static substitution proof; Phase 4B.2 is not started.
