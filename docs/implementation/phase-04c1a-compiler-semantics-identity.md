# Phase 4C.1A — Compiler Semantic Content Schema

All candidate definitions are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED
ABI / NOT IMPLEMENTED**. MODEL-B02 defines content/role identity and support;
binary trust and release policy stay with B05/B07.

## Closed schema

~~~text
SemanticRole = enum {
  static_analysis; contract_validation; semantic_ir_validation; dataflow;
  leaf_resolution; control_algebra; root_recovery; root_export;
  plan_lowering; runtime_semantics_profile;
}
ModelMissingReason = enum { not_formalized; future_dependency; }
RoleDeclaration = variant { defined: SemanticRoleDefinition; model_missing: ModelMissingReason; }
SemanticRoleDefinition = record {
  analysis_semantics_version: Text;
  normative_specifications: set<SpecId, utf8>;
  required_schema_pins: set<SchemaRef, SchemaRefKey>;
  depends_on: set<SemanticRole, RoleRank>;
}
SemanticRoleEntry = record { role: SemanticRole; declaration: RoleDeclaration; }
SourceContentIdentity = record {
  specification_id: SpecId; logical_path: LogicalPath;
  content_sha256: PlainChecksum; content: Bytes;
}
RegistryContent = record { content_sha256: PlainChecksum; content: Bytes; }
RegistryContentSet = record {
  specifications: RegistryContent; rules: RegistryContent;
}
ClosedTableContentSet = record {
  hash_domains: RegistryContent; abi_numbers: RegistryContent;
  rule_areas: RegistryContent; afidl_primitives: RegistryContent;
}
RuleContentIdentity = record {
  rule_id: Text; normative_source: LogicalPath;
  source_anchor: Text; short_statement: Text;
}
RuleDependencyEdge = record { dependent: Text; prerequisite: Text; }
IncompleteAttribution = record {
  listed_rules: set<RuleContentIdentity, rule_id>;
}
CompleteAttributionClaim = record {
  direct_rules: set<RuleContentIdentity, rule_id>;
  transitive_rules: set<RuleContentIdentity, rule_id>;
  dependency_edges: set<RuleDependencyEdge, (dependent,prerequisite)>;
}
AttributionProfile = variant {
  full_content_revalidation: IncompleteAttribution;
  rule_complete_attribution: CompleteAttributionClaim;
  portable_certificate: unit;
}
StaticIdentityProjection = record {
  analysis_semantics_version: Text;
  normative_specification_ids: set<SpecId, utf8>;
  applied_rules: set<RuleContentIdentity, rule_id>;
}
CompilerSemanticsManifest = record {
  registries: RegistryContentSet;
  closed_tables: ClosedTableContentSet;
  normative_sources: set<SourceContentIdentity, specification_id>;
  semantic_roles: set<SemanticRoleEntry, RoleRank>;
  attribution: AttributionProfile;
  static_identity_projection: StaticIdentityProjection;
}
~~~

There is exactly one entry for each of the ten SemanticRoles. defined records
are content declarations, **not** caller-established implementation support.
The local supported-semantic table computes Implemented, DefinedButNotImplemented,
ModelMissing or Unsupported. Definition dependencies must be acyclic and
complete; only defined roles can discharge the selected required role set.

Versions are nonempty exact NFC strings, compared literally. SemanticRole rank
is the declaration order above. Source/rule strings obey the existing registry
grammars and exact rows. No rule is trusted merely because its ID parses.

Every defined role's normative_specifications and the static projection's
selected IDs must resolve to the included source/registry rows. Every
required_schema_pin must resolve to an exact SchemaAsset and to the compiled
schema body of that supported source profile; it does not implicitly insert
that schema into a ProofContext. All attribution rules must match the embedded
Rule Registry row in ID/source/anchor/statement. Dependency-edge endpoints must
occur in the union of direct_rules and transitive_rules; this graph has unique
edges and is acyclic. Structural graph validity never proves its completeness.
The source/path membership equals the full Specifications Registry, not just
the sixteen current projection entries. No extra unregistered source or
synthetic rule is admitted into the tuple.

## Complete source and registry binding

All source files named by the embedded Specifications Registry are included
inline, one exact SourceContentIdentity per row. At this HEAD there are 41.
Verify SpecId/path/raw SHA-256/content against its registry row internally at
D3; match the full tuple with the local supported profile at D4. Source bytes,
including line endings, are not Unicode- or
JSON-normalized.

The two full registries are raw byte identities, not a selection of convenient
rows. The four closed tables are also exact inline bytes; their contents must
match the supported domain/ABI/primitive/rule-area models. They do not authorize
caller-created domains or changed meanings. Registered schema bodies for
known machine families must match canonical compilation of those exact AFIDL
sources. Custom context schema families remain explicit data, with separate
origin requirements.

Embedded registries/tables use their existing typed document formats. Their
JSON parser rejects duplicate keys and unknown required semantic structures.
Their plain checksum is a file-content identity; ordinary JSON bytes are never
used as an IR/contract/Bundle HashV1 canonical encoding.

