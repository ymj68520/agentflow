"""Reviewed G2 allocations. Registration metadata only; never constructs input G."""
import copy, hashlib, json
from pathlib import Path
from oracle.structural_binding_registration import catalogue, children, closure, nominal

R=Path(__file__).resolve().parents[1]
D='spec/conformance/g01-native-source-staging/'
P='phase-04c1b4c4r3c2r5g2-'
OWNER='agentflow.root-export-checker@9'
MODEL='agentflow.computation.proof-report.model@8'
REPORT='agentflow.g01.report-shape@8'
OBLIGATION='agentflow.g01.obligation@8'
OLD='agentflow.computation.proof-report.model@7'
SLOTS=['effect_resource','recovery_usage','flow_execution']
SOURCES={
 'NB':('agentflow.computation.native-relation-source-binding.semantics@1','schema_language','spec/computation/native-relation-source-binding.md','Exact pre-C native Source nominal child binding; registration only.'),
 'OS':('agentflow.computation.proof-obligation.semantics@8','schema_language','spec/computation/proof-obligation-semantics-v8.md','Successor Source/Subject consumer and failure contract; no producer.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v8.md','Closed Source binding delta over exact complete Model7; no actual ranks.'),
 'PP':('agentflow.portable.proof.inputs.semantics@9','schema_language','spec/portable/portable-proof-inputs-v9.md','Explicit Analysis9 current-only non-authoritative registration entry and isolated legacy8 compatibility.')}
CLAUSES=[
 ('NB','native-input','Actual native kind, family, revision, complete pin, original bytes and full catalog are supplied before G from the same actual Context slot, never reconstructed from C or history.',['AF-SEMANTIC-0286','AF-SEMANTIC-0287','AF-SEMANTIC-0293','AF-SEMANTIC-0308'],['NB-DRAFT-01']),
 ('NB','nominal-edges','All three successor Context Some child declarations target exact NativeRelationSnapshotSource; None emits no Source and alias body equality never permits a nominal cast.',['AF-SEMANTIC-0302','AF-SEMANTIC-0303','AF-SEMANTIC-0293'],['NB-DRAFT-02']),
 ('NB','stage-reference','Immutable G precedes actual C; C.source in output_record and w_value resolves the same Subject slot Source in prior G as exact g_external and never writes G.',['AF-SEMANTIC-0297','AF-SEMANTIC-0306','AF-SEMANTIC-0307','AF-SEMANTIC-0308'],['NB-DRAFT-03','NB-DRAFT-04']),
 ('NB','association','Native A equality, typed content sharing, originating Owner/Occurrence association and C observation remain distinct; shared equal values never delete real parent or Origin edges.',['AF-SEMANTIC-0281','AF-SEMANTIC-0296','AF-SEMANTIC-0298'],['NB-DRAFT-06']),
 ('OS','subject-closure','The sole RawValidationSubject input root reaches Source through actual Context slot Some, with unchanged exact Subject propagation, minimal typed interning and canonical bottom-up ranks.',['AF-SEMANTIC-0217','AF-SEMANTIC-0305','AF-SEMANTIC-0310','AF-SEMANTIC-0311'],['NB-DRAFT-02','NB-DRAFT-05']),
 ('OS','fail-closed','Source or stage failures use applicable existing closed domains and paths, never port verdicts; every Some without actual C Valid forbids complete Fact while retaining actual A and port observations.',['AF-SEMANTIC-0291','AF-SEMANTIC-0295','AF-SEMANTIC-0309'],['NB-DRAFT-08']),
 ('RM','closed-delta','Exact complete base contents, five type replacements, four site replacements and full rebased nominal/reference/field coverage define the closed successor; unknown or two-sided omitted required content rejects.',['AF-SEMANTIC-0214','AF-SEMANTIC-0220','AF-SEMANTIC-0312'],['NB-DRAFT-02','NB-DRAFT-04','NB-DRAFT-05']),
 ('PP','edition-isolation','Analysis9 registration uses exact Model8 Report8 Obligation8 and complete registry/source identity; legacy8 compatibility retains its exact snapshot and cannot masquerade as successor current.',['AF-SEMANTIC-0221','AF-SEMANTIC-0223','AF-SEMANTIC-0313'],['NB-DRAFT-07']),
 ('PP','non-activation','Registration and bounded handwritten metadata never implement successor G, produce native receipts or production ranks, admit full Raw facts, activate support, issue CompleteView or authorize positive proof.',['AF-SEMANTIC-0212','AF-SEMANTIC-0227','AF-SEMANTIC-0314'],['NB-DRAFT-08'])]
