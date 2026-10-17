# G1 normative amendment DRAFT — immutable source plus current validation

**Unregistered design only.** Names below are explanatory draft names, not new
Rust types, supported descriptor variants, RuleIds or reserved Edition numbers.
No frozen definition is replaced in place. A separately authorized successor
must register the exact closed grammar/constraints before implementation.

## Decision and three distinct identities

The unique recommendation is **A+C**: A for the native source layer, C for an
independent, current-context validation layer. B is documented but not selected.

1. **SchemaFamilyIdentity** is the native SCHEMA_ID. It identifies a nominal
   family, not a field-level schema definition and not a constructor event.
2. **ConstructionSchemaIdentity**, if ever supported, is the full TypeSchemaRef
   actually used at construction with an authentic association to that call.
   Current five-field snapshots do not retain it. This design marks the claim
   unavailable and never fills it. A future B receipt needs separate authority.
3. **CurrentContextSchemaIdentity** is the exact current module root definition
   against which this same source snapshot was actually revalidated. It may
   differ from any original construction definition, without changing native
   equality. It is an output observation, not an input source pin.

These meanings MUST NOT occupy one untagged `pin.schema`. In the successor draft,
the source is explicitly native-family; the derived validation result has its
own schema_ref. A family wrapper and TypeSchemaRef MUST remain nominally distinct.
The current model@5 field stays untouched and unsupported for direct native
family projection. No interpretive union or Rust-only variant is added now.

## Closed source layer (A)

The proposed source node, owned by an as-yet-unallocated successor model, is:

```text
NativeRelationSnapshotSource = {
  kind: closed EffectResource | RecoveryUsage | FlowExecution,
  pin: { family: RelationFamilySpecificationId,
         revision: NonemptyNfcText,
         canonical_content_checksum: {algorithm: Sha256, hex: LowerHex64} },
  canonical_content: Bytes,
  catalog: closed per-kind FULL native catalog value
}
```

This is a complete **source observation**, not a validated-schema assertion.
`RelationFamilySpecificationId` wraps the strict existing SpecificationId grammar
and is constrained by kind to the three exact families in the slot matrix.
Catalog is a closed tagged per-kind payload, not an opaque reflection blob;
its fields/variants are the exact complete T declared in af-contract/universe.rs
and its imported native semantic types. An exhaustive successor descriptor and
reference closure must be generated/reviewed at registration, not inferred by
structural similarity at capture. Optional slots remain three named options,
not an unordered sequence that loses slot identity.

The source projection MUST retain and check agreement of schema(), pin.schema
and T::SCHEMA_ID; revision() and pin.revision; full bytes and plain checksum.
No canonical reordering of the **observed** catalog is silently substituted.
Full native T is retained; subsequent canonicalization must compare equal to
that source and reproduce its exact bytes. The stored duplicate fields are
validated for agreement, not dropped without a justified injective projection.
For valid public native snapshots they agree by construction; malformed future
wire inputs MUST fail source admission with their actual conflicting values.

### Exact catalog source grammar (no opaque payload)

`native-and-rule-inventory.json` preserves the exact17 native catalog/row/policy
declarations, actual source lines/hash and exact applicable Registry rule rows.
Those declarations define the draft's per-kind payload contents, not an arbitrary
JSON object. The following is the complete local member/variant roster; Seq
preserves the observed native Vec order and values before C checks canonicality:

```text
EffectResourceCompatibilityCatalog {complete_operations:Seq<EffectOperationRef>,
                                   supported_pairs:Seq<SupportedEffectResourcePair>}
SupportedEffectResourcePair {operation:EffectOperationRef, resource_type:ResourceTypeRef,
                            selector_keys:Seq<SelectorKeyRule>,
                            allowed_qualifier_profiles:Seq<EffectQualifierProfile>}
SelectorKeyRule {selector_key:String,resource_attribute_key:String,required:Bool,
                 values:ScopeValueRule}
ScopeValueRule = FiniteExactValues(Seq<String>) | OpaqueNfcExactString
EffectQualifierProfile {qualifiers:Seq<EffectQualifier>}
RecoveryUsageCoverageCatalog {rules:Seq<RecoveryUsageCoverageRule>}
RecoveryUsageCoverageRule {action:RecoveryActionIdentity,complete_meters:Seq<UsageMeterRef>}
InformationFlowExecutionCatalog {closed_regions:Seq<RegionRef>,
  closed_trust_domains:Seq<TrustDomainRef>,confidentiality_rules:Seq<ConfidentialityExecutionRule>,
  taint_rules:Seq<TaintExecutionRule>,authority_rules:Seq<AuthorityExecutionRule>,
  sink_rules:Seq<FlowExecutionRule>}
RegionPair {execution_region:RegionRef,residency_region:RegionRef}
RegionPairPolicy = FiniteExactPairs(Seq<RegionPair>) | AllPairsInClosedRegionSnapshot
TrustDomainPolicy = ExactAllowlist(Seq<TrustDomainRef>)
                  | AllDomainsInClosedTrustSnapshot | NoAdditionalTrustRequirement
SandboxFloor = NoAdditionalMinimum | Minimum(SandboxLevel)
ConfidentialityExecutionRule {label:ConfidentialityLabelRef,region_pairs:RegionPairPolicy}
TaintExecutionRule {taint:TaintRef,trust_domains:TrustDomainPolicy,sandbox:SandboxFloor}
AuthorityExecutionRule {authority:InstructionAuthority,trust_domains:TrustDomainPolicy,
                        sandbox:SandboxFloor}
FlowExecutionSink = WorkflowOutput(U32) | ExactEffect(EffectAtom)
FlowExecutionRule {kind:FlowKind,sink:FlowExecutionSink,
                   trust_domains:TrustDomainPolicy,sandbox:SandboxFloor}
```

