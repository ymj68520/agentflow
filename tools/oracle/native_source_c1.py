"""Independent C1 native-source NFC admission/semantic identity audit.

Not a repair and not full Seed conformance. Unchanged original/F1 Oracles
remain read-only. Expected comes from closed nfc_text and ACE1 normalization.
"""
import copy,importlib.util,json,re,unicodedata
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('c1_frozen_native',ROOT/'tools/oracle/native_guard_capture.py')
NG=importlib.util.module_from_spec(spec);spec.loader.exec_module(NG)

def nfc_tree(x):
    if isinstance(x,str):return unicodedata.normalize('NFC',x)
    if isinstance(x,dict):return {nfc_tree(k):nfc_tree(v) for k,v in x.items()}
    if isinstance(x,list):return [nfc_tree(v) for v in x]
    return x

def vectors():
    out=[]
    for name,action in [('nfc-control','\u00e9'),('nfd-action','e\u0301')]:
        c=NG.base('string');entries=c['effects']['row']['closed']
        entries[0]['guard']=NG.condition('string','\u00e9')
        entries[1]['guard']=NG.condition('string',action)
        out.append({'case_id':'c1-native-source/'+name,'schema_primitive':'string',
                    'raw_declaration':c,'rust_input':NG.rust_input(c),'canonical_admission':True})
    return out

def expected(v):
    # Frozen checker diagnostics operate on their genuine physical source.
    # The registered source domain is checked separately; encoding equivalence
    # is not permission to replace that source after the judgment was made.
    c=NG.CC.canonical_contract(v['raw_declaration']);trace=NG.expected(v)
    # This field is explicitly the parsed ACE1 bytes in the diagnostic
    # projector; keep the separate physical source and raw-kernel operands.
    trace['canonical_declaration']=json.loads(bytes.fromhex(trace['canonical_bytes_hex']))
    for run in trace['runs']:
        witness=run['record']['countermodel']
        if witness is not None:
            # The inherited *diagnostic* projector renders this contract
            # through ACE1 too. The physical offending guard remains explicit
            # above and is rejected before formal capture; this is not a
            # claim of lossless witness capture for the rejected source.
            witness['source_contract']=nfc_tree(witness['source_contract'])
    admitted=all(unicodedata.is_normalized('NFC',e['guard']['input_equals']['expected']['string']) for e in c['effects']['row']['closed'])
    return {'stored_guards':[e['guard'] for e in NG.wire_numbers(c)['effects']['row']['closed']],
            'observation':trace,'capture_admitted':admitted,'capture_failure':None if admitted else 'invalid_facts',
            'canonical_source_status':'proven' if admitted else None,'root_export_capability':False}

def compare(actual):
    model=json.loads(re.search(r'```json\s*(.*?)\s*```',(ROOT/'spec/tables/proof-report-model-v3.md').read_text(encoding='utf-8'),re.S)[1])
    owner='agentflow.computation.proof-report.model@1/'
    assert model['types'][owner+'$String']=={'kind':'nfc_text'}
    primitive=model['types'][owner+'af-semantic::PrimitiveConstant']
    assert dict(primitive['variants'])['String']['fields']==[['0',{'ref':owner+'$String'}]]
    cases={v['case_id']:v for v in vectors()};assert len(actual)==2
    differences=[];matched=0
    for row in actual:
        want=expected(cases[row['case_id']]);got={k:copy.deepcopy(row[k]) for k in want}
        for run in got['observation']['runs']:run['record'].pop('reason')
        d=NG.first_difference(want,got)
        if d:differences.append({'case_id':row['case_id'],'first_differing_field':d,'expected':want,'actual':row})
        else:matched+=1
    return {'kind':'IndependentNativeSourceNfcAdmissionAudit;not full Seed Oracle','outcome':'ADMISSION_MISMATCH' if differences else 'SOURCE_ADMISSION_MATCH',
            'source_head':'98c7f0a8df1da880da104d0f322aa0727d0bcfb4','executed':2,'matched':matched,
            'unexpected_mismatch':len(differences),'differences':differences,
            'equal_canonical_bytes':actual[0]['observation']['canonical_bytes_hex']==actual[1]['observation']['canonical_bytes_hex'],
            'full_seed_conformance':False,'positive_root_export':'NotAuthorized'}

def main():
    import sys
    if sys.argv[1:]==['--vectors']:print(json.dumps(vectors(),ensure_ascii=True));return
    report=compare(json.load(sys.stdin));print(json.dumps(report,ensure_ascii=True))
    if report['unexpected_mismatch'] and sys.argv[1:]!=['--report']:sys.exit(1)
if __name__=='__main__':main()
