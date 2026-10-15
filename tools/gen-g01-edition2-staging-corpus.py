"""Independent T1/T2 inputs/Expected from registered source declarations, never Rust Actual."""
from pathlib import Path
import importlib.util,json,copy,hashlib
from g01_registration_data import DATA
from g01_rule_dependencies import dependencies
from portable_proof_python_oracle import canonical_bytes,sha256
from edition2_registration_inputs import pinned as pinned_edition2
ROOT=Path(__file__).resolve().parent.parent
OUT=ROOT/"spec/conformance/g01-edition2-staging"
SOURCES={"OS":("agentflow.computation.proof-obligation.semantics@1","spec/computation/proof-obligation-semantics.md"),"RM":("agentflow.computation.proof-report.model@1","spec/tables/proof-report-model-v1.md"),"PP2":("agentflow.portable.proof.inputs.semantics@2","spec/portable/portable-proof-inputs-v2.md")}
def save(path,raw):
 if not path.is_file() or path.read_bytes()!=raw:
  raise ValueError("frozen Edition2 output mismatch;refusing overwrite: "+str(path))
def js(path,value):save(path,(json.dumps(value,ensure_ascii=False,sort_keys=True,indent=2)+"\n").encode())
def load_module(name,filename):
 spec=importlib.util.spec_from_file_location(name,ROOT/"tools"/filename);m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);return m
codec=load_module("e1_codec_input_builder","gen-portable-proof-codec-corpus.py")
spec_bytes=pinned_edition2("specifications");rule_bytes=pinned_edition2("rules")
specs=json.loads(spec_bytes);rules=json.loads(rule_bytes)
ids=sorted(codec.STATIC_SPECS+[x[0] for x in SOURCES.values()]+["agentflow.portable.proof.inputs.semantics@1"])
rule_ids=sorted(codec.STATIC_RULE_IDS+[f"AF-SEMANTIC-{n:04}" for n in list(range(173,188))+[189]+list(range(191,229))])
assert len(ids)==20 and len(rule_ids)==138
profile={"version":"agentflow.root-export-checker@2","spec_ids":ids,"rule_ids":rule_ids,"specification_bytes":spec_bytes,"rule_bytes":rule_bytes}
base=codec.bundle_and_context(profile)
assert len(base["compiler_semantics_manifests"][0]["content"]["normative_sources"])==46
cases=[]
def portable(slug,value,stage,family,raw=None):
 filename=slug+".ace1";save(OUT/"portable"/filename,raw if raw is not None else canonical_bytes(value))
 cases.append({"case_id":"g01-edition2-staging/"+slug,"kind":"portable_admission","file":"portable/"+filename,"expected_stage":stage,"expected_family":family,"expected_capability":False})
