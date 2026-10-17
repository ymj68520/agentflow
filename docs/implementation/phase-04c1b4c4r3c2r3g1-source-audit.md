# G1 — exact native source, existing binding and consumer audit

## Evidence reviewed first

The prior phase final report, binding-gap report, complete parsed
closure-and-binding-audit.json, incremental-instance-ledger.json and final
runtime receipt were reviewed, together with the complete independent
`tools/oracle/raw_relation_schema_binding.py` and normative
`spec/component/relation-catalogs.md`. The two ledgers retain all1750 main and
714 additional root entries; reading their descriptors/counts is not new
runtime nested coverage. Their formal finding is R3-CTX-REL-01/P1/BLOCKING,
not false-Proven. New diagnostic evidence is compared to its original input-
derived Expected, never used to rewrite it.

## Source versus nominal type

| Exact owner/source | Declared field/content | What it establishes |
| --- | --- | --- |
| af-contract; `src/proof.rs`, `RelationCatalogPin.schema` | Rust String | actual family SpecificationId, not a TypeSchemaRef |
| af-contract; `src/universe.rs`, `RelationCatalogSnapshot<T>` | schema, revision, canonical_bytes, catalog, pin | all five stored fields; construction schema absent |
| af-contract; `RelationCatalogWire` | SCHEMA_ID, ROOT_TYPE, canonicalized, to_value | kind-specific family/root and full native value mapping; no schema digest receipt |
| af-semantic; `src/types.rs`, TypeSchemaRef | specification_id:String, type_name:String, schema_digest:DigestRefValue | full named schema authored-against reference; not latest lookup |
| af-semantic; DigestRefValue | algorithm, domain, hex (Strings) | complete digest triple, not naked hex |
| model@5; RelationSnapshotContent | pin.schema:ref af-semantic::TypeSchemaRef; revision:$Text; checksum algorithm/hex:$Text; canonical_content:$Bytes | registered shape requires full pin, but does not establish native source conversion |
| model@5; Raw(RelationSnapshotContent) | corresponding Raw TypeSchemaRef/Text/Bytes refs | raw transformation does not resolve missing provenance |
| model@5; ContractProofContextContents | exact nine fields, three named Opt<RelationSnapshotContent> | full actual native Context projection, not a resolver |
| model@5; SpecificationId | nfc_text nominal SpecificationId | nominal family carrier; does not add type_name/digest |

`descriptor-census.json` records **every** current descriptor, rank and reference
edge, and every incoming reference to the selected binding types. Exact model
owner is `agentflow.computation.proof-report.model@5`; imported Rust names such
as `af-semantic::TypeSchemaRef` are names in that registered owner, not an
interchangeable wire type with the same three field names. The portable type
`agentflow.semantic.core@1/TypeSchemaRef` is separately nominal.

Native constructor signature is `new(revision, catalog:T, modules:&CompiledModuleSet)`.
It does NOT receive a TypeSchemaRef. It validates family syntax, revision
nonempty/NFC, canonicalizes the catalog, queries
`modules.shape_of(T::SCHEMA_ID + "/" + T::ROOT_TYPE)`, encodes the full native
value, computes plain SHA256 and stores only the five fields above. Accessing
a schema during this call is not retention of that schema. No subsequent
getter exposes an original schema. Derived native PartialEq/Eq compares those
stored values and the full T; no external construction event participates.

Family grammar comes from af-idl `src/specification.rs`: lowercase ASCII kebab
namespace segments separated by dots, @positive-u32 major with no leading zero,
no whitespace/Unicode/case folding. Revision is an opaque nonempty NFC token,
not a version-to-schema map. In the three actual Context slots the permitted
families are closed by concrete T. The public generic trait can have other
implementors, but that does not extend these three slots or the proposed closed
projection. A legal arbitrary SpecificationId does not admit an unknown kind.

## Non-recoverability proof, with exact limits

Actual compiled A has supported_pairs:list<SupportedEffectResourcePair>; B
changes this one field in memory to list<string>. The independent diagnostic
compiles both from actual17 AFIDL inputs and computes complete schema ACE1 and
HashV1 type-schema digests. For empty catalogs at family/revision equality:

`S(A,empty,r) == S(B,empty,r)`, while `TypeSchemaDigest(A) != TypeSchemaDigest(B)`.

This equality includes the native pin, catalog and canonical bytes, not just a
display/hash summary. Thus a function of stored native values alone cannot
return the correct construction schema for both histories: equal inputs would
need unequal outputs. Revision and catalog checksum cannot repair this missing
information. The digests have different preimages; there is no hash collision.
It is not a requirement that all distinct schemas produce equal snapshots,
only a counterexample to unique recovery for all legal values.

With each snapshot put into its own otherwise value-equal current A Context,
each Raw call uses that Context's actual same analysis/validation object. Two
separate equal contexts are NOT aliases. Captures compare equal subjects and
both reach true port-zero Violation/zero witness. A third call retains Some
snapshot but omits the relation module (and has empty schema/resource universes)
while retaining the real ComponentContract target: it also reaches the same
port-zero Violation. These are THREE OLD diagnostic executions, not48 new
executed vectors or complete Fact captures. Dynamic evidence covers only
effect_resource; no dynamic credit for recovery_usage or flow_execution.

