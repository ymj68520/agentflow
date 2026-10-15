# Complete production producer inventory at 293066e865e25dad695cd2d774f558b70eac43f3

Read-only implementation facts, **not normative justification for the proposed model**. Source lines are at this exact baseline. Future design mapping is in [compatibility](phase-04c1b4c1-existing-rust-compatibility.md); frozen premises in [gap analysis](phase-04c1b4c1-normative-gap-analysis.md).

All abbreviated `af-*/src/...` paths below are relative to `crates/`.

## 1. Direct RootExport string writers: all six sites

All six are in `af-computation/src/root_export.rs::analyze_root_export_assertion` (function786, storage839, returned report866–880). No other production function mutates/extends the returned `unresolved_obligations`. Search covers all `crates/*/src`, not current-case traces.

| ID / source line | Exact trigger / dimension / structured source facts | Exact produced string | Source identity, duplicate and insertion behavior | Rule / normative basis / candidate |
| --- | --- | --- | --- | --- |
| U1 root_export.rs:844 | Exists any nine-dimension proof whose verdict is Unproven, INCLUDING the9 preflight-blocked placeholders; full dimension report is available | `one or more static dimensions require missing or unsupported typed evidence` | Whole assertion only; no failing dimension/node/ordinal in string. One push irrespective of number of causes; first if present | Dimension's own0162–0170; root-export per-dimension sections. New model per originating evaluated fact, or explicit blockers |
| U2 :853 | Exists any dimension RequiresEvidence | `trusted implementation, subflow body, or runtime evidence is not part of this static checker` | Whole assertion; many causes collapse. One push after U1 if present | Actual affected dimension/rule, subflow-body/static-runtime boundary. Separate source-specific evidence requirements |
| U3 :855 | Unconditional, even no Scope node or failed preflight | `Scope authority attenuation is Unproven because frozen ScopeBody has no authority ceiling` | Root boundary, not an actual per-Scope occurrence; exactly one push |0172, scope-static-obligations0148; root authority-boundary identity |
| U4 :859 | Unconditional | `Scope budget attenuation is Unproven because frozen ScopeBody has no budget ceiling` | Root boundary, exactly one, after U3 |0172/0148; distinct budget-boundary identity |
| U5 :863 | Unconditional; returned body and implementation are separately RequiresEvidence | `trusted component implementation/subflow body conformance is RequiresEvidence and separate from static envelope compatibility` | Two independent root requirements collapsed in one sentence; no call-site identity; exactly one |0171/0172; separate root body, implementation and applicable call-site body identities |
| U6 :864 | Unconditional; returned runtime/publisher statuses NotEstablished, runtime evidence RequiresEvidence | `static compatibility creates no EffectGrant, ResourceHandle, reservation, receipt, or runtime authorization` | Safety prohibition, not six asserted missing runtime objects. Exactly one final push |0172 static-runtime-boundary. Keep prohibition as invariant; derive runtime and publisher requirements from their structured statuses, not by parsing object names |

Insertion order is U1?→U2?→U3→U4→U5→U6. Therefore4–6 strings, no sorting/deduplication, no per-source multiplicity. A sole Violation produces neither U1 nor U2 unless another dimension also has the relevant verdict. No producer reads preflight directly; preflight failures affect U1 through `unproven_dimensions`. Runtime/publisher have structured statuses but no dedicated strings. The same boundary strings appear with static Proven or non-positive results. These observations explain incompatibility; they are not reasons to select this ordering as normative.

## 2. All inherited obligation constructors

`root_recovery.rs` is the typed K data model/accessors (proof field182, accessor207); its proof constructors are actually in `control.rs`, not hidden emitters in that file.

| ID / function / source | Trigger and payload | Aggregation, order and duplicates | Rule / source premise |
| --- | --- | --- | --- |
| H1 `derive_missing_leaf`, control.rs:3476 | Each non-Proven proof built for missing leaf: source_node, dimension, verdict, RuleId, cloned optional reference/reason | Collect in constructed proof order; no deduplication; no full source contract/ordinal in this obligation struct | Proof propagation0142, missing leaf never empty; control-unsupported-not-empty / propagation |
| H2 `inherited_from_leaf`, :3756 | Every leaf proof whose verdict !=Proven, including Unproven, Violation and RequiresEvidence | Leaf source_node plus cloned proof fields, leaf report order; no deduplication | Leaf0111–0123 actual proof's rule; body evidence0120 and typed interface/source identity |
| H3 `inherit_child_proofs`, :4315–4340, pushes4333 | Every non-Proven child proof across ALL child reports | Child traversal then proof order. Push cloned-status parent proof at4325 and obligation at4333; reason prefixed `inherited from child computation#N: ...`. Original source_node/reference retained; ancestors may expose same origin again. No origin-occurrence token or dedup | Original child's exact rule retained; control-proof-obligation-propagation / occurrence identity |

