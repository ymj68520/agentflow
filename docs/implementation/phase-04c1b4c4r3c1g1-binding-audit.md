# C1G1 — Registered binding audit / actual source

Status: design adjudication only. Baseline/code-gate HEAD
`85b67afee2a8489df2c0ea1f8d593357fd93f675`. No registration or implementation.
The machine selection in `registered-selection.json` copies current descriptors,
not proposed replacements. All original 2012 nonignored files remain frozen.

## Actual replay and provenance limit

The frozen `c1_direct_raw_native_validation_subject_binding_gap` test constructs
`FixtureContext::new()`, installs `CaptureInput` as Int64 with the independently
pinned schema, borrows one `c = f.view()`, and deserializes the independent vector's
`rust_input` into a `ComponentContract`. Input port ordinal0 has `port_no=0`.
The complete vector is retained in `replay-input.json`; exact freshly regenerated
wire bytes (hex/SHA256) are retained separately in `replay-wire.json`.
These are **fresh replay transport bytes**, not a claim that historical physical
input bytes were recorded. The direct native API receives a typed reference, not
bytes or a source document. The original C1 test did not save the subprocess's
stdout bytes. Lossless historical physical-byte provenance therefore is unavailable.
JSON reserialization MUST NOT be represented as the original physical input.

The existing test separately calls canonicalize, analyze, check, and observe. These
are distinct invocations using the same context object, not one traced call chain.
Within `analyze_contract`, the actual chain is:

```text
analyze_contract(raw, &c)                         validate.rs:65
  canonicalize_contract(raw, &c)                  canonical.rs:49
    validate_component_identity                  validate.rs:97
    canonical_ports(raw.inputs, schemas, input)  canonical.rs:238
      clone and sort by port number
      number==0 || number==previous
      ContractError(AF-CONTRACT-0004,
                    invalid_or_duplicate_port,
                    detail).at("0")              canonical.rs:253
  Err -> invalid_contract_report(error, &c)        validate.rs:121
    schema-validity Violation
    nine later carrier placeholders Unproven/not evaluated
    attach_prerequisites; relation snapshot pins; no guard records
```

The rule is `agentflow.component.contract.semantics@1`, `AF-CONTRACT-0004`,
`spec/component/component-contract.md#component-contract-ports` (source SHA256
`7f847a770dd0d92c289a06a6133e55bd239e5247d7eea63e1c9bc98bcf231f22`).
The wire shape is `agentflow.component.contract@1/ComponentContract`; u32 port0
fits its width but violates the substantive positive-port rule. It is not an
ACE1 encode failure. No input type resolve occurs for port0; no outputs, nested
canonicalizers, `prove_canonical_contract`, Native Guard, or Checked creation are
reached by that analyze call. `check_contract` invokes analyze again and returns
Err; it does not reach its second canonicalize/Checked construction.
`observe_canonical_guard` separately rejects at canonical validation.

`ContractError` has exactly `rule_id:&'static str`, `code:&'static str`,
`reference:Option<String>`, `detail:String`. This is structured diagnostic data,
**not** a complete typed rejection witness: it lacks direction, original ordinal,
sorted occurrence and predecessor. Its reference "0" is not an Origin. The replay
has enough declared input to independently locate ordinal0, but that does not prove
the legacy validator captured a typed path. Existing schema carrier V is genuine;
no Guard/Root countermodel or complete G01 witness is claimed.

## Nine actual members and identity

All members are shared borrows from the actual fixture. Source identity is the
whole originating object and complete contents, not its address string or digest.

| Rust member | Full Rust referent type | Model field | Actual fixture source |
|---|---|---|---|
| modules | af_idl::CompiledModuleSet | compiled_modules | f.modules; compiled sources listed in fixture modules() |
| schema_universe | af_semantic::TypeSchemaUniverse | type_schemas | f.schemas; compiled modules + CaptureInput Int64 |
| effects | af_effect::EffectCatalogUniverse | effect_catalogs | f.effects |
| resources | af_resource::ResourceTypeUniverse | resource_types | f.resources; compiled module schemas |
| flow | af_flow::FlowCatalogs | flow_catalogs | f.flow |
| usage_meters | af_usage::UsageMeterUniverse | usage_meters | f.usage |
| recovery | af_recovery::RecoveryOperationUniverse | recovery_operations | f.recovery |
| execution | af_execution::ExecutionUniverses | execution_universes | f.execution |
| relations | af_contract::RelationSnapshots | relation_snapshots | f.relations |

`StaticContractProofContext<'a>` is exactly these nine borrowed fields
(`crates/af-contract/src/validate.rs:20`). Native `relations` has named optional
effect_resource/recovery_usage/flow_execution snapshots; complete snapshot content
is required, not only `pins()`. `ContractProofContextContents` describes these
nine complete fields. Old `Context` has ten, and its relation_snapshots is a
sequence, not that three-option record. Neither schema can be coerced into the other.

For the actual analyze call, Analysis Context and Canonical Validation Context
are the **same `&c` object**; there is no Checked Context role or resolver role.
Actual context aliasing is checked in-process by object association. A newly
created `f.view()` with identical referents/contents is a different context object;
content equality does not establish aliasing. Portable design retains full context
contents and the observed role equivalence partition, never native addresses.
Complete portable projection remains G02 work, not implemented here.

## Resolver adjudication

Semantic answer: resolver dependencies are **not inputs to direct native raw
canonical validation**. The type explicitly has no I/O/resolver. Ports consume
the supplied schema universe, not a resolver query. Resolver manifests are
applicable to resolver-backed caller/root/checked-source association stages.

