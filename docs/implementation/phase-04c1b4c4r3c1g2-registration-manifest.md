# C1G2 — Formal registration and safety manifest

Status: REGISTERED STAGING, not Capture support or activation. Baseline
`875e6e41f16845c78d4f6d31573826c547d48c07`; final complete-code gate
`306d22253c2a2f7482ed51b5f7160bebe362e500`. Actual complete gate outcomes and
final evidence HEAD are checked after execution in the ignored runtime ledger.

## Pre-registration authority check and allocations

Before Registry mutation, the original native registration/denial test succeeded
(`target/phase-04c1b4c4r3c1g2-pre-registration-safety.log`). Source review covered
CompilerSemanticsIdentity, model admission, current-only native/Portable D4 routing,
native analyzer/direct checker/private constructor and complete support states.
EditionSupport has only NotImplemented; Activation/Release only NotAuthorized;
can_issue_capability cannot become true from registration or caller metadata.
Portable D4 independently refuses support after exact content admission. No atomic
activation dependency exists. Raw Source admission returns unit only; complete
profile admission remains Err(NotImplemented), and root conversion UnsupportedModel.

Collision census used the frozen complete54 Specifications/468 Rules, existing
registered model tuples, initial2021 paths, descriptor names and existing RuleIds.
Highest registered Semantic RuleId was0260; new names/paths and owner tuple were
unoccupied. Full checks and actual allocations, not a presumption of next numbers:

| Role | Registered successor identity | Source |
|---|---|---|
| Analysis owner | agentflow.root-export-checker@5 | compiler implementation and portable semantics |
| Proof model | agentflow.computation.proof-report.model@4 | spec/tables/proof-report-model-v4.md |
| Report | agentflow.g01.report-shape@4 | new model/portable tuple |
| Obligation | agentflow.g01.obligation@4 | new model/obligation tuple |
| Raw substantive profile | agentflow.component.raw-validation.semantics@1 | spec/component/raw-contract-validation.md |
| G01 semantics | agentflow.computation.proof-obligation.semantics@4 | spec/computation/proof-obligation-semantics-v4.md |
| Portable semantics | agentflow.portable.proof.inputs.semantics@5 | spec/portable/portable-proof-inputs-v5.md |

Four new Specification records and eleven rules yield **58/479/478/1/0**.
Report/Obligation/Analysis identifiers are edition roles, not extra phantom
Specification records. No new ABI assignment, AFIDL module, ACE1 grammar or HashV1
domain was added. Exact current compiler selected-source/applied counts are32/174;
its registered source support view includes all58 source contents and four full
closed tables. Attribution is not a transitive dependency closure; F03 stays open.

## Accepted clause -> source -> rule -> executed registration case

Each row has its exact registered source/anchor/statement/dependencies in
`tools/raw_validation_registration_data.py` and `rule-registration.json`.
The Rust registration test resolves every full Registry row and unique source
anchor. These executions prove registration correspondence, not full T3 semantics.

| Accepted/new clause | RuleId | Source anchor | Executed CaseId suffix |
|---|---|---|---|
| RC-SCOPE | AF-SEMANTIC-0261 | raw-subject | rule-0261 |
| RC-CONTEXT | AF-SEMANTIC-0262 | raw-context | rule-0262 |
| RC-STAGES | AF-SEMANTIC-0263 | raw-stages | rule-0263 |
| RC-PREDICATE | AF-SEMANTIC-0264 | raw-port-predicate | rule-0264 |
| RC-LOCATION | AF-SEMANTIC-0265 | raw-origin | rule-0265 |
| RC-WITNESS | AF-SEMANTIC-0266 | raw-witness | rule-0266 |
| RC-LINKS | AF-SEMANTIC-0267 | raw-links | rule-0267 |
| RC-HOOK (contract only) | AF-SEMANTIC-0268 | raw-hook-contract | rule-0268 |
| RC-FRAGMENT (OS extension) | AF-SEMANTIC-0269 | raw-fragment | rule-0269 |
| RC-DESCRIPTORS (table closure) | AF-SEMANTIC-0270 | raw-descriptors | rule-0270 |
| RC-MIGRATION | AF-SEMANTIC-0271 | raw-migration | rule-0271 |