RULES=[dict(source=s,anchor=a,statement=text,dependencies=deps,clauses=cs,rule_id=f'AF-SEMANTIC-{315+i:04}',case_id=f'g01-native-source-staging/rule-{315+i:04}') for i,(s,a,text,deps,cs) in enumerate(CLAUSES)]

def rebase(v):
 if isinstance(v,dict):return {k:rebase(x) for k,x in v.items()}
 if isinstance(v,list):return [rebase(x) for x in v]
 return MODEL if v==OLD else v

def source_delta():
 old=catalogue();t=copy.deepcopy(old['types']);r=rebase(copy.deepcopy(old['structural_binding_registration']))
 context=t['ContractProofContextContents'];fields=dict(context['fields']);relation=fields['relation_snapshots']
 for slot,d in relation['fields']:
  assert slot in SLOTS and d['item']=={'ref':'RelationSnapshotContent'}
  d['item']={'ref':'NativeRelationSnapshotSource'}
 changed=['ContractProofContextContents'];site_changes=[]
 for row in r['sites']:
  if row['source_catalogue_key']!='ContractProofContextContents' or row['source_path'][0]!=['field','relation_snapshots']:continue
  path=row['source_path'];d=relation if len(path)==1 else dict(relation['fields'])[path[1][1]]
  assert len(path) in [1,2]
  row['descriptor']=copy.deepcopy(d);t[row['alias']]=copy.deepcopy(d);changed.append(row['alias'])
  row['definition']['source_model']=MODEL;row['definition']['parent_source']=dict(owner=MODEL,name='ContractProofContextContents')
  if len(path)==2:
   assert len(row['definition']['children'])==1
   row['definition']['children'][0]['value_type']=dict(owner=MODEL,name='NativeRelationSnapshotSource')
  site_changes.append(copy.deepcopy(row))
 assert len(changed)==5 and len(site_changes)==4
 r['input_types']=sorted(closure(t,['RawValidationSubject']))
 r['output_types']=sorted(closure(t,['RawPortCheckBasis','RawPortRejectionWitness','CurrentContextRelationSchemaValidation','RelationSubjectAdmission']))
 aliases={s['alias'] for s in r['sites']}
 r['value_bindings']=[b for b in r['value_bindings'] if b['catalogue_key'] in set(r['input_types'])|set(r['output_types'])|aliases]
 for b in r['value_bindings']:
  key=b['catalogue_key'];g=key in r['input_types'] or key in aliases and next(s for s in r['sites'] if s['alias']==key)['definition']['eligibility']=='g_input'
  b['eligibility']=['g_input','w_output'] if g else ['w_output']
 r['source_closure_types']=sorted(closure(t,['StructuralDeclaredInputs']))
 r['source_model']=OLD;r['changed_types']=changed
 r['stage_contract']['status']['production_binder']='NotImplemented'
 r['coverage_types']=sorted((set(r['coverage_types'])-{'RelationSnapshotContent'})|{'NativeRelationSnapshotSource'})
 delta=dict(format='closed_native_source_delta_v1',base_source=dict(specification_id=OLD,logical_path='spec/tables/proof-report-model-v7.md',plain_sha256=hashlib.sha256((R/'spec/tables/proof-report-model-v7.md').read_bytes()).hexdigest()),
  edition=dict(analysis=OWNER,model=MODEL,report=REPORT,obligation=OBLIGATION),
  type_replacements={n:t[n] for n in changed},registration=r,
  source_contract=dict(fields=[n for n,_ in t['NativeRelationSnapshotSource']['fields']],native='StaticContractProofContext.relations.<slot> actual immutable Snapshot',stage='before_G',root='RawValidationSubject',slots=SLOTS,source_node_target='NativeRelationSnapshotSource',none_source_nodes=0,c_source_domain='g_external'),
  canonical_constraints=[c.replace('all70 source bindings here,44 selected specifications and217 applied rules','all74 source bindings here,48 selected specifications and226 applied rules') for c in old['canonical_constraints']]+['Successor Source Some children are exact NativeRelationSnapshotSource, not alias casts; Source is pre-C and g_external consumers reference the same immutable Subject slot. Legacy8 current_* APIs are compatibility only; successor registration has a separate non-authoritative entry.'],
  no_new_type_ranks=True,no_new_node_body=True,production_binder='NotImplemented',production_dagranks_generated=0,raw_full_field='INCOMPLETE',root_export='NotAuthorized')
 return delta

def sections(key):
 return ''.join('\n<a id="'+r['anchor']+'"></a>\n\n## '+r['rule_id']+'\n\nMUST: '+r['statement']+'\n\nDirect normative dependencies: '+', '.join(r['dependencies'])+'.\nRegistration conformance: '+r['case_id']+'.\n' for r in RULES if r['source']==key)

