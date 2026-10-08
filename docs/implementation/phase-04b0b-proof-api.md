# Phase 4B.0B — Future Cross-Dimension Proof API

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT IMPLEMENTED**

The following is the future pure static surface for `af-contract`. It defines
proof boundaries and data flow only; this phase creates no crate, production
type, checker, digest, or runtime behavior.

## Core operations

```text
canonicalize_component_contract(raw)
    -> Result<CanonicalComponentContract, StructuralViolations>

validate_component_contract(
    contract: CanonicalComponentContract,
    universes: ExplicitContractUniverses
) -> ComponentValidationReport

check_cross_dimension_consistency(
    checked: ComponentValidationReport,
    universes: ExplicitContractUniverses
) -> CrossDimensionReport

check_component_substitution(
    new: CanonicalComponentContract,
    old: CanonicalComponentContract,
    universes: ExplicitContractUniverses
) -> ComponentCompatibilityReport
```

`canonicalize_component_contract` validates closed structure, identity syntax,
positive/unique ports, exact-key duplicates, canonical order and all
component-local invariants. It does not invent missing cross-dimension
universes. A structural defect is a Violation; it is never converted into an
empty row.

`validate_component_contract` resolves exact TypeSchemaRefs and existing
dimension catalogs, then returns per-dimension results. It does not issue an
authorization. `check_cross_dimension_consistency` runs the explicit links in
the order below. `check_component_substitution` requires both contracts to be
independently checked under the same exact relation snapshots; it does not
compare version strings or digests as a substitute for semantic proof.

## Typed input context

`ExplicitContractUniverses` is a closed typed record, not a map of strings:

```text
type_schemas: TypeSchemaUniverse
effects: EffectCatalogUniverse
resources: ResourceTypeUniverse
flow: FlowCatalogs
usage_meters: UsageMeterUniverse
recovery_usage_coverage: RecoveryUsageCoverageCatalog
effect_resource_compatibility: EffectResourceCompatibilityCatalog
flow_execution_relation: InformationFlowExecutionCatalog
execution: ExecutionUniverses
```

Each catalog is read-only for one analysis. Its identity, revision and
canonical-content checksum are captured in the proof report. The checksum is
plain catalog-content SHA-256, not HashV1; it is an identity pin, not a
signature or trust proof. Missing catalog data is Unproven. Catalog
authentication/authority is a separately trusted input assumption.

Candidate proof-context pin (non-wire analysis type):

```text
RelationCatalogPin {
    schema: core.TypeSchemaRef
    revision: string
    canonical_content_checksum: CatalogContentChecksum
}
```

`schema` pins the exact relation-catalog schema; `revision` and the plain
canonical-content checksum pin the exact immutable instance. The checksum is
not an implementation artifact digest or HashV1 digest. Neither value is
stored in ComponentContract or authenticates its source.
`revision` is a non-empty NFC release token compared only by exact equality;
it has no lexical/latest ordering semantics. A checksum or schema mismatch
makes two proof contexts different; absent an explicit catalog-evolution
relation, cross-context substitution is Unproven.

## Verdict model and phase separation

```text
enum Verdict {
    Proven,
    Violation,
    Unproven,
    RequiresEvidence
}

DimensionVerdict {
    dimension: ComponentDimensionOrRelation
    verdict: Verdict
    reason: ClosedReason
    assumptions: list<ExactSnapshotPin>
    required_evidence: list<EvidenceKind>
}
```

The report retains a separate verdict for identity, I, O, Φ, ε, R, F, U, K,
X and every required cross-dimension relation. A list of dimension verdicts
is canonicalized by closed dimension number then exact subject identity. It is
not hashed into ComponentContract and is not itself an attestation.

The aggregate rule is conservative: all required checks Proven yields
`Proven`; a known Violation remains a Violation even if another check is
Unproven; absent a Violation, any Unproven remains Unproven; otherwise
outstanding independent evidence yields RequiresEvidence. Per-dimension
results are retained regardless of the aggregate so one verdict never hides
another.

The API distinguishes:

```text
Schema Valid
Cross-Dimension Consistent
Statically Compatible
Attested
Authorized To Execute
```

