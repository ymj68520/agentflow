# Phase 4B.0 — ComponentContract Pre-assembly Audit

Status: the original audit findings are closed at design level: P0 in Phase
4B.0A and all eight P1 decisions in Phase 4B.0B. **Ready to implement Phase
4B.1, but no ComponentContract implementation or schema has been created.**

Audit baseline: commit `5dc75eb` (`docs(execution): record phase 4a7 decisions`),
clean worktree. The audit adds only design documents and a read-only audit
script. It creates no ComponentContract wire schema, production type, ABI row,
digest, or HashV1 value.

## Decision

At the audit baseline, one verified P0 defect made EffectAtom equality
unsound and eight P1 design blockers prevented an unambiguous candidate ABI.
Phase 4B.0A closed the P0. Phase 4B.0B then accepted designs for all eight
P1s, including exact I/O/Alias policy, operation/resource relation,
selector-grain Flow bindings, complete Recovery action/Usage bindings and the
F/X relation. Those decisions make an additive candidate expressible without
altering old schemas. The checker, relation catalogs and conformance
implementation remain future work; the readiness disposition is recorded in
[`phase-04b0b-readiness-report.md`](phase-04b0b-readiness-report.md).

The detailed artifacts are:

* [Field matrix](phase-04b0-field-matrix.md)
* [Port namespace matrix](phase-04b0-port-namespace-matrix.md)
* [Compatibility matrix](phase-04b0-compatibility-matrix.md)
* [Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md)
* [Counterexamples](phase-04b0-counterexamples.md)
* [ABI proposal](phase-04b0-abi-proposal.md) — design proposal only
* [Decisions](phase-04b0-decisions.md)

## Audit basis and evidence classes

Read the frozen architecture research v6, Phases 3A/3B/3C/3D and 4A.1–4A.7
decision and implementation records, all twelve existing AFIDL modules, the
static crate models/canonicalizers/validators/algebras/compatibility APIs,
the Subflow derived-interface code, and the relevant frozen conformance
corpora. The architecture document says its later v1 machine baseline takes
precedence over earlier conceptual prose; actual registered specs and ABI
remain the machine authority.

The Phase 4A.1B guarded-join correction is taken from the amended D-15 and
Effect corpus revision 2. The Phase 4A.4B unknown-provenance correction is
taken from the F decision record and Information Flow corpus revision 2:
`unknown_external` and `influence_unknown` remain Unproven and are never
normalized to clean/empty provenance.

Every statement in the matrices is classified as one of:

* **FROZEN REQUIREMENT** — stated by the architecture baseline or a
  registered normative source.
* **MATERIALIZED ABI** — present in an existing AFIDL module and its frozen
  TypeSchemaDigest/ABI assignments.
* **DERIVED ANALYSIS** — non-wire Rust result or projection.
* **IMPLEMENTATION ASSUMPTION** — supplied by an explicit in-memory universe,
  profile, or caller; not independently authenticated by these crates.
* **UNDEFINED** — no frozen rule or API currently establishes it.
* **DEFERRED RUNTIME** — requires admission, attestation, policy, a worker,
  enforcement, or an external effect result.

An AFIDL field being serializable is not sufficient reason to hash it as
ComponentContract semantics. See the field matrix for the current status of
every proposed field.

## Baseline inventory

At `5dc75eb`, the formal baseline is 30 Specifications, 296 Rules, 400 active
ABI assignments, 12 AFIDL modules, 14 HashV1 domains, and 67 conformance
suites. The exact pins and artifacts are checked by
[`audit-component-contract-readiness.py`](../../tools/audit-component-contract-readiness.py).

