# Phase 3D — Subflow Interface Derivation: Decision Register

> Status: Phase 3D implementation record. Binding on the Phase 3D
> implementation; the normative source is `spec/semantic/semantic-ir.md`
> section 15. The wire model is untouched: `spec/idl/semantic-ir.afidl` stays
> byte-identical at
> `9fb3292a0fe90b207db037575c50ef3ca50174411b3c5a461c09c37d53f616a3`.

## D-1 Derived interface, not a wire object

`SubflowInterface` projects the target's own three public tables —
`run_parameters` → inputs, `workflow_outputs` → outputs, every declared
`ResourceSlot` → resources. No `SubflowInterface.afidl`, no interface
digest, no ABI row, no Hash domain. The type lives in `subflow.rs`, is
plain Rust (no serde derives — statically audited), and appears nowhere in
`model.rs`, `canonical.rs` or the schema module. The task's
`ResolvedSemanticType` is materialized as the **existing**
`af_semantic::StructuralType` (same closed shape: Primitive/Schema/List/
MapString/Option, derived structural equality, frozen `canonical_string()`),
so no duplicate enum was created.

## D-2 Expansion is re-derived locally, and stays naive on purpose

`af_semantic`'s Phase 3A resolution normalizes schema leaves through the
universe (aliases collapse to their target). The subflow expansion instead
keeps the **pinned** `TypeSchemaRef` and compares pins exactly — the task
mandates exact `TypeSchemaRef` equality (specification_id + type_name +
schema_digest), which makes alias-vs-target mismatch a rejection, a
deliberately strict exact-contract posture. Expansion is a small memoized
walk over the public `SemanticTypeNodeBody`, valid because Phase 3A
guarantees DAG acyclicity; threading `CheckedDocument`'s private resolved
table through `CheckedIrDocument` would have changed the public lifetime
signature for no rule benefit.

## D-3 Universe validity is type-enforced, not error-enforced

`SubflowIrUniverse::insert(&CheckedIrDocument, &CompiledModuleSet)` makes
Phase 3B validity a type-level fact and runs `analyze_dataflow()` itself,
so a target that fails Phase 3C is **uninsertable** — the recommended
`invalid_subflow_target` runtime kind is unnecessary (the task explicitly
prefers making invalid state hard to construct). The digest is computed
from the target document over the explicitly supplied module set; callers
never pass a trusted digest string. Duplicate insertion: same digest + same
canonical bytes → idempotent; same digest + different bytes →
`subflow_target_collision` (never silently overwritten). Keys are full
`DigestRefValue`s (algorithm + domain + hex), not bare hex.

## D-4 Canonical port mappings

Inputs: `RunParameterId` ascending → ports 1..N. Outputs: canonical
NFC-name byte order → ports 1..N. Resources: `ResourceSlotId` ascending →
ports 1..N. Derivation sorts by the canonical keys itself, so even a
non-canonical source table yields the canonical interface (and canonical
tables are unaffected). The three namespaces are independent; ordinals are
external coordinates, never aliases for target-local ids; names ride along
as diagnostics only.

## D-5 All declared ResourceSlots are the resource interface

Including currently-unused slots: the slot is the Phase 3A contract
surface, and an unused slot is an authoring defect to fix before
publishing, not something the interface layer hides. The caller must bind
ports 1..N exactly and each bound caller slot's `requirement_type` must be
the exact `TypeSchemaRef` of the target port. No value-type conflation:
resources never compare as `SemanticTypeId`s.

## D-6 Exact port sets, seven stable kinds

Minimum kind set, exactly as recommended: `unknown_subflow_target`,
`subflow_input_port_mismatch`, `subflow_input_type_mismatch`,
`subflow_output_port_mismatch`, `subflow_output_type_mismatch`,
`subflow_resource_port_mismatch`, `subflow_resource_type_mismatch`, plus
`subflow_target_collision` for the universe invariant (D-3). Missing/extra
ports share one kind per dimension with a missing/unexpected diagnostic;
types carry expected/actual rendered structural types in the detail (not
ABI). Phase 3B kinds (unknown contract, wrong kind, duplicate ports) are
untouched.

## D-7 Pass ordering is structural

`check_subflow_interfaces(&self, &SemanticDataflowAnalysis,
&SubflowIrUniverse)` consumes the checked document plus its own analysis —
the Phase 3C result is a required input, so the pass cannot run before it.
All 12+16 corpus cases pass Phase 3B and Phase 3C first (the generator
re-validates both layers for every document).

## D-8 No recursion, no resolver

Target interfaces are derived without validating the target's own
CALL_SUBFLOW edges; subflow-graph cycles are not judged. Dependency
closure and graph acyclicity stay with the future Resolver / LockGraph;
this phase never becomes a half resolver and never reads deployments or
workflow names.

## D-9 The corpus carries a second fixture schema

`input/schema-digest-mismatch` and `resource/schema-digest-mismatch` need
two universe-valid refs with the same record name but different pins — a
fake digest would die in Phase 3A (`universe.resolve` checks pins). The
corpus therefore freezes `agentflow.test.semantic.v2@1 / User` (same shape,
different specification) as a `fixture_schemas` block; the Rust harness
compiles the frozen source and pins the frozen digests into its universe.
The projections in all three files are labeled NON_NORMATIVE CONFORMANCE
PROJECTION — the interface is never serialized in production.

## D-10 Registry shape

15 additive rules `AF-SEMANTIC-0096..0110` (existing RuleIds verbatim),
4 suites `SUITE-SUBFLOW-*`, each rule on a dedicated anchor. Totals: 168
rules, 28 suites, 14 specifications, 232 ABI rows, 4 AFIDL modules, 14
Hash domains — only the semantics specification's source digest updated,
as required.

## D-11 Zero drift, again enforced by pins

The Phase 3B/3C corpora, semantic-core, ACE1/HashV1 corpora, AFIDL corpus
revision 5, the 22 golden digests and the 20+37 dataflow expectations are
all `git diff`-empty; generators run twice with zero diff; the afidl sha
pin test still guards the schema bytes.
