# Existing Node/DagRank binding audit

## Authority, not implementation-plan labels

The machine authority inventory gives each actual SpecificationId, RuleId,
source path/anchor and statement. No draft clause below is a registered RuleId.
The descriptor application owner is
`agentflow.computation.proof-report.model@6`; original declarations/RuleIds retain
their original source owners, including model@1 anchors and 97 historical imports.

| Registered source / actual anchor | Binding conclusion |
|---|---|
| model@1, AF-SEMANTIC-0214 `model-closed-descriptors` | Full closed field/width/option/variant descriptors are mandatory |
| model@1, 0215 `model-typed-order` | Typed comparison, not container traversal or decimal-spelling order |
| model@1, 0217 `model-minimal-value-dag` | Finite minimal reachable typed input DAG; canonical bottom-up ranks |
| model@1, 0218/0219 `model-no-partial-output` / `model-closed-decode` | Resource/unknown/noncanonical failures do not return truncated success |
| model@1, 0220/0221 `model-independent-fullfield` / `model-shape-version` | Independent complete Expected; representation changes need an exact successor |
| OS@1, 0198/0200/0210 `g01-exact-scope` / `g01-subject-origin` / `g01-origin-copy-merge` | Complete inputs/associations; occurrence and observation distinctions |
| OS@4, 0269 `raw-fragment`; OS@6, 0297 `input-output-dag` | Outputs cannot establish input identity; separate finite graphs |
| Raw validation@1, 0261–0268 `raw-subject` … `raw-hook-contract` | Actual 13-field input, actual shared Context, real port prefix, source-bound evidence |
| Native guard@1, 0255 `native-links`, NG-LINKS prose | Existing Native T3 may expose lifetime-bound PendingScope/local references; complete FactKey/Scope is T4/T5 |
| Native guard@1, 0256 `native-imports` | Historical imports bind exact owner/name/full source/recursive descriptor; no same-name latest fallback |
| PP@2, 0223–0228 `edition-*`; PP@7, 0299/0300 | Exact contents/current-only staging and capability refusal |

Supplementary exact authority for SB-02's0256 reference (the machine authority
array lists0255 and0257 but does not repeat this middle row): SpecificationId
`agentflow.component.native-recovery-guard.semantics@1`, source
`spec/component/native-recovery-guard.md#native-imports`, RuleId
`AF-SEMANTIC-0256`. Registry-pinned source SHA-256:
`a78b148a57babd488c7ab27348461a8f934e2a8ee87818cfe8d93b448f7da38f`.
Its application to model@6's97 selected historical imports does not replace
their actual declaring owners with the descriptor application's current owner.

The source `spec/tables/proof-report-model-v6.md#closed-model` / 0301 registers
new closed descriptors and metadata tests, not executable full Raw capture.
`LocalTypedFactSeed` is **not** a registered ModelType key. Existing local Rust
seed types and planning documents do not supply an undocumented canonical Node
constructor. The Native registered PendingScope contract is real, but is not
permission to relabel that view as the complete Raw Node carrier required here.
`docs/implementation/phase-04c1b4c2-producer-adaptation-map.md` explicitly says
PLANNING ONLY; its site IDs are not rules or stage ownership grants.

## Four completeness predicates

Rust-value completeness means all actual storage fields/source are retained.
Registered-type completeness additionally checks the entire logical descriptor,
nominal owner/case/width/constraints. Node-carrier completeness additionally binds
the value to a valid typed rank in the exact finite canonical graph/environment.
Full G01 Fact conformance additionally compares all applicable Fact fields,
prerequisites, references, witnesses, consumers and observations independently.
None of these extra obligations follows merely from the preceding predicate.

## Exact first-field finding and limitations

The R4 machine finding, not an invented path, records:

```text
Source.raw_validation.subject : Node<RawValidationSubject>
RawValidationSubject.context : RawContextBinding
RawContextBinding.contents : ContractProofContextContents
ContractProofContextContents.compiled_modules
    : Map<SpecificationId, CompiledModuleContents>
```