`minimal-expected.json` is independently regenerated from source inputs by the
UNCHANGED diagnostic Oracle. Its schema_a_ace1/schema_b_ace1 and both complete
digest triples provide exact machine evidence, not construction data recovered
from Rust Actual. Direct native pin projection into frozen model shape fails
UnsupportedModel because schema:String is not its required record.

## Existing binding audit (Outcome B rejected)

`candidate-matrix.json` evaluates all nine requested candidate families plus
the portable header; each row has Registered, SemanticallyApplicable,
ExactSource, TypeMapping, ConstructionHistory, CurrentContextDependence and
Lossless. Lossless Yes for a native family means only that **family source**
is retained; it does not claim a lossless full pin. Portable row completeness
is restricted to a caller-supplied exact header, not conversion from native.

- AF-SEMANTIC-0198 / proof-obligation-semantics.md#g01-exact-scope preserves
  complete compiler/context/resolver/root/assertion contents and role partition.
  It does not say a current compiler was the unrecorded construction compiler.
- AF-SEMANTIC-0200 requires full consistent Source/Target/Coordinate/origin;
  AF-SEMANTIC-0206/0208/0209/0210/0211/0212 require real prerequisite/evidence/
  primary/merge/disposition/normalization relations, not provenance invention.
- AF-SEMANTIC-0261 preserves true source documents; 0262 requires same actual
  nine-member Context object; 0263 separates stages; 0264 keeps ports before
  SchemaResolve; 0265 identifies real occurrence/target; 0266 keeps actual
  zero/duplicate witnesses; 0267/0268 prohibit manufactured premise/evidence.
  None specifies family→TypeSchemaRef or construction/current interpretation.
- relation-catalogs.md#component-relation-catalogs-snapshots requires exact
  schema/revision/content/checksum and changed-content revalidation; different
  catalog pins are Unproven absent explicit evolution. It does not register
  missing native root/digest source or guarantee module presence in every Context.
- AF-CONTRACT-0013/0016/0018 consume exact typed catalog semantics; 0019 concerns
  exact relation pins. They do not supply an unretained construction schema.
- Actual AFIDL relation roots provide definitions; actual current compiled
  contents can provide current definitions only. Complete compiler identity
  retains normative source bytes, not a time-stamped snapshot constructor input.
- Existing portable `RelationContentHeader` under portable.proof.inputs@1
  explicitly supplies `core.TypeSchemaRef`. `build_relation` checks family/root,
  actual current compiled digest, strict decode, native construction, byte and
  checksum re-encoding. That is evidence for **a supplied current reconstruction**,
  not a registered reverse mapping from already-created native snapshots.

Metadata shape admission does not prove any of these missing semantic steps.
No existing exact native-source binding with complete missing-schema policy was
found. Outcome B would need such a rule and independent execution; we do not
claim it. A frozen current descriptor cannot be weakened under its old identity.

## Actual consumers: no family-only precision downgrade

The complete lexical source inventory (including tests and normative hits) is
machine-retained with raw hashes in source-inventory.json. The direct consumers
and transitive identity-bearing carriers divide as follows:

| Consumer / source | Uses today | Design consequence |
| --- | --- | --- |
| RelationCatalogSnapshot::new (af-contract/universe.rs) | actual T canonicalization and actual module shape; immutable bytes/pin | retain exact source; C must independently repeat applicable checks against current inputs |
| prove_canonical_contract / check_effect_resource / check_recovery_usage / check_flow_execution (validate.rs) | full catalog rows and exact resource/meter/label/action semantics; missing evidence Unproven | family equality is never sufficient; retain full typed catalog plus schema binding for complete formal projection |
| RelationSnapshots::pins / ProofReport.relation_catalogs | family/revision/checksum equality/reporting; different pins not automatically V | preserve all old native values and behavior, including empty/no-event reports |
| ProofContextReference and resolved/control summaries (summary.rs) | actual pointer identity plus complete relation pins | C content cannot replace native object association; no content⇒alias inference |
| ProofContextManifest::capture and RootExport exact context preflight (root_export.rs) | complete compiled modules/universes, pin+bytes and resolver associations | family alone cannot replace complete context; exact schema-using replay must retain full precision |
| Raw subject/context/basis/origin; standalone substitution and native guard subjects | same actual nine-member Context; complete source transitively used in identity/evidence | draft source exactness and C binding are separate; no output can fake an incomplete source |
| model@5 RelationSnapshotContent, Context, Raw counterpart, Fact reference closure | explicitly full TypeSchemaRef requirement | successor design required; old types/ranks/refs/Expected not edited |
| portable validation/content.rs checksums; reconstruction.rs build_relation | exact caller header, compiled root digest, ACE1 decode/round-trip and native catalog construction | C Valid must retain full schema ref + module closure; do not replace portable full pin with family |
| portable root revalidation/output serialization | full reconstructed static context for analysis; some JSON presentations print pins/counts | presentation summaries do not define complete provenance; do not count them as full-field closure |
| support.rs/schema.rs/pipeline.rs D0–D4/current-only routing | exact supported compiler/model/schema sets; unsupported input denied | draft observations confer no support; future routing requires separate versioned registration/tests |

No current native semantic consumer requires retaining a historical constructor
receipt as such. Several consumers require full *current* schema precision or
complete source values. Therefore neither A alone (precision loss) nor B alone
(unavailable history and not current validity) meets the whole contract. C alone
would omit the actual unvalidated native source and could pretend observation
is history. A+C is selected for these semantic reasons, not implementation ease.