Local declaration owner is af-contract. Imported member owners remain af-effect
(EffectOperationRef, EffectQualifier, EffectAtom), af-resource (ResourceTypeRef),
af-usage (UsageMeterRef), af-flow (FlowKind, ConfidentialityLabelRef, TaintRef,
InstructionAuthority), af-execution (RegionRef, TrustDomainRef, SandboxLevel),
and af-contract (RecoveryActionIdentity). Use their exact complete native
declarations/import mappings, including full semantic TypeSchemaRef/digest fields;
do not reduce effect atoms to operation names or actions to ordinals. Current
model@5 already declares these imported member types; any new local catalog
descriptor grammar and its complete transitive nominal closure belongs only to
the separately reviewed successor. The draft does not reuse an incompatible
AFIDL root or infer imports by matching field names.

Construction history is **not a source field with an invented None**. The source
contract explicitly has no construction claim. If a consumer asks for one,
answer Unavailable/unsupported for this source kind, not an empty schema pin.
An independently authentic construction receipt would be a different source
contract, not a label attached by a caller to an old snapshot.

## Current validation layer (C): inputs, algorithm and closed results

Inputs are the exact source node, its exact slot, the actual complete nine-member
Context, and the actual complete compiler/validation algorithm identity. For
revision comparison only, an **actually supplied** expected catalog pin is a
separate optional input. Schema definitions themselves do not contain a catalog
revision and therefore do not supply such an expected token.

The closed result is a derived observation, not a capability:

```text
CurrentRelationValidation = {
  source_ref, context_input_ref, slot, validation_algorithm_identity,
  expected_catalog_pin: explicit optional actual input,
  outcome:
    NotReached {failed_prior_stage, actual_cause}
  | Missing {queried_family, queried_root}
  | Ambiguous {queried_identity, complete_conflicting_definitions}
  | FamilyMismatch {source_family, slot_family, actual_definition_if_found}
  | RevisionMismatch {actual_pin, actual_expected_pin}
  | Invalid {resolved_schema_ref, step, typed_failure, offending_input_refs}
  | Valid {resolved_schema_ref, complete_validation_witness}
}
```

`Found` is a distinct lookup-stage record containing the actual complete root
schema and resolution closure. It MUST NOT be represented as Valid. If a stage
after Found fails, its final result is Invalid with the actual resolved pin and
typed failure; stages never reached carry no fabricated successful evidence.
Association failures before an exact current schema is found do not have an
Invalid schema result or a guessed pin. Source-integrity refusal is separately
typed as SubjectAdmission failure; NotReached records that C did not execute.

Proposed deterministic steps:

1. Check complete actual source/slot/kind association and nine-member Context
   association. Raw analysis/canonical_validation MUST be the same actual native
   object. Refuse mismatches before inventing schema observations. No Resolver
   is added; no context digest/address is serialized as proof of aliasing.
2. Check family grammar, exact closed slot family, nonempty NFC revision,
   duplicated native-field agreement and plain SHA256 of the complete bytes.
   Refuse corrupt source integrity with structured source values.
3. If an actual expected pin is provided, compare its exact family/revision/
   checksum under that consumer's policy. RevisionMismatch is an association/
   comparison failure (no invented schema-revision field); different pins keep
   the existing Unproven/evolution policy, not automatic semantic Violation.
   With no supplied expected pin, no revision-equality premise is manufactured.
4. Query **only** actual_context.modules by exact source family plus the closed
   slot root name. The name table selects the query; it is not evidence that a
   definition exists. Return Missing if module or named root is absent. Do not
   search schema_universe, resources.schemas, global Registry, disk or latest
   as fallback. Those other members retain their own distinct native meanings.
