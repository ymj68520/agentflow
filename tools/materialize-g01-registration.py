"""One-time normative materializer. Inputs are approved declarations, not Actual."""
from pathlib import Path
import json,sys,re
from g01_registration_data import DATA
from g01_rule_dependencies import dependencies
ROOT=Path(__file__).resolve().parent.parent
sys.path.insert(0,str(ROOT/"tools"))
from portable_proof_python_oracle import AFIDL
from oracle.rust_type_inventory import declarations
PATHS={"OS":"spec/computation/proof-obligation-semantics.md","RM":"spec/tables/proof-report-model-v1.md","PP2":"spec/portable/portable-proof-inputs-v2.md"}
IDS={"OS":"agentflow.computation.proof-obligation.semantics@1","RM":"agentflow.computation.proof-report.model@1","PP2":"agentflow.portable.proof.inputs.semantics@2"}
def write(path,text):
 p=ROOT/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(text.rstrip()+"\n",encoding="utf-8",newline="\n")
sources=[(p.relative_to(ROOT).as_posix(),p.read_text(encoding="utf-8")) for p in sorted((ROOT/"spec/idl").rglob("*.afidl"))]
_,modules=AFIDL.compile_module_set(sources)
imports=[x for key in sorted(modules) for x in modules[key]]
native=declarations(edition="legacy-1")
def predicate_literals(filename, constructor):
 text=(ROOT/filename).read_text(encoding="utf-8").split("#[cfg(test)]")[0]
 return sorted(set(re.findall(re.escape(constructor)+r'::new\(\s*"([^"]+)"',text)))
closed_predicates={
 "root_preflight":["assertion_input","exact_pin","proof_context","compiler_identity","subflow_dependency"],
 "root_dimension":["I","O","Phi","epsilon","R","F","U","K","X"],
 "execution_child":["source","feasibility","security"],
 "root_boundary":["body","implementation","scope_authority","scope_budget","runtime","publisher"],
 "control":predicate_literals("crates/af-computation/src/control.rs","ControlDimensionProof"),
 "leaf":predicate_literals("crates/af-computation/src/resolution.rs","LeafDimensionProof"),
 "prerequisite":["closed_contract_schema","imported_module_closure","schema_universe","positive_unique_ports","failure_pins","effect_operation_membership","resource_universe","effect_resource_relation","flow_catalogs","io_resource_namespaces","additive_meter_units","recovery_usage_exhaustiveness","closed_effect_action_coverage","exact_k_identity","execution_universes","flow_execution_relation","required_dimensions","exact_relation_pins","epsilon_nonempty","flow_constraints_require_relation"]}
closed_predicates["root_recovery"]=[case for case in closed_predicates["control"] if case.startswith(("root-recovery-","root-k-"))]
constraints=[]
for paragraph in DATA["semantic_constraints"]:
 paragraph=re.sub(r'\[([^\]]+)\]\(phase-[^)]+\)',r'\1 (closed predicate roster below)',paragraph)
 paragraph=paragraph.replace("the exact named predicates in each row of the 92-site map (closed predicate roster below)","the exact cases in closed_predicates")
 constraints.append(paragraph)
