# Phase 4C.1A — Complete Portable Bundle Schema

All definitions are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**. This is a language-neutral closed schema DSL, not Rust
reflection and not a registered AFIDL source.

## DSL and common field contract

`record` declares exactly its listed fields. Every field is required, including
Option fields. `variant` is exactly one named case with its typed payload.
`enum` is a closed name set. `seq<T>` preserves semantic order; `set<T,key>`
encodes as a list strictly increasing by the declared primary key followed
by full canonical element bytes, rejecting identical duplicates. The listed
identity key additionally rejects distinct rows with the same unique identity
at D2; a tie-breaker permits ordering of negative raw rows, not their acceptance.
All collections are finite; table IDs are uint32, logical list lengths and
ordinals are uint64. A local resource limit may be smaller and yields
ResourceLimit, not a positive result.
list<T> is a synonym for seq<T>; set<T,key> adds the declared strict-order
constraint. Generic/inline DSL forms are finite schema macros, not recursive
AFIDL types.

Candidate family name: `agentflow.portable.proof.inputs@1`. This is a proposed
identifier only. Candidate field/case/value numbers are the one-based ordinal
of declarations in each block below and its linked type-definition blocks.
They are a deterministic materialization proposal, not ABI assignments.
4C.1B must register the additive schema before using its nominal digest; none is
computed here. ACE1 value bytes use field/case names, not these numbers.
Imported types and the existing type-schema blob grammar retain their existing
bindings. For AFIDL materialization, generate finite generic instances and
inline auxiliary shapes under reserved prefix PpiAux. Materialize each
occurrence separately; do not deduplicate equal anonymous shapes. The name is
PpiAux + owning top-level named type + P + its traversal path. A path appends
F<number> for a record field, C<number> for a variant payload, L1/M1/O1 for a
list/map/option element, and G1 when expanding a generic macro at that site.
Traverse declarations in their stated order. Stop at an existing named type
reference; that type owns its own definition. set adds validation to list,
not a different container representation. Emit an auxiliary named type only
for an anonymous record/enum/variant or instantiated generic, and retain
primitive/list/map/option as AFIDL expressions. Generic parameters are
substituted before recursively processing that shape.

For example the CompleteCollection at PortableContextContent field 1 is
PpiAuxPortableContextContentPF1G1; its complete payload is list<ModuleContent>.
Inline primitive enum fields similarly receive their unique field path.
The prefix is reserved from user declarations; a collision in the complete
candidate inventory is an allocation error, never a serializer-dependent
rename. Each generated shape assigns its own positive declaration-ordinal
numbers. Named aliases retain their own nominal declaration. This mechanical
proposal is not registration; 4C.1B must verify the full expansion before
allocating ABI entries.

For **every field** in this package:

| Property | Exact default rule, overridden only explicitly below |
| --- | --- |
| Owner | Its containing record; referenced table rows owned by this Bundle |
| Exact type | Listed DSL type or exact imported nominal type |
| Presence | Required record member; Option means explicit none/some only |
| Cardinality | Scalar exactly one; Option zero/one payload; list finite as declared |
| Namespace | Type alias/table scope; identical integers in other namespaces are unrelated |
| Encoding | Schema-directed ACE1 from the encoding document |
| Ordering | Field keys by UTF-8; sequences preserved; sets by the listed comparator |
| Validation | Closed shape, canonical scalars, declared invariants and cross-references |
| Cross-reference | Exact typed key in the indicated local table/scope; no ambient lookup |
| Failure | D0/D1 shape/canonical errors, D2 reference/invariant errors, or later semantic failure; never a default value |

## Primitive and identity aliases

~~~text
SchemaVersion = enum { portable_proof_inputs_v1 }
Text = NFC string
Bytes = canonical unpadded base64url byte sequence
U32 = uint32
U64 = uint64
Ordinal64 = U64                         // zero-based index, checked against source
ContextId = U32                        // positive, Bundle-local
ResolverId = U32                       // positive, Bundle-local
CompilerManifestId = U32               // positive, Bundle-local
AssertionId = U32                      // positive, Bundle-local
StructuralTypeId = U32                 // positive, distinct from IR SemanticTypeId
SpecId = Text satisfying existing AFIDL SpecificationId grammar
TypeName = Text satisfying existing AFIDL TYPE-NAME grammar
LogicalPath = Text                     // relative "/" path; no empty, ".", "..",
                                       // backslash, NUL, drive or absolute segment

