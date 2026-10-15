# Proposed exact analysis semantics edition

**PROPOSED — NOT REGISTERED. Support/materialization changes: NOT IMPLEMENTED.**

Proposed owner `agentflow.root-export-checker@2`. This is not "v1 with new text". Full identity is the exact tuple below; checksums are only consistency checks on retained bytes. Candidate final bytes/checksums cannot be known until separately approved sources exist.

| Component | Precise proposal |
| --- | --- |
| Normative source bundle | Every entry in future46-entry Specification Registry, sorted specification_id, complete logical_path/source bytes/plain SHA256. Shared paths remain distinct specification bindings. Preserve all43 old rows/sources plus OS,RM,PP2 |
| Specification registry | Exact canonical whole future specifications.json bytes; old43 entries unchanged, append3 sorted entries |
| Rule registry | Exact canonical whole future rules.json bytes, all old404 descriptors unchanged plus32 proposed rows with real conformance refs; coverage summary updated consistently |
| Closed tables | Exact current hash_domains,abi_numbers,rule_areas,afidl_primitives bytes unchanged; RM descriptor table is a new normative source, not an ABI/hash-domain table edit |
| Owner static identity specification projection | Existing16 applicable specs + PP1 semantics@1 reused non-policy constraints + OS + RM + PP2 =20 exact source bindings; every selected source includes full bytes |
| Applied rule attribution | CONTRACT0001–0022(22),SEMANTIC0111–0172(62),SEMANTIC0173–0187(15),0189(1),0191–0196(6),0197–0228(32): total138. Exclude old policy0188/0190 for owner@2 explicitly in PP2. No old RuleId redefined |
| Attribution profile | Existing full_content_revalidation/Profile A *shape*, new exact content edition. Lowering and Runtime remain ModelMissing.138 applied is an explicit applied list, NOT complete dependency closure; F03 remains OPEN/P2 |
| Compiler support table | A single implementation-owned entry for owner@2, exact46-source/whole-registry/table tuple,20 selected source identities,138 rule descriptors,10 role statuses,RM model and G01 obligation editions; unsupported historical dispatch absent |
| Report shape | agentflow.g01.report-shape@1 plus exact RM/OS content |
| Obligation identity edition | agentflow.g01.obligation@1, embedded in complete Scope and every identity; no /draft or latest fallback |
| Canonical codec | Existing ACE1 grammar; report-local descriptors/order as RM. Bundle schema remains portable.proof.inputs@1 |
| Enforcement | PP2 admission and OS strict normalization-before-private-construction. Every private route and cross-process report identifies the same owner |

138 is a **proposed direct applied count**, not a present production fact or claim that dependency closure increased from84. If a later review identifies more applied rules, change the explicit approved table and expected count before activation, never disguise them as closure. OS's role/predicate dependencies cannot be inferred from Rust report strings.

Exact successor semantic-role table (existing enum order): static_analysis,contract_validation,semantic_ir_validation,dataflow,leaf_resolution,control_algebra,root_recovery,root_export are defined;plan_lowering and runtime_semantics_profile are model_missing:not_formalized. Every defined role carries owner@2,the same20 exact selected specification IDs and all314 current named AFIDL schema pins (17 modules unchanged;RM is non-wire and adds no AFIDL named types). Dependency sets preserve the existing ordered role graph: contract_validation=[],semantic_ir_validation=[],dataflow=[semantic_ir_validation],leaf_resolution=[contract_validation,semantic_ir_validation,dataflow],control_algebra=[leaf_resolution],root_recovery=[dataflow,control_algebra],root_export=[contract_validation,semantic_ir_validation,dataflow,leaf_resolution,control_algebra,root_recovery],static_analysis=[contract_validation,semantic_ir_validation,dataflow,leaf_resolution,control_algebra,root_recovery,root_export]. Missing roles claim no definitions. Model registration does not imply execution of any role on a submitted bundle.

No normative file includes its final compiler-manifest checksum, final report or digest of itself. Raw registry source digests bind files; compilation builds the tuple from already frozen sources. Registry rules do not contain a new self-referential content identity. RM refers to semantic types by exact type/source bindings, not an own-file checksum field. Thus no circular hash fixed point.

## Current implementation deltas required

- `af-computation/src/root_export.rs`: owner version, full selected sources/applied-rule roster and content checks; capture/normalization/private constructor.
- `af-portable/src/support.rs`: remove assumptions current43/16/84; exact owner@2 table, explicit PP2 supersession, distinguish consistency from support.
- `af-portable/build.rs`: embed future46 source identities, retain unchanged Bundle set inventory; RM closed descriptor generation must be independently checked, not uncontrolled reflection.
- Production getters/projections at G02; all producer types/adapters at T3–T5.
- Generators/independent new-version corpus manifests only in new namespaces; old ones execute in old-code environment.
- `gen-registry-instances.py`: approved entry descriptors/real coverage references only; never regenerate old portable fixtures using new current registry.

Portable carries owner version and all full content using existing fields. Report/obligation editions are uniquely bound by registered RM and OS bytes in that tuple; no new Bundle field is needed. If a caller supplies report bytes separately, their explicit shape/obligation tokens must match the supported tuple. Label swaps never change owner attribution. Assertion-claimed stale compiler is a D10 comparison operand in an in-process supported owner; it is not the owner tuple and must remain intact.

## Compatibility equation

Old exact tuple != new exact tuple even if all semantic IR/ComponentContract bytes are unchanged. Old@1 source registry43 and rules404 differ from proposed46/436. ABI775,AFIDL17,HashV1 domains14 do not change. Public bundle grammar may be identical while support differs: encoding identity is not compiler semantics identity. The20-source projection and138 rule list must be constructed from complete registries, not substitute for the full46-source support tuple.