5. Establish unique complete canonical root and its actual imported definition
   closure. Native CompiledModuleSet is a private unique-key, acyclic resolved
   set; ambiguity cannot be constructed through its valid API. Ambiguous is a
   fail-closed boundary for malformed portable/imported candidates, never a
   fabricated reachable native branch. Missing imported definitions leave
   closure incomplete; cycles/duplicate definitions refuse context admission.
6. Record Found. Require actual root.schema == source family == slot family and
   actual root.name == closed slot root. Compute exact canonical TypeSchema ACE1
   and `SHA256(b"AF\0type-schema\0" || ACE1(schema))`; retain algorithm=sha256,
   domain=type-schema and lowercase full hex. Digest comes from this actual
   definition, never family text, catalog checksum, a registry default or reason.
   A root digest may use named references rather than inline imported bodies;
   full current module closure MUST also be retained, not replaced by that digest.
7. Independently validate actual complete native catalog semantics (including
   duplicate/malformed exact rows and closed snapshots), verify canonicalized T
   equals observed T, then encode `T.to_value()` against this current shape.
   Strictly decode the original canonical bytes against the same shape, recover
   the kind-specific catalog with exact closed fields/variants, and require
   complete equality with observed T and exact canonical round-trip bytes.
   This excludes merely checking that an empty checksum matches. Apply all
   registered per-kind canonicalization rules, not just shape acceptance.
8. Valid exists only when every applicable check completes. Its witness retains
   full source/definition/context associations, algorithm identity, exact
   schema ref, actual validation steps and equal bytes/catalog values. Invalid
   retains the resolved schema, exact failing step and structured offending
   values/paths; no diagnostic prose is parsed. Later checks are NotReached.

Valid is a **pointwise claim about this exact catalog value under this exact
current schema**, not a theorem that every possible native T value shares that
wire schema. Thus empty A/B catalogs may both validate while carrying distinct
current schema refs. A schema-identity/ABI consumer must additionally compare
the complete schema ref/closure to its actual required supported schema; it
cannot infer universal structural compatibility from one empty round-trip.
This preserves field-level precision rather than using emptiness or Family
equality to erase it. Nonempty Pair-valued data under B's list<string> fails.

This algorithm is specified, **not implemented or executed in this phase**.
The future independent Oracle must compute it independently. The existing
portable reconstruction code supplies a useful consumer precedent, not already
executed proof for this proposed judgment.

Missing is incomplete evidence/association, NOT an automatically evaluated
Schema Validation Violation. Invalid means a real completed validation step
failed; it is NOT the Raw positive_unique_io_ports verdict and does not allocate
a new G01 predicate or authorize a new Producer. Successor fact/predicate
registration must separately specify any validation-judgment exposure.

## Admission and consumer policy (no precision weakening)

Source capture may retain a complete A observation even when C is Missing.
However, **complete successor G01 Subject admission requires C Valid for every
actual Some slot**; otherwise return incomplete/refused admission with the actual
failure, never a complete Fact with a fabricated TypeSchemaRef. This conservative
policy preserves full-schema precision for typed Subject consumers. An exact
schema-dependent replay/portable consumer also requires C Valid plus its own
exact supported-version/schema policy. A Valid current B observation is not
permission to accept B in a route supporting only A.

None MUST remain the actual None. It does not become Some/Valid and does not
prove a required relation predicate. All None can satisfy the optional source
shape but never establishes conditional semantic catalog coverage. A single
invalid/missing Some blocks complete Subject admission even if another slot
has Valid. No observed Some is replaced by None to pass this gate.

For Raw, the local branch remains strictly PositiveFresh→Proven, Zero→Violation,
Duplicate→Violation, with actual input-before-output stable numeric sorting and
true original/sorted coordinates before SchemaResolve. C does not execute as a
new prerequisite in the old port checker and cannot rewrite its verdict. Its
gate is on complete formal Subject/Fact materialization, not on whether the
existing port-zero event actually occurred. The diagnostic zero with missing
current relation module is thus a real Violation event and an inadmissible full
typed Subject under the proposed policy. Both statements are consistent.

## Three exact optional slots

Extracted from model@5 ContractProofContextContents, not guessed:

| Slot | Actual native T | Exact family / root in actual modules |
| --- | --- | --- |
| effect_resource | EffectResourceCompatibilityCatalog | agentflow.effect.resource.compatibility@1 / EffectResourceCompatibilityCatalog |
| recovery_usage | RecoveryUsageCoverageCatalog | agentflow.recovery.usage.coverage@1 / RecoveryUsageCoverageCatalog |
| flow_execution | InformationFlowExecutionCatalog | agentflow.flow.execution.relation@1 / InformationFlowExecutionCatalog |

