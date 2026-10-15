# G01 formal obligation contract — draft edition 1

**DRAFT — NOT REGISTERED NORMATIVE.** Accepted design recommendation only. The words MUST/SHALL below describe a proposed future contract, not current registered behavior. Clause labels `D01`–`D16`, type names, tags and ranks are local draft definitions: no RuleId, SpecificationId, AFIDL number, HashV1 domain or support version is allocated by this document. Activation requires separate registration, implementation and migration authorization.

Related documents: [identity](phase-04c1b4c1-typed-obligation-identity.md), [encoding](phase-04c1b4c1-encoding-and-ordering.md), [prerequisites/references](phase-04c1b4c1-prerequisite-reference-semantics.md), [compatibility](phase-04c1b4c1-existing-rust-compatibility.md), [vectors](phase-04c1b4c1-decision-vectors.md).

## D01. Objects, authority and completeness

Select **Option A: canonical typed identity with a mandatory lossless canonical string projection**. Formal collection = finite **Set** of complete obligation identities. Its serialized view is a deterministically ordered sequence, NOT an ordered multiset. Human explanations are a separate projection.

An obligation is a still-undischarged named proof predicate at an exact source/target occurrence. It is neither a verdict nor evidence satisfying that predicate. A violation is a known contradiction, not missing evidence; its remediation obligation does not erase the contradiction.

Inputs are `StructuredFacts(scope, facts, prerequisites, references, boundaries, blockers)`. A fact has:

```text
Fact = {
  key: FactKey, role: Role, kind: Kind, requirement: Requirement,
  source: Source, target: Target, coordinate: Coordinate,
  rule: RuleRef, evaluation: Evaluation, status: Status,
  prerequisites: Set<Prerequisite>, references: Set<EvidenceRef>,
  contradiction: Option<TypedContradiction>, observations: Set<ObservationSite>
}
Role = mandatory_preflight | static_dimension | execution_child |
       independent_boundary | supporting_fact
Evaluation = evaluated | blocked_by(nonempty Set<FactKey>)
Status = proof(Proven|Violation|Unproven|RequiresEvidence) |
         authority(NotEstablished)
```

`FactKey=(stage, assertion_path, kind, dimension, source, target, coordinate, rule, requirement, origin)` under one exact Scope; it is assigned by the proof predicate's declaration/source coordinates, never a Rust function/line number, iteration accident, UUID, clock, process address or reason string. Identity is this key prefixed by edition and complete scope. `ObservationSite` identifies a report owner/field that exposes the same fact; it does not create another obligation. Status domains retain their existing meanings; NotEstablished is not inserted into ProofVerdict.

The fact inventory SHALL be complete for the selected report profile before derivation. A missing fact, unsupported predicate, missing required coordinate, conflicting repeated fact, unknown applicability or incomplete context yields `IncompleteFacts`/`UnsupportedModel`/`ResourceLimit`, not an empty collection and not a fifth proof verdict. No caller-supplied report establishes facts. Future adapters obtain facts from validators and independently interpretable declarations.

## D02. Exact scope and identity

```text
Scope = {
  contract_edition, analysis_semantics_version,
  caller_ir: DigestRef, top_export: DigestRef,
  compiler_identity: COMPLETE CompilerSemanticsIdentity,
  context_manifest: COMPLETE ProofContextManifest,
  resolver_associations: COMPLETE checked caller/node/target association set,
  root_provenance: {source_ir, root_node, COMPLETE root_resolver_identity,
                    COMPLETE root_context_contents, COMPLETE root_K_context_contents},
  context_association: equivalence partition of analysis_context/checked_export/snapshot/root/root_K roles,
  assertion_inputs: COMPLETE typed assertion input graph
}
Identity = {
  edition, scope, assertion_path, stage, kind, dimension,
  source, target, coordinate, rule, requirement, origin
}
Obligation = { identity: Identity, status: Status, disposition: Disposition,
               prerequisite_links: Set<PrerequisiteKey>, references: Set<EvidenceRef>,
               contradiction: Option<TypedContradiction>, observed_at: Set<ObservationSite> }
Disposition = missing_static_proof | needs_independent_evidence |
              contradiction_remains | authority_not_established
```

Every field of Identity is semantic. Reason, localized presentation and source-code line numbers are not identity fields. Status is a required value, not identity: the same predicate may change status after legitimate fresh proof; a different evidence requirement, source occurrence or rule is a different identity. Different statuses for the same fact in one immutable report are an error, not silently merged by verdict priority.

