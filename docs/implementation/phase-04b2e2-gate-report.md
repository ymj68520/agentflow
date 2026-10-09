# Phase 4B.2E.2 — Final Gate Report

## Decision and scope

Phase 4B.2E.2 implements an in-process, non-wire `RootExportAssertion` and a
fail-closed, nine-dimension static compatibility checker. Its strongest claim
is `Static Root/Export Contract Compatibility: Proven` for the exact checked
root, export, compiler semantics, and proof context. It does not prove
implementation or subflow-body conformance and does not grant runtime authority.

The implementation was gate-tested at code HEAD
`f011eef49c0c3b0e92aae061c28743c70f99087d`, on top of the requested starting
HEAD `bf5d37e2b3d99d866da8979b2d988caea298fdbe`. This report is committed in a
following documentation-only commit; the final repository tip is reported in
the completion message and Git history.

## Model, identity, and proof context

- `RootExportAssertion` is typed, in-process, non-wire state. There is no new
  digest domain, publisher identity, persistence format, or caller-supplied
  `compatible`/`proven` escape hatch.
- Structural assertion identity includes exact SemanticIrDigest and
  ComponentContractDigest pins, compiler semantics, the complete typed proof
  context manifest, every typed coordinate map, and exact subflow pins.
  Canonical ordering preserves duplicates for rejection and never relies on a
  Debug string or pointer identity as a stable cross-process identity.
- The checked contract, resolver snapshot, root derivation, and root recovery
  summary must belong to the same pointer-identical
  `StaticContractProofContext`. The manifest separately snapshots complete
  typed universe/catalog content, relation pins and content, and resolver
  dependencies. Context, catalog, map, IR/export pin, compiler, recovery-source,
  or subflow-pin changes invalidate the assertion/report for reuse.
- `CompilerSemanticsIdentity` pins the checker semantics version, normative
  specification identities and full source bytes/checksums, registry bytes,
  and the full applicable RuleId set plus rule content. The current identity
  checks 84 applicable rules; equal RuleIds alone are not accepted as proof of
  equal semantics. No compiler binary attestation is claimed.

## Mapping and nine-dimension proof

The production API exposes detailed analysis and a checked static-only result.
Input/output/resource/effect/Flow/recovery/execution maps are typed and checked
against the actual exact root/export witnesses; omitted, duplicate, aliased,
or unsupported coordinates fail closed.

- **I / O:** Input maps require the complete exact public-input/root-parameter
  bijection and checked use-site evidence. Output maps retain WorkflowOutput,
  expression, producer, type, scope, dominance, and all-path obligations.
- **R:** Resource mappings preserve source component port → scoped slot → root
  coordinate → exported requirement port, including nominal resource/schema
  types, attributes, aliases, scope, selector, and effect linkage.
- **Phi / epsilon:** Failure containment uses nominal types. Effect comparison
  uses full atoms, qualifiers, resource selector, guard, and occurrence bounds.
  Guard counterexamples are not treated as reachable when the root tree has
  SELECT, RACE, or HANDLE control-sensitive paths; that case stays Unproven.
- **F / X:** Typed source/sink, FlowKind, label, taint, decision, and effect-flow
  partition coordinates are checked. Unknown external influence remains
  unknown. Every child gets separate execution feasibility and security
  results; neither result admits a Worker.
- **U / K:** Forward and recovery usage are compared separately by exact phase,
  meter, unit, and bound. Root K source/action/profile/factor multisets and
  safety obligations reuse RootRecoverySummary. K action and usage projections
  are coverage checks against existing epsilon/U envelopes and are never added
  a second time.

Every dimension has its own verdict, evidence, coordinate witnesses,
preconditions, missing evidence, violations, and applicable rules. Unknown is
not empty; a failed containment routine is not automatically a counterexample.
Subflow dependency closure checks exact target/export pins and missing/cyclic
transitive dependencies. Static compatibility can be Proven while body and
implementation conformance remain RequiresEvidence.

## Conformance, oracle, and comparator