All three construct `InheritedControlObligation` (type945–952); there are no other production struct literal sites. `derive_node`3095, `derive_leaf_node` and missing-leaf assembly attach these vectors. H3's parent proof is itself inherited at the next ancestor: this is observational duplication, not proof of a new independent source. A report cannot recover lost origin ordinals by counting repeated English messages.

## 3. Closed prerequisite writer domain

`af-contract/src/validate.rs::attach_prerequisites`323–366 overwrites each vector according to dimension. It is called after invalid-report assembly166 and normal-report assembly315. **All11 arms /20 literal occurrences**:

| ID / line / dimension | Exact strings in current insertion order | Facts / rule / proposed semantic premises |
| --- | --- | --- |
| P1 :326 schema-validity | `closed ComponentContract@1 schema`; `all exact imported schema modules compiled` | Contract0001, exact closed shape and module closure |
| P2 :330 io | `explicit TypeSchemaUniverse`; `positive unique per-direction ports` |0004, explicit schema universe and port namespaces |
| P3 :334 failure | `exact failure TypeSchemaRef pins`; `explicit TypeSchemaUniverse` |0005, nominal failure pins and universe |
| P4 :338 effects | `exact EffectOperationRef catalog coverage` |0006, operation membership |
| P5 :339 resources | `exact ResourceTypeUniverse`; `EffectResourceCompatibilityCatalog snapshot when epsilon is non-empty` |0007/0013, resource universe and conditional relation premise |
| P6 :343 information-flow | `explicit label and effect-sink catalogs`; `exact I/O/R port namespaces` |0008/0012, flow catalog and namespace premises |
| P7 :347 usage | `explicit additive meter/unit universe`; `exhaustive recovery coverage snapshot for every action` |0009/0016, meter and action-specific coverage premises |
| P8 :351 recovery | `closed effect row for complete action coverage`; `exact K clause and action identities` |0010/0014/0015/0021/0022, complete row and full action identities |
| P9 :355 execution | `explicit execution capability/profile universes`; `exact F/X relation snapshot when flow constraints require it` |0011/0018/0019, universes and conditional F/X premise |
| P10 :359 cross-dimension | `every required dimension is Proven`; `all relation catalog pins match this proof context` |0017, full required dimension roster and exact relation pins |
| P11 :363 fallback | `explicit proof context` |Generic implementation fallback; no approved extra dimension semantics. Proposal unknown dimensions fail UnsupportedModel, never treated as proven by this text |

These are *lists of preconditions*, including for Proven proofs, not assertions that every listed condition is missing. No satisfaction bit or producer-origin token is stored in each string. No dedup; same universe description is repeated across dimensions. Ordering follows source literals, not an existing normative token-order rule. Conditional P5/P9 wording cannot be interpreted as an unconditional missing requirement.

`DimensionProof::{proven,unproven,violation}` in `af-contract/src/proof.rs:37,53,69` initialize empty prerequisites and None reference. `compatibility.rs:303–325` creates twelve substitution dimension rows with empty prerequisites/None reference; setters change verdict/reason, not prerequisite collections. `LeafDimensionProof::new` (`af-computation/src/proof.rs:34–48`) and `ControlDimensionProof::new` (`control.rs:509–525`) always initialize empty private prerequisite vectors; no production mutator exists in those types. RootExport `proof_for`:1946 and `unproven_dimensions`:1275 use those contract constructors, so RootExport proofs currently have empty prerequisites. Empty here is an implementation fact, not a universal formal law.

## 4. All reference-producing forms / carriers

| ID / sites | Trigger / current form and structured identity | Duplicate / ordering / normative disposition |
| --- | --- | --- |
| R1 root_export.rs:961; helper983–1020 | Root/current resolver identities differ: six return branches: component pin sets differ; target absent; direct dependencies differ; target export differs(root/current full formatted digests or absent); additional current targets; fallback identity differs | At most one selected text, first structural difference in helper order; no typed difference enum in reference. Rule0160. Future typed resolver difference retains both manifests/target, no English matching |
| R2 validate.rs:140,160; error.rs:23 | Invalid contract reference copied into schema/dimension proof. canonical.rs:257 bare numeric port;266 `port N` | Error context chooses source; same port numbers across I/O are ambiguous without containing dimension/direction. Rule0004, full contract/port namespace required |
| R3 validate.rs:507,636 via set_dimension_at1669 | Missing selector resource or invalid qualifier profile: `effect-atom:{AtomKey:?}` | Debug is not normative encoding; full atom exists at branch. Rule0013 |
| R4 validate.rs:521 | Operation outside catalog: `operation:catalog/name` | Requires exact named ref and relation coverage, not slash parsing. Rule0013 |
| R5 validate.rs:538 | Unsupported operation/resource pair: `operation:...;resource:spec/name/schema_hex` | Complete typed operation and ResourceTypeRef already in branch. Rule0013 |
| R6 resolution.rs:508,534,600; control.rs:base3196/interface functions | Caller-local `contract-ref:N`, scoped `computation:N`, raw digest hex, opcode name, or SchemaRef Display text | None has standalone complete scope. Node/digest/source fields and caller asset must supply association. Leaf0111–0123/Control0124–0158 apply |
| R7 control.rs:3767 `target_reference` | Component contract hex; subflow export hex or target IR hex fallback; AWAIT SchemaRef Display | Different digest domains/target roles can share textual form; preserve tagged target rather than infer from hex |
| R8 control.rs:4282 `downgrade`,4320 inheritance; proof constructors | Clone/retain reference or None with verdict propagation | None is not unknown-to-empty proof; source/status remain. Copies retain full originating identity under future model |
| R9 af-portable root_revalidation.rs:926,1419,1583–1614; root_revalidation/conformance.rs:150 | Getter-only copies of obligations, prerequisites, references and guard reason into JSON | No new obligation semantics, no sorting/dedup. pipeline.rs:980–1004 retains report/private capability outcome; earlier-stage errors use PortableProofError path, not a RootExport obligation |