Identity edition must equal its scope's contract edition. Repeated checked source/digest/node/ordinal fields inside a coordinate must agree with their declared source and origin; deliberate expected-versus-observed comparison operands use explicitly tagged sides, not contradictory aliases. Each status domain follows its existing producer: execution-child compatibility has only its existing three verdicts; authority has NotEstablished, never a newly invented ProofVerdict interpretation.

Scope equality uses complete typed content, not lengths/checksums or pointer equality. Existing IR/contract digests retain algorithm, domain and every byte. Compiler registry/source/rule bytes and context universe/relations/resolver contents are retained, not replaced by a new hash. Process-local pointer consistency remains a separate checked premise. G02 must eventually provide lossless getters; this draft does not implement them. No RootContractDigest/ObligationDigest/ReportDigest is introduced.

`compiler_identity`/analysis version identify the **analysis owner**. The compiler/context/pin claims inside `assertion_inputs` remain separately retained comparison operands, including stale/invalid claims. A current in-process checker may therefore emit a compiler-identity Unproven fact for a stale assertion; do not reject it merely by treating that assertion's claimed version as the analysis owner's version. By contrast an unsupported Portable manifest rejected at D4 has no D10 report at all. Root/current resolver identities and context-role association partitions are distinct values even when node sets or content counts agree.

`assertion_path` is the ordered root-to-target list of `(caller_ir, call_site, target_ir, target_export Option)` edges. Root has `[]`. Equal target digests at two caller nodes have different paths. Input local table IDs and memory addresses do not substitute for semantic associations. Acyclic sharing may be represented by a validated canonical value DAG; sharing versus two copies of equal values is not identity. Original semantic list order, including SEQUENCE children, is preserved. Malformed mapping order is retained as the subject of a failing preflight, never repaired by sorting the assertion itself.

The displayed Scope record is the `root_export` variant (rank1). To review prerequisites without fabricating a root, the closed scope sum also has `contract_validation` (rank2: edition/version/owner compiler, complete context/association, complete raw candidate contract, optional old checked contract for substitution) and `computation` (rank3: edition/version/owner compiler, caller IR, complete context/resolver/root provenance and owned root node). These variants have no pretend root-export digest or assertion. D05's full-root roster/gate applies ONLY to root_export; other profiles use the per-fact emission rule in a tagged fragment and cannot issue a RootExport capability. This includes invalid D6 reports and standalone D9 facts. Early rejection still has no invented D10 report.

## D03. Closed semantic kinds and requirement clauses

The following ranks define the proposal's closed kind vocabulary. They are internal draft ranks, NOT AFIDL assignments. Each non-Proven fact SHALL identify one of the stated normative predicates; a new unrelated predicate needs a new contract edition/registration, not a generic catch-all string.

| Rank/tag | Predicate and required subject/coordinate | Existing normative basis / rule family |
| --- | --- | --- |
| 1 assertion_input | mapping uniqueness/order; whole assertion or offending mapping occurrence | root-export exact-assertion-pins;0159 |
| 2 exact_pin | exact IR/export/root pin association; typed role + full expected/observed pins | same;0159 |
| 3 proof_context | complete content/same-context premise; exact scope + catalog/relation subject | proof-context-manifest;0160 |
| 4 resolver_provenance | same caller/source/target/export/dependency association | proof-context-manifest;0160 |
| 5 compiler_identity | exact analysis/spec/rule/registry content | compiler-semantics-identity;0161 |
| 6 subflow_dependency | exact call-site/target/export/closure edge | subflow-body-separation;0171 |
| 7 input_mapping | export input, root parameter/slot, producer/use-site/type | root-input-mapping;0162 |
| 8 output_producer | export output, named workflow output/expression/path/type | root-output-mapping;0163 |
| 9 failure_containment | exact nominal failure membership | failure-containment;0165 |
| 10 effect_containment | source occurrence/full atom/guard/bounds and export coordinate | effect-containment;0166 |
| 11 resource_mapping | source port → root slot → export port, type/attribute/alias | root-resource-mapping;0164 |
| 12 information_flow | exact source/sink/kind/label/taint/influence/partition | information-flow-non-relaxation;0167 |
| 13 usage_coverage | phase/meter/unit/aggregation/bound and K coverage role | usage-containment;0168 |
| 14 recovery_coverage | clause below, complete source/action/profile/factor identity | recovery-compatibility;0169; D9 0150/0154–0158 |
| 15 execution_source | caller/node/contract and exact requirement source coverage | execution-compatibility;0170 |
| 16 execution_feasibility | each child requirements/quantity/profile comparison | same;0170 |
| 17 execution_security | each child trust/region/sandbox preservation comparison | same;0170 |
| 18 body_conformance | root or exact call-site target body, never just export declaration | subflow-body-separation;0171/0172; D9 0120 |
| 19 implementation_conformance | exact root/export implementation claim | static-runtime-boundary;0172 |
| 20 scope_authority | root boundary or exact Scope node's absent ceiling | static-runtime-boundary;0172; Scope 0148 |
| 21 scope_budget | same, budget ceiling distinct from authority | same |
| 22 runtime_authority | exact analysis boundary, separate authority/evidence statuses | static-runtime-boundary;0172 |
| 23 publisher_authentication | exact analysis boundary, authentication not established | same |
| 24 control_evidence | named existing Control predicate, not an unregistered opcode meaning | control-proof-obligation-propagation;0124–0158 |
| 25 prerequisite | a separately originated missing/contradicted/evidence prerequisite when no existing supporting fact already names it | component-contract cross-dimension and existing0001–0019 premise rules |

