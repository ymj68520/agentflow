# Phase 4C.1A — Complete Proof Context Data Model

All candidate definitions are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED
ABI / NOT IMPLEMENTED**. This document covers every actual field of
StaticContractProofContext, including independent nested universes.

## Complete versus unavailable

~~~text
UnavailableReason = enum { not_supplied; content_not_retained; unsupported_projection; }
CompleteCollection<T> = variant { complete: list<T>; unavailable: UnavailableReason; }
PortableContextContent = record {
  compiled_modules: CompleteCollection<ModuleContent>;
  type_schemas: CompleteCollection<SchemaRef>;
  effect_catalogs: CompleteCollection<EffectCatalogContent>;
  resource_types: ResourceUniverseContent;
  flow_catalogs: FlowCatalogContent;
  usage_meters: CompleteCollection<UsageMeterContent>;
  recovery_operations: RecoveryUniverseContent;
  execution_universes: ExecutionUniverseContent;
  relation_snapshots: RelationSnapshotSlots;
}
ModuleContent = record {
  specification_id: SpecId; named_types: set<SchemaRef, local_type_name>;
}
~~~

Each concrete complete list follows the member order/uniqueness rules below.
Complete is a raw declaration of the supplied closed set; it is not a claim of
authorized catalog origin. Positive reconstruction requires every universe
collection Complete. Complete([]) is known empty; Unavailable is not empty.
Missing the record field is a Decoder error.

Compiled modules and context TypeSchemaUniverse are distinct memberships.
Named schema assets retain canonical bodies/digests, including field/case/value
numbers and full named-reference identities. Within each scope a logical type
identity has exactly one selected SchemaRef. Module import graphs may cycle
when the **named type graph** is acyclic; schema recursion remains forbidden.

## Effect / Resource / Flow data

~~~text
EffectCatalogContent = record {
  specification_id: SpecId; operations: set<Text, utf8>;
}
ResourceTypeContent = record {
  schema: SchemaRef; recognized_attributes: set<Text, utf8>;
}
ResourceUniverseContent = record {
  schemas: CompleteCollection<SchemaRef>;
  descriptors: CompleteCollection<ResourceTypeContent>;
}
FlowEffectSinkContent = record {
  operation: EffectOperationRef; decision_capable: bool;
}
FlowCatalogContent = record {
  confidentiality_labels: CompleteCollection<ConfidentialityLabelRef>;
  taints: CompleteCollection<TaintRef>;
  effect_sinks: CompleteCollection<FlowEffectSinkContent>;
}
~~~

Resource schemas reproduce `ResourceTypeUniverse.schemas` separately from
`context.schema_universe`. Descriptor schema pin/name must resolve in that
resource scope. A legal schema without a descriptor is not a recognized
resource type. Descriptors have exact closed NFC attribute vocabularies.

Operation grammar is the existing dotted lowercase [a-z0-9-] grammar, bounded
by the existing 256-byte rule. Flow sink operation membership and its
decision_capable classification are independent declared data; that bool
cannot authenticate the catalog or grant authority. Label/taint nominal
memberships remain separate and have no invented hierarchy.

## Usage and Recovery data

~~~text
UsageUnitRef = record { catalog: SpecId; name: Text; }
UsageAggregation = enum { additive; unsupported; }
UsageMeterContent = record {
  meter: UsageMeterRef; unit: UsageUnitRef; aggregation: UsageAggregation;
}
ReconciliationOperationRef = record { catalog: SpecId; name: Text; }
RecoveryUniverseContent = record {
  effect_catalogs: CompleteCollection<EffectCatalogContent>;
  reconciliation_operations: CompleteCollection<ReconciliationOperationRef>;
}
~~~

Recovery's effect catalogs are the actual separate
`RecoveryOperationUniverse.effects`, not assumed equal to context effects.
Reconciliation membership is an exact catalog/name set. Unsupported Usage
aggregation is faithfully represented; resolving a used unsupported meter
fails as in the existing validator. It never becomes additive or zero.
Reconciliation names are nonempty/no-whitespace NFC text, preserving the
current recovery constructor's broader name vocabulary; the Effect catalog's
dotted lowercase grammar is not imposed on this distinct membership.

## Execution data

~~~text
ExecutionCapabilityContent = record {
  capability: ExecutionCapabilityRef; exact_version: Text;
  supported_versions: set<Text, utf8>;
  attributes: set<CapabilityAttribute, (key,value)>;
}
RegionCompatibility = record { from: RegionRef; to: RegionRef; }
ExecutionRuntimeProfileContent = record {
  profile: RuntimeProfileRef; family: Text; features: set<Text, utf8>;
}
ExecutionUniverseContent = record {
  capabilities: CompleteCollection<ExecutionCapabilityContent>;
  trust_domains: CompleteCollection<TrustDomainRef>;
  regions: CompleteCollection<RegionRef>;
  region_compatibility: CompleteCollection<RegionCompatibility>;
  runtime_profiles: CompleteCollection<ExecutionRuntimeProfileContent>;
}
~~~

