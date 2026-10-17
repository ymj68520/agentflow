"""G2 allocation after the recorded exact collision census."""
BASELINE='2ad4359410c087462f8366f1ddb5c38bebfef681'
OWNER='agentflow.root-export-checker@7'
MODEL='agentflow.computation.proof-report.model@6'
SHAPE='agentflow.g01.report-shape@6'
OBLIGATION='agentflow.g01.obligation@6'
PREFIX='g01-relation-schema-staging/'
SOURCES={
 'RS':('agentflow.component.relation-snapshot-validation.semantics@1','schema_language','spec/component/relation-snapshot-validation.md','Native family source and independently current-context relation schema validation; non-authoritative admission only.'),
 'OS':('agentflow.computation.proof-obligation.semantics@6','schema_language','spec/computation/proof-obligation-semantics-v6.md','Separate immutable relation input and derived current validation observations with exact source, witness and DAG propagation.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v6.md','Closed relation family/current-validation successor model retaining the exact frozen model@5 and historical imports.'),
 'PP':('agentflow.portable.proof.inputs.semantics@7','schema_language','spec/portable/portable-proof-inputs-v7.md','Exact current relation successor tuple with isolated history, metadata-only support and no RootExport activation.')}
CLAUSES=[
 ('RS','native-source','A preserves actual kind, schema family, revision, complete pin, bytes and full typed native catalog without a construction schema claim.',['AF-CONTRACT-0019','AF-SEMANTIC-0198']),
 ('RS','source-integrity','Native family follows exact SpecificationId grammar; duplicate stored family/revision values agree and plain SHA256 binds complete canonical bytes.',['AF-CONTRACT-0019']),
 ('RS','current-lookup','C queries only the actual Context modules by exact named Slot/Family/Root; no global, latest, schema-universe or construction-cache fallback.',['AF-SEMANTIC-0262']),
 ('RS','definition-closure','Current lookup requires unique full canonical root and complete actual acyclic imported-definition closure; malformed candidate ambiguity rejects before native context assembly.',['AF-SEMANTIC-0217']),
 ('RS','schema-digest','Current TypeSchemaRef contains actual specification_id, type_name and sha256/type-schema HashV1 of complete canonical root; catalog checksum is not schema identity.',['AF-CONTRACT-0019']),
 ('RS','staged-results','NotReached, Missing, Ambiguous, FamilyMismatch, applicable RevisionMismatch, Found, Invalid and Valid have separate closed typed payloads and actual stage applicability.',['AF-SEMANTIC-0263']),
 ('RS','catalog-revalidation','Valid requires actual full native catalog semantic canonicalization and current-schema encode/decode/native-catalog equality and exact byte round-trip; Found alone is not Valid.',['AF-CONTRACT-0013','AF-CONTRACT-0016','AF-CONTRACT-0018']),
 ('RS','three-slots','Each of effect_resource, recovery_usage and flow_execution has its exact closed native type/family/root association and independent Some/None validation; absent snapshots are not positive relation evidence.',['AF-SEMANTIC-0262']),
 ('RS','context-association','Analysis and canonical_validation refer to the same actual nine-member native Context object; equal contents or digests never establish aliases and no resolver is invented.',['AF-SEMANTIC-0262']),
 ('RS','subject-admission','Every actual Some requires C Valid for the relation portion of complete Subject admission; preserve actual source and failure otherwise without issuing a Fact or rewriting port verdicts.',['AF-SEMANTIC-0264','AF-SEMANTIC-0268']),
 ('OS','source-propagation','A source and C observation retain distinct identities; Raw Source/Origin/Basis/Witness/Contradiction/EvidenceRef share the exact Subject while Raw Target and port coordinates retain their actual meanings.',['AF-SEMANTIC-0200','AF-SEMANTIC-0265','AF-SEMANTIC-0266','AF-SEMANTIC-0267']),
 ('OS','input-output-dag','Immutable source/context inputs cannot reference their derived validation/admission/Fact outputs; outputs only reference prior inputs and retain minimal reachable typed DAG rules.',['AF-SEMANTIC-0217']),
 ('RS','construction-history','Equal legacy snapshots remain equal; missing construction provenance is unavailable and cannot be recovered from a new edition, current schema, revision or checksum.',['AF-CONTRACT-0019']),
 ('PP','edition-isolation','New Analysis/Model/Report/Obligation require exact new registered compiler contents; old and mixed tuples reject without relabeling and all old normative records remain exact.',['AF-SEMANTIC-0271']),
 ('PP','capability-denial','Registered relation Metadata Admission never enables complete Raw, Scope, Normalizer, publication or Checked RootExport; default D0 limits and explicit bounded D4 denial remain separate.',['AF-SEMANTIC-0212','AF-SEMANTIC-0218','AF-SEMANTIC-0244']),
 ('RM','closed-model','Every new source/result/field/variant and required association has a closed nominal descriptor and executed registration coverage; shape success is not actual lookup or producer conformance.',['AF-SEMANTIC-0199','AF-SEMANTIC-0208','AF-SEMANTIC-0209'])]
RULES=[dict(source=s,draft='RS-%02d'%(i+1),anchor=a,statement=t,dependencies=deps,rule_id='AF-SEMANTIC-%04d'%(286+i),case_id=PREFIX+'rule-%04d'%(286+i)) for i,(s,a,t,deps) in enumerate(CLAUSES)]
SLOTS=[('effect_resource','EffectResourceCompatibilityCatalog','agentflow.effect.resource.compatibility@1'),('recovery_usage','RecoveryUsageCoverageCatalog','agentflow.recovery.usage.coverage@1'),('flow_execution','InformationFlowExecutionCatalog','agentflow.flow.execution.relation@1')]
