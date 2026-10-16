"""Collision-checked C1G2 allocations; static registration, never Capture."""
BASELINE='875e6e41f16845c78d4f6d31573826c547d48c07'
OWNER='agentflow.root-export-checker@5'
MODEL='agentflow.computation.proof-report.model@4'
SHAPE='agentflow.g01.report-shape@4'
OBLIGATION='agentflow.g01.obligation@4'
SOURCES={
 'RC':('agentflow.component.raw-validation.semantics@1','schema_language','spec/component/raw-contract-validation.md','Raw input source admission, one exact port predicate, declarative origin and non-authoritative typed evidence.'),
 'OS':('agentflow.computation.proof-obligation.semantics@4','schema_language','spec/computation/proof-obligation-semantics-v4.md','Independent Raw fragment, explicit Origin extension and input/output separation without CompleteView.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v4.md','Closed Raw validation successor descriptors and unchanged embedded historical models.'),
 'PP':('agentflow.portable.proof.inputs.semantics@5','schema_language','spec/portable/portable-proof-inputs-v5.md','Exact Raw successor tuple, current-only content identity and unconditional staging capability denial.')}
ALLOCATIONS=[
 ('RC','RC-SCOPE','raw-subject','Raw subjects preserve all thirteen parsed Raw fields, real declarative documents and physical byte associations without inventing historical input.', ['AF-SEMANTIC-0198','AF-SEMANTIC-0200']),
 ('RC','RC-CONTEXT','raw-context','Raw analysis and canonical validation use the actual same nine-member context object and exact partition without a fabricated resolver.', ['AF-SEMANTIC-0247']),
 ('RC','RC-STAGES','raw-stages','Raw physical bytes, parsed declarations, diagnostic presentation, canonical validation and Checked states remain distinct; parser failure is not a port predicate.', ['AF-CONTRACT-0003','AF-SEMANTIC-0213']),
 ('RC','RC-PREDICATE','raw-port-predicate','The single positive_unique_io_ports case preserves the actual input-before-output stable numeric sort and reached positivity/uniqueness check before schema resolution.', ['AF-CONTRACT-0004','AF-SEMANTIC-0199']),
 ('RC','RC-LOCATION','raw-origin','Raw Source, exact schema constraint Target, original and sorted port Coordinate and both declarative paths identify the same real occurrence.', ['AF-SEMANTIC-0200','AF-SEMANTIC-0210']),
 ('RC','RC-WITNESS','raw-witness','Zero and duplicate violations retain complete typed U32 occurrences and real predecessor evidence; positive facts have no contradiction and diagnostic strings cannot supply evidence.', ['AF-CONTRACT-0004','AF-SEMANTIC-0211']),
 ('RC','RC-LINKS','raw-links','Raw prerequisites and typed member references retain original stage and origin, all observations and consumers; one local positive port does not prove namespace coverage.', ['AF-SEMANTIC-0206','AF-SEMANTIC-0208','AF-SEMANTIC-0209']),
 ('RC','RC-HOOK','raw-hook-contract','Future branch-created evidence is borrowed before diagnostic reduction; source, context and evidence lifetimes cannot produce a Checked token or trusted caller assertion.', ['AF-SEMANTIC-0197','AF-SEMANTIC-0213']),
 ('OS','RC-FRAGMENT','raw-fragment','Raw input Scope excludes generated facts and witnesses, whose separate finite DAGs and non-root fragment cannot convert to CompleteView.', ['AF-SEMANTIC-0257']),
 ('RM','RC-DESCRIPTORS','raw-descriptors','All Raw descriptors, parent extensions and member paths are closed, width-exact, ranked and field-complete while historical descriptors retain their original owners.', ['AF-SEMANTIC-0214','AF-SEMANTIC-0258']),
 ('PP','RC-MIGRATION','raw-migration','The exact Raw successor binds complete registered content with separate Registered, NotImplemented and NotAuthorized states; old and mixed identities fail closed.', ['AF-SEMANTIC-0259','AF-SEMANTIC-0260'])]
RULES=[dict(source=s,draft=d,anchor=a,statement=t,dependencies=deps,rule_id=f'AF-SEMANTIC-{i:04}',case_id=f'g01-raw-validation-staging/rule-{i:04}') for i,(s,d,a,t,deps) in enumerate(ALLOCATIONS,261)]