DigestRef = existing agentflow.semantic.core@1/DigestRefValue
SchemaRef = existing agentflow.semantic.core@1/TypeSchemaRef
IrPin = DigestRef(domain = semantic-ir, algorithm = sha256)
ContractPin = DigestRef(domain = component-contract, algorithm = sha256)
SchemaPin = DigestRef(domain = type-schema, algorithm = sha256)
LowerHex64 = Text                       // exactly 64 lowercase ASCII hex digits
PlainChecksum = record { algorithm: enum { sha256 }; hex: LowerHex64 }
~~~

Digest hex is exactly 64 lowercase ASCII hex digits. A PlainChecksum is plain
SHA-256(content bytes), never HashV1 and never a signature. A SchemaRef retains
specification_id, type_name and its SchemaPin. No untyped digest string is used.

## Bundle and complete inline assets

~~~text
PortableProofInputBundle = record {
  schema_version: SchemaVersion;
  root_assertion_id: AssertionId;
  schema_assets: set<SchemaAsset, SchemaRefKey>;
  ir_assets: set<IrAsset, TypedContentRefKey>;
  contract_assets: set<ContractAsset, TypedContentRefKey>;
  structural_types: set<StructuralTypeNode, id>;
  proof_contexts: set<ProofContextEntry, id>;
  resolver_dependency_manifests: set<ResolverEntry, id>;
  compiler_semantics_manifests: set<CompilerManifestEntry, id>;
  root_export_assertions: set<AssertionEntry, id>;
}
TypedContentRef = record { digest: DigestRef; type_schema: SchemaRef; }
IrAsset = record { reference: TypedContentRef; canonical_content: Bytes; }
ContractAsset = record { reference: TypedContentRef; canonical_content: Bytes; }
SchemaAsset = record { reference: SchemaRef; canonical_content: Bytes; }
ProofContextEntry = record { id: ContextId; content: PortableContextContent; }
ResolverEntry = record { id: ResolverId; content: PortableResolverManifest; }
CompilerManifestEntry = record {
  id: CompilerManifestId; content: CompilerSemanticsManifest;
}
AssertionEntry = record {
  id: AssertionId;
  ir_asset: TypedContentRef;
  export_contract_asset: TypedContentRef;
  assertion: PortableRootExportAssertion;
}
~~~

Definitions of PortableContextContent, PortableResolverManifest,
CompilerSemanticsManifest and PortableRootExportAssertion are owned by their
linked companion schema documents; those definitions are part of this schema,
not open extension points.

IrAsset's digest domain is semantic-ir and root nominal type is exactly
`agentflow.semantic.ir@1/SemanticIrDocument` for the initial support profile.
ContractAsset uses component-contract and
`agentflow.component.contract@1/ComponentContract`. The complete SchemaRef
must match that context's compiled root schema. Different root schema pins
cannot be chosen by looking only at the content digest.

SchemaAsset contains existing CanonicalTypeSchema ACE1 bytes. Recompute
HashV1(type-schema, bytes) and require its schema/name to equal the reference.
An IR/contract asset recomputes its existing domain digest, then its decoded
typed value must re-encode to the same bytes. The canonical schema blob grammar
is fully specified in the encoding document.

### Root field ownership and references

| Required fact | Exact owner / cross-reference |
| --- | --- |
| Root semantic_ir_content/digest | root AssertionEntry.ir_asset and exactly matching IrAsset; assertion.exact_semantic_ir_digest must equal reference.digest |
| Export contract content/digest | root AssertionEntry.export_contract_asset and matching ContractAsset; asserted export pin must equal reference.digest |
| Root assertion | root_assertion_id resolves to exactly one AssertionEntry |
| Proof context | assertion.proof_context_manifest.context_id resolves to ProofContextEntry |
| Resolver manifest | assertion.proof_context_manifest.resolver_id resolves to ResolverEntry, whose context_id/caller match that assertion |
| Compiler content | assertion.compiler_semantics_id resolves to CompilerManifestEntry |