The first8 anchors are in Raw semantics, fragment in new G01 semantics,
descriptors in model@4, migration in portable semantics@5. CaseId prefix is
`g01-raw-validation-staging/`. The substantive port rule remains
**AF-CONTRACT-0004 / component-contract-ports**, not a replacement new governing
port algorithm. There is exactly one initial raw_contract_validation case:
positive_unique_io_ports. Kind prerequisite; supporting_fact; static_declaration;
contract_validation stage; exact substage canonical_validation/io_port_check;
Requirement raw_validation:positive_unique_io_ports. It is not schema validity,
type-pin validity, the whole canonicalizer or generic invalid_contract_report.

## Full subject, physical source, context and stage boundary

All31 new authored types are concrete in the683-type/2018-reference model; all
references resolve. The97 historical selected imports and19 frozen source bundles
are copied unchanged;90 live declarations are separately compared. Old652 type
ranks form an exact preserved prefix. Only the declared parent append operations
occur in the new model; the complete old model@3 is embedded unchanged.

RawValidationSubject contains complete compiler, RawParsedInput and RawContextBinding.
RawParsedInput has the complete13-field RawCandidateContract, RawInputOrigin and
RawPhysicalAssociation. Document variants are physical_json_utf8{bytes} and
declared_native_document{declarations}; no opaque fallback or arbitrary unparsed
JSON replaces the typed candidate. RawDocumentMember byte-name/index path is not
implicitly cast to DeclaredStep. Physical content is original bytes, never the
reserialized parsed value, display text or a canonical contract digest.

The physical source admission API deliberately consumes the registered **non-wire
Raw metadata JSON representation**, with exact byte arrays, fields, variants,
options and widths; it is not an added AFIDL/Rust wire decoder. It rejects duplicate
JSON keys recursively, malformed UTF8/structure and unknown/missing typed fields.
It checks the selected complete candidate at the actual supplied path. Native
admission instead borrows actual ComponentContract declarations and checks the
full candidate against the real selected declaration. Neither API constructs a
document by default, reconstructs historical physical bytes or manufactures a
trusted validator result. A fresh source association is not historical provenance.

No source document -> MissingDeclarativeOrigin. Physical not_supplied is legitimate
for native input but is not KnownEmpty. Same parsed values with different original
physical bytes remain different source contents. Source APIs borrow immutable input;
they do not normalize NFD or mutate the original bytes. nfc_text applies only at its
own formally declared field stage. A diagnostic NFC display is output, not source.

The complete actual nine-member native context uses compiled_modules/type_schemas/
effect_catalogs/resource_types/flow_catalogs/usage_meters/recovery_operations/
execution_universes/relation_snapshots. Relation snapshots retain the three named
optional effect_resource/recovery_usage/flow_execution slots and full content.
Both roles analysis and canonical_validation refer to the same actual context
object; partition [[analysis,canonical_validation]] is exact. Metadata checks the
closed partition; source APIs use std::ptr::eq on real borrowed contexts. Equal
contents of another object do not pass. No address is serialized, and no resolver,
caller IR or synthetic root is added. Existing resolver-bearing subjects are not
weakened; an extra KnownEmpty resolver is rejected by the native-context shape.

The five stages remain separate: physical input, parsed Raw declaration, diagnostic
presentation, actually canonical validated declaration, actually Checked contract.
An early rejection has neither a successful canonical identity nor Checked. A full
legacy positive contract check may still legitimately create ComponentContract
Checked, but no new local port P/source API produces it or RootExport capability.

## Port execution, source location and evidence

The unchanged validator first checks component identity, then inputs, then outputs.
Each namespace uses its actual stable numeric U32 sort, preserving original
occurrences. Positivity/uniqueness is checked before the corresponding schema query.
Original ordinal, sorted ordinal, direction, U32 port number and real predecessor
are distinct fields. Sentinel previous_number=0 is not an actual predecessor.
Fresh positive ->P;zero/duplicate ->V; input/output number equality is not a duplicate.
Earlier identity rejection, schema failure after a positive port, stopped later
events and empty namespaces have explicit applicability, not fabricated evaluated U.

