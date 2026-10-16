"""Independent R2 registration/admission Expected from R1/declarations.

Never consumes Rust Actual. No Fact Seed/Scope identity is generated. The
model's own emitted rows are NOT copied to expected_case/expected_dimension.
"""
from pathlib import Path
import base64,copy,hashlib,importlib.util,json,re
from substitution_registration_data import OWNER,SHAPE,OBLIGATION,SOURCES,BASELINE
from portable_proof_python_oracle import canonical_bytes
from edition3_registration_inputs import pinned_bytes
ROOT=Path(__file__).resolve().parent.parent;OUT=ROOT/"spec/conformance/g01-successor-staging"
def sha(b):return hashlib.sha256(b).hexdigest()
def save(path,raw):
 path.parent.mkdir(parents=True,exist_ok=True)
 if path.exists() and path.read_bytes()==raw:return
 path.write_bytes(raw)
def js(path,v):save(path,(json.dumps(v,ensure_ascii=False,sort_keys=True,indent=2)+"\n").encode())
def load(name,path):
 s=importlib.util.spec_from_file_location(name,ROOT/"tools"/path);m=importlib.util.module_from_spec(s);s.loader.exec_module(m);return m
def source_profile(sbytes,rbytes,version,ids,rids):return dict(specification_bytes=sbytes,rule_bytes=rbytes,version=version,spec_ids=ids,rule_ids=rids)