The first three are conditional static results. `Attested` is an external
evidence verification result. `Authorized To Execute` is a later policy /
EffectGrant / admission result and is not returned by this API.

## Cross-dimension pass order

1. **Structural canonicalization.** Reject malformed or duplicate closed
   fields, invalid port coordinates and noncanonical semantic identities.
2. **Exact schema resolution.** Resolve every public I/O TypeSchemaRef,
   FailureTypeRef and ResourceTypeRef by full identity and digest. Semantic
   TypeIds remain local to their own IR document.
3. **Namespace resolution.** Build separate I, O and R maps. Resolve INVOKE
   bindings, F input/resource/output references, ε selectors, and guards in
   their owning namespace. Equal integer port numbers never cross-link.
4. **Effect/resource proof.** For every closed EffectAtom, first verify its R
   port exists, then separately check R attribute requirements, selector
   scope rules and exact operation/resource support. Relation snapshots are
   explicit inputs; open/unknown ε is Unproven.
5. **Effect/flow proof.** Resolve every F sink. Expand legacy operation
   sinks to all matching exact atoms unless an `EffectFlowPartition` narrows
   that closed set. Apply the complete aggregate F summary to every permitted
   sink. Unknown provenance/influence remains Unproven.
6. **Recovery action proof.** Match every K forward clause to an exact ε
   AtomKey; require exactly the actions prescribed by K; require each full
   action EffectAtom in closed ε; reject action cycles.
7. **Recovery usage proof.** Resolve exact action coverage profiles, require
   explicit meter limits, apply activation/attempt multipliers, accumulate
   conservatively, and compare against explicit U recovery coordinates.
   Missing is Unknown, never finite zero.
8. **Flow/execution proof.** Apply exact label/taint/authority/sink mappings
   to X. For region residency check all declared allowed execution/residency
   pairs, not one existentially convenient pair. Exact profile matching is
   still conditional on the supplied descriptor.
9. **Dimension and whole-contract proofs.** Apply each frozen direction
   independently and return every result; do not turn an unavailable callee,
   universe, action profile or mapping into an empty/pure row.

## Substitution directions

For new implementation `N` replacing old `O`:

```text
I_old <: I_new
O_new <: O_old
Phi_new ⊆ Phi_old
epsilon_new ⊑ epsilon_old
R_new does not strengthen R_old under an explicit port map
F_new does not relax F_old, including exact EffectFlowPartitions
U_new does not increase U_old for each exact phase/meter
K_new does not weaken K_old and preserves exact RecoveryAction coverage
X feasibility and X security-preservation both prove
every cross-dimension relation proves under the same exact snapshots
```

V1 I/O proves only exact closed port sets and exact pins. Different aliases
are not structurally collapsed. R uses its frozen candidate-set direction,
not ε's narrower-permission direction. Flow compares effective exact allowed
sink sets. K action identity and U action profiles are compared by complete
typed identities. Different or unavailable relation snapshots make the
affected relation Unproven.

No release version order, ComponentDigest equality, implementation digest,
provider name, type short name, operation prefix or same port number proves
substitution.

## Fail-closed result distinctions

| Condition | Result |
| --- | --- |
| Malformed component schema, duplicate exact identity, invalid closed enum | Violation |
| Known missing referenced I/O/R port in a closed contract | Violation |
| Exact operation/resource relation explicitly denies the pair | Violation |
| Flow partition lists an atom absent from ε or widens absent old F sink | Violation |
| Required Recovery action is absent from closed ε or binding is extra | Violation |
| Explicit U ceiling is below known derived recovery usage | Violation |
| Type pin, relation snapshot, action coverage, or meter coordinate unavailable | Unproven |
| ε is Open/Unknown for a proof requiring exhaustive operation coverage | Unproven |
| F contains unknown provenance/influence for a positive flow proof | Unproven |
| Relation snapshots differ across substitution sides without an evolution rule | Unproven |
| Exact static descriptor matched, but independent worker/publisher evidence is due | RequiresEvidence in the evidence report, not a static pass |

The static checker does not establish that declarations describe the binary,
that the relation catalog is authoritative, that a worker is trusted, that
runtime enforcement occurred, or that a policy approved dispatch. Those
remain attestation, admission, runtime enforcement and authorization planes.