model={
 "model_id":IDS["RM"],"model_edition":"agentflow.g01.report-shape@1","obligation_edition":"agentflow.g01.obligation@1",
 "model_language":{"record":"all named fields required, no extra fields; written declaration order for semantic comparison",
  "variant":"one closed case and its complete payload; case declaration order supplies rank",
  "Opt":"explicit none:unit or some:T; missing is not None",
  "Seq":"ordered semantic sequence, duplicate occurrences retained",
  "Set":"unique complete typed values in typed order; conflicting equal keys rejected",
  "Map":"Set of complete key/value records with unique typed keys",
  "Node":"uint64 canonical value-DAG rank of the exact declared model type",
  "nominal_ids":"distinct uint32 Computation/Expression/RunParameter/ValueSlot/ResourceSlot/Port/ContractRef; ordinals/indices checked uint64",
  "Raw":"recursively replace untrusted text positions with exact UTF8 bytes, retain native enum/option/numeric types and all sequence order",
  "imports":"complete frozen declarations below are normative logical shape input; storage references flatten to owned logical values, never native addresses",
  "unknown":"reject unknown type, field, variant, width, role, clause or edition; no reflection fallback"},
 "core_type_definitions":DATA["type_definitions"],
 "semantic_constraints":constraints,
 "closed_predicates":closed_predicates,
 "closed_domains":DATA["closed_domains"],
 "imported_afidl_declarations":imports,
 "semantic_model_declarations":{key:row for key,row in sorted(native.items()) if not row.get("ambiguous")},
 "scope_excluded_descendants":["Fact","Obligation","Scope","CompleteView","final_report","CheckedRootExportCompatibility"],
 "required_types":["Scope","FactKey","Fact","ObligationIdentity","Obligation","PrerequisiteKey","EvidenceRef","TypedContradiction","ObservationSite","Source","Target","Coordinate","Requirement","CanonicalValueDag"],
 "bounds":{"assertion_nodes":1024,"depth":96,"combined_fact_prerequisite_obligation_keys":65536,"work":4194304,"canonical_view_bytes":134217728,"value_nodes":1048576,"source_occurrences":65536,"reference_observation_dependency_edges":262144}
}
common="# "+IDS["OS"]+" — typed obligation semantics\n\nStatus: normative;semantic major1. This is a non-wire analysis model. Registration does not assert production support. No AFIDL/ABI/HashV1 assignment is introduced. The authoritative complete types/ranks are the closed table agentflow.computation.proof-report.model@1. Existing frozen substantive proof rules remain prerequisites,not replaced by report presentation.\n\n"
body=common+"""## Objects and authority

Facts are validator-origin applications of closed predicates to exact typed inputs. A fact is not an observation or a sentence. CompleteView is defined only for the full root-export roster;contract-validation/computation fragments are tagged and cannot issue RootExport capability. Unknown applicability/content is not an empty set. Caller assertions,report text and support flags cannot establish facts.

Identity has exactly edition,complete scope,assertion path,stage,kind,dimension,source,target,coordinate,rule,requirement,origin. Status is not identity but remains a required semantic value together with disposition,prerequisites,references,primary,contradiction,companion evidence and all observations. Scope binds full compiler/source/registry/role content,complete universes/relations,resolver caller/source/target associations,root provenance,raw assertion input DAG and context-role partition. No count/checksum-only equality is permitted.

Origins use declared semantic roles/sequence ordinals,never code lines,UUIDs,pointers,clock or push order. Equal atoms from independent declarations keep distinct origins. Copies of the same key must agree on every semantic field except observation sets,which are unioned. Conflicting status/support/countermodel is InvalidFacts,not verdict-max repair.

## Derivation predicate

For each complete originating fact:blocked_by facts yield exact blocker records and no evaluated obligation;Proven yields no unmet obligation;Unproven yields missing_static_proof;RequiresEvidence yields needs_independent_evidence;Violation requires its typed witness and yields contradiction_remains;authority(NotEstablished) yields authority_not_established. Every required non-Proven evaluated fact must be covered exactly once. Missing sources/required coordinates,unsupported models/cases,cyclic support and inconsistent fact sets fail explicitly.

The full root roster has five mandatory core checks:assertion-input validity,exact pins,proof-context/resolver association,compiler identity,subflow dependencies. Core failure retains all failed keys and nine non-evaluated blocked dimension records. Core success evaluates exactly I,O,Phi,epsilon,R,F,U,K,X. The post-gate body carrier is not a sixth mandatory preflight. Known X sources retain separate feasibility/security facts and full requirements;an unresolved source is not a fabricated evaluated child.

Six root independent boundaries always remain under static-only inputs:body/implementation RequiresEvidence,Scope authority/budget Unproven,runtime/publisher NotEstablished. Runtime's RequiresEvidence companion belongs to the same runtime predicate. Each call-site body requirement additionally retains caller/call/target/export path. Proof.Proven never means Established authority. No supplied string discharges these boundaries.

A prerequisite retains originating stage,predicate,source,target,coordinate,rule,requirement,origin,status,supporting fact and all consumer edges. An established prerequisite may accompany Proven;an unmet mandatory prerequisite cannot. Conditional false has no relation instance;unknown applicability has an explicit unresolved premise. Equal text does not merge predicates whose keys differ. Required references are typed evidence coordinates,not RuleIds or proof certificates. Derive their full set from subjects;primary is the typed minimum,except six root whole-boundary primaries None. Missing is not None.

## Capability condition

ProofEligible means five core checks and nine unique evaluated dimensions Proven. Checked additionally requires CompleteAndValid fact capture and successful bounded normalization/encoding under this exact model. These are additional availability preconditions,not a restatement of old sufficiency. ResourceLimit/IncompleteFacts returns no Checked and does not rewrite a proof verdict as Violation. The private checker alone can construct Checked;normalizers and decoded reports cannot. This registered condition does not authorize activation before T3–T8.

## Dependency direction

Proof predicates -> structured facts -> typed obligations -> canonical report. There is no report-text -> proof-verdict edge. Scope contains input values only;canonical DAG interning is finite bottom-up equality,not a report/digest fixed point. Complete identity string equality iff complete typed identity equality does not excuse omission of status,links or observations from full-field comparison.

## Clause allocation

D01–D16 are design provenance labels only. The independently testable invariants below have permanent RuleIds. Full substantive conformance remains required before activation;T1/T2 registration tests do not claim T3–T8 implementation.

"""
rules_by={source:[row for row in DATA["rules"] if row["source"]==source] for source in IDS}
def rule_sections(rows):
 return "\n".join('<a id="'+row["anchor"]+'"></a>\n\n## '+row["rule_id"]+'\n\nMUST: '+row["statement"]+'\n\nDirect normative dependencies: '+', '.join(dependencies(row["rule_id"]))+'. This is not a transitive-closure certificate.\n\nConformance responsibility:registered/staged case '+row["case_id"]+' checks the actual registration/staging boundary;full semantic laws '+row["full_semantics_design_laws"]+' require their separate implementation-layer conformance before T8.\n' for row in rows)