| Dimension | Existing wire authority | Rust surface | Current proof limit |
| --- | --- | --- | --- |
| I/O | semantic-core + semantic-ir | `SemanticTypeNode`, `StructuralType`, `RunParameterDecl`, `WorkflowOutputDecl`, `InputBinding`, `ComputationOutputBinding` | Local typed dataflow; no Component input/output ABI or INVOKE contract checker |
| Φ | failure-row | `FailureRow`, exact `FailureTypeRef`, set algebra | Exact nominal containment; INVOKE/CALL_SUBFLOW envelope is Unproven without full contract |
| ε | effect-signature | `EffectSignature`, `EffectAtom`, `EffectCatalogUniverse`, algebra | Static effect ceiling; no ResourceType compatibility relation; P0 AtomKey issue below |
| R | resource-requirement | `ResourceRequirementRow`, `ResourceTypeUniverse` | Exact type and attribute predicates; no cross-component row mapping or handles |
| F | information-flow | `InformationFlowContract`, label and sink universes | Static summary/ceiling; source refs are positive coordinates only; external sink is operation-level |
| U | usage-signature | `UsageSignature`, usage algebra/envelopes | Additive bounds and control helpers; missing is not zero; calls/subflows remain unknown without declarations |
| Cost | cost-projection | price snapshot, projection, static ceiling | Analysis under a supplied fixed price snapshot; no reservation or live billing |
| K | recovery-contract | `RecoveryContract`, safety obligations, compatibility | Exact clause coverage for closed ε rows; recovery action identities and U coverage remain incomplete |
| X | execution-requirements | requirements, explicit universes, static profile matching, two compatibility verdicts | Conditional proof over supplied descriptor; not attestation, admission, or placement |

The specific file/type/API details and canonical-field recommendation are in
the field matrix. The namespace separation and proposed mappings are in the
port matrix.

## P0 — Semantic soundness defect (audit finding; closed in Phase 4B.0A)

`EffectAtom` identity is defined as operation + ResourceSelector + qualifier
list. The frozen `EffectIdempotencyQualifier` has two distinct members,
`natural` and `none`, each with a different `value_no`. But
`af_effect::AtomKey::of` stores only `EffectQualifier::family_no()`. Both
members therefore map to the same key. That key drives duplicate rejection,
effect containment and algebra matching/merging, and Recovery clause coverage.
For example, a single declared row containing otherwise-identical `natural`
and `none` atoms is classified as duplicate even though the canonical ACE1
member values differ.

A temporary probe against the existing public API confirmed that a
`RecoveryContract` clause for the `natural` atom, an ε row containing the
otherwise identical `none` atom, and `Applied` evidence for that ε atom can
produce `RecoveryProof::ProvenStaticConstraint`. Exact recovery coverage
should not match different qualifier values. The probe source was removed
after verification; the reproducible vector is preserved in the counterexample
document.

This also exposes a normative ambiguity: the Effect specification calls the
atom an exact tuple of qualifiers, but describes the canonical key as a list
of qualifier *families*. Resolve the wording in favor of the full qualifier
member identity, correct `AtomKey` to retain family and member value, and add
canonical/algebra/recovery vectors before any contract composition relies on
it. This is P0 because it can turn an incompatible recovery clause into a
positive static proof.

No production code was changed during this audit.

### Phase 4B.0A closure record (2026-10-08)

The audit-era public reproducer was rerun at starting HEAD `956e020` before
any production edits. It confirmed both exact observations: the Recovery
clause key and ε atom key printed with `qualifiers: [1]`, and
`analyze_recovery_safety` returned `ProvenStaticConstraint` for a natural
clause against a none-qualified effect with Applied evidence. The typed
`AtomKey` fix now retains the complete `EffectQualifier`; canonical ordering,
duplicate detection, Effect Algebra and Recovery lookup all share it.
Revision-3 Effect corpus adds 15 cases and revision-2 Recovery corpus adds 9
cases. All 50 prior Effect and 34 prior Recovery cases retain their semantic
and Expected fields unchanged; only their declared corpus revision changed.
The new cases freeze both mismatch directions, exact matches, distinct-row
algebra, guard/bounds boundaries and end-to-end public Recovery rejection.
See [Phase 4B.0A counterexamples and regression vectors](phase-04b0-counterexamples.md).

At the close of 4B.0A, this closed only the P0 and the eight P1s still blocked
schema freeze. Phase 4B.0B later accepted all eight P1 designs and recorded
the implementation readiness gate in
[phase-04b0b-readiness-report.md](phase-04b0b-readiness-report.md). No
ComponentContract schema was created in either design phase; readiness is not
implementation or runtime verification.