`Requirement` is a closed pair `(clause, evidence_class)`, not free text and not a verdict alias. Clause is one of the named predicates in the table. For recovery the clauses are `source_closure, action_binding, usage_profile, multiplicity, guard_compatibility, epsilon_coverage, recovery_usage_coverage, eligibility, commit_order`; for control: `resolved_leaf, topology, output_interface, selector_totality, effect_envelope, failure_row, resource_provenance, information_flow, usage, retry_safety, child_settlement, forward_history, compensation_coverage, root_export_boundary, unsupported_control, recovery_blocked_outcome`. Other kinds use their identical tag as clause. `evidence_class=static_declaration | typed_counterexample_resolution | trusted_body | implementation_attestation | provider_receipt | authority_attestation | publisher_authentication | control_terminal_state | committed_effect_order | recovery_outcome | provider_honoring | application_idempotency`, with ranks in that order. The last five distinguish Scope settlement, committed-order evidence, recovery-blocked outcome and provider/application retry guarantees described by the frozen Scope/Recovery rules; they are not all provider receipts. Allowed classes follow the predicate's frozen substantive boundary: no action-specific F/X or receipt existence is invented.

Kind25 instead uses the closed prerequisite clauses `closed_contract_schema, imported_module_closure, schema_universe, positive_unique_ports, failure_pins, effect_operation_membership, resource_universe, effect_resource_relation, flow_catalogs, io_resource_namespaces, additive_meter_units, recovery_usage_exhaustiveness, closed_effect_action_coverage, exact_k_identity, execution_universes, flow_execution_relation, required_dimensions, exact_relation_pins`, in this order. These are the18 distinct predicates represented by P1–P10's19 literal occurrences. Conditional relation clauses retain explicit applicability facts; unresolved applicability is a named missing premise, not absence. Unknown P11 fallback dimensions have no new clause by default.

For a known static contradiction, the *predicate* retains its original class and status Violation; `disposition=contradiction_remains` and typed contradiction identify what must be resolved. `typed_counterexample_resolution` is reserved for an explicit prerequisite that asks to resolve a counterexample, not automatically assigned to every violation. An unsupported kind/clause/class combination is UnsupportedModel. Control clauses preserve their original dimension/rule and source scope; a body/effect/runtime concern is not folded into generic control_evidence.

## D04. Source, target and coordinates

Source is a closed union: `root{ir,root_node}`; `component{caller_ir,node,contract,entry_ordinal Option}`; `subflow{caller_ir,call_site,target_ir,export_state}`; `scope{caller_ir,node}`; `context{exact_scope,catalog_role Option}`; `compiler{exact_scope}`; `control{caller_ir,node}`. None of a required source digest/node/ordinal means unknown. Unresolved leaf/source predicates use the known control source and a known declared request, not an invented component digest. A whole-root aggregate legitimately uses root source and an exact aggregate subject; it must not pretend to enumerate lost per-source causes.