The corpus contains **35 CaseIds**: X01–X22 design-vector mappings and 13
cross-dimension/lifecycle cases. Reachability distribution is **34** production
RootExport checker cases plus **1** evidence-gated ComponentContract rejection.
All 35 manifest cases execute their declared production stage and are fully
compared: 8,337 semantic fields, 0 skipped, 0 mismatches, 0 uncovered required
fields. The independent Python oracle constructs expected values from
declarative inputs and mutations; it does not call Rust or derive Expected from
the production projection.

The isolated comparator mutation suite detected **38/38** corruptions,
including exact pins, maps, schemas, resource source/alias coordinates, full
EffectAtom qualifiers/guards, recovery profiles, proof-context/compiler
identity, Flow partitions, child X, body evidence, and runtime-authority
promotion.

The preserved production suites pass unchanged:

| Suite | Result |
| --- | ---: |
| Control | 58/58 |
| Advanced Control | 72/72 |
| Scope / Closure | 55/55 |
| Root Recovery | 44/44 |

The positive fixture proves all nine dimensions and static compatibility while
both body and implementation conformance are `RequiresEvidence`, and runtime
authority remains `NotEstablished`.

## Registry and frozen surfaces

| Surface | Starting baseline | Final |
| --- | ---: | ---: |
| Specifications | 40 | 41 |
| Normative rules | 366 | 380 |
| Rule coverage | — | 379 covered, 1 intentionally uncovered, 0 missing |
| Active ABI assignments | 477 | 477 |
| AFIDL modules | 16 | 16 |
| HashV1 domains | 14 | 14 |

The added rules cover RootExport pins, context/compiler identity, complete
coordinates, nine dimensions, source closure, and non-authority boundaries.
The base-to-final path comparison found **0 changed paths** in AFIDL,
ComponentContract/Semantic IR frozen conformance surfaces, ABI assignments, and
HashV1 domain tables. Existing digest goldens remain under the full test suite.

## Final gates at code HEAD `f011eef49c0c3b0e92aae061c28743c70f99087d`

| Gate | Result |
| --- | --- |
| `cargo fmt --all -- --check` | Pass |
| `cargo clippy --all-targets --all-features -- -D warnings` | Pass |
| `cargo test --all` | Pass, including 58/72/55/44 and RootExport differential |
| `cargo run -p af-spec --bin spec-check -- check` | Pass; 41 specs, 380 rules, 0 missing coverage, 477 ABI rows, 16 AFIDL modules |
| All `tools/gen-*.py`, two consecutive rounds | Pass; 23 generators, 986 spec files, round 1 = round 2 and pre-run = post-run SHA-256 snapshots |
| Frozen wire/digest path diff from starting HEAD | Pass; 0 changed paths |

The same gates and generator idempotence check were rerun after the categorized
implementation/test/documentation commits, including guard-path fail-closed
commit `f011eef`.

## Findings and boundaries

- **P0:** None known.
- **P1:** None known. No required static proof is promoted when the accepted
  model cannot express it; unsupported relations remain Unproven.
- **P2:** No in-scope conformance or required-field gap remains. Deliberate
  fail-closed limitations include transformed input identity, unsupported
  Recovery monotonicity, incomplete source/catalog facts, and action-specific
  Flow/X evidence not expressible by the current model.
- **P3:** Component implementation and subflow body evidence, publisher
  authentication, Scope authority/budget attenuation, Worker admission,
  authorization, runtime receipts, and runtime recovery outcomes are outside
  this static phase. Authority/budget are Unproven where the frozen model has
  no ceiling; runtime authority is NotEstablished.

**Static freeze readiness:** Yes, for the implemented static Root/Export
compatibility subset only. This is not authorization to proceed to publication
or runtime. Because the checker combines nine dimensions and proof-context
invalidation, an independent Root Export Proof Audit is recommended before
relying on the result as a release control.

## Categorized commits

1. `b47b679` — `feat(computation): add static root export proofs`
2. `4d0314a` — `test(computation): add root export differential corpus`
3. `dbb18b3` — `docs(computation): document root export proof boundaries`
4. `f011eef` — `fix(computation): fail closed on guarded root paths`
5. Gate report — documentation-only commit following the code HEAD above.

No WorkflowVersion, publication, ExecutablePlan, Journal, Reducer, Scheduler,
Worker, or Runtime phase was started.
