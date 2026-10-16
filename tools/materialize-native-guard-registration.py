"""Static G2 registration materializer. No runtime facts, codec or normalizer."""
import copy,hashlib,json,re,subprocess,sys
from pathlib import Path
from closed_g01_model import Graph,parse_defs,enum,ref,UNIT
from native_guard_registration_data import *
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/'spec/conformance/g01-native-guard-staging'
def sha(b):return hashlib.sha256(b).hexdigest()
def raw_at(path):return subprocess.check_output(['git','show',BASELINE+':'+path],cwd=ROOT)
def model(path):return json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/path).read_text(encoding='utf-8'),re.S)[1])
def write(p,b):
 p=ROOT/p;p.parent.mkdir(parents=True,exist_ok=True)
 if not p.exists() or p.read_bytes()!=b:p.write_bytes(b)
def js(p,x):write(p,(json.dumps(x,ensure_ascii=False,sort_keys=True,indent=2)+'\n').encode())
def walkrefs(x):
 if isinstance(x,dict):
  if 'ref' in x:yield x['ref']
  for v in x.values():yield from walkrefs(v)
 elif isinstance(x,list):
  for v in x:yield from walkrefs(v)
def qualify(x):
 if isinstance(x,dict):return {k:HISTORICAL+'/'+v if k=='ref' else qualify(v) for k,v in x.items()}
 if isinstance(x,list):return [qualify(v) for v in x]
 return x
