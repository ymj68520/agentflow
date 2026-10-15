"""R2 static normative materializer; no Fact producer, encoder or normalizer."""
import copy,hashlib,json,re,sys
from pathlib import Path
from closed_g01_model import Graph,parse_defs,enum,UNIT
from substitution_registration_data import *
from substitution_type_declarations import declarations
ROOT=Path(__file__).resolve().parent.parent

def load_model(path):
 text=path.read_text(encoding="utf-8")
 return json.loads(re.search(r"```json\s*(.*?)\s*```",text,re.S)[1])

def errors():
 out={}
 for domain in ("resource","flow","usage","recovery","execution"):
  p=ROOT/f"crates/af-{domain}/src/error.rs";raw=p.read_bytes();text=raw.decode()
  out[domain]=[dict(constant=a,kind=b,payload={"kind":"closed_domain_case","detail":"diagnostic_text"},nominal_identity=f"af-{domain}::{a}",source=p.relative_to(ROOT).as_posix(),source_sha256=hashlib.sha256(raw).hexdigest(),applicability="typed dependency failure;only existing dispatcher branches can select Violation",unknown="UnsupportedModel") for a,b in re.findall(r'pub const (\w+): &str = "([^"]+)";',text)]
 assert [len(x) for x in out.values()]==[12,11,11,13,22]
 return out

def model():
 legacy=load_model(ROOT/"spec/tables/proof-report-model-v1.md");err=errors()
 defs=declarations(legacy,err)
 families="root_preflight root_dimension execution_child root_boundary leaf control root_recovery prerequisite".split()
 defs["PredicateFamily"]=enum([(x,UNIT) for x in families+["standalone_substitution"]])
 g=Graph(legacy,defs,ROOT)
 graph=g.finish(list(defs))
 return dict(model_id=SOURCES["RM"][0],model_edition=SHAPE,obligation_edition=OBLIGATION,analysis_owner=OWNER,
  language={"record":"exact named fields;semantic declaration order;ACE1 UTF8 field-name output order","enum":"closed ranked variants;unit enums use case token;payload variants use existing single-case-object ACE1","option":"explicit {none:null} or {some:payload};bare null is invalid","sequence":"declared order preserved","set":"complete typed uniqueness;conflicting payloads reject","map":"unique typed keys/full values","dag_ref":"uint64 rank of exact target descriptor;not pointer","raw":"text claim positions become bytes,actual bytes/numerics/options unchanged","unknown":"UnsupportedModel;no reflection or opaque fallback"},
  descriptor_roots=list(defs),types=graph,
  inherited_registered_model=legacy,
  dependency_errors=err,closed_predicates={**legacy["closed_predicates"],"standalone_substitution":CASES},
  bounds=legacy["bounds"],profile="standalone_substitution",registration_only=True,
  producer_status="NotImplemented",normalizer_status="NotImplemented",lossless_scope_status="NotImplemented",positive_capability="NotAuthorized",
  case_descriptors=[case_descriptor(i,c) for i,c in enumerate(CASES)],dimension_descriptors=[dimension_descriptor(i,d) for i,d in enumerate(DIMENSIONS)],
  canonical_constraints=successor_constraints(legacy)+[
   "The successor profile uses the registered appended ranks and exact source/content tuple;no old edition is relabelled.",
   "Standalone assertion_path is exactly empty;ApplicationOrigin binds actual applications[index] input path,not report fields.",
   "Context-role partition retains all three roles exactly once and actual native association;equal content is not shared identity.",
   "Kind,clause,role,dimension,source,target,coordinate and all optional fields must match the closed case descriptor.",
   "Scope/ApplicationDocument input DAG excludes Fact,Obligation,Scope,Report and Checked capability descendants;FactKey links are outside input Scope.",
   "NonaggregateSubstitutionWitness excludes aggregate;Violation requires a valid typed actual source witness.",
   "DependencyErrorPayload.domain agrees with its domain-tagged kind;detail is diagnostic only and never witness input.",
   "Metadata shape admission does not establish validator-origin facts,shared native context,CompleteAndValid,or a checked capability."])