Target is `analysis_boundary | export{digest,port Option} | component{digest} | subflow{ir,export_state} | relation{schema,revision,content} | proof{FactKey} | unresolved_request{caller_ir,node,request}`. A request is the exact existing declared contract-ref/digest, signal/schema pin, source lookup, or named universe membership query. Its unresolved state is explicit and is never interpreted as a resolved target. `export_state=provided(DigestRef)|absent|unresolved(request)` distinguishes a declared absence from missing resolution; existing declared Optional export pins in assertion paths preserve their literal absence, not an invented digest. Coordinate is `whole | input{export_port,run_parameter,slot,use_site Option} | output{export_port,name,expression,path} | resource{source_port,slot,export_port,type,attributes,aliases} | effect{full_source_identity,export_atom,export_guard,bounds} | flow{source,sink,kind,labels,taints,partition,ordinal} | usage{phase,meter,unit,aggregation,coverage_role} | recovery{full_action_source,action,profile,factors,path} | execution{source_node,contract,requirements,aspect} | dependency{call_site,target,edge_ordinal} | control{node,operator,predicate,edge_path} | context_member{catalog,member_identity} | pin{role,expected,observed}`. Types are the existing complete frozen semantic types; all applicable fields are present. Option is genuine non-applicability or an explicitly declared absent optional input, never an unknown-content substitute. A coordinate variant belonging to another kind/dimension is invalid.

Origin distinguishes independent occurrences: `(predicate, declared_occurrence_path, source_ordinal Option, mapping_ordinal Option)`. The path uses named semantic roles plus numeric declared sequence indices. An inherited copy keeps the originating path, not a new parent-array index. Two independent same-valued declaration rows retain different origin indices even when all other payload fields match. If the origin was lost, return IncompleteFacts; do not reconstruct it from text or final push order.

## D05. Derivation relation

```text
derive(F):
  validate exact scope/profile, complete fact roster, closed variants and links
  validate unique FactKeys, evidence domains, evaluation provenance and acyclic blockers
  core := exactly {assertion_input, exact_pin, proof_context, compiler_identity,
                   subflow_dependency} at root mandatory_preflight role
  if any core is not Proven:
      require nine static facts marked blocked_by(exact failed core keys)
      forbid evaluated-success claims or per-dimension missing-evidence guesses
  else:
      require exactly I,O,Phi,epsilon,R,F,U,K,X evaluated static facts
  O := empty Set; B := empty Set; C := empty Set
  for each distinct originating fact f (not every observation copy):
      if f.evaluation is blocked_by(xs): B += (f.key,xs); continue
      if f.status is proof(Proven): continue
      if f.status is proof(Unproven): disposition := missing_static_proof
      if f.status is proof(RequiresEvidence): disposition := needs_independent_evidence
      if f.status is proof(Violation):
          require normative typed contradiction; C += contradiction
          disposition := contradiction_remains
      if f.status is authority(NotEstablished): disposition := authority_not_established
      O += obligation(identity(f), f.status, disposition, exact links/evidence)
  for each unresolved prerequisite not already the same originating fact:
      apply the same rule to its separately identified predicate fact
  require coverage of every required non-Proven evaluated fact, all six boundaries,
          all known execution-child aspects and each subflow body source
  return {scope, obligations: sort_typed(O),
          identity_strings: [ACE1(o.identity) for o in sort_typed(O)],
          blockers: sort_typed(B),
          contradictions: sort_typed(C), fact_and_prerequisite_index}
```

This is a projection of independently established facts, not an algorithm to solve unknown E/F/K/X relations. An unproven aggregate requires complete `cause_fact_keys` or an explicitly declared root-aggregate predicate. If its model calls for per-occurrence facts but current output lost them, derivation fails; it must not synthesize N detailed causes from one sentence. Existing verdict precedence applies to the proof report only, never to erase obligations in other dimensions.

## D06. Preflight, dimensions, execution and subflows

Five core preflights are mandatory static predicates. The current `subflow-body-evidence` item is an **independent_boundary**, even though its v1 carrier is `preflight`. Each dependency is separately represented by its caller/call-site/target/export path. Boundary Unproven/RequiresEvidence statuses do not prevent the otherwise permitted static compatibility capability.

Blocked nine-dimension placeholders produce **blocker records**, not nine new unsupported-evidence obligations. The vector's Unproven status remains visible. All failed preflight obligations remain, not merely the first or highest priority one. Independent body/implementation/scope/runtime/publisher boundary facts remain even if preflight fails or a static dimension violates its ceiling.

Each execution child retains two separate aspect facts. Missing child mapping is execution_source; unavailable feasibility/security relations are not silently omitted. A separate aggregate X fact is retained only when it asserts a different predicate (e.g. complete coverage); duplicate observation of the same aspect is not an additional obligation. No worker quantity sum or action-specific X semantics is introduced.

