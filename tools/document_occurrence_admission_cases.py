"""Hand-authored metadata admission fixtures; NOT a Producer fact Oracle."""
import copy,json
from pathlib import Path
R=Path(__file__).resolve().parent.parent
def text(s):return list(s.encode('utf-8'))
def cases():
 design=json.loads((R/'docs/implementation/phase-04c1b4c4r3c2g1-independent-vectors.json').read_bytes())
 candidate=design['definitions']['physical_metadata_candidate'];second=copy.deepcopy(candidate);second['inputs'][0]['port_no']=1
 native={'declared_native_document':{'declarations':[candidate,second]}}
 physical=design['definitions']['documents']['physical_P']
 domain={'namespace':text('team'),'domain_key':text('submission-7')}
 def owner(name,kind,doc):
  field={'declaration_bundle':('bundle_name','declarations','declaration_name'),'import_manifest':('manifest_name','imports','import_name'),'workflow_source_collection':('collection_name','sources','source_name')}[kind]
  a,b,c=field
  return {'anchor':{'domain':copy.deepcopy(domain),'owner_key':text(name),'revision_key':text('revision-3')},'container':{kind:{a:text(name),b:[{c:text('entry0'),'document':copy.deepcopy(doc)},{c:text('entry1'),'document':copy.deepcopy(doc)}]}}}
 def package(doc=native,kind='declaration_bundle'):
  return {'domain':copy.deepcopy(domain),'owners':[owner('A',kind,doc),owner('B',kind,doc)]}
 rows=[]
 def add(name,expect='Admitted',**changes):
  row=dict(case_id='g01-document-occurrence-staging/admission-'+name,operation='admit',package=package(),source_owner=None,slot=0,borrow_owner=0,borrow_slot=0,path_entry=0,physical='not_supplied',expected=expect,declared_only=True,capture_case=False)
  row.update(changes);rows.append(row);return row
 for kind in ['declaration_bundle','import_manifest','workflow_source_collection']:
  add('native-'+kind,package=package(kind=kind))
  add('physical-'+kind,package=package(physical,kind),physical='matching')
 add('same-slot-repeat',operation='repeat',repeat=2)
 add('different-slots',operation='different_slots')
 add('different-owners',operation='different_owners')
 add('missing-package','MissingDocumentOccurrenceBinding',missing_package=True)
 add('missing-owner','MissingDocumentOccurrenceBinding',missing_owner=True)
 add('missing-slot','MissingDocumentOccurrenceBinding',slot=None)
 for key in ['namespace','domain_key']:
  row=add('missing-'+key,'MissingDocumentOccurrenceBinding');row['package']['domain'].pop(key)
 for key in ['owner_key','revision_key']:
  row=add('missing-'+key,'MissingDocumentOccurrenceBinding');a=copy.deepcopy(row['package']['owners'][0]['anchor']);a.pop(key);row['source_owner']=a
 row=add('owner-domain-mismatch','InvalidOccurrenceAssociation');row['package']['owners'][0]['anchor']['domain']['namespace']=text('other')
 row=add('unknown-owner','UnsupportedOwnerVariant');row['package']['owners'][0]['container']={'durable_source_handle':{'handle':text('claimed')}}
 row=add('unknown-domain-schema','UnsupportedSourceDomain');row['package']['domain']['unknown_domain']=True
 row=add('undeclared-owner','InvalidOccurrenceAssociation');row['source_owner']=copy.deepcopy(row['package']['owners'][0]['anchor']);row['source_owner']['owner_key']=text('undeclared')
 add('out-of-range','InvalidOccurrenceAssociation',slot=2)
 row=add('duplicate-owner','MalformedSourcePackage');row['package']['owners'].append(copy.deepcopy(row['package']['owners'][0]))
 other=copy.deepcopy(native);other['declared_native_document']['declarations'][0]=copy.deepcopy(second)
 add('wrong-content','BindingConflict',external_document=other)
 add('equal-copy-wrong-borrow','InvalidOccurrenceAssociation',clone_document=True)
 add('wrong-owner-borrow','InvalidOccurrenceAssociation',borrow_owner=1)
 add('wrong-declaration','InvalidOccurrenceAssociation',candidate_override=second)
 add('wrong-internal-entry','InvalidOccurrenceAssociation',path_entry=1,candidate_override=candidate)
 add('native-supplied','UnsupportedPhysicalAssociation',physical='matching',physical_document_override=physical)
 add('physical-not-supplied','UnsupportedPhysicalAssociation',package=package(physical))
 add('physical-cross-occurrence','InvalidOccurrenceAssociation',package=package(physical),physical='other')
 add('context-split','InvalidContextBinding',context_split=True)
 add('compiler-wrong','InvalidIdentity',compiler_wrong=True)
 bad=copy.deepcopy(physical);bad['physical_json_utf8']['bytes']=[255]
 add('physical-invalid-utf8','InvalidPhysicalInput',package=package(bad),physical='matching',candidate_override=candidate)
 bad=copy.deepcopy(physical);bad['physical_json_utf8']['bytes']=list(b'{"x":0,"x":1}')
 add('physical-duplicate-json-key','InvalidPhysicalInput',package=package(bad),physical='matching',candidate_override=candidate)
 bad=copy.deepcopy(physical);bad['physical_json_utf8']['bytes']=list(b'{')
 add('physical-malformed-json','InvalidPhysicalInput',package=package(bad),physical='matching',candidate_override=candidate)
 add('physical-wrong-candidate','InvalidOccurrenceAssociation',package=package(physical),physical='matching',candidate_override=second)
 add('physical-bytes-conflict','BindingConflict',package=package(physical),physical='matching',external_document=design['definitions']['documents']['physical_P_newline'],candidate_override=candidate)
 add('joint-same',operation='joint_same')
 add('joint-conflict','OwnerBindingConflict',operation='joint_conflict')
 row=add('context-digest-is-not-owner','MissingDocumentOccurrenceBinding');row['source_owner']={'context_digest':text('not-owner')}
 for name in ['pointer','call-counter','random-uuid','fabricated-wrapper']:
  add(name+'-only','MissingDocumentOccurrenceBinding',missing_package=True,source_owner={'claimed':name})
 row=add('package-output-backedge','UnsupportedModel');row['package']['generated_fact']=0
 row=add('key-invalid-utf8','UnsupportedModel');row['package']['domain']['namespace']=[255]
 row=add('key-empty','MissingDocumentOccurrenceBinding');row['package']['domain']['namespace']=[]
 row=add('key-resource-limit','ResourceLimit');row['package']['domain']['namespace']=[120]*4097
 row=add('native-full-thirteen-raw-not-checked');row['package']['owners'][0]['container']['declaration_bundle']['declarations'][0]['document']['declared_native_document']['declarations'][0]['inputs'][0]['port_no']=0
 row=add('caller-authentication-claim','UnsupportedModel');row['package']['externally_authenticated']=True
 row=add('trusted-name-is-only-declared');row['package']['domain']['namespace']=text('trusted.publisher')
 for o in row['package']['owners']:o['anchor']['domain']['namespace']=text('trusted.publisher')
 return rows