## P1 — Audit findings (resolved at design level in Phase 4B.0B)

The numbered items below preserve the original audit inventory and wording.
Their accepted decisions and exact candidate fields now live in the Phase
4B.0B Decision Register; they are not claims of implemented proof APIs.

1. **I/O ABI is absent.** Semantic IR inputs are run parameters; outputs are
   named workflow outputs; `SubflowInterface` is a derived view, not a
   Component ABI. There is no frozen component port namespace or general
   subtype/variance model. Phase 4B.1 needs a closed I/O representation and a
   rule that returns Unproven for unequal types unless a formal relation is
   supplied.
2. **ε↔R semantic relation is missing.** Positive port existence can be
   checked at assembly, but `EffectCatalogUniverse` and `ResourceTypeUniverse`
   do not say which resource types an operation supports or which selector
   scope keys are valid for that type. Same numeric port is only a link, never
   proof of operation/resource compatibility.
3. **ε↔F identity grain is too coarse.** `FlowSink::external_effect` pins
   `EffectOperationRef`, while ε distinguishes the operation, ResourceSelector
   and qualifier values. One F sink currently applies to both
   `data.write@R1` and `data.write@R2`; it cannot allow one and reject the
   other. A selector-level Flow sink is needed before the contract can promise
   that resource-sensitive information-flow ceiling.
4. **K recovery-action identity is incomplete.** Forward recovery clauses
   pin a full EffectAtom, but `CompensationRequirement` contains only an
   EffectOperationRef and `ReconciliationRequirement` is only `none/required`.
   The declared reconciliation-operation universe is not connected to a
   contract clause. Neither recovery action can be proved to lie inside the
   complete ε envelope.
5. **K↔U recovery coverage is undefined.** U separates forward and recovery
   meters, but K does not identify the meter envelope of each reconciliation
   or compensation action and no recovery-operation-to-usage profile exists.
   Missing recovery coverage must remain Unproven, never a zero/free cost.
6. **F↔X security mapping is undefined.** F labels/taints and X regions,
   trust domains, and sandbox levels use distinct explicit identities. No
   trusted catalog relation maps confidentiality/residency or influence
   assurance requirements across them. Region names and trust labels cannot
   supply that relation.
7. **Public type identity has two existing policies.** Phase 3A expands
   aliases to their value target, while the Phase 3D Subflow interface keeps
   exact pinned TypeSchemaRef leaves and rejects alias-vs-target. Choose one
   Component I/O ABI comparison policy before freezing substitution behavior.
8. **Component identity/version/digest semantics need a final choice.** The
   design must state whether stable ComponentIdentity is inside canonical
   bytes, keep ContractVersion separate from exact digest, and define how
   content-identical releases under one logical identity are represented.

The ABI proposal gives the minimal candidate shapes for these decisions; it
does not resolve them by inventing a total order or a universal `Any` field.

## P2 — Static proof surface not yet materialized

* There is no ComponentContract validator that checks I/O references, F
  sources/sinks, ε selectors, K coverage, U recovery envelopes, or X/F/R
  relations as one closed object.
* There is no canonical combined compatibility result retaining per-dimension
  `Violation`, `Unproven`, and `RequiresEvidence`. Existing result shapes
  differ: ε containment has Proven/Unproven only; F returns a proof or a
  relaxation error; R splits errors from Unproven; K has four safety outcomes
  but only conservative compatibility; X splits feasibility/security.
* `af-ir` has a typed dataflow analysis and exact Subflow I/O/R checker, but
  INVOKE output/input matching against ComponentContract is deferred. It also
  does not derive Φ/ε/F/U/K/X across the full Control tree.
* Control helpers exist independently. They do not form a single cross-layer
  walk, and there are intentional holes: component calls and subflows require
  resolved full contracts; F, R, K, and X have no general Control propagation
  engine; X intentionally has no blind whole-tree resource aggregation.