def successor_constraints(legacy):
 # Historical prose remains byte-exact inside inherited_registered_model. The
 # active successor constraints must not claim the old closed roster/counts.
 out=[]
 for value in legacy["semantic_constraints"]:
  if value.startswith("Kind ranks1"):
   value+=" Successor appends substitution_evidence at rank26;old ranks remain unchanged."
  elif value.startswith("Dimension ="):
   value+=" Successor appends substitution:SubstitutionDimension after supporting;its twelve cases use the registered numeric order."
  elif value.startswith("Requirement={"):
   value+=" Successor appends substitution:SubstitutionClause with exactly the26 registered non-context cases;context_association remains direct:proof_context."
  elif value.startswith("PredicateFamily ="):
   value+=" For this successor the family roster additionally appends standalone_substitution with exactly the27 case_descriptors;the preceding closed roster describes inherited families only."
  elif value.startswith("FullSupportManifest="):
   value=value.replace("all46 source bindings here,20 selected specifications and138 applied rules", "all50 source bindings here,24 selected specifications and149 applied rules").replace("hidden 46->20 content reduction","hidden 50->24 content reduction")
  elif value.startswith("Context={"):
   value="Inherited root/computation Context only (not standalone contract context): "+value+" Standalone uses ContractProofContextContents with its exact nine fields and no resolver_dependencies."
  elif value.startswith("ContextPartition="):
   value+=" Standalone uses its separate ContextRolePartition and ordered SubstitutionContextRole analysis0,old_checked1,replacement_checked2;analysis aliases replacement_checked."
  elif value.startswith("ModelType={"):
   value=value.replace("or RM@1", "or the exact inherited RM@1 or successor RM@2")
  out.append(value)
 return out

def case_descriptor(rank,case):
 context=case=="context_association";result=case.startswith("result_")
 return dict(family="standalone_substitution",case=case,rank=rank,
  kind="proof_context" if context else "substitution_evidence",
  requirement={"family":"direct" if context else "substitution","clause":"proof_context" if context else case,"evidence_class":"static_declaration"},
  stage="standalone_substitution",role="mandatory_preflight" if case in ("checked_operand","context_association") else "static_dimension" if result else "supporting_fact",
  applicability="all valid subjects" if case in ("checked_operand","context_association") or result else "context Proven" if case=="whole_body_equality" else "context Proven and body changed",
  source_type="Source",target_type="Target",coordinate_type="Coordinate",origin_type="Origin",
  source_variant="contract_operand" if result or case in LOCAL or case=="checked_operand" else "substitution_application",
  target_variant="contract_operand" if result or case in LOCAL or case=="checked_operand" else "substitution_pair",
  operand_roles=["old","replacement"] if case=="checked_operand" else ["replacement","old"],
  assertion_path="known-empty",origin_path="ApplicationDocument.applications[index];old/replacement field only for checked consumption",
  supporting_relations=["actual original validator facts"] if case=="checked_operand" else ["checked_operand(old)","checked_operand(replacement)"] + ([] if context else ["context_association"]),
  status_rules="frozen three-branch matrix;local and final judgments are distinct;no invented RequiresEvidence",
  evaluation="context false:blocked result;body equal:reflexive result;changed:actual evaluated candidate/fold",
  witness="exact typed witness iff Violation;never recover from reason",ordering="rank;full typed payload and source/application origin",governing_rule="AF-CONTRACT-0020")

def dimension_descriptor(rank,d):
 return dict(dimension=d,rank=rank,case="result_"+d.replace("-","_"),local_predicate=LOCAL[rank] if rank<11 else None,
  old_operand_paths=["old.contents.contract."+x for x in OPERANDS[d]],replacement_operand_paths=["replacement.contents.contract."+x for x in OPERANDS[d]],
  governing_rule="AF-CONTRACT-0020",source_anchor="component-contract-substitution",subordinate_rules=SUBORDINATE[d],
  role="static_dimension",applicability="every valid application;blocked under failed context",evaluation="blocked|exact-body-reflexive|changed-body-evaluated",
  candidate_rules=DECISIONS[rank],floor="Unproven",result_fold="outer max(Unproven,local)" if rank<11 else "outer fold eleven distinct non-cross results",
  source="replacement checked operand",target="old checked operand",coordinate=COORDINATES[rank],
  witness="SubstitutionWitness exact applicable variant only",error_basis="UnprovenBasis;full known domain/kind/payload;unknown UnsupportedModel",
  dependencies=["checked_operand(old)","checked_operand(replacement)","context_association"]+(["changed_pair_evidence_floor",LOCAL[rank]] if rank<11 else ["eleven distinct final results;no self edge"]))