portable("new-complete-staging",base,"D4","UnsupportedSemantics")
old=codec.bundle_and_context()
portable("old-complete-rejected",old,"D4","UnsupportedSemantics")
# New inputs are built independently from fields/sources; no old Expected relabelling.
mixed_profile={**profile,"specification_bytes":codec.pinned_bytes("specifications"),"rule_bytes":codec.pinned_bytes("rules"),"spec_ids":codec.STATIC_SPECS,"rule_ids":codec.STATIC_RULE_IDS}
portable("new-label-old-content",codec.bundle_and_context(mixed_profile),"D4","InvalidIdentity")
portable("old-label-new-content",codec.bundle_and_context({**profile,"version":"agentflow.root-export-checker@1"}),"D4","InvalidIdentity")
portable("unknown-edition",codec.bundle_and_context({**profile,"version":"agentflow.root-export-checker@999"}),"D4","UnsupportedSemantics")
missing=copy.deepcopy(base);missing["compiler_semantics_manifests"][0]["content"]["normative_sources"].pop()
portable("missing-source",missing,"D3","IncompleteContent")
portable("noncanonical-space",base,"D1","InvalidCanonicalEncoding",b" "+canonical_bytes(base))
forbidden=copy.deepcopy(base);forbidden["edition_supported"]=True
# Inserting an unknown field deliberately violates the closed schema.
portable("caller-support-flag",forbidden,"D0","InvalidCanonicalEncoding",canonical_bytes(forbidden))
changed=copy.deepcopy(base)
blob=changed["compiler_semantics_manifests"][0]["content"]["registries"]["rules"]
import base64
raw=json.loads(base64.urlsafe_b64decode(blob["content"]+"="*(-len(blob["content"])%4)))
raw["registry_revision"]+=1
changed_raw=(json.dumps(raw,sort_keys=True,indent=2)+"\n").encode()
blob.update(codec.checksum_blob(changed_raw))
portable("changed-registry-same-label",changed,"D4","InvalidIdentity")
# Current identity bytes are independently assembled from registry/source fields.
def native_identity(sbytes,rbytes,version,sids,rids):
 s=json.loads(sbytes);r=json.loads(rbytes)
 source_rows=[]
 for row in s["specifications"]:
  if row["specification_id"] in sids:
   p=row["normative_source"]["logical_path"];raw=(ROOT/p).read_bytes()
   source_rows.append({"specification_id":row["specification_id"],"logical_path":p,"content_sha256":sha256(raw),"content_hex":raw.hex()})
 return {"analysis_semantics_version":version,"registry_hex":sbytes.hex(),"rule_registry_hex":rbytes.hex(),"specifications":source_rows,"rules":[codec.rule_identity(row) for row in r["rules"] if row["rule_id"] in rids]}
js(OUT/"compiler-current.json",native_identity(spec_bytes,rule_bytes,profile["version"],ids,rule_ids))
js(OUT/"compiler-legacy.json",native_identity(codec.pinned_bytes("specifications"),codec.pinned_bytes("rules"),"agentflow.root-export-checker@1",codec.STATIC_SPECS,codec.STATIC_RULE_IDS))
registration=[]
for row in DATA["rules"]:
 sid,path=SOURCES[row["source"]]
 registration.append({"case_id":row["case_id"],"kind":"registered_rule","rule_id":row["rule_id"],"specification_id":sid,"source":path,"anchor":row["anchor"],"statement":row["statement"],"dependencies":dependencies(row["rule_id"]),"executed_scope":"registration-and-staging-only","full_semantic_conformance":"NOT IMPLEMENTED;T3-T8 required"})
js(OUT/"registration.json",{"baseline":DATA["baseline"],"cases":registration,"new_specifications":[{"specification_id":sid,"path":path,"sha256":sha256((ROOT/path).read_bytes())} for sid,path in SOURCES.values()],"model_scope":"non-wire","expected_counts":{"specifications":46,"rules":436,"selected_sources":20,"applied_rules":138,"active_abi":775,"afidl_modules":17,"hash_domains":14}})
native_cases=[("current-identity",None),("new-label-old-native-content","InvalidIdentity"),("old-label-new-native-content","InvalidIdentity"),("old-identity","UnsupportedSemantics"),("unknown-native-edition","UnsupportedSemantics"),("missing-selected-source","IncompleteContent"),("changed-rule-native-content","InvalidIdentity"),("identity-resource-limit","ResourceLimit"),("unknown-report-shape","UnsupportedSemantics"),("unknown-obligation-edition","UnsupportedSemantics")]
js(OUT/"manifest.json",{"edition":profile["version"],"cases":cases,"case_count":len(cases),"registration_case_count":len(registration),"native_cases":[{"case_id":"g01-edition2-staging/"+name,"mutation":name,"expected_failure":failure,"expected_capability":False} for name,failure in native_cases],"expected_stage_state":{"registration":"Registered","support":"NotImplemented","activation":"NotAuthorized","release":"NotAuthorized"},"design_vectors_48_plus_16":"Not Executed"})
print("g01 Edition2 staging",len(registration),"registration cases,",len(cases),"portable cases;positive capability NOT AUTHORIZED")