## D07. Boundary roster and discharge

A full root-export view requires six root boundary facts: body, implementation, Scope authority, Scope budget, runtime authority (with its separate existing evidence status), publisher authentication. Existing frozen static-only inputs leave body/implementation RequiresEvidence, Scope authority/budget Unproven, runtime/publisher NotEstablished. Runtime's RequiresEvidence companion is linked to the same runtime predicate, not a duplicate authority identity.

Exact subflow body requirements additionally use distinct call-site subjects. Root body and a subflow body are different predicates; two observations of the same target-call body requirement share one identity. A boundary is removed only after the *same scoped predicate* is legitimately Proven by a separately authorized evidence-consuming validator. No such evidence input is introduced into the current checker by this draft. Unsupported submitted attestation strings cannot remove an obligation.

The preceding discharge condition applies to proof-domain predicates only. The closed authority domain here has **only NotEstablished** and no discharge transition. A later authority/authentication model would require separate status/evidence semantics and version authorization; this draft cannot remove a runtime/publisher obligation by interpreting a static Proven as Established. Frozen static body/implementation/Scope boundaries likewise cannot be discharged by a caller-supplied flag or by changing a reason string.

## D08. Prerequisites

Prerequisite = `{key=(scope, stage, predicate, source, target, coordinate, rule, requirement, origin), status=established|missing|requires_evidence|contradicted, supporting_fact Option}`. Stage names the originating prerequisite check, not every consumer stage. Consumer dimension belongs to the retained consumer edge, so a genuinely shared universe premise can serve several dimensions. It shares predicate identity components with an obligation, but has a distinct role/tag; it is not automatically an unresolved obligation.

An established prerequisite may accompany a Proven proof. A Proven evaluated fact with a missing/contradicted/RequiresEvidence prerequisite is inconsistent and rejected, not downgraded or promoted. Prerequisite lists have finite-set semantics with canonical typed order; they do not prescribe evaluation/SEQUENCE execution order. Reuse across dimensions shares a prerequisite identity only if every identity field agrees; consumer edges are preserved separately. For a fact's own missing prerequisite, the missing fact's identity is used once rather than a second generic "prerequisite missing" obligation.

If supporting_fact identifies that existing missing predicate, re-use its full FactKey and obligation. Otherwise the complete declared prerequisite itself supplies a kind25 fact, preserving its source/rule/coordinate/origin and original check stage. Contradicted prerequisites retain their typed contradiction. This is explicit producer data, not an algorithm that infers a missing precondition from a constructor's empty list.

The current20 literal strings are descriptions, not an approved token registry. Their proposed semantic translation is in [prerequisite semantics](phase-04c1b4c1-prerequisite-reference-semantics.md). Today no classification or executable coverage rule changes.

## D09. References and diagnostics

`rule_id` identifies a governing normative rule; `reference` locates a typed evidence subject. Reference is **RequiredSemantic evidence coordinate**, not the whole proof identity and not a substitute rule-dependency closure. EvidenceRef is a closed union of scoped computation, IR/contract digest, full SchemaRef, typed input/output/resource port, operation, full effect atom, caller-local contract-ref plus resolved pin, control edge, opcode, resolver difference, FactKey, scope_member or unresolved_request, in that order. Scope-member paths use closed schema-field steps, typed set-member keys or numeric sequence indices (variant ranks0/1/2). Resolver difference retains typed old/new/target contents; English descriptions are reason presentation only in the future version.

None means no singled-out primary reference; all required source/subject facts still exist. A missing required reference is an incomplete fact, not equivalent to None. Prerequisite and reference strings receive canonical typed projections under their own shape tags; no natural-language parser recovers identity. Current legacy values remain required under v1 until an explicit approved migration. DiagnosticOnly reasons remain presence/type-checked; removing reason must preserve status/basis/countermodel/identity/coordinates/links. Any essential datum found only in reason is a projection defect, not permission to ignore it.

Reference derivation is deterministic, not a producer's optional choice. Enumerate the typed subjects named by source/target/coordinate: root/control/scope nodes yield scoped-computation refs; component sources also yield their full contract pins; subflow sources yield call-site/target/declared export refs; context/compiler subjects yield complete scope-member refs; each nominal schema/port/operation/atom/control edge in the predicate yields its typed ref; unresolved requests and explicit resolver-difference operands yield their tagged refs. Deduplicate only complete equal refs. Under kind18–23 with root source, whole coordinate and analysis-boundary target, primary=None (the independent boundary has no singled-out external evidence object), while identity/source links remain complete. In every other predicate, primary is the minimum typed member of that derived reference set; an unexpectedly empty set is IncompleteFacts. New-reference string projection is Some(ACE1{edition,complete_scope,typed_coordinate}) or actual None according to that policy. Different legal primary selection is not allowed; full references remain required even though one is primary.