def build():
 old=model('spec/tables/proof-report-model-v2.md');hist=old['inherited_registered_model']
 hg=Graph(hist,{},ROOT);ht=hg.finish(NATIVE_NAMES+['af-semantic::SchemaEntry'])
 imports={HISTORICAL+'/'+n:qualify(v) for n,v in ht.items()}
 defs=copy.deepcopy(old['types']);authored=parse_defs(DEFINITIONS)
 def inline_unit_payloads(x):
  if isinstance(x,dict):
   if x.get('kind')=='enum':
    for row in x['variants']:
     if row[1]=={'ref':'unit'}:row[1]=copy.deepcopy(UNIT)
   for v in x.values():inline_unit_payloads(v)
  elif isinstance(x,list):
   for v in x:inline_unit_payloads(v)
 inline_unit_payloads(authored);defs.update(authored)
 defs['SchemaEntry']=ref(HISTORICAL+'/af-semantic::SchemaEntry')
 defs['NativeGuardCase']=enum([(c,UNIT) for c in CASES]);defs['NativeGuardClause']=enum([(c,UNIT) for c in CLAUSES]);defs['NativeGuardBranch']=enum([(c,UNIT) for c in BRANCHES])
 defs['ImportedActionGuardCompatibility']=ref(HISTORICAL+'/'+NATIVE_NAMES[0]);defs['ImportedTypedGuardCountermodel']=ref(HISTORICAL+'/'+NATIVE_NAMES[2])
 defs.update(imports)
 extra=parse_defs('''
ScopeSubject = native_recovery_guard:NativeRecoveryGuardSubject
Source = native_guard_subject {subject:Node<NativeRecoveryGuardSubject>} | native_guard_binding {subject:Node<NativeRecoveryGuardSubject>,occurrence:NativeGuardOccurrence}
Target = native_guard_subject {subject:Node<NativeRecoveryGuardSubject>} | native_guard_binding {subject:Node<NativeRecoveryGuardSubject>,action_binding_ordinal:U64,action_identity:RecoveryActionIdentity}
Coordinate = native_guard {subject:Node<NativeRecoveryGuardSubject>,clause:NativeGuardCoordinateClause,occurrence:Opt<NativeGuardOccurrence>,guard_role:Opt<NativeGuardRole>,member:NativeGuardMemberPath}
EvidenceRef = native_guard_member {subject:Node<NativeRecoveryGuardSubject>,member:NativeGuardMemberPath}
TypedContradiction = native_guard {witness:NativeGuardWitness}
PrerequisitePredicate = native_guard:NativeGuardPrerequisiteClause
RequirementClause = native_guard:NativeGuardClause
''')
 for k,v in extra.items():defs[k]['variants'].extend(v['variants'])
 for n,c in [('PredicateFamily','native_recovery_guard'),('Kind','native_recovery_guard_evidence'),('ObservationOwner','native_recovery_guard')]:defs[n]['variants'].append([c,UNIT])
 defs['ClosedPredicateCase']['variants'].append(['native_recovery_guard',ref('NativeGuardCase')])
 defs['Fragment']['fields'][1][1]['variants'].append(['native_recovery_guard',UNIT])
 defs['Fact']['fields'].append(['native_guard_basis',{'_generic':'Opt','args':[ref('NativeGuardBasis')]}])
 g=Graph(hist,defs,ROOT);types=g.finish(list(defs))
 manifest=[]
 for n,v in ht.items():
  native=hist['semantic_model_declarations'].get(n)
  manifest.append({'owner':HISTORICAL,'name':n,'successor_key':HISTORICAL+'/'+n,'historical_declaration':native,'historical_descriptor':v,'successor_descriptor':imports[HISTORICAL+'/'+n],'recursive_dependencies':sorted(set(walkrefs(v))),'source':native['source'] if native else 'spec/tables/proof-report-model-v1.md','root':n in NATIVE_NAMES})
 sources={p:{'sha256':sha((ROOT/p).read_bytes()),'content_hex':(ROOT/p).read_bytes().hex()} for p in sorted({x['source'] for x in manifest})}
 sources['spec/tables/proof-report-model-v1.md']={'sha256':sha((ROOT/'spec/tables/proof-report-model-v1.md').read_bytes()),'content_hex':(ROOT/'spec/tables/proof-report-model-v1.md').read_bytes().hex()}
 clauses={x['draft']:x['rule_id'] for x in RULES}
 rows=[]
 for i,c in enumerate(CASES):
  context=c=='context_association';subject=i<2
  label={'canonical_source':'NG-SUBJECT','context_association':'NG-CONTEXT','action_sources':'NG-SOURCES','action_bounds':'NG-BOUNDS','guard_inputs':'NG-INPUTS','guard_containment':'NG-CONTAINMENT','guard_evaluation':'NG-EVALUATION'}[c]
  rows.append(dict(family='native_recovery_guard',case=c,rank=i,kind='proof_context' if context else 'native_recovery_guard_evidence',requirement={'family':'direct' if context else 'native_guard','clause':'proof_context' if context else c,'evidence_class':'static_declaration'},stage='contract_validation',role='mandatory_preflight' if subject else 'supporting_fact',dimension='supporting exact PredicateId',source_variant='native_guard_subject' if subject else 'native_guard_binding',target_variant='native_guard_subject' if subject else 'native_guard_binding',coordinate_variant='native_guard',origin_type='Origin',assertion_path='known-empty',parameter_roles=['forward','action'] if c=='guard_inputs' else [],governing_rule=clauses[label],substantive_rule='AF-CONTRACT-0022',applicability=['canonical validation admitted','actual analysis/canonical validation association','each real binding','source entries and usage resolved','bounds ready;both eager roles','valid inputs and nonzero potential;actual call','each actual native result'][i],evaluation='actual native evaluation only;never synthesize skipped calls',witness='exact applicable NativeGuardWitness iff V;no V from frozen false',source_type='Source',target_type='Target',coordinate_type='Coordinate'))
 out=copy.deepcopy(old)
 out.update(model_id=MODEL,model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,types=types,descriptor_roots=list(defs),inherited_registered_model=old,profile='native_recovery_guard',native_case_descriptors=rows,historical_selected_imports=manifest,historical_import_sources=sources)
 out['closed_predicates']={**old['closed_predicates'],'native_recovery_guard':CASES}
 out['canonical_constraints']=[v.replace('all50 source bindings here,24 selected specifications and149 applied rules','all54 source bindings here,28 selected specifications and163 applied rules').replace('hidden 50->24 content reduction','hidden 54->28 content reduction') for v in old['canonical_constraints']]+[
  'Native profile appends seven cases/six clauses at contract_validation;root five preflights/nine dimensions/six boundaries and substitution meanings are unchanged.',
  'Native subject is complete compiler,canonical declaration+bytes,nine-member contract context,and analysis0/canonical_validation1 same-object partition;no serialized addresses and full Scope remains Pending G02.',
  'Native facts/results/assignments are outputs outside Scope;each witness DAG has its own finite minimal reachable rank space. No root CompleteView conversion.',
  'Native ModelType/ModelRef selected imports use exact historical owner/name keys only;current owner cannot alias those historical names. Unknown owners/names/fields/variants reject.',
  'Native Fact requires native_guard_basis Some exactly for native family;substitution_basis None. Existing families require native_guard_basis None.',
  'Guard input outcomes are both eager but primary forward error wins;port validation precedes zero. Containment false is U,not V;kernel may be skipped while native algorithm reports its actual branch.',
  'Native local WellTypedComponentInputDomain never proves Root-global reachability. Root consumers require real Checked body/context/pin association;negative diagnostics confer no authority.',
  'Canonical declarations require actual validator provenance;metadata validation proves no validator fact,context aliasing,normalization,support,activation or capability.',
 ]
 return out