Source.raw_validation retains the complete subject. Target.raw_declaration_constraint
has exact compiled declaration schema/full rule reference/direction/constraint;
this association is not an invented runtime shape query. Coordinate.raw_port names
the actual occurrence and port_no member. RawValidationOrigin has the complete real
document/path/direction/original ordinal. The new required optional Origin field is
Some exactly for this family and None for unaffected successor facts. The genuine
typed DeclaredStep path through input.candidate and RawDocumentMember path must
resolve to the same original port. No runtime call count, ProbeId, report-array
position, Rust line, UUID, fabricated node or contract digest stands for Origin.

RawPortCheckBasis retains all closed inputs/condition/error/predecessor/rule.
Witness.zero has the actual U32(0),minimum U32(1) and complete occurrence; duplicate
has both real original positions and the same positive number/namespace with actual
sorted-predecessor relation. P has no contradiction; V requires the matching typed
witness. It contains no Guard assignment, resolver result, Root countermodel, receipt
or Checked token. Diagnostic detail cannot select zero versus duplicate.

Fact.raw_validation_basis, TypedContradiction.raw_validation, EvidenceRef.
raw_validation_member and ObservationOwner.raw_contract_validation are closed new
extensions. RuleRef is not EvidenceRef. Input Scope excludes generated Fact/Witness
descendants; witness output has its own finite reachable rank space. Prerequisites
keep original stage/source/origin/support/consumers. One local port P cannot prove
full namespace coverage; later Root observers need genuine consuming source/Checked
association. Identical originating copies add observations, not facts; independent
declaration entries remain distinct. Normalization/merging is not implemented here.

The future hook contract requires branch-created typed evidence inside canonical_ports
before string reduction; the outer analyze_contract hook only assembles it before
invalid_contract_report. Raw/context/evidence lifetimes bound any future local view.
This registration implements **no hook, branch evidence producer or Raw Fact Seed**.
Metadata shape/association success returns unit and does not satisfy that missing
execution premise. Complete profile support remains NotImplemented.

## Conformance, routing, limits and freeze

262 new registration CaseIds =241 schema-derived shape cases +11 rule cases +8
portable matrix cases +2 runtime portable adversarial cases. The129 required new
field/variant sites have exact bidirectional links to executed shape CaseIds.
Manual shaped witnesses do not establish actual validator-produced evidence.
Additional real source-admission/port observations are Rust test groups, not extra
certified Capture CaseIds. Existing2 NFC diagnostics/1 Raw gap probe also are not
new complete Capture. The original40 G1 design vectors stay unchanged Not Executed;
applicable registration laws were separately materialized, not relabeled en masse.

Current-only full-content identity routing rejects old-label/new-content, new-label/
old-content, missing model source, foreign ModelType/ModelRef owners, unknown variants
and editions. The extra caller supported flag and noncanonical current bytes are
tested against the unchanged closed portable wire decoder. Source/metadata errors
do not become ProofVerdict V. No empty set or metadata success supplies support.

Production decoder defaults remain unchanged (1 MiB scalar,32 MiB input). New model
source is about5 MiB, with a hex transport scalar over8 MiB, so new D4 denial tests
explicitly request16 MiB scalar through the existing API; default ResourceLimit is
a separate case. Native identity retains32 MiB aggregate limit; shape probes retain
depth96/work262144. Physical source admission has8 MiB input/path96, and native
document admission has262144 declaration bound. Resource errors return no success.
No new cap rewrites a validator Verdict or activates production support.

The initial2021 original path/raw-byte map was captured before changes. The committed
freeze manifest protects2011 paths; the10 reviewed exceptions are two append-only
Registry files, six Metadata/routing/Cargo source files and two generator files.
All old Expected, original Oracles, Port Validator and Bytes repair sources remain
byte-exact. Raw checkout hashes are used, not Git-normalized text: existing CRLF
documents/Expected bytes are not silently canonicalized. New58-source pins are all
verified. Old54/468 records compare as complete JSON objects, not sampled fields.

The old Native generator now uses the exact frozen Edition4 Registry, preserving
all889 old cases and their compiler/portable Expected. Historical1/2/3 and old4 run
their exact immutable builds. New Source metadata/registry cases run separately.
G01/T3 incomplete; G02 P1/G03 P2 open; F01/F02 closed; F03/F04 P2 open; original
B01–B07 blocking. R3-NG-B01 remains closed on fresh53/62/40 gates. Registration
only establishes a formal model prerequisite for separately requesting R3 resumption.
