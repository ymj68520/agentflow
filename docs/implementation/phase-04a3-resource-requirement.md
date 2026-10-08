# Phase 4A.3 — Resource Requirement Row / Static Resource Contract

Status: complete (closed 2026-10-08). The R dimension of
`Computation<I, O, Φ, ε, R>` is materialized as the static declaration
side only. Companion document: `phase-04a3-decisions.md` (D-1..D-17).

## 1. Deliverables

| Artifact | Path |
|---|---|
| AFIDL module (`agentflow.resource.requirement@1`) | `spec/idl/resource-requirement.afidl` |
| Rust crate | `crates/af-resource` |
| Normative semantics | `spec/resource/resource-requirement.md` |
| Conformance corpus (revision 1, 34 cases) | `spec/conformance/resource-requirement/` |
| Independent Python oracle | `tools/gen-resource-requirement-corpus.py` |
| Decision register | `docs/implementation/phase-04a3-decisions.md` |

## 2. Wire model

4 record types / 7 ABI rows (`agentflow.resource.requirement@1`):

- `ResourceTypeRef { 1 schema: core.TypeSchemaRef }` — nominal identity,
  exact pin;
- `ResourceAttributeConstraint { 1 key: string; 2 value: string }` —
  closed exact-value predicate;
- `ResourceRequirement { 1 port_no: uint32; 2 resource_type:
  ResourceTypeRef; 3 required_attributes: list<...> }` — one requirement
  at an interface port;
- `ResourceRequirementRow { 1 requirements: list<ResourceRequirement> }`
  — the closed row.

No handle/binding/adapter/grant/credential type exists; the AFIDL
self-hosts (ABI numbers active in `spec/registry/abi-numbers.json`,
TypeSchemaDigests frozen in the AFIDL corpus case `resource-requirement`,
corpus revision 8, purely additive).

## 3. Static analysis surface (`crates/af-resource`)

Dependencies exactly `af-core`, `af-idl`, `af-semantic`, `serde`,
`unicode-normalization` (NFC). No `af-ir` / `af-effect` / `af-failure` /
`af-spec`, no I/O, no clock, no network.

- `model.rs` — the four wire types + `nominal_key` / `is_same_type`;
- `universe.rs` — `ResourceTypeUniverse` (exact schema resolution +
  explicit recognition descriptors with per-type closed attribute
  vocabulary; stale descriptors rejected);
- `canonical.rs` — `canonicalize_resource_requirement_row` (NFC
  normalize-then-check, positive unique ports ascending, attribute keys
  in unsigned UTF-8 order, duplicates rejected) + canonical ACE1 bytes;
- `validate.rs` — canonicalization + exact universe resolution +
  attribute-vocabulary check (`unknown_resource_attribute`);
- `compatibility.rs` — `satisfies_resource_requirement` (exact type +
  (key, value) superset) and `requirement_not_stronger` (namespace-aware
  non-strengthening proof);
- `error.rs` — 12 stable kinds.

## 4. Non-strengthening (the frozen algebra)

Principle: `{candidates satisfying old} ⊆ {candidates satisfying new}`.
Per retained port: exact nominal type kept (else
`Unproven(retained_port_resource_type_changed)`), all NEW-side attribute
keys in the descriptor vocabulary (else
`Unproven(unknown_attribute_semantics)`; OLD-side unknown keys never
block — they only shrink the old candidate set), and
`new_pairs ⊆ old_pairs` as full (key, value) pairs (violation =
`resource_requirement_strengthened`). New port = strengthened; removed
port never strengthens in R. Cross-namespace comparison fails closed
(`resource_requirement_unproven`). Direction is the opposite of the 4A.1
effect-scope containment and is never interchanged with it.

## 5. Conformance corpus

34 frozen cases, revision 1: 8 golden (canonical ACE1 + is_empty +
count; empty row, single/multiple ports, exact/multiple attributes,
same-type distinct ports, same-short-name different family, canonical
ordering), 11 negative (port 0, duplicate port, unknown type, wrong
digest, schema-not-recognized, duplicate key, NFC duplicate key, unknown
attribute, candidate missing/mismatched attribute, candidate wrong
type), 12 compatibility (identical rows, removes port / adds port,
removes / adds attribute, changed nominal type, same short name
different digest, unknown-attribute implication Unproven,
old-unknown-does-not-block, cross-namespace fail-closed, 2 satisfaction
positives), 3 equivalence groups (port order, attribute order, NFC
text). Rust consumes the frozen bytes; the oracle computes all expected
values independently.

## 6. Registry delta

- Rule area `RESOURCE` added; rules `AF-RESOURCE-0001`..`0018` (18).
- Specifications +2: `agentflow.resource.requirement@1` (schema),
  `agentflow.resource.requirement.semantics@1` (schema_language) — 20
  total.
- ABI assignments +7 (273 active total); suites +6 (`SUITE-RESOURCE-*`,
  44 total).
- spec-check: 20 specifications / 221 rules / 220 covered + 1
  intentional / 0 missing / 273 ABI / 7 AFIDL modules / 0 cross-reference
  errors.

## 7. Zero drift

Old conformance corpora byte-identical (encoding, digest, semantic-core,
semantic-dataflow, semantic-ir, subflow-interface, spec-registry,
effect-signature, failure-row); effect corpus revision 2 and failure
corpus revision 1 unchanged; AFIDL corpus bump 7→8 purely additive (no
old TypeSchemaDigest removed); semantic-ir normative digest stays
`9fb3292a…`; hash-domains.json unchanged (14 domains); no resource
digest of any kind exists.

## 8. Gates

- `cargo fmt --all -- --check`: clean.
- `cargo clippy --all-targets --all-features -- -D warnings`: 0 errors.
- `cargo test --all`: 234 passed / 0 failed (10 in `af-resource`: 4
  conformance + 6 static-surface audits).
- spec-check: exit 0 (numbers above).
- All 11 generators run twice consecutively: zero diff over the 410-file
  spec tree.

## 9. Boundary

Not implemented (structurally absent, static-audited):
ComponentContract, ResourceBinding / ResourceHandle / ResourceAdapter /
physical resolution, EffectGrant / credentials / leases, authority
evaluation, policy/deployment/binding snapshots, scheduling / placement
/ quota / usage, InformationFlow / RecoveryContract /
ExecutionRequirements, plan / journal / reducer / runtime, effect-port
validation. The `component-contract` HashV1 domain stays reserved and
untouched.