* `ImplementationAbiRequirement` checks TypeSchemaRef grammar/digest syntax
  but does not resolve the reference against a supplied TypeSchemaUniverse
  or prove that a selected implementation exports it.
* Static proof inputs are caller-supplied universes/profiles. A future proof
  record should identify their exact immutable snapshots and rule-set version;
  this is not a ComponentContract digest or attestation.
* Resource binding to execution is also absent: R declares logical external
  needs; X describes an executor. Neither says a profile can bind/access a
  ResourceType. This is P2 until a deployment/adapter profile supplies evidence;
  it remains distinct from EffectGrant.

## P3 — Runtime / policy / attestation obligations

Publisher declarations and static profile descriptors do not independently
prove implementation behavior. Publisher identity/signature, contract tests,
evaluation gates, compatibility attestation, deployment binding, worker
admission, runtime sandbox enforcement, current trust/region evidence, live
capacity, policy decisions, EffectGrant, ResourceHandle and budget reservation
remain distinct later evidence/authority objects. A static `Proven` result
must be worded as conditional on its explicit inputs.

## Cross-dimension conclusion

The isolated wire types can be referenced together in a future closed schema.
Phase 4B.0B defines the missing identity and binding models without changing
those isolated schemas. This is a design-readiness conclusion only; a
ComponentContract is not yet a verified runtime artifact. See the
cross-dimension register and the 37 Phase 4B.0B decision vectors.

## Normative consistency review

No pre-existing AFIDL field or RuleId was changed. The audit found the Effect
qualifier identity ambiguity described in P0. The registered F schema
deliberately references operation-only sinks, and the Recovery schema
deliberately uses operation-only reconciliation/compensation references; these
are not implementation contradictions but known underspecified cross-dimension
interfaces. Resolve them as explicit future schema changes, with new corpus
revision and compatibility consequences documented before freezing the new
ABI. The architecture's general I/O subtype formulas are also stronger than
the materialized structural equality API and cannot be declared proven by
today's types.

Architecture v6 §25.7 calls its runtime/authority consistency list closed;
the listed checks concern Journal/State, Commands, Worker/Effect, ownership,
checkpoint, audit and SDK boundaries. It does not define the 4A contract
cross-links listed in this audit. This is a scope distinction, not evidence
that ε↔R, ε↔F, K↔U or F↔X has already been proved. Phase 4A decision records
explicitly defer those links to ComponentContract assembly.

The alias comparison point is likewise treated as a boundary decision rather
than a proven contradiction between registered specs: Phase 3A value typing
expands aliases, while Semantic IR §15 requires exact schema-leaf comparison
for Subflow ports. Phase 4B.0B resolves the public Component boundary to exact
nominal TypeSchemaRef equality; it does not change either existing rule.

### Phase 4B.0B design resolution

The P1s are **ACCEPTED as design decisions**, not as implemented
functionality. Component identity is hashed with the semantic body while
release version/implementation/deployment stay separate; public I/O uses
exact pinned types and nominal aliases; ε↔R and F↔X use exact versioned
relation catalogs; F's old operation sink is refined only by a closed exact
EffectAtom allowlist; K actions and recovery-meter profiles are exact
Component-level bindings. The unchanged legacy schemas remain valid but
cannot claim a more precise proof than they encode.

The candidate ABI and proof API are explicitly non-normative. Phase 4B.1 may
materialize the additive ComponentContract and the three typed relation
catalog schemas together; no Phase 4B.0C is needed provided those catalogs are
not replaced by partial, unpinned data. This audit does not implement any of
them or compute a ComponentContractDigest.

## Zero drift and verification

The audit script compares the protected wire/registry/corpus paths with the
Phase 4B.0 baseline commit, and checks the frozen counts and required API/model
surface. It explicitly labels code-presence checks as inventory evidence, not
semantic proof. After 4B.0B, its missing-code/P1 inventory still reports
implementation facts; it is not the design-decision status. The accepted
design status and implementation boundary are recorded in
[phase-04b0b-readiness-report.md](phase-04b0b-readiness-report.md).