write(PATHS["OS"],body+rule_sections(rules_by["OS"]))
rm="# "+IDS["RM"]+" — closed non-wire semantic model\n\nStatus: normative. The single JSON block under Closed Descriptor Table is the complete registered model declaration. Its generic constructors have the exact closed meanings below;unknown materialization fails closed. Full fixed imported logical declarations are included,not only their names/digests. Native lifetimes/borrows are not wire values;the explicit logical Context/Compiler/Scope definitions override storage layout. Raw(T) is a closed companion-family transformation,not runtime reflection. Every concrete reachable descriptor must be materialized and validated before support can be activated. No AFIDL field/case numbers or HashV1 domains are assigned.\n\n"
rm+="## Canonical interpretation\n\nACE1 is schema-directed. Products emit all fields in unsigned UTF8 field-name order;semantic comparison follows declaration order. Numerics retain nominal widths and exact canonical text;bytes are unpadded base64url;options retain explicit tags;invalid raw claims remain bytes. Sets use typed keys/full payload;sequences keep declaration order. Identity order is edition,scope,assertion path,stage,derived role,dimension,kind,source,source ordinal,mapping ordinal,target,coordinate,rule,requirement,origin. Role is uniquely derived from predicate roster and full scope. Numeric2 precedes10. Rank order is not a compatibility lattice.\n\nFinite DAG construction validates types/cycles/reachability,interns equal typed values bottom-up,orders by height then type/nonrecursive payload/ordered child ranks,and rejects redundant nodes or alternative numbering. Sharing and equal copies denote the same value;independent declaration origins stay distinct. No Scope/final-report descendant or recursive digest fixed point is allowed. Exact decimals compare as rationals,finite floats mathematically after frozen zero canonicalization;None<Some,text/bytes unsigned lexicographic,records/tuples/lists by declared fields/prefix.\n\nWork charges every node/edge/typed comparison/sort comparison/exact64-bit numeric limb and64-byte scanned/emitted block. The output bound includes every repeated full Scope in standalone identity strings and typed records. All limits intersect earlier D0–D10 limits;no limit raises those limits. Overflow,unknown model,incomplete inventory,invalid graph or overbudget returns an explicit failure and no partial successful view/capability.\n\n## Closed Descriptor Table\n\n```json\n"+json.dumps(model,ensure_ascii=False,sort_keys=True,indent=2)+"\n```\n\n"+rule_sections(rules_by["RM"])
write(PATHS["RM"],rm)
pp="# "+IDS["PP2"]+" — Edition2 current-only staging\n\nStatus: normative;successor policy for agentflow.root-export-checker@2. Portable Bundle schema agentflow.portable.proof.inputs@1 and ACE1/HashV1 meanings are unchanged. PP1 source/RuleIds remain immutable. For owner@2 this source supersedes only AF-SEMANTIC-0188/0190 current-version/content/attribution policy;0173–0187,0189,0191–0196 remain applicable.\n\n"
pp+="""## Exact edition binding

Owner is agentflow.root-export-checker@2;report shape agentflow.g01.report-shape@1;obligation edition agentflow.g01.obligation@1. Exact complete identity binds whole specification/rule registry bytes,all46 source bindings in the Portable manifest,20 selected source contents,138 applied rule descriptors,closed tables,all ten semantic roles and the registered model bytes. Source checksums are ordinary SHA256 consistency checks,not semantic HashV1 domains or replacements for content.

Selected IDs are the prior sixteen static semantic IDs plus portable.proof.inputs.semantics@1,proof-obligation.semantics@1,proof-report.model@1,portable.proof.inputs.semantics@2. Applied rules are CONTRACT0001–0022,SEMANTIC0111–0187,0189,0191–0228,total138. They are explicit attribution only;F03 remains OPEN/P2,not complete transitive closure. All eight defined roles bind the same20 source IDs and314 named existing AFIDL schema pins;plan_lowering/runtime_semantics_profile remain model_missing:not_formalized. Preserve the exact prior role dependency graph.

## Distinct registration/support/activation states

Registered means actual sources/rules/model are present and content-valid. Supported means implemented proof/fact/normalizer/lossless projection behavior has passed its required conformance. Activated means separately authorized T8 release. These are distinct implementation-owned states,not a caller Boolean or editable support flag. T1/T2 state is Registered,NotImplemented,NotAuthorized;release is NotAuthorized. Exact valid Edition2 input in this state must not receive full static Checked capability.

All direct and Portable capability paths must enforce the same support/activation denial. A legacy diagnostic Unproven carrier,if retained for API compatibility,must have typed staging rejection metadata and must not be advertised as normalized G01 facts/view. Permanent Unproven is not a substitute for the future implementation;T3 producers,T4 normalizer,T5 lossless projection,T6 independent oracle,T7 audit,T8 activation remain separately required.

## Current-only admission and priority

Bounds/canonical closed decode precede reference/content consistency,which precede exact support. Malformed ACE1 is InvalidCanonicalEncoding;resource bounds/overflow ResourceLimit;missing required normative source IncompleteContent;known version mixed with wrong full registry/content InvalidIdentity;internally valid old/unknown edition UnsupportedSemantics. Exact new content with incomplete implementation is explicit unsupported/not-implemented staging,not ordinary static Violation. Never relabel old bytes or reuse old positive results as Edition2 conformance. Early rejection produces no complete RootExport/G01 report.

## Historical evidence and staging boundary

Keep all old43 Specification and404 Rule records,775 ABI assignments,17 AFIDL modules,14 HashV1 domains and historical Expected unchanged. Old229 algebra cases may run unchanged;old82 RootExport-sensitive and201 Portable cases execute in an exact pinned old-code environment. New input/Expected has a distinct namespace and independent normative derivation. Versioned original-directory generators must regenerate historical output from pinned historical identity/declaration inputs,not new registry bytes.

The staging commit is an indivisible T1/T2 safety boundary,not a release. New registry+old silently issuing checker+relabelled manifests is forbidden. Roll back the entire staged identity transaction if any route can mint an incomplete-edition Checked. Full activation remains T8 only;G02/G03 and B01/B02 conformance remain blocking.

"""
write(PATHS["PP2"],pp+rule_sections(rules_by["PP2"]))
for key in IDS:print(IDS[key],PATHS[key],(ROOT/PATHS[key]).stat().st_size)