`set_dimension_at`1679–1684 first calls verdict arbitration and attaches reference only if retained rule AND **reason string equality** identify the winning update. Thus internal text is used as an implementation arbitration check; do not claim no code ever reads reason. This does not turn returned reason prose into a formal identity. Future typed cause IDs should control reference attachment, preserving returned semantic facts without relying on sentence equality.

## 5. Full branch coverage argument, including unvisited branches

RootExport dimensions are exclusively assembled at `prove_all_dimensions`1330. Their complete producers are `prove_inputs`1522, `prove_outputs`1779, `prove_failures`1974, `prove_resources`2041, `prove_effects`2396, `prove_flow`2893, `prove_usage`3292, `prove_recovery`3532, `prove_execution`3852, with `proof_for`1946 handling all four verdict variants. Their branch results affect U1/U2 ONLY through the exact existential predicates above. No hidden branch directly emits strings. Missing/partial/open rows, invalid maps/types/availability, unsupported guards/relations, missing meters/units, K source/profile/factor/coverage, and child feasibility/security retain their complete structured provenance requirements in the proposal, regardless of which of the26 fixtures reach them.

Core preflight901: mapping order/duplicates (`canonical_order && no_duplicate_targets`); recomputed pins (match/mismatch/recompute-error); context/pointer/manifest/resolver equality; compiler exact-current match; dependency closure success/failure. Failure invokes `unproven_dimensions`1275 for all nine with empty source facts; it is **not** evidence of nine independent evaluated failures. The body item at830–837 is appended AFTER `preflight_ok`799/static aggregation813 and never creates a seventh top-string producer. Returned statuses868–876 are unconditional root boundary facts, including publisher without a dedicated sentence.

Control/RootRecovery source families cover missing leaf, unsupported opcode, every supported control operator/interface/effect/failure/resource/flow/usage/source propagation, Scope authority/budget/settlement, CompensationScope missing child/conditional actions/local bounds/commit order/blocked outcome, retry safety and root closure/guard/multiplicity/coverage/eligibility. No-case-based filter is used. Constructor census (production before cfg(test)) is65 Control sites and27 leaf sites. These are upstream fact producers, NOT65+27 extra top-string producers. Complete call-site line census:

```text
control.rs:1297,1305,1588,1602,1610,1618,1626,1634,1642,1658,1669,1680,
3076,3088,3154,3162,3198,3206,3234,3243,3271,3288,3316,3387,3396,
3467,3497,3511,3525,3544,3557,3572,3586,3653,3702,3859,3881,3900,3912,
4288,4325,4502,4512,4524,4531,4542,4890,5048,5523,5614,5716,5724,5732,
5740,5750,5870,6225,6247,6259,6274,6282,6311,6494,6527,6664
resolution.rs:356,367,377,508,534,535,600,640,647,662,663,667,677,687,
718,719,760,779,815,838,894,937,941,985,1000,1013,1017
```

Factory argument slots admit all four existing LeafVerdict states, typed source nodes, optional references and private empty prerequisites. Inheritance is universally filtered by !=Proven, so all present/future data combinations flowing through these current sites are covered, including Violation/RequiresEvidence branches not in the26 corpus. RootRecovery types themselves have no string writer. af-contract canonical/validator/substitution factories are covered by P/R rows above; af-portable content/reference/support/limits errors reject at their true stage and have no RootExportProofReport producer. No report is invented after an early rejection.

Reproduce census read-only with `rg -n 'unresolved_obligations|InheritedControlObligation\s*\{|prerequisites|set_dimension_at|reference'` over the named production sources. Inspect generic constructors and callers, not only test coverage. Source line numbers and exact existing sentences are audit evidence only; none becomes a proposed identity component.