def dag_cases():
 # Bounded read-only subgraph laws over registered scalar/ordinal types.
 # NOT a whole SourcePackage Scope encoder or T4 normalizer.
 from document_occurrence_registration_data import MODEL,PREFIX
 def n(name,body):return {'type':{'owner':MODEL,'name':name},'body':body}
 nodes=[n('$U64',{'scalar':{'uint64':2}}),n('$U64',{'scalar':{'uint64':10}}),n('DocumentSourceKey',{'scalar':{'bytes':[88]}}),n('DocumentOccurrenceKey',{'record':[{'field':'entry_ordinal','value':0}]}),n('DocumentOccurrenceKey',{'record':[{'field':'entry_ordinal','value':1}]})]
 base={'nodes':nodes,'roots':[2,3,4]};rows=[]
 def add(slug,value,expected=None):rows.append({'case_id':PREFIX+'dag-'+slug,'value':value,'expected_failure':expected,'classification':'Registered bounded scalar/ordinal subgraph validation; NOT full Scope or normalization'})
 add('shared-scalar-and-distinct-ordinals',copy.deepcopy(base))
 bad=copy.deepcopy(base);bad['nodes'].append(copy.deepcopy(nodes[2]));bad['roots'].append(5);add('duplicate-equal-node',bad,'InvalidFacts')
 bad=copy.deepcopy(base);bad['nodes'][0],bad['nodes'][1]=bad['nodes'][1],bad['nodes'][0];add('alternate-rank-order',bad,'InvalidFacts')
 bad=copy.deepcopy(base);bad['roots']=[3,4];add('unreachable-content',bad,'InvalidFacts')
 bad=copy.deepcopy(base);bad['nodes'][3]['body']['record'][0]['value']=3;add('self-cycle',bad,'InvalidFacts')
 bad=copy.deepcopy(base);bad['nodes'][3]['body']['record'][0]['value']=99;add('dangling-reference',bad,'InvalidFacts')
 bad=copy.deepcopy(base);bad['nodes'][2]['type']['name']='Fact';add('output-node-in-input',bad,'UnsupportedModel')
 bad=copy.deepcopy(base);bad['nodes'][2]['random_source_salt']=7;add('unknown-salt-field',bad,'UnsupportedModel')
 return rows