def main():
 m=model();raw=json.dumps(m,ensure_ascii=False,separators=(",",":"))
 print("MATERIALIZED",len(m["types"]),"types",len(raw.encode()),"JSON bytes",len(m["case_descriptors"]),"cases",len(m["dimension_descriptors"]),"dimensions")
 if "--audit" in sys.argv:return
 if "--emit" not in sys.argv:raise SystemExit("Pass --audit or --emit")
 emit(m)

def write_new(path,raw):
 p=ROOT/path;p.parent.mkdir(parents=True,exist_ok=True)
 if p.exists() and p.read_bytes()==raw:return
 p.write_bytes(raw)

def emit(m):
 archive=ROOT/"target/phase-04c1b4c4r2-edition2/source"
 # Exact old input snapshots,not successor Expected or replacement old rows.
 for name in ("specifications","rules"):
  old=(archive/f"spec/registry/{name}.json").read_bytes()
  write_new(f"spec/conformance/g01-successor-staging/edition2-{name}.json",old)
 oldspec=json.loads((archive/"spec/registry/specifications.json").read_bytes())["specifications"]
 oldrules=json.loads((archive/"spec/registry/rules.json").read_bytes())["rules"]
 assert not ({r["specification_id"] for r in oldspec}&{x[0] for x in SOURCES.values()})
 assert not ({r["rule_id"] for r in oldrules}&{x["rule_id"] for x in RULES})
 policy=(archive/"spec/portable/portable-proof-inputs-v2.md").read_text(encoding="utf-8")
 assert OWNER not in policy and SHAPE not in policy and OBLIGATION not in policy
 common="Status: normative, registered non-wire semantics. Registered is not Supported or Activated. R3 producers, T4 normalizer, G02 full lossless Scope, independent full oracle and T8 activation are NOT IMPLEMENTED/NOT AUTHORIZED by this staging. No AFIDL, ABI, ACE1 grammar or HashV1 domain changes.\n\n"
 bodies={
 "OS": """## Profile authority and applicability

For this successor only,the exact model is agentflow.computation.proof-report.model@2. Old model/source meanings remain frozen under their old editions. All existing object/authority,complete Scope,origin,derivation,status/reference/prerequisite and bounded-failure principles are retained. The complete root_export roster remains five mandatory core checks plus nine dimensions and six independent boundaries. The new standalone_substitution profile is a distinct tagged non-root fragment. It cannot issue CheckedRootExportCompatibility,even for equal complete checked contracts. No diagnostic report or successful schema check establishes validator facts or authority.

Root/controller/leaf source meanings are unchanged;successor Fact adds the registered optional substitution_basis which is Some exactly for this new family. Full status,typed basis,witness,support,reference,observation and consumer content remain required semantic payload. A local fact differs from a conservative result judgment;no status-max repair merges conflicting same-origin facts. Scope input DAG contains only declared inputs,not report/fact/capability descendants. Actual full-scope binding and Canonical ObligationIdentity production remain unimplemented.

""",
 "SUB": """## Exact subject

Substitution compares replacement then old,as the frozen check_substitution signature does. Complete ComponentIdentity has exactly catalog/name;it is not a contract/schema/IR digest. Both ordered subjects include all13 canonical contract fields,canonical bytes,existing complete pins,actual validator source and context bindings. No caller IR,ComputationId,RootExportAssertion,workflow root or resolver instance is required. ContractProofContextContents has exactly the nine actual contract context components;it does not silently instantiate a root resolver. Full contents and the three-role partition are required,never counts/checksums or native addresses.

Analysis aliases replacement_checked.context;actual in-process shared context with old is a mandatory premise. Native pointer identity is preserved. Equal contents do not prove the same checked object. Missing source/application/context/coordinate is an explicit failure,not an empty trusted collection.

## Closed profile and branch semantics

The case_descriptors and dimension_descriptors in agentflow.computation.proof-report.model@2 are normative,including every source/target/coordinate,role,class,stage,applicability,rule,dependency,witness and rank. Exactly27 cases,26 non-context clauses and12 result dimensions are defined. checked_operand is parametrized by old/replacement,not two new arbitrary cases. Context uses direct(proof_context);all other clauses use substitution(the exact closed case). Unknown case/type/clause/role fails UnsupportedModel.

Branch C:after valid checked sources/application,failed context produces an evaluated Unproven association and12 blocked_by(context) results with legacy U placeholders;no comparison was evaluated. Branch B:valid context and complete canonical body equality produce12 Proven reflexive results without invented local calls. Branch M:changed body retains the Unproven floor and all eleven pre-arbitration local candidate judgments;each non-cross result is outer_max(U,local),P<U<RequiresEvidence<V. Cross folds11 distinct non-cross final results and never itself. The current comparator has no reachable RequiresEvidence candidate;the four-value verdict type is unchanged.

Local lineage_equality is evaluated P on complete catalog/name equality and U otherwise,never V. It is a different predicate from result_identity,which remains U on changed-body paths even if local lineage is P. Execution has one actual comparison;its two aspects remain separate and combined U dominates V,unlike the outer fold. An Err yields actual unavailable/fallback U judgments sharing the exact typed error,not three invented successful calls. Usage first checks exact usage AND recovery-profile equality;on fallback it uses the existing usage relation without a newly imposed profile-equality gate. Recovery's P branch additionally requires exact action bindings;Weakened maps V,other unsupported differences U. Every remaining relation is the exact per-dimension registered matrix.

## Origin and evidence

ApplicationOrigin binds the full declarative ApplicationDocument and real applications[index] path. checked_operand consumption adds only the actual old/replacement field;other origins end at the application declaration and use their closed PredicateId/Dimension. No fictional report results/comparisons/operands field,UUID,time,probe ID,Rust line/function,call counter or push order is origin. assertion_path is known empty for this non-root profile. Distinct declared entries retain distinct origins;copies add observations only. Conflicting same-origin payloads reject.

Consumption premises use standalone_substitution stage;their actual original validation supporting facts retain original stage/path. Missing old validation history is never invented;future adapters must fail or perform explicitly fresh instrumented validation under each operand's own original context without erasing a context mismatch. Prerequisites retain applicability,status,source/target/coordinate/rule/requirement/origin,supporting fact and all consumers. Reference is the complete typed operand/application/member coordinate,not a RuleId or parsed diagnostic string;derive the full set and typed minimum primary,without a root-boundary exception.

## Basis,witness and error obligations

The model completely declares SubstitutionBasis,UnprovenBasis,SubstitutionWitness and the parent Fact,TypedContradiction,ObservationSite extensions. Every V requires the exact applicable structured witness at the original branch. Lineage mismatch,effect unprovability,changed nominal resource type,unknown influence/meter/relation or unsupported binding alone do not become V. Aggregate witnesses refer to real violating supporting facts,not themselves;nonaggregate witness excludes aggregate.

All69 known error kinds from the five frozen source domains are explicitly materialized with nominal domain/variant,payload,source binding,applicability and unknown behavior. Error detail is required diagnostic text,not identity or a source of countermodels. Only the existing resource-strengthened/flow-relaxed Err branches select V;other errors U. Unknown kind is UnsupportedModel;missing required payload IncompleteFacts. No UnknownError(String),Other(String),layout reflection or reason parser is permitted.

""",
 "RM": """## Closed descriptor language and canonical identity

This single JSON block materializes the complete reachable logical descriptor graph,not a runtime reflection plan. Named refs must resolve;records have exactly every ordered field;enums/variants have closed full payloads;options use explicit none:null/some tags;sets/maps retain full typed key/value semantics. Native lifetimes/borrows denote owned logical content,not addresses. Imported frozen declarations remain complete and source-bound. All successor field/type/rank identities are distinct from unchanged old editions.

ACE1 is unchanged:record fields emit unsigned UTF8 name order;enums use their exact names and variants single-case objects;option None is {\"none\":null},Some is {\"some\":payload},never bare null. Numerics keep nominal widths and exact values;2 precedes10 by numeric ordering,not text. NFC and raw UTF8-byte claim positions are distinct;bytes use existing canonical byte form. Sets reject duplicate complete identity/conflicting payload;sequences preserve declared order. Only identical complete origins may unite observations in the future normalizer.

Canonical value DAGs are finite,minimal,reachable,acyclic and bottom-up interned by complete typed values,ranked by height/type/nonrecursive payload/ordered children. No rank/hash-only shortcut replaces semantic typed equality. Scope excludes fact/obligation/report/capability descendants and has no recursive digest equation. All count/depth/work/output limits intersect existing earlier bounds;overflow/resource excess fails with no partial success or capability. Encoding,normalization,complete-scope binding and witness production are not implemented by descriptor registration or metadata shape validation.

## Closed Descriptor Table

""",
 "PP": f"""## Allocated exact identity

Owner:{OWNER};report shape:{SHAPE};obligation:{OBLIGATION}. Allocation was checked against the exact starting registry/policies and existing positive semantic-major naming rules;it is not an assumed automatic next number. Full identity binds complete old+new registered source contents,the whole actual specification/rule registry bytes,the complete model graph,all closed tables,selected source contents,actual applied rule descriptors and exact support/role policy. Labels/counts/checksums alone do not establish equality. HashV1/digest meanings are unchanged.

For owner@3 only,this policy supersedes owner@2-specific AF-SEMANTIC-0223–0227 admission/governance. Those exact old rows/sources remain historical authority,not current applied policies. Other applicable common invariants remain;the applied set is recomputed from actual registration and remains explicit attribution,not complete transitive dependency closure (F03 OPEN/P2). Full defined semantic roles bind the successor-selected sources and unchanged AFIDL pins;lowering/runtime remain model_missing:not_formalized.

## Controlled states and failure priority

Registered,Supported,Activated and release are distinct implementation-owned states. The successor is Registered/NotImplemented/NotAuthorized/release NotAuthorized. Direct analyzer,native checker,sole private Checked constructor and Portable D4 independently deny full positive capability. No caller supported/activation flag,standalone fragment,legacy report or successful model-shape check creates it. R3,T4,G02,G03,T6–T8 remain separate prerequisites and authorizations.

Bounds/canonical closed decoding precedes exact content and support. Resource excess=>ResourceLimit;noncanonical ACE1=>InvalidCanonicalEncoding;missing required source/descriptor=>IncompleteContent;known label mixed with wrong complete registry/model bytes=>InvalidIdentity;well-formed old/unknown edition in the current-only build=>UnsupportedSemantics. Model unknowns=>UnsupportedModel and missing fact shape payload=>IncompleteFacts in the non-authoritative descriptor check. No admission failure is a substitution Violation. Missing context-role metadata is not a shared-context proof. Standalone fragments cannot be interpreted as root CompleteView.

## Historical isolation

Edition1 512 cases use the exact abc91d5 source/registry/Expected. Edition2 51 staging cases use the exact pre-R2 {BASELINE} source/46-source/436-rule identity. They are independently executed,not relabelled as successor positives. The successor has a separate input/Expected namespace;only registration/shape/identity/denial cases execute here. R1's48 full semantic design vectors remain NotExecuted unless an explicitly identified registration-only law is separately materialized;none becomes a typed producer conformance claim.

"""}
 for key,(sid,kind,path,_) in SOURCES.items():
  text="# "+sid+"\n\n"+common+bodies[key]
  if key=="RM":text+="```json\n"+json.dumps(m,ensure_ascii=False,separators=(",",":"))+"\n```\n\n"
  for rule in (x for x in RULES if x["source"]==key):
   text+=f'<a id="{rule["anchor"]}"></a>\n\n## {rule["rule_id"]}\n\nMUST: {rule["statement"]}\n\nDirect normative dependencies: '+", ".join(rule["dependencies"])+f'.\n\nRegistered/staged responsibility:{rule["case_id"]};full producer/normalizer/oracle conformance remains R3/T4–T8,not executed by this reference.\n\n'
  write_new(path,(text.rstrip()+"\n").encode())
 print("EMITTED",len(SOURCES),"new sources",len(RULES),"new normative clauses;old rows untouched")
if __name__=="__main__":main()