Capability exact_version must be nonempty and a member of supported_versions.
It is not derived as the smallest version: the current constructor stores the
first registered version. Reconstruction registers it first, then all other
supported versions in canonical order using the same attribute set.
Attributes are exact (key,value) set entries; duplicate pairs fail. A key with
distinct values is not silently collapsed into a string map.

Region compatibility is directional. Both endpoints must be in regions;
Decoder must not let constructor auto-registration hide a missing member.
Trust and runtime profile keys resolve exactly. Runtime profile family/features
are static execution-catalog descriptors, **not** the missing full runtime
semantic profile of MODEL-B02/B04.

## Relation snapshots

~~~text
RelationSnapshotSlots = record {
  effect_resource: option<EffectResourceRelationContent>;
  recovery_usage: option<RecoveryUsageRelationContent>;
  flow_execution: option<FlowExecutionRelationContent>;
}
RelationContentHeader = record {
  schema: SchemaRef; revision: Text; canonical_content_checksum: PlainChecksum;
}
EffectResourceRelationContent = record {
  header: RelationContentHeader; canonical_content: Bytes;
}
RecoveryUsageRelationContent = record {
  header: RelationContentHeader; canonical_content: Bytes;
}
FlowExecutionRelationContent = record {
  header: RelationContentHeader; canonical_content: Bytes;
}
~~~

Each slot uses its exact existing root nominal type:
EffectResourceCompatibilityCatalog, RecoveryUsageCoverageCatalog or
InformationFlowExecutionCatalog in the imported families. Decode its bytes
against that root, reconstruct the typed catalog, invoke
RelationCatalogSnapshot::new with the exact revision and compare regenerated
bytes/plain checksum/header. Unknown root/schema or a revision-only claim is
not a snapshot.

The existing closed relation shapes carry all data: complete operation sets,
selector-key/value rules, qualifier profiles, exact resource pins; recovery
action/meter sets; closed regions/trust domains and confidentiality/taint/
authority/sink policies. FiniteExactValues, OpaqueNfcExactString,
FiniteExactPairs, AllPairsInClosedRegionSnapshot, ExactAllowlist,
AllDomainsInClosedTrustSnapshot, NoAdditionalTrustRequirement and SandboxFloor
cases retain their existing meanings. The algorithm interpreting those cases
is a supported semantic role, not serialized code.

None means no relation supplied. Some(empty typed catalog) is a supplied
content-bound empty relation and differs in identity. A required relation
missing at validation yields Unproven; no default empty snapshot is created.

## Member keys and closure

| Collection | Unique key and strict order |
| --- | --- |
| modules | SpecificationId UTF-8 |
| schema membership | (specification_id,type_name), full pin checked |
| effect catalogs | SpecificationId; operation names UTF-8 |
| resource descriptors | (specification_id,type_name), full SchemaPin |
| confidentiality / taints | (catalog,name), separate collections |
| effect sinks | operation (catalog,name) |
| meters | meter (catalog,name) |
| reconciliation operations | (catalog,name) |
| execution capabilities | (catalog,name) |
| trust domains / regions / runtime profiles | (catalog,name) |
| region compatibility | (from catalog/name,to catalog/name) |
| version / feature / attribute sets | exact UTF-8 element or (key,value) |

No referenced type/member may be fetched from a default registry. The producer
must expose all members of the original in-memory scopes, including unused
members. Dropping an unused member changes the context projection even when
the final verdict coincides. An attacker who declares a newly weakened but
internally complete context has changed input identity; reconstruction alone
cannot approve that context's origin.

## Portability and implementation bridges

All present universes are data. Built-in membership, compatibility, guard,
scope, arithmetic and profile matching are rebuilt by exact supported
algorithms and their content versions. No arbitrary function pointer exists
in the reviewed context fields. An unsupported algorithm version cannot be
restored as a default function.

Read-only enumeration APIs are absent for several private maps/sets, including
resource schemas/descriptors, label/taint/sink sets, Usage descriptors,
reconciliation, trust/region/profile sets. A future exporter needs lossless
snapshot accessors. Those are 4C.1B implementation work; this phase changes
no Rust. The full field inventory and constructor recipes remove model
ambiguity without claiming those bridges exist.

The v1 projection domain requires canonical NFC data members. If an
in-memory set admitted a non-NFC member through a permissive constructor or
public mutation, an exporter returns NoncanonicalContextMember rather than
normalizing away its identity. Malformed/inconsistent host state is not a
portable positive input. This restriction does not change existing validators.