The table form replaces redundant top-level content copies; it does not
replace content with an incomplete external reference. **v1 permits inline
assets only**. A locator, checksum-only catalog or future parent pointer cannot
supply missing bytes. Parent-owned reference transport is deferred to an
explicit versioned extension; v1 rejects it as UnsupportedContentTransport.

Asset uniqueness is by the full typed reference. Duplicate identical rows are
rejected as noncanonical, not silently deduplicated. Same full reference with
different bytes is a conflict. Two scopes may hold different schema pins for
the same logical type name; inside each universe that logical name must select
one exact body. Cross-scope pin equality is never inferred.

## Finite structural type witnesses

~~~text
WorkflowPrimitive = existing agentflow.semantic.core@1/SemanticPrimitiveType
StructuralTypeNode = record { id: StructuralTypeId; body: StructuralTypeBody; }
StructuralTypeBody = variant {
  primitive: WorkflowPrimitive;
  schema: SchemaRef;
  list: StructuralTypeId;
  map_string: StructuralTypeId;
  option: StructuralTypeId;
}
~~~

The table is acyclic and all references resolve. Expanding a node gives exactly
the current StructuralType. Equal expanded witnesses may use distinct local
IDs, but a referenced witness must equal the freshly derived type. IDs are
input-local identity, not inferred SemanticTypeIds.

Table rows sort by id; IDs are not renumbered while decoding. Different ID
labelings may yield different evidence bytes while expanding to the same
semantic facts. This distinction is intentional and appears in the equality
rules. It does not affect deterministic encoding of the same typed input.

## Graph and ownership constraints

Each ID namespace is separate and positive. Assertions reference other
assertions only through optional target IDs. Their graph must be finite and
acyclic before expansion into current boxed identities. Root and all referenced
target rows retain their own exact context/compiler/resolver scopes.

Every Bundle table row/asset must be reachable through the root assertion,
its target-identity graph, or explicit complete context/resolver inventory.
An inventory may intentionally retain an unused schema/component/target; its
membership is still a proof input and must not be pruned. Unreferenced transport
padding is rejected. No row may refer to another Bundle's local IDs.

## Exact import inventory

These existing nominal shapes are imported field-for-field, with their
SchemaRefs checked against the selected compiled modules:

| Family | Imported shapes used by candidate fields |
| --- | --- |
| semantic.core@1 | DigestRefValue, TypeSchemaRef, PrimitiveConstant, SemanticPrimitiveType and nominal ValueSlot/Expression/ResourceSlot IDs |
| semantic.ir@1 | SemanticIrDocument, ComputationId and RunParameterId |
| effect.signature@1 | EffectAtom, ResourceSelector, ScopeBinding, EffectGuard, EffectQualifier, OccurrenceBounds/Ceiling, EffectOperationRef |
| resource.requirement@1 | ResourceTypeRef |
| information.flow@1 | FlowSourceRef, FlowConstraint, FlowKind, FlowSink, ConfidentialityLabelRef, TaintRef, InstructionAuthority |
| usage.signature@1 | UsageMeterRef and UsageSignature |
| component.contract@1 | ComponentContract, EffectFlowPartition, RecoveryActionIdentity/Kind, RecoveryUsageBinding |
| execution.requirements@1 | ExecutionRequirements, ExecutionCapabilityRef, CapabilityAttribute, TrustDomainRef, RegionRef, RuntimeProfileRef |
| effect.resource.compatibility@1 | EffectResourceCompatibilityCatalog |
| recovery.usage.coverage@1 | RecoveryUsageCoverageCatalog |
| flow.execution.relation@1 | InformationFlowExecutionCatalog |

The prefix `agentflow.` applies to every shortened family above. Imports include
the transitive existing named-type dependencies of each listed shape, with
their full frozen specification/type identities and schema pins. Existing ABI
numbers/variant meanings remain unchanged. No existing import is interpreted
as Any or a host JSON object. All other candidate type names are closed
definitions in this package.

## Submission versus proof identity

Origin claims, compiler build claims and submitted diagnostics use the separate
submission envelope in the origin document. They cannot alter the root
checker input or discharge its facts. The future parent must inline this
complete Bundle; there is no new Bundle/Assertion/Context HashV1 domain or
standalone proof digest.