Schema answer: the registered old `contract_validation.context:Node<Context>`
unconditionally requires `resolver_dependencies:ResolverDependencyManifest`.
There is no conditional field omission or NotSupplied tag in that record.
Conditional semantic need does **not** license ignoring this closed descriptor.
NotSupplied != supplied KnownEmpty. Empty relation/IR/resolver objects cannot be
invented, and Pending G02 cannot make a missing input exist. Other subjects retain
their strict resolver associations unchanged.

## Five candidates — complete required subject fields

All five use the current exact compiler/model tuple. Full nested descriptors are
copied in `registered-selection.json`; references resolve in the frozen model@3.

| Candidate | Required subject fields and states | Applicability to this replay |
|---|---|---|
| root_export | caller_ir DigestRef, top_export DigestRef, compiler Node<Compiler>, context Node<Context10>, resolver_associations Node<ResolverAssociations>, root_provenance Node<RootProvenance>, associations ContextPartition, assertion_inputs Node<AssertionGraph>. Checked/root source associations and five preflights apply. | Not applicable: no caller IR/root/export/assertion/resolver/provenance input. A port error is not a root preflight. |
| contract_validation | compiler Node<Compiler>, context Node<Context10>, associations ContextPartition, candidate Node<RawCandidateContract>, old_checked Opt<Node<CheckedContractContents>>. Raw allowed; old_checked can genuinely be None. | Partly reusable, not exact: first incompatible field is context.resolver_dependencies, additionally relation_snapshots shape differs. Raw permission alone does not repair Context, Source or Witness. |
| computation | compiler Node<Compiler>, caller_ir DigestRef, context Node<Context10>, resolver_associations Node<ResolverAssociations>, root_provenance Node<RootProvenance>, associations ContextPartition, owned_root ComputationId. | Not applicable: no IR/root/node/resolver supplied. |
| standalone_substitution | owner_compiler CompleteCompiler, old OperandSubject, replacement OperandSubject, contexts ContractContextSet, associations ContextRolePartition, application ApplicationOrigin. Each operand has role/Checked contents/identity/context_role; contexts analysis/old_checked/replacement_checked. | Not applicable: neither two Checked operands nor a declarative application exists. Case checked_operand consumes actual Checked, not a Raw validation substitute. |
| native_recovery_guard | owner_compiler CompleteCompiler, declaration CanonicalNativeContractDeclaration{contract,canonical_bytes}, context ContractProofContextContents9, context_roles NativeGuardContextPartition. | Not applicable: canonical admission fails; declaration cannot be constructed honestly. Seven closed cases do not include raw positive-port validation. |

Additional predicate/source review, not exclusion merely by names:

* `PredicateFamily` is exactly root_preflight/root_dimension/execution_child/
  root_boundary/leaf/control/root_recovery/prerequisite/standalone_substitution/
  native_recovery_guard. There is **no generic contract_validation family**.
  The subject/report stage name does not declare one.
* Existing `prerequisite/positive_unique_ports` is a substantive dependency with
  `AF-CONTRACT-0004`, but lacks an exact direct-raw profile/source/context/witness
  mapping for this call. It cannot be promoted by report prose alone.
* Root/leaf/control/recovery sources require actual IR/Computation/source bindings;
  standalone source/target requires operands/application; Native source/target and
  Coordinate.native_guard require an admitted native subject. Context/compiler
  source variants cannot stand for the Raw Declaration. Target.analysis_boundary
  or Coordinate.whole loses the offending port. Target.relation is not the
  declaration's positive-port constraint. Existing port evidence requires a valid
  owner digest, which this rejected input does not have.
* Origin has predicate/declared_path/source_ordinal/mapping_ordinal; it does not
  itself supply an absent Raw document. Existing DeclaredStep field/entry/member
  can address a known document, but cannot create one. TypedContradiction's
  invalid_shape/invalid_mapping do not by themselves bind this substantive u32
  positivity/uniqueness failure and its actual source path.
* Permitted root families have their root stages/subjects and typed witnesses;
  standalone has its 27 cases, substitution stage/basis/witness; Native has its
  seven cases at contract_validation stage and NativeGuardWitness. None permits
  replacing their required inputs with an unvalidated Raw candidate.

Rules anchoring the rejection of coercion: AF-SEMANTIC-0198 (complete Scope),
0199 (closed predicate),0200 (subjects/origin),0206 (prerequisites),0208 (typed
references),0210 (copy merge),0219 (closed decode),0230 (Checked operands),
0245/0246 (canonical Native subject/seven cases),0257/0258 (fragment/closed model).
No exact Option A mapping exists under the current frozen tuple.

## Options and unique decision

| Option | Assessment |
|---|---|
| A: existing exact binding | Rejected by the field/predicate/source review above. Filling resolver=empty or canonical_source=P is unsound. |
| B: raw/canonical/checked stage variants inside existing subject | Technically possible only in a successor, never in-place. However no existing contract_validation PredicateFamily supplies the desired meaning; the change still needs new Source/Coordinate/Witness and explicit stage applicability. Merely changing Context or tagging the old carrier does not suffice. Adding all three stages would broaden this minimal amendment and risk relaxing old resolver guarantees. |
| C: independent narrow raw_contract_validation profile | **Recommended.** One exact substantive case initially: positive_unique_io_ports, parameterized by actual direction/original occurrence. Closed Raw subject/context/origin/witness, no Root/Checked authority. Explicitly does not claim all canonicalizer failures or complete contract validation. |

Recommendation is a design choice, not a claim that B is impossible to specify.
The new profile is independent because the existing closed predicate roster has
no direct Raw validation family. Its substantive rule is reused, not redefined.
Other canonical checks (e.g. exact type pins or nested usage) need their own
future exact cases before being captured; unknown failures fail UnsupportedPredicate,
not a generic schema-invalid catch-all. See amendment draft and Oracle design.
