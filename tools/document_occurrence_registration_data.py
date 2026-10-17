"""C2G2 allocation following the frozen Registry/path collision census."""
BASELINE='e623aab705c5f8269807201e112a32e67e6869bf'
OWNER='agentflow.root-export-checker@6'
MODEL='agentflow.computation.proof-report.model@5'
SHAPE='agentflow.g01.report-shape@5'
OBLIGATION='agentflow.g01.obligation@5'
PREFIX='g01-document-occurrence-staging/'
SOURCES={
 'DO':('agentflow.component.document-occurrence.semantics@1','schema_language','spec/component/document-occurrence.md','Declared SourcePackage, immutable owner anchors, exact document slots and fail-closed non-authoritative source association.'),
 'OS':('agentflow.computation.proof-obligation.semantics@5','schema_language','spec/computation/proof-obligation-semantics-v5.md','Occurrence-aware input provenance and unchanged same-origin observation, input DAG and proof authority boundaries.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v5.md','Closed occurrence successor descriptors with exact embedded frozen model@4 and unchanged selected historical imports.'),
 'PP':('agentflow.portable.proof.inputs.semantics@6','schema_language','spec/portable/portable-proof-inputs-v6.md','Exact occurrence successor tuple, explicit old-source nonmigration and Registered/NotImplemented/NotAuthorized admission.')}
ALLOCATIONS=[
 ('DO','DO-01','document-identities','Complete document content, declared occurrence, internal member and output observation are distinct identities; equal content does not imply equal occurrence.',['AF-SEMANTIC-0200','AF-SEMANTIC-0210']),
 ('DO','DO-02','actual-source','Occurrence admission requires an actual supplied complete SourcePackage and real owner revision and document slot; missing provenance cannot be synthesized from a candidate or invocation.',['AF-SEMANTIC-0261']),
 ('RM','DO-03','closed-owners','Owner containers are exactly declaration_bundle, import_manifest and workflow_source_collection with complete closed entries; unknown domains, owner schemas and variants reject.',['AF-SEMANTIC-0270']),
 ('DO','DO-04','declared-authority','Exact UTF8 namespace, domain, owner and revision keys establish only declared source identity and never external authentication or global ownership.',['AF-SEMANTIC-0213']),
 ('DO','DO-05','owner-conflicts','All submitted owners must match the package domain and have unique complete anchors; conflicting complete records in explicit joint packages are owner binding conflicts.',['AF-SEMANTIC-0200']),
 ('DO','DO-06','occurrence-conflicts','Owner anchor and actual U64 document-entry ordinal define occurrence; equal content reuses it and unequal content at the same key fails BindingConflict.',['AF-SEMANTIC-0210']),
 ('DO','DO-07','member-paths','External document slot, internal document path, port original and sorted ordinals and observation site remain separate and all selected input members must agree.',['AF-SEMANTIC-0265']),
 ('DO','DO-08','physical-native','Physical sources require original bytes and supplied matching occurrence; native sources require complete declarations and not_supplied physical history; other combinations reject UnsupportedPhysicalAssociation.',['AF-SEMANTIC-0263']),
 ('OS','DO-09','identity-propagation','Occurrence binding starts at RawInputOrigin and propagates through subject, source, raw origin, FactKey, basis, witness and references without mixed occurrences.',['AF-SEMANTIC-0200','AF-SEMANTIC-0267']),
 ('OS','DO-10','input-dag','Equal typed content still interns under the finite minimal reachable DAG; different real occurrence bindings are different typed values and source packages cannot refer to generated outputs.',['AF-SEMANTIC-0217','AF-SEMANTIC-0269']),
 ('OS','DO-11','observation-copy','Only equal complete scoped originating facts with equal non-observation payload union legitimate observations; conflicting payloads fail InvalidFacts and observations never generate occurrences.',['AF-SEMANTIC-0210','AF-SEMANTIC-0211']),
 ('DO','DO-12','frozen-port-context','The one Raw port predicate, actual execution prefix and nine-member same-object context partition remain unchanged; no context role, resolver or generic schema profile supplies document identity.',['AF-SEMANTIC-0262','AF-SEMANTIC-0264']),
 ('PP','DO-13','migration','Exact old content-only sources retain old-edition meaning and cannot acquire occurrence history without actual extra evidence; old and mixed tuple contents reject without relabelling.',['AF-SEMANTIC-0271']),
 ('PP','DO-14','staging-denial','Registered occurrence metadata and shape admission produce no predicate fact, Checked, full Scope, normalization, external authority or RootExport capability; all unsupported and resource failures remain fail-closed.',['AF-SEMANTIC-0212','AF-SEMANTIC-0218','AF-SEMANTIC-0244'])]
RULES=[dict(source=s,draft=d,anchor=a,statement=t,dependencies=deps,rule_id=f'AF-SEMANTIC-{i:04}',case_id=PREFIX+f'rule-{i:04}') for i,(s,d,a,t,deps) in enumerate(ALLOCATIONS,272)]
