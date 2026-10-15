# Ordered operands, contexts and application identity

**PROPOSED — NOT REGISTERED.** Logical non-wire types; not Rust APIs or wire schemas.

```text
OperandRole = old | replacement
ContractContents = existing full13-field ComponentContract + exact canonical input bytes
                   + existing full ComponentContractDigest algorithm/domain/bytes
OperandSubject = {role:OperandRole, contents:ContractContents,
                  identity:complete ComponentIdentity, context_role:ContextRole}
SubstitutionProofSubject = {owner_compiler:CompleteCompiler,
  old:OperandSubject(old), replacement:OperandSubject(replacement),
  contexts:{analysis:FullContext,old_checked:FullContext,replacement_checked:FullContext},
  associations:ContextRolePartition, application:ApplicationOrigin}
```

Complete Compiler uses the existing RM logical definition. **FullContext in this entire proposal means the new closed ContractProofContextContents below**, not the root/Portable Context's resolver-augmented layout. Repeated identity/pin/bytes must agree with canonical operands. Existing digests remain consistency pins, not replacements for full content or proof. Stage-origin evidence from real private validators is retained in the fact graph; a decoded record does not become a Checked operand. Borrowed local source binding remains PendingLosslessScopeProjection until G02. No Scope normalization is implemented here.

```text
ContractProofContextContents = {
 compiled_modules:Map<SpecificationId,CompiledModuleContents>,
 type_schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,
 effect_catalogs:Map<SpecificationId,Set<Text>>,
 resource_types:{schemas:Map<LogicalTypeIdentity,CanonicalTypeSchema>,
                 descriptors:Map<LogicalTypeIdentity,ResourceTypeDescriptor>},
 flow_catalogs:{confidentiality:Set<ConfidentialityLabelRef>,taints:Set<TaintRef>,
               sinks:Map<EffectOperationRef,Bool>},
 usage_meters:Map<UsageMeterRef,{meter:UsageMeterRef,unit:UsageUnitRef,
                               aggregation:UsageAggregation}>,
 recovery_operations:{effects:Map<SpecificationId,Set<Text>>,
                      reconciliation_operations:Set<{catalog:Text,name:Text}>},
 execution_universes:ExecutionUniverseContents,
 relation_snapshots:{effect_resource:Opt<RelationSnapshotContent>,
                     recovery_usage:Opt<RelationSnapshotContent>,
                     flow_execution:Opt<RelationSnapshotContent>}
}
```

Imported complete module/type/resource/execution/relation content definitions are the exact existing logical RM declarations. This record matches the nine actual StaticContractProofContext fields,including complete schema bodies/universe entries/recognized attributes/relation pins AND canonical contents. Optional relation absence is genuine supplied absence. No resolver_dependencies field exists in this new context descriptor:the actual standalone contract checker has no resolver,so do not fill a fictitious empty root resolver or request a Caller IR. This is an explicit successor descriptor addition,not an omission from a reused old type. Full projection still depends on G02;the design does not implement getters.

The actual two-argument checker has no third context input. Define its implicit analysis context as **replacement.proof_context()**, explicitly an alias of a real reference, not an invented checked object. Context-role order is analysis,old_checked,replacement_checked. Valid shared partition={{analysis,old_checked,replacement_checked}}. Mismatch has {{analysis,replacement_checked},{old_checked}}, even if all projected context contents are equal. Store all roles' complete contents and the partition; never native addresses. A future wrapper accepting an independent analysis argument must additionally require it pointer-identical to replacement and old; it cannot silently change this profile.

C is a mandatory profile premise. On mismatch the legacy report is still12 U; new typed dimension facts are blocked_by(the evaluated context-association U fact), not twelve evaluated U relations. Actual type-level Checked validity is a different premise from C. Missing/forged operand proof, invalid association or incomplete contents fails infrastructure/model validation before a trustworthy fragment, not a fabricated static V. No RootExport report is invented after an earlier D6/D8 rejection.

## Declarative, cross-process application origin

```text
ApplicationDocument = {descriptor:approved successor ModelRef,
  owner:exact declaring SpecificationId,
  applications:Seq<ApplicationDeclaration>}
ApplicationDeclaration = {name:NfcText, old:ContractContents,
  replacement:ContractContents, context_requirement:same_checked_context}
ApplicationOrigin = {document:complete ApplicationDocument,
  declaration_path:Seq<DeclaredStep>}  // exactly applications[index]
```

Document text names are declarations, not trusted approval flags. Names must be nonempty/unique in the document. An origin binds full document content and exact uint64 declaration index; no document checksum-only shortcut, UUID, timestamp, function name, source line, probe ID, call count or insertion order. Document contains input declarations only, never Fact/Report/Scope/Capability descendants. Declared old/replacement values must match actual checked values in their ordered roles. Two identical pairs in two declared entries have distinct application origins. Reobserving the same complete document+path+scope is the same static application, even across processes; repeating a runtime invocation is not automatically a new semantic declaration.

Existing check_substitution has no such application declaration parameter. A future explicitly authorized diagnostic capture adapter must receive/validate this declarative envelope against the real operands; without it, return IncompleteFacts for missing origin rather than inventing it. The ordinary legacy two-argument API may still return its legacy report but cannot automatically manufacture a complete G01 origin. This is an explicit R3 adaptation dependency, not an unresolved source choice or an implemented wrapper.