Each registered slot is `option<ref RelationSnapshotContent>` with minimum0;
each actual native slot is `Option<RelationCatalogSnapshot<T>>`. All use the same
constructor contract and preserve Family in A, not construction schema. All
query exact actual modules as specified in C and require independent canonical
and per-kind semantic validation. Missing current root in any Some yields
Missing/incomplete admission. Their native payload rules differ: effect/resource
complete operations/pairs/selectors/qualifiers, recovery/action exhaustive meter
rules, flow closed regions/domains and label/taint/authority/sink policies. Those
rules are not collapsed into one generic empty-catalog algorithm.

Machine three-slot-matrix.json includes source shape, descriptor/owner, kind,
family/root, construction/current lookup, missing policy and typed projection.
Only the inherited effect_resource counterexample is dynamically exercised.

## Full source-to-output propagation

field-source-matrix.json specifies actual evidence, stage, nominal owner, exact
type, complete digest, association and validation status for every proposed pin
member and its parent Raw carriers. Its source names are explanatory until the
successor is registered. The normative propagation requirements are:

- Source.raw_validation carries the exact occurrence-bound Raw subject,
  package/owner/document/declaration/candidate, compiler and full nine-member
  Context, with A in each actual slot. C is referenced separately as an output
  admission witness; it never mutates that source Context or adds a resolver.
- Target retains the exact compiled **ComponentContract** schema, actual rule
  AF-CONTRACT-0004 and true port constraint. A relation TypeSchemaRef is not a
  replacement target. C's target is its own actual relation root and input slot.
- Coordinate retains direction, original ordinal, stable sorted ordinal and
  actual U32 port occurrence. Relation slot coordinates are closed named fields,
  not ordinal aliases. Full Source/Target/Coordinate share the same subject.
- RawValidationOrigin uses the actual source occurrence and declared paths;
  schema validation output does not become a constructor/document origin.
  Typed Source identity includes full source/context values, while C identity
  additionally includes actual validation inputs/algorithm/outcome. Equal
  source values are not independent-document occurrences or pointer aliases.
- RawPortCheckBasis preserves actual declared port and predecessor/prefix facts
  before diagnostic reduction. ZeroWitness and DuplicateWitness keep complete
  subject association and both real duplicate members. No schema-source failure
  is repackaged as a zero/duplicate contradiction or reason-derived witness.
- References include the complete registered subject/member evidence set;
  duplicate includes both actual original members. Primary is the typed minimum
  of that set under the registered successor ordering, not first printed record.
  Prerequisites and cause_fact_keys describe only actual predicate queries:
  there is no fabricated earlier schema Proven fact before port zero. C's
  admission witness is not silently inserted as a port logical premise.
- Observations and consumers retain all real sites; same-origin merge requires
  complete payload consistency. Conflicting C output for identical complete
  inputs/algorithm refuses collection; different current contexts legitimately
  give different observations. Complete Fact cross-field equality, ordering and
  reference closure still require future independent execution, not this design.

## Canonical input DAG versus output witness graph

The graphs are different. No output witness belongs to canonical input identity:

```text
Input:  compiler definitions     source package/document
                  |                       |
        full nine-member Context ---- Raw subject
            |          |
        modules      A source snapshot (full typed catalog + bytes)
            \          /
Output:      C validation observation
                         |
               complete Subject admission receipt
                         |
                 Raw Fact / Basis / Witness / EvidenceRefs
```

Arrows indicate dependencies, not mutable links written into input nodes.
Canonical input nodes use exact nominal descriptors and full typed contents;
typed references are to prior lower-ranked input nodes only. C points to its
immutable source/context/schema inputs. Admission and Fact outputs may reference
C and input nodes, never the reverse. Input cannot point to its own validation,
Fact, contradiction or primary; C cannot depend on admission/Fact derived from
it. The logical prerequisite/consumer graph is also output metadata, not an
input Schema dependency. Typed projection and DAG cycle/rank/order validation
must be registered before T4; no normalizer or canonical encoder is implemented
here. The draft does not claim G02/T5 cross-process Context association solved.

## Alternatives not selected

A alone preserves actual native information and old equality, but cannot meet
field-level schema consumers; pretending it meets model@5 would be a downgrade.
C alone can bind a current schema but must not overwrite/omit actual native
source or call it history. Their composition has independent stage/identity/
consumer roles and supplies both requirements.

B would extend future constructors/storage with an actual construction schema
ref and its association (potentially full definition/closure evidence). It must
change native equality deliberately under a new contract: equal old A/B values
could have unequal new construction receipts. It need not change catalog ACE1
or plain checksum if receipt is separate, but changing serialized snapshot
identity, ABI, report/context grammar and any hashed carrier requires explicit
new edition analysis. A root ref alone is not the full imported closure. No
legacy snapshot receives a guessed receipt from current modules, revision or
bytes. Missing authentic history is permanently unmigratable for that claim,
not a mutable resolver retry. B does not prove current validity and would still
need C where consumers require it. Existing semantics do not require historical
retention, so its invasive native change is neither necessary nor selected.