## Exact current CompilerSemanticsIdentity projection

The portable projection yields:

- analysis_semantics_version from StaticIdentityProjection;
- exact specifications/rules registry bytes from registries;
- full SourceContentIdentity values selected by normative_specification_ids,
  with content_sha256.hex projected to the existing scalar checksum;
- exact applied RuleContentIdentity rows.

For the initial current static verifier, version is
`agentflow.root-export-checker@1`. It selects the same 16 normative source IDs:

~~~text
agentflow.component.contract.semantics@1
agentflow.component.relation-catalogs.semantics@1
agentflow.effect.signature.semantics@1
agentflow.execution.requirements.semantics@1
agentflow.failure.row.semantics@1
agentflow.information.flow.semantics@1
agentflow.recovery.contract.semantics@1
agentflow.resource.requirement.semantics@1
agentflow.semantic.advanced-control-contract-algebra.semantics@1
agentflow.semantic.control-contract-algebra.semantics@1
agentflow.semantic.ir.semantics@1
agentflow.semantic.leaf-contract-resolution.semantics@1
agentflow.semantic.scope-compensation-contract-algebra.semantics@1
agentflow.semantic.types@1
agentflow.usage.signature.semantics@1
agentflow.computation.root-export-contract-proof.semantics@1
~~~

Every initial static role conservatively binds that full 16-source set under
the aggregate checker token. This overbinding is deliberate; it makes no
minimal per-rule dependency claim and invents no existing independent
processor-version constants. Exact imported schema pins additionally bind
their full current AFIDL definitions.

For every initial defined role, required_schema_pins is the complete named
schema set obtained by canonical compilation of all registered AFIDL schema
sources in the supported Specifications Registry, sorted by SchemaRefKey.
The dependency sets are fixed: contract_validation and semantic_ir_validation
have none; dataflow depends on semantic_ir_validation; leaf_resolution depends
on contract_validation, semantic_ir_validation and dataflow; control_algebra
depends on leaf_resolution; root_recovery depends on control_algebra and
dataflow; root_export depends on all preceding validation/derivation roles;
static_analysis depends on all seven other static roles. Missing lower/runtime
roles contribute no invented definitions.

The initial applied set remains 22 AF-CONTRACT-0001..0022 and 62
AF-SEMANTIC-0111..0172 rules, exactly matching the whole Rule Registry's
source/anchor/statement rows and projection. The full-content attribution list
must equal static_identity_projection.applied_rules.

## Role support at the starting HEAD

| Role | Actual status | Current identity boundary |
| --- | --- | --- |
| StaticAnalysis | Implemented | Aggregate current static checker and explicit inputs |
| ContractValidation | Implemented | Contract/dimension/relation validators, aggregate content binding |
| SemanticIrValidation | Implemented | Existing semantic IR/type rules and schema pins |
| Dataflow | Implemented | Phase 3C scoped producer/availability relations |
| LeafResolution | Implemented | Exact component/subflow/signal resolution |
| ControlAlgebra | Implemented | Frozen 13-opcode static algebra |
| RootRecovery | Implemented | Source/action/profile/multiplicity closure |
| RootExport | Implemented | Current exact preflight and all nine comparisons |
| PlanLowering | ModelMissing | No formal lowering/PlanCore model or producer |
| RuntimeSemanticsProfile | ModelMissing | Static execution profile catalog is not a runtime semantic model |
| Portable input codec, separate bootstrap | DefinedButNotImplemented | This candidate schema; production support begins only in 4C.1B |

Static revalidation requires the first eight roles. Explicit ModelMissing
Lowering/Runtime entries do not become support and do not invalidate a
well-defined *static-only* result. Omitting either entry is a shape/role-set
error. A Plan/Publish operation requiring them remains NotImplemented/
Unsupported and blocked by B03–B07.

## Attribution profiles and F03

Profile A full-content revalidation is selected. The 84-ID attribution remains
explicitly incomplete; **AUDIT-F03 stays OPEN/P2**.

Profile B's direct/transitive rule graph is a raw complete-attribution claim.
IDs, source rows and edges can be structurally validated, but current code has
no justified rule-closure verifier. The initial support profile marks this
positive requirement UnsupportedAttributionProfile/Unproven; it cannot be
met by supplying the old 84 IDs.

Profile C is a known unsupported selector with unit payload; it introduces
no certificate schema or verifier. Certificate data is not accepted by this
Bundle. Full Rule Dependency Closure becomes mandatory before Positive
Subproof Cache, Portable Certificate Verification, Rule-complete Release Policy
or Proof-step Attribution Requirement is enabled.

## Binary and registration boundaries

Semantic content identity excludes compiler executable identity. The separate
build/origin interfaces are in the origin document. Same source tuple does
not prove two binaries behave identically; qualification/conformance is a
separate premise.

Future schema registration changes the complete registries and therefore
changes this content tuple even when RootExport code is unchanged. 4C.1B must
construct and explicitly support its post-registration profile. It cannot
replace registry/version fields in an old saved proof to “refresh” it.
Historical exact support requires the version-support mechanism below.