## D10. Equality and multiplicity

Identity equality is componentwise typed equality of all D02 fields. Obligations with different computation, contract, ordinal, mapping target, rule, stage, evidence requirement, context/resolver contents or subflow path are distinct. Repeated observations of the same complete origin/status are unioned with the set of observations and references. Same identity with conflicting status or contradiction is InvalidFacts. Independently repeated source occurrences have different origin coordinates and are retained even if the rendered English is identical.

The canonical collection decoder rejects duplicate complete identities and noncanonical order; only the **derivation** unions equal observations. Thus no transport deduplication hides an invalid duplicate. Counts represent set cardinality, never an implicit number of effects, retries or executions. Control repetition remains the existing explicit multiplicity fields, not duplicated obligation list entries.

A transformed/strengthened parent predicate is not an inheritance copy. In particular a `downgrade` application or a new selector/path condition creates an explicitly scoped application-origin/coordinate with its own predicate, preserving a dependency edge to the child's fact. Same source node/RuleId/dimension text does not establish same FactKey. Otherwise legitimate differently scoped evaluations would be misclassified as conflicting copies. Transformation provenance must be captured at the application, never guessed from the appended reason.

## D11. Canonical typed order

Compare the following tuple lexicographically, by typed comparison:

```text
(scope, assertion_path, stage_rank, role_rank, dimension_rank,
 kind_rank, source_variant_rank, source_payload,
 source_ordinal Option, mapping_ordinal Option,
 target_variant_rank, target_payload, coordinate_variant_rank, coordinate_payload,
 rule, requirement, origin)
```

Stage ranks D0..D10 are numeric0..10. Role ranks follow D01 order. Dimension rank = none0,I1,O2,Phi3,epsilon4,R5,F6,U7,K8,X9, followed by named supporting dimensions in unsigned UTF-8 order in a separate supporting-dimension variant. Kind ranks are D03. Source/target/coordinate variant ranks are their declaration order in D04; fields compare in declaration order. EvidenceRef variants follow D09 order. Requirement clause/class ranks follow D03 declaration order (recovery/control subclauses as listed). Rule comparison = `(specification_id, rule_id, anchor)` unsigned UTF-8, with complete compiler scope fixing the rule's content.

Role is a fixed function of the originating predicate in the validated profile roster, not caller-selectable and not changed by the report field where a copy is observed. The identity fields plus that roster determine its role uniquely. A conflicting role for an identical origin is InvalidFacts, so this comparison key cannot disagree with identity equality. Context association partitions compare canonical role classes (classes sorted by minimum role rank, members numerically); actual process addresses are never encoded. A stale root resolver and a current snapshot are both retained in scope, not reduced to their common node set.

Context-role ranks are distinct from fact-role ranks: root_export uses analysis_context0,checked_export1,snapshot2,root3,root_K4; contract_validation uses analysis_context0,candidate_validation1,old_checked_contract2 when applicable; computation uses analysis_context0,snapshot1,leaves2,root3,root_K4. Exactly the applicable roles occur once in the partition; missing/duplicate roles are invalid, not "same arena" defaults. Including the analysis argument is essential: four mutually equal cloned contexts are still different from the analyzer's supplied context. Source/digest/ordinal/coordinate optional fields precede payload by their own declared option tag. All payload record comparisons follow the written declaration field order, not the canonical JSON key order.

Unsigned integers compare mathematically within nominal widths, never by their encoded decimal strings (`2<10`). Text compares unsigned NFC UTF-8 bytes; byte strings unsigned byte lexicographic; option None<Some; tuples/lists lexicographic with proper prefix first; records by their declared ordered field vector; tagged sums by rank then payload. Digest comparison is `(algorithm,domain,raw_digest_bytes)`; preserve full SchemaRef nominal pins. Nested paths are semantic edge sequences and are not sorted as sets. Presentation strings and insertion order do not participate. Serialized lexical order need not equal this typed collection order.