This is the **first machine-recorded reproduced site**, not a global execution,
canonical-height or rank order. A declared-field depth-first static audit reaches
`RawValidationSubject.owner_compiler -> CompleteCompiler -> Compiler.specifications`
earlier: an anonymous sequence of full SpecificationContentIdentity values.
Its complete runtime compiler projection was not materialized in R4. It must be
reported STATIC REACHABLE, not promoted to an earlier actual failed binding.
No frozen rule specifies a universal producer-field traversal order that makes
one of these a global first failure.

`first-unbound-field.json` contains the full actual 16-module map value, exact
R4 Actual path/checksum provenance, expected enclosing Node and U64 rank domain,
native source types, source file paths, producing stage and missing binding.
The complete reviewed R4 `target/phase-04c1b4c4r3c2r4-binding-actual.json`
raw SHA-256 is
`b75784770fd8c1ef7eaec84a36819b68b407c7cdb2010b677e93f8e50395bb83`.
Its `/context/compiled_modules` value is copied as source evidence, not an Oracle
Expected. Fresh current diagnostics are separately executed in this phase;
reviewing this prior actual file is not counted as new vector execution.
Native source: `StaticContractProofContext.modules: &CompiledModuleSet` in
`crates/af-contract/src/validate.rs`; `CompiledModuleSet` privately owns
`BTreeMap<String, CompiledSchema>` in `af-idl/src/resolve.rs`; each CompiledSchema
has full specification and canonical type definitions (`af-idl/src/canonical.rs`).

R4's real zero event stays native Violation/AF-CONTRACT-0004. The same actual
Context object is used by analysis/canonical validation. Complete logical Context
metadata passes current shape admission. Proposed unregistered child names and
an invented Map body fail `UnsupportedModel`; a sequence-body shape may pass
only the metadata probe. No full DAG checker has accepted a malformed graph.

## Exhaustive current carrier constraints

Current `DagNode={type:ModelType,body:NodeBody}`; ModelType is exact owner/name,
not a generic structural expression or field-path constructor. Node<T> means
a typed DagRank whose referenced descriptor is exactly T. Rank is U64, scoped to
the declared finite graph, not a global ID or native address. Current NodeBody
has scalar/record/variant/sequence/set/option_none/option_some, **no map case**.
No registered named Map declaration resolves the demonstrated child descriptor.

The complete model constraints require: exact node types; all required reference
targets; acyclicity; semantic set validation; bottom-up full typed-value interning
without hash-only equality; scalar/empty-container height zero, otherwise one
plus maximum local-child height; height then type/nonrecursive-payload/ordered
child-rank ordering; contiguous ranks from zero; strictly smaller local child
ranks; profile-declared root order; reachability; no redundant equal nodes or
alternative numbering. Equal roots may repeat in the declared root roster.
Ranks are representation, never a substitute for semantic numeric comparison.

AF-ACE1-0001 supplies schema-directed encoding, not a generic graph allocator.
ACE1 0005–0007 govern NFC string-map keys, their unsigned UTF8 order and duplicate
rejection; they do **not** define arbitrary-key G01 Map<K,V> Node bodies. The
model's typed order governs arbitrary key/value domains. Existing Set body and
semantic uniqueness are present; Set is not proven to have the identical missing
body-case issue as Map. Anonymous Set/sequence/option/case payload occurrences
can still lack a closed child ModelType binding. R4's 147 static sites cover
these distinct questions, not 147 runtime failures.

The machine roster distinguishes ACTUAL REPRODUCED logical values, STATIC
REACHABLE descriptor occurrences, VALUE/VARIANT-DEPENDENT applicability and
NOT VERIFIED carrier binding. It includes all nine Context fields and full
FullSupportManifest descriptors. Existing eight/four Native-universe defaults in
the minimal probe are actual constructed empties, not omissions repaired to empty.

## Existing-stage adjudication

T3 currently has genuine local source/events, not an implemented canonical input
carrier. AF-SEMANTIC-0217 specifies the graph law/algorithm mathematically; it
does not assign an implemented pre-capture allocator to T3. G02/T4 are explicitly
pending in registered sources. The complete binding of anonymous nodes and
arbitrary maps is not supplied by a constructor/source path in the current code.
Thus Option A cannot be established. T3/T4/G02 are roadmap names rather than
new registered predicate Stage cases; existing facts keep contract_validation.
This is not proof that logical values cannot be preserved, nor permission to
implement G02. The user's exact Node/DagRank requirement remains unchanged.
