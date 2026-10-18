"""Allocation after G2-A..F review and recorded zero-collision G2-G census."""
BASELINE='8dc4f508c2f97b431053528ab14d2bd9263b3737'
OWNER='agentflow.root-export-checker@8'
MODEL='agentflow.computation.proof-report.model@7'
SHAPE='agentflow.g01.report-shape@7'
OBLIGATION='agentflow.g01.obligation@7'
PREFIX='g01-structural-binding-staging/'
ARTIFACT='phase-04c1b4c4r3c2r4g2-'
SOURCES={
 'SB':('agentflow.computation.structural-binding.semantics@1','schema_language','spec/computation/structural-binding.md','Independent rank-free source admission and pre-capture structural input binding contract; registration staging only.'),
 'OS':('agentflow.computation.proof-obligation.semantics@7','schema_language','spec/computation/proof-obligation-semantics-v7.md','Candidate input data versus admitted Subject and explicit immutable input / witness consumer environments.'),
 'RM':(MODEL,'closed_table','spec/tables/proof-report-model-v7.md','Closed structural sites, map body, witness edges, source/stage/failure declarations and exact frozen historical owners.'),
 'PP':('agentflow.portable.proof.inputs.semantics@8','schema_language','spec/portable/portable-proof-inputs-v8.md','Exact structural-binding successor contents and current-only non-activated staging.')}
CLAUSES=[
 ('SB','sites','Every eligible inline value site has one exact owner/parent/selector binding, complete descriptor and child types; missing or extra source-less sites reject.',['AF-SEMANTIC-0214','AF-SEMANTIC-0221'],['SB-01','SB-03','SB-05']),
 ('SB','nominal-order','Nominal types and site identities remain exact; aliases delegate declaration bodies without casts and type-order integration preserves all old ranks.',['AF-SEMANTIC-0215','AF-SEMANTIC-0256'],['SB-02','SB-06']),
 ('SB','map-body','Map uses typed key/value child entries with canonical typed order and duplicate canonical key rejection before interning; native module mapping is full and source-exact.',['AF-SEMANTIC-0214','AF-SEMANTIC-0215','AF-ACE1-0007'],['SB-04']),
 ('SB','canonical-ranks','Only the future pre-capture binder constructs bounded minimal input graphs under0217 full typed equality, canonical height/type/body/child ordering and exact roots.',['AF-SEMANTIC-0217','AF-SEMANTIC-0218','AF-SEMANTIC-0219'],['SB-06','SB-07']),
 ('SB','input-environment','G references resolve exact typed inputs in their fixed complete carrier; ordinary integers, other arenas, output backedges and synthetic roots never repair missing bindings.',['AF-SEMANTIC-0269','AF-SEMANTIC-0297'],['SB-08']),
 ('SB','witness-environment','W local/input edges are explicit; external G targets use full typed comparison without W-local height and complete envelope equality includes G and exact tuple.',['AF-SEMANTIC-0217','AF-SEMANTIC-0297'],['SB-08']),
 ('SB','source-stage','Complete rank-free declared inputs and genuine native source associations precede immutable G, actual C admission and complete Fact assembly; T4 cannot backfill input ranks.',['AF-SEMANTIC-0198','AF-SEMANTIC-0262','AF-SEMANTIC-0268','AF-SEMANTIC-0278'],['SB-09','SB-10','SB-13']),
 ('SB','failures','Structural diagnostics use closed applicable domain/reason/stage/source paths, preserve complete source and never become evaluated port predicates or partial successful graphs.',['AF-SEMANTIC-0218','AF-SEMANTIC-0219','AF-SEMANTIC-0225'],['SB-12']),
 ('OS','candidate-admission','Successor candidate Subject data may be bound before C but complete Fact use requires genuine source admission and every actual Some C Valid; failures preserve port events without Facts.',['AF-SEMANTIC-0295','AF-SEMANTIC-0297'],['SB-09','SB-11']),
 ('OS','consumer-propagation','Raw ScopeSubject declares its exact Subject input root and all affected Source/Target/Origin/Basis/Witness/Evidence/C references retain explicit G/W domains without output-dependent input identity.',['AF-SEMANTIC-0200','AF-SEMANTIC-0269','AF-SEMANTIC-0296'],['SB-03','SB-08','SB-13']),
 ('RM','closed-model','Independent source-derived site/field/variant/node rosters and two-sided omission rejection cover the closed current model; metadata success is not production capture.',['AF-SEMANTIC-0220','AF-SEMANTIC-0301'],['SB-15']),
 ('PP','edition-isolation','Structural binding uses exact successor source/rule/model/report/obligation contents with frozen previous records and current-only refusal of old or mixed identity.',['AF-SEMANTIC-0221','AF-SEMANTIC-0223','AF-SEMANTIC-0299'],['SB-14']),
 ('PP','non-activation','Registered structural staging leaves production binder, full Raw, Scope, normalizer and positive proof/publication/runtime unimplemented or unauthorized; default resource and bounded D4 gates remain separate.',['AF-SEMANTIC-0212','AF-SEMANTIC-0227','AF-SEMANTIC-0300'],['SB-16']),
]
RULES=[dict(source=s,anchor=a,statement=text,dependencies=deps,sb_clauses=sb,rule_id='AF-SEMANTIC-%04d'%(302+i),case_id=PREFIX+'rule-%04d'%(302+i)) for i,(s,a,text,deps,sb) in enumerate(CLAUSES)]
assert len(RULES)==13 and {s for r in RULES for s in r['sb_clauses']}=={'SB-%02d'%i for i in range(1,17)}