def main():
 m=build();refs=list(walkrefs(m['types']));assert all(x in m['types'] for x in refs)
 print('G2 MODEL',len(m['types']),'TYPES',len(refs),'REFS','IMPORTS',len(m['historical_selected_imports']))
 if '--emit' not in sys.argv:return
 oldspec=json.loads(raw_at('spec/registry/specifications.json'));oldrules=json.loads(raw_at('spec/registry/rules.json'))
 ids={r['specification_id'] for r in oldspec['specifications']};rids={r['rule_id'] for r in oldrules['rules']}
 assert not ids&{v[0] for v in SOURCES.values()} and not rids&{v['rule_id'] for v in RULES}
 assert all((ROOT/p).exists() for k,(_,_,p,_) in SOURCES.items() if k!='RM')
 body='# '+MODEL+'\n\nRegistered non-wire native staging, NOT Typed Capture or authority. Frozen prior models are embedded unchanged below. All exact source/import/field/variant/rank and DAG constraints are normative;metadata admission checks shape only.\n\n```json\n'+json.dumps(m,ensure_ascii=False,separators=(',',':'))+'\n```\n\n'
 rr=next(x for x in RULES if x['source']=='RM')
 body+=f'<a id="{rr["anchor"]}"></a>\n\n## {rr["rule_id"]}\n\nMUST: {rr["statement"]}\n\nDirect normative dependencies: '+', '.join(rr['dependencies'])+f'.\n\nRegistration conformance: {rr["case_id"]};not producer conformance.\n'
 write(SOURCES['RM'][2],body.encode())
 specs=copy.deepcopy(oldspec);rules=copy.deepcopy(oldrules)
 for sid,kind,path,desc in SOURCES.values():
  specs['specifications'].append(dict(specification_id=sid,kind=kind,status='frozen',description=desc,normative_source={'logical_path':path,'content_digest':{'algorithm':'sha256','hex':sha((ROOT/path).read_bytes())}}))
 specs['specifications'].sort(key=lambda x:x['specification_id'])
 for row in RULES:
  rules['rules'].append(dict(area='SEMANTIC',conformance_refs=[row['case_id']],normative_source=SOURCES[row['source']][2],requirement_level='must',rule_id=row['rule_id'],short_statement=row['statement'],source_anchor=row['anchor']))
 rules['rules'].sort(key=lambda x:x['rule_id'])
 js('spec/registry/specifications.json',specs);js('spec/registry/rules.json',rules)
 js('spec/conformance/g01-native-guard-staging/historical-imports.json',{'kind':'Registration Conformance;NOT Capture','selected_imports':m['historical_selected_imports'],'sources':m['historical_import_sources']})
 for name in ['specifications','rules']:write(f'spec/conformance/g01-native-guard-staging/edition3-{name}.json',raw_at(f'spec/registry/{name}.json'))
 print('REGISTERED',len(specs['specifications']),len(rules['rules']))
if __name__=='__main__':main()