Signed integer/timestamp/duration values compare mathematically in their nominal domain; bool false<true; unit/null have one value. Canonical decimals compare as exact rational coefficient/scale values without floating conversion; finite float64, if present in a scope snapshot, compares its mathematical finite value after the existing zero canonicalization. Nominal type tags precede values, so equal-looking numbers in different nominal types are not cross-assigned. Enum ranks are canonical-order codes only, never a new Recovery/trust/sandbox compatibility lattice.

## D12. Canonical representation

Use existing **schema-directed ACE1**, not arbitrary JSON or Debug. Define a report-local closed shape `ObligationIdentityDraft1` with the exact D02 fields; its string projection is UTF-8 text of `ACE1(identity)`. Edition is literal `g01.obligation/1-draft`; activation must allocate a non-draft edition separately. Every record key is emitted in ACE1 field-name order; all required fields appear; machine IDs/ordinals/ranks are canonical decimal **strings** at their declared u32/u64 widths; bool/unit/bytes/options/variants follow the existing ACE1 grammar exactly. Digest bytes render as64 lowercase hexadecimal characters under existing full DigestRef shapes, not a new hash calculation. Existing TypeSchemaRef includes its specification/name/complete nominal digest. Nested records and coordinates use their complete closed field shapes from D02–D04.

For existing non-AFIDL exact snapshot values, the draft fixes a transparent schema-directed value algebra: `scalar{nominal_type,value}`, `record{model_type,fields}`, `variant{model_type,case,payload}`, `sequence{element_type,items}`, `set{element_type,items}`, `option{element_type,value}`. Model types/fields/cases are the closed existing semantic data-model declarations at the baseline plus this draft's new records, not arbitrary host reflection; private universes require complete logical-member getters. Scalar nominal widths are preserved. Fields are a record of all equality-bearing model fields; byte contents are bytes, not checksums. Set members use the domain's existing typed key plus full typed payload; sequence fields preserve semantic order. Unknown fields/cases/opaque snapshots are UnsupportedModel. This is a *mathematical report-shape contract*, not an installed AFIDL codec or wire ABI; future adapter implementation and registration must explicitly materialize and test these shapes. G02 is the blocked getter dependency, not a license to encode an opaque object by Debug.

For byte-exact untrusted assertion claims, the scalar descriptor is `raw_utf8_bytes` with ACE1 bytes payload, not normalized semantic string. Preserve every original UTF-8 byte and original sequence order. Thus an invalid/NFD identifier, noncanonical claimed digest spelling or stale registry row remains distinguishable and can be the subject of a failed preflight; normalization must not repair it. Validated source/target DigestRefs retain the64-lowercase-hex rule; an invalid asserted pin is encoded as a complete raw claim record, not cast to a validated DigestRef. Actual set types are canonicalized as sets; a Vec whose order is checked by preflight is preserved as a sequence, not silently sorted.

Maps are encoded as a typed set of complete key/value records, with unique typed keys and the existing domain's key order; arbitrary dynamic keys are not mistaken for fixed record fields. Existing enums use the variant form with unit payload. Native usize ordinals become checked u64 values; overflow or an unsupported native index is an explicit resource/representation failure, never a truncating cast.

Recursive assertion values use a finite canonical value DAG, not exponential inline expansion: validate acyclicity; bottom-up intern exactly equal typed nodes; rank distinct leaves by full typed key, then increasing height and `(nonrecursive typed fields, ordered child ranks)`; require minimal reachable nodes and a root rank. Equal unfolded values share one entry irrespective of process sharing. No native address or submitted arbitrary node numbering is authoritative. A standalone identity string contains its complete scope/value-DAG, not an unresolved local table index. A future factored document may share that scope, but equality then requires the complete document, not strings alone. No new digest domain is necessary.

Canonical Unicode/escaping are ACE1 NFC/minimal escapes; integers exclude leading zeros,+,-0 and floating intermediates; Option is `{"none":null}` or `{"some":...}`, never missing or bare null. Unknown tokens/editions, extra/missing/duplicate fields, non-NFC strings, padded base64url and noncanonical numeric text fail closed. Decode/re-encode must reproduce every byte. A reason-only edit changes no identity bytes.

## D13. Bounds and failures

Proposed report-local safety ceiling:1024 distinct assertion value nodes, depth96,65536 fact/obligation identities,4194304 graph/comparison-work steps and128MiB total canonical view bytes. Validate counters/overflow and source/mapping cardinalities before expansion; stream output and stop before allocation beyond budget. Stricter local limits are permitted and must be declared. DAG keys are interned without recursive string expansion. Repeated scope strings charge the full byte budget; no unbounded exemption for shared values.