TEXT={
 'NB':'''# Exact Native Relation Source Nominal Binding

This successor registers only the accepted direct-child Option A. Actual native A is the immutable Snapshot already supplied in the actual Context, not C output or lost construction history. Getter projection order is kind, schema_family, revision, pin, canonical_content, catalog. Native Snapshot equality, family pin, bytes, revision, full typed catalog and current C/port semantics remain unchanged.

The three ContractProofContextContents.relation_snapshots options have exact NativeRelationSnapshotSource Some children in Model8. RelationSnapshotContent remains a distinct registered alias identity for historical/other uses; it is not cast, retagged, assigned Source rank or an extra root. None is absent, not positive relation evidence. Every source field comes from the same actual typed slot borrow. Source integrity and source/Context/native association precede binding.

Declared inputs -> source admission -> immutable pre-capture G -> actual C -> complete Raw Fact -> future T4. Current module data is an input, but C results are not. C.source output_record and w_value stay g_external and must resolve the exact Source under the same Subject/Context/slot. No W-local or output-local fallback, alias, checksum-only identity, UUID, address, temporary ordinal or output backedge is permitted.

Equal complete typed A values may intern; all real parent edges and complete package/Owner/Occurrence/member/Origin associations survive. Source equality is not observation identity. Same A under different current modules can yield Valid or Missing without modifying A or G. No current Schema can recover missing constructor history. Three closed slot/family/root/catalog associations remain those of AF-SEMANTIC-0293.
''',
 'OS':'''# Source-bound obligation semantics successor8

The sole Raw input root remains RawValidationSubject. It reaches native Source through context/contents/relation_snapshots/actual-slot/Some. Subject is complete candidate input before C, not admitted proof. Source/Target/Origin/Basis/Witness/Evidence consumers retain exact same Subject and explicit G/W environments. Source records use existing record/variant/option/scalar/sequence/set/map bodies. No new Body case, root or type rank is introduced.

AF-SEMANTIC-0217 remains exact: full nominal typed equality, full fields/children, finite minimal reachable DAG, children strictly earlier, zero-based contiguous canonical height/type/body/child ranks. ModelType rank and source ordinal never replace DagRank. All successor input ranks must be independently constructed later under its exact tuple. Registration metadata and handwritten subgraphs are not production ranks or native receipts.

Failures reuse StructuralFailureDomain/Reason/Stage and complete path/site/type payloads. Missing native source/known node is IncompleteFacts/missing_node; unknown type UnsupportedModel/unknown_nominal_type; missing site IncompleteContent/missing_site; wrong child/domain/slot/rank/association/backedge are applicable InvalidFacts reasons; limits ResourceLimit/resource_bound; unavailable production NotImplemented/production_not_implemented. Portable outer taxonomy remains InvalidIdentity/UnsupportedSemantics. No Source failure becomes a port predicate or empty evidence. Every Some requires actual full C Valid for complete Subject admission. Missing/Invalid keeps real source and port verdict, issues no full Fact. None performs no snapshot validation.
''',
 'PP':'''# Analysis9 explicit registration entry and legacy8 isolation

Exact successor tuple is agentflow.root-export-checker@9 / agentflow.computation.proof-report.model@8 / agentflow.g01.report-shape@8 / agentflow.g01.obligation@8. Explicit successor metadata entry binds complete live specification/rule registry bytes, all registered source contents, selected sources/rules and unchanged four closed tables. A version string or checksum is insufficient. The existing legacy8 compatibility APIs remain pinned to exact complete Analysis8 registry/source identity and are not the successor current entry. They cannot accept or produce successor graphs.

All old70 specification and522 rule objects and old normative contents remain unchanged. Old Model7/Analysis8 graphs, type ranks, Source/Origin meanings and Expected are not retagged or auto-migrated. Source-less historical data cannot be upgraded from current schemas; genuinely re-supplied complete inputs require future fresh binding/C under the new tuple. Cross-edition inputs reject. Native/direct/private/alternate, D0/default and explicitly bounded D4 permissions are unchanged. Unsupported successor Portable inputs continue rejecting, never falling back to legacy8 positive proof.

Registration is distinct from support/activation/release. All successor production G/Raw/full seed/Standalone/other T3/normalizer/Scope remain NotImplemented or incomplete; Source production DagRanks=0; CompleteView and ObligationIdentity NotIssued; RootExport, Portable positive proof, publication and runtime NotAuthorized. No caller flags or successful metadata comparison seal a native source receipt or activate any path. G02/G03 and independent prior findings are not closed by registration.
'''}