def main():
 sbytes=pinned_bytes('specifications');rbytes=pinned_bytes('rules')
 specs=json.loads(sbytes);rules=json.loads(rbytes)
 e2s=(OUT/"edition2-specifications.json").read_bytes();e2r=(OUT/"edition2-rules.json").read_bytes()
 js(OUT/"edition2-input-pins.json",{"baseline":BASELINE,"specifications":sha(e2s),"rules":sha(e2r)})
 codec=load("successor_input_codec","gen-portable-proof-codec-corpus.py")
 old_extra=["agentflow.computation.proof-obligation.semantics@1","agentflow.computation.proof-report.model@1","agentflow.portable.proof.inputs.semantics@1","agentflow.portable.proof.inputs.semantics@2"]
 old_ids=sorted(codec.STATIC_SPECS+old_extra)
 old_rules=sorted(codec.STATIC_RULE_IDS+[f"AF-SEMANTIC-{n:04}" for n in list(range(173,188))+[189]+list(range(191,229))])
 ids=sorted(old_ids+[x[0] for x in SOURCES.values()])
 rids=sorted([x for x in old_rules if x not in {f"AF-SEMANTIC-{n:04}" for n in range(223,228)}]+[f"AF-SEMANTIC-{n:04}" for n in range(229,245)])
 profile=source_profile(sbytes,rbytes,OWNER,ids,rids)
 old_profile=source_profile(e2s,e2r,"agentflow.root-export-checker@2",old_ids,old_rules)
 def identity(p):
  ss=json.loads(p["specification_bytes"]);rr=json.loads(p["rule_bytes"])
  return {"analysis_semantics_version":p["version"],"registry_hex":p["specification_bytes"].hex(),"rule_registry_hex":p["rule_bytes"].hex(),"specifications":[{"specification_id":x["specification_id"],"logical_path":x["normative_source"]["logical_path"],"content_sha256":sha((ROOT/x["normative_source"]["logical_path"]).read_bytes()),"content_hex":(ROOT/x["normative_source"]["logical_path"]).read_bytes().hex()} for x in ss["specifications"] if x["specification_id"] in p["spec_ids"]],"rules":[codec.rule_identity(x) for x in rr["rules"] if x["rule_id"] in p["rule_ids"]]}
 js(OUT/"compiler-successor.json",identity(profile));js(OUT/"compiler-edition2.json",identity(old_profile))
 base=codec.bundle_and_context(profile);portable=[]
 def pc(slug,b,stage,family,raw=None):
  file="portable/"+slug+".ace1";save(OUT/file,raw if raw is not None else canonical_bytes(b))
  portable.append(dict(case_id="g01-successor-staging/"+slug,file=file,expected_stage=stage,expected_family=family,expected_capability=False))
 pc("complete-successor-denied",base,"D4","UnsupportedSemantics")
 pc("exact-edition2-rejected",codec.bundle_and_context(old_profile),"D4","UnsupportedSemantics")
 pc("edition2-label-new-content",codec.bundle_and_context({**profile,"version":"agentflow.root-export-checker@2"}),"D4","InvalidIdentity")
 pc("successor-label-old-content",codec.bundle_and_context({**old_profile,"version":OWNER}),"D4","InvalidIdentity")
 pc("unknown-owner",codec.bundle_and_context({**profile,"version":"agentflow.root-export-checker@999"}),"D4","UnsupportedSemantics")
 missing=copy.deepcopy(base);missing["compiler_semantics_manifests"][0]["content"]["normative_sources"]=[x for x in missing["compiler_semantics_manifests"][0]["content"]["normative_sources"] if x["specification_id"]!=SOURCES["RM"][0]]
 pc("missing-new-model-source",missing,"D3","IncompleteContent")
 pc("noncanonical-space",base,"D1","InvalidCanonicalEncoding",b" "+canonical_bytes(base))
 fake=copy.deepcopy(base);fake["supported"]=True;pc("caller-support-flag",fake,"D0","InvalidCanonicalEncoding")
 fake=copy.deepcopy(base);fake["activation_requested"]=True;pc("caller-activation-flag",fake,"D0","InvalidCanonicalEncoding")
 changed=copy.deepcopy(base);blob=changed["compiler_semantics_manifests"][0]["content"]["registries"]["rules"]
 rr=json.loads(rbytes);rr["registry_revision"]+=1;blob.update(codec.checksum_blob((json.dumps(rr,sort_keys=True,indent=2)+"\n").encode()))
 pc("changed-registry-same-label",changed,"D4","InvalidIdentity")

 # Independent literal/accepted-R1 model expectations,not emitted descriptor rows.
 role_doc=(ROOT/"docs/implementation/phase-04c1b4c4r1-role-origin-prerequisites.md").read_text(encoding="utf-8")
 cases=re.search(r"Closed family standalone_substitution cases \(exact declaration order\): ([^.]+)\.",role_doc)[1].split(',')
 dims="identity io failure effects resources information-flow usage recovery execution execution-feasibility execution-security cross-dimension".split()
 local="lineage_equality io_relation failure_relation effects_relation resources_relation flow_relation usage_relation recovery_relation execution_combined_relation execution_feasibility_relation execution_security_relation".split()
 expected=[]
 for rank,c in enumerate(cases):
  context=c=="context_association";result=c.startswith("result_")
  expected.append(dict(case_id="g01-successor-staging/case-"+c,expected_case={
   "family":"standalone_substitution","case":c,"rank":rank,"kind":"proof_context" if context else "substitution_evidence",
   "requirement":{"family":"direct" if context else "substitution","clause":"proof_context" if context else c,"evidence_class":"static_declaration"},
   "stage":"standalone_substitution","role":"mandatory_preflight" if c in ("checked_operand","context_association") else "static_dimension" if result else "supporting_fact",
   "applicability":"all valid subjects" if c in ("checked_operand","context_association") or result else "context Proven" if c=="whole_body_equality" else "context Proven and body changed",
   "source_type":"Source","target_type":"Target","coordinate_type":"Coordinate","origin_type":"Origin",
   "source_variant":"contract_operand" if result or c in local or c=="checked_operand" else "substitution_application",
   "target_variant":"contract_operand" if result or c in local or c=="checked_operand" else "substitution_pair",
   "operand_roles":["old","replacement"] if c=="checked_operand" else ["replacement","old"],
   "assertion_path":"known-empty","origin_path":"ApplicationDocument.applications[index];old/replacement field only for checked consumption",
   "supporting_relations":["actual original validator facts"] if c=="checked_operand" else ["checked_operand(old)","checked_operand(replacement)"]+([] if context else ["context_association"]),
   "status_rules":"frozen three-branch matrix;local and final judgments are distinct;no invented RequiresEvidence",
   "evaluation":"context false:blocked result;body equal:reflexive result;changed:actual evaluated candidate/fold",
   "witness":"exact typed witness iff Violation;never recover from reason","ordering":"rank;full typed payload and source/application origin","governing_rule":"AF-CONTRACT-0020"}))
 js(OUT/"case-registration.json",{"authority":"accepted R1 roster and matrix;no Rust Actual","cases":expected})
 paths=[['component_identity'],['inputs','outputs'],['failures'],['effects'],['resources','component_identity'],['information_flow','effect_flow_partitions'],['usage','recovery_usage_bindings'],['recovery','recovery_action_bindings'],['execution'],['execution'],['execution'],['component_identity','inputs','outputs','failures','effects','resources','information_flow','usage','recovery','execution','effect_flow_partitions','recovery_action_bindings','recovery_usage_bindings']]
 coords="lineage io failure effects resources flow usage recovery execution execution execution whole_pair".split()
 decision_text=[
  "exact catalog AND name equality:P;otherwise:U;never:V",
  "equal I/O port sets and pins:P;added required input or missing old output:V;otherwise:U",
  "new nominal failure set subset old:P;otherwise:V",
  "existing exact effect containment:P or typed U;never manufacture V",
  "existing non-strengthening Ok:P/U;strengthened error:V;other error:U",
  "flow_relaxed error:V;Ok(P) and partitions equal:P;otherwise:U",
  "equal usage and profiles:P;otherwise existing usage_not_increased P/V/U;Err:U;no extra fallback profile gate",
  "K equivalent and action bindings equal:P;Weakened:V;otherwise:U",
  "either aspect U:U;else either V:V;else:P;shared Err:U",
  "existing feasibility P/V/U;shared Err:U","existing security P/V/U;shared Err:U",
  "outer fold of eleven distinct non-cross final results;V>RE>U>P;no self edge"]
 subordinate=[[],[],['AF-FAILURE-0008'],['AF-EFFECT-0009'],['AF-RESOURCE-0012','AF-RESOURCE-0013','AF-RESOURCE-0018'],['AF-INFOFLOW-0017'],['AF-USAGE-0013'],['AF-RECOVERY-0013'],['AF-EXECUTION-0014'],['AF-EXECUTION-0012'],['AF-EXECUTION-0013'],[]]
 dimension_cases=[]
 for i,d in enumerate(dims):
  expected_dimension=dict(dimension=d,rank=i,case="result_"+d.replace('-','_'),local_predicate=local[i] if i<11 else None,
   old_operand_paths=["old.contents.contract."+p for p in paths[i]],replacement_operand_paths=["replacement.contents.contract."+p for p in paths[i]],governing_rule="AF-CONTRACT-0020",source_anchor="component-contract-substitution",subordinate_rules=subordinate[i],role="static_dimension",applicability="every valid application;blocked under failed context",evaluation="blocked|exact-body-reflexive|changed-body-evaluated",candidate_rules=decision_text[i],floor="Unproven",result_fold="outer max(Unproven,local)" if i<11 else "outer fold eleven distinct non-cross results",source="replacement checked operand",target="old checked operand",coordinate=coords[i],witness="SubstitutionWitness exact applicable variant only",error_basis="UnprovenBasis;full known domain/kind/payload;unknown UnsupportedModel",dependencies=["checked_operand(old)","checked_operand(replacement)","context_association"]+(["changed_pair_evidence_floor",local[i]] if i<11 else ["eleven distinct final results;no self edge"]))
  dimension_cases.append(dict(case_id="g01-successor-staging/dimension-"+d,expected_dimension=expected_dimension))
 js(OUT/"dimension-registration.json",{"cases":dimension_cases})
 error_cases=[]
 for domain in ("resource","flow","usage","recovery","execution"):
  text=(ROOT/f"crates/af-{domain}/src/error.rs").read_text(encoding="utf-8")
  for constant,kind in re.findall(r'pub const (\w+): &str = "([^"]+)";',text):
   error_cases.append(dict(case_id="g01-successor-staging/error-"+domain+"-"+kind,domain=domain,kind=kind,constant=constant,probe_type=domain+"ErrorKind",value=kind))
 js(OUT/"error-registration.json",{"cases":error_cases})
 shape_cases=[]
 def sc(slug,ty,value,failure=None):shape_cases.append(dict(case_id="g01-successor-staging/shape-"+slug,type=ty,value=value,expected_failure=failure))
 sc("unknown-case","SubstitutionCase","future_case","UnsupportedModel")
 sc("roles-shared","ContextRolePartition",[["analysis","old_checked","replacement_checked"]])
 sc("roles-distinct","ContextRolePartition",[["analysis","replacement_checked"],["old_checked"]])
 sc("roles-missing","ContextRolePartition",[["analysis","replacement_checked"]],"IncompleteFacts")
 sc("roles-duplicate","ContextRolePartition",[["analysis","old_checked","replacement_checked","old_checked"]],"IncompleteFacts")
 sc("alias-conflict","ContextRolePartition",[["analysis"],["old_checked"],["replacement_checked"]],"InvalidFacts")
 sc("unknown-model","not_registered",{},"UnsupportedModel")
 sc("unknown-role","OperandRole","caller","UnsupportedModel")
 sc("option-none","Source",{"contract_operand":{"role":"old"}},"IncompleteFacts")
 sc("ordinal-two","$U64",2);sc("ordinal-ten","$U64",10)
 sc("nominal-u32-overflow","ComputationId",2**32,"UnsupportedModel")
 sc("raw-utf8-claim","Raw($Text)",[101,204,129])
 sc("nfd-checked-text","$Text","e\u0301","UnsupportedModel")
 for domain in ("resource","flow","usage","recovery","execution"):
  sc("unknown-error-"+domain,domain+"ErrorKind","future_error","UnsupportedModel")
 sc("domain-kind-conflict","DependencyErrorPayload",{"domain":"resource","kind":{"usage":"unknown_usage_meter"},"detail":"diagnostic"},"InvalidFacts")
 sc("missing-error-detail","DependencyErrorPayload",{"domain":"resource","kind":{"resource":"unknown_resource_type"}},"IncompleteFacts")
 sc("unknown-model-ref","ModelRef",{"owner":SOURCES['RM'][0],"name":"not_registered"},"UnsupportedModel")
 sc("wrong-option-tag","ProofVerdict",{"some":"Proven"},"UnsupportedModel")
 js(OUT/"shape-registration.json",{"cases":shape_cases,"meaning":"metadata shape only;NOT trusted Fact Seed"})
 rule_cases=[]
 for n in range(229,245):
  row=next(x for x in rules["rules"] if x["rule_id"]==f"AF-SEMANTIC-{n:04}")
  rule_cases.append(dict(case_id=f"g01-successor-staging/rule-{n:04}",rule_id=row['rule_id'],normative_source=row['normative_source'],anchor=row['source_anchor'],statement=row['short_statement'],scope="registered-staging-only;full semantic laws NOT EXECUTED"))
 js(OUT/"rule-registration.json",{"cases":rule_cases})
 native=[("current",None),("exact-edition2","UnsupportedSemantics"),("edition2-label-new","InvalidIdentity"),("successor-label-old","InvalidIdentity"),("missing-model-source","IncompleteContent"),("changed-model-content","InvalidIdentity"),("unknown-owner","UnsupportedSemantics"),("unknown-shape","UnsupportedSemantics"),("unknown-obligation","UnsupportedSemantics"),("resource-limit","ResourceLimit")]
 native=[dict(case_id="g01-successor-staging/native-"+k,mutation=k,expected_failure=v) for k,v in native]
 profile_cases=[dict(case_id="g01-successor-staging/profile-"+p+("-root" if root else ""),profile=p,as_root=root,expected_failure="UnsupportedModel" if p=="future" or (p=="standalone_substitution" and root) else "NotImplemented") for p,root in [("standalone_substitution",False),("standalone_substitution",True),("root_export",True),("contract_validation",False),("computation",False),("future",False)]]
 all_ids=[x['case_id'] for group in [expected,error_cases,shape_cases,rule_cases,native,portable,profile_cases] for x in group]+["g01-successor-staging/dimension-"+d for d in dims]
 assert len(all_ids)==len(set(all_ids))
 js(OUT/"manifest.json",{"kind":"successor registered staging;NOT Typed Fact Capture","edition":OWNER,"shape":SHAPE,"obligation":OBLIGATION,"case_ids":all_ids,"case_count":len(all_ids),"portable_cases":portable,"native_cases":native,"profile_cases":profile_cases,"selected_specifications":ids,"applied_rule_ids":rids,"r1_semantic_vectors":"48 Not Executed","sources":[dict(specification_id=sid,path=path,sha256=sha((ROOT/path).read_bytes())) for sid,_,path,_ in SOURCES.values()]})
 print("successor staging",len(all_ids),"distinct CaseIds;registration/shape/admission only")
if __name__=="__main__":main()