Exceeding limits, unresolved/cyclic references, missing necessary members, inconsistent statuses or unknown descriptors yields an explicit infrastructure/model failure; no truncated view or empty substitute, no semantic Violation invented, no capability issued from an incomplete normalization. Limits are design proposals, not edits to current production defaults. Report encoding is not allowed to become a path to granting a capability that proof did not justify.

## D14. Capability and consistency laws

```text
EligibleStatic(F) := F.scope.variant = root_export
                 AND CompleteAndValid(F)
                 AND all five mandatory_preflight facts are Proven
                 AND exactly nine unique evaluated static dimensions are Proven
```

This restates, rather than broadens, the existing static capability's conditions. It does not add body/runtime evidence to the static gate. Creating the actual private Checked capability remains exclusively the authorized production checker. Empty O is NEVER sufficient. A full current static-only report normally has the six boundary obligations even when EligibleStatic is true. A static-only subset view may be empty but is explicitly tagged as a subset and cannot impersonate a complete root view. Violation priority remains unchanged; blockers, independent boundaries and lower-priority facts cannot be erased by an overall result.

CompleteAndValid includes consistency with the frozen predicate relations: a Proven aggregate cannot have an unmet **mandatory** source/prerequisite fact for that same predicate. Independent boundary/support facts (e.g. D9's lack of an IR-embedded root-export declaration) are not silently turned into new mandatory D10 premises. Roles and dependency edges identify the distinction; a caller cannot forge an aggregate Proven to bypass missing required sources. No generic fact-status field is trusted merely because it appears in a report.

Required invariants: `obligation → exactly identified non-Proven evaluated fact`; `required non-Proven evaluated fact → obligation`; `blocked fact → explicit blocker, not fabricated evaluation`; `same origin observed twice → one identity with both observation edges`; `different origin → distinct identities`; `canonical strings equal iff full typed identities equal`; `no status/evidence upgrade by serialization`.

The identity string list is an exact order-preserving bijection with the required typed obligation records; it is not the whole report. Status/disposition/prerequisite/reference/contradiction/observation values and fact index must also be compared. Identity equality intentionally does not erase a changed status; comparing only the canonical identity strings would NOT satisfy full-field conformance. Prerequisite strings similarly project their complete typed key/status/support record under a separate edition tag.

## D15. Compatibility and invalidation

Current v1 strings are not retrospectively canonical. Future production must emit typed cause/origin/prerequisite/reference facts before aggregation, then the proposed projection. Existing four-to-six top strings cannot be split losslessly into per-source obligations by a postprocessor. This is **production adaptation required**, not merely a comparator-format update.

Any change to this contract's identity field, kind/clause/rank, derivation, order, encoding or prerequisite/reference meaning changes its edition and exact compiler semantics profile. Normative source/registry/rule/role/dependency/schema byte changes invalidate old exact-profile assumptions even if a version label is unchanged. Recommended migration is a newly identified analysis semantics edition with **current-only fail-closed** support; historical exact support is not silently added. Historical artifacts/Expected stay preserved; a new edition gets new independent Expected and explicit version-partitioned regression, never rewritten old failures. Detailed alternatives and transaction dependencies are in [compiler impact](phase-04c1b4c1-compiler-version-impact.md).

## D16. Future conformance and registration obligations

Before implementation closure: independently derive all required facts and exact identity bytes; test all direct and inherited producer branches, every predicate/status/blocked/boundary case, injectivity counterexamples, ordering including2/10, duplicates versus copies, decoder negatives, bounded DAG/resource failures and cross-process exact identity. Register new normative content/RuleIds/support mappings with explicit migration authorization; provide full G02 projection and G03 nonempty-subflow Oracle separately; execute real production differential. The [design vectors](phase-04c1b4c1-decision-vectors.md) are declarative and all Actual Status = Not Executed. They add zero production conformance cases.

Existing normative dependencies: RootExport0159–0172 and exact pins/context/containment/subflow/runtime anchors; component contract0001–0022 and cross-dimension/proof composition; Control0124–0142 and propagation/occurrence identity; advanced0143–0146; Scope/RootRecovery0147–0158; portable0173–0196, particularly0177/0186–0194; ACE1 exact grammar. These identify existing premises, not a claim to complete rule closure. F03/F04 remain OPEN/P2. New draft clauses require separately allocated RuleIds; none is registered now.
