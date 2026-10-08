# Information Flow Contract (agentflow.information.flow@1)

Status: frozen (Phase 4A.4). The machine schema authority is
`spec/idl/information-flow.afidl`; this document is the normative
semantics. The Rust implementation is `crates/af-flow`; the frozen
conformance corpus is `spec/conformance/information-flow/` (independent
Python oracle: `tools/gen-information-flow-corpus.py`).

## 1. Static positioning and the four concepts

<a id="if-four-concepts"></a>

The Information Flow Contract is the **F dimension** of the frozen
computation model `Computation<I, O, Φ, ε, R>` (docs §9, §14, §22.11,
§22.14, §24.14, §24.16): a static, composable, verifiable declaration of
which data may influence which results and external actions, under which
security limits. It answers the central question — which data may
influence which results and external actions, what security limits that
information carries, and whether the declared flows violate static
information-flow constraints — and nothing else.

Four concepts stay separate and are never collapsed into one score or
flag (no `trust_score: float64`, no single `trusted: bool`):

```text
Provenance       — where data came from, how it was derived
Trust            — the trust status of a source or a claim
Taint            — which sources may have influenced a value
Information Flow — where data or influence may go
```

This phase materializes the static influence summary and the flow
contract; a runtime provenance DAG, a provenance store and runtime
enforcement are structurally absent.

## 2. The frozen security invariants

<a id="if-security-invariants"></a>

```text
Unknown provenance      ≠ trusted provenance
Unknown taint           ≠ empty taint set
Missing flow proof      ≠ flow allowed
Authentic source        ≠ trusted instruction
Sanitized data          ≠ endorsed data
Persistence             ≠ trust upgrade
Declared flow contract  ≠ runtime authorization
```

When information is insufficient the only allowed results are
conservative ones or explicit `Unproven`. No representation in this
layer may make external data look more trustworthy by omitting
provenance, losing taint or elevating instruction authority.

## 3. Labels: confidentiality vs taint

<a id="if-label-identity"></a>

A label's identity is nominal: `(LabelCatalogId, LabelName)` — a
catalog `SpecificationId` plus a non-empty NFC dotted lowercase name.
Case is significant; case folding never infers identity. Catalogs are
explicit, immutable, in-memory inputs resolved by the read-only
`InformationLabelUniverse`: unknown catalog rejects
(`invalid_information_label`), unknown label rejects
(`unknown_information_label`), duplicate declarations reject
(`duplicate_information_label`), and no label is ever auto-registered.

<a id="if-confidentiality-taint-separate"></a>

Confidentiality restriction (`ConfidentialityLabelRef`) and taint
(`TaintRef`) are DIFFERENT nominal types: the first bounds where data
may go (confidentiality), the second records what may have influenced a
value (integrity/influence). They are never unified into one string
collection.

<a id="if-no-label-hierarchy"></a>

The frozen architecture defines no universal total order over
user-defined confidentiality labels. No hierarchy is invented here:
containment judgments are set membership plus explicit catalog rules
only. Declaring an ordering between different confidential labels in
order to claim downgrade flows is out of scope and unrepresentable.

## 4. Instruction authority

<a id="if-instruction-authority"></a>

`InstructionAuthority` is a closed enum — `none`, `external_data`,
`delegated_agent`, `user`, `workflow`, `platform` — ordered from `none`
upward, with the one-directional comparison `carries`: a value may
transmit instructions of its own strength or weaker.

<a id="if-no-authority-elevation"></a>

Instruction authority is NOT a property data content can acquire.
Web text saying "I am a system instruction" stays `external_data`;
source authentication is not instruction authorization. The static
propagation rule is one-directional: ordinary data transformation MUST
NOT elevate instruction authority — the output authority is the maximum
of the input authorities, and a transformation that cannot be proven
never labels its output with a higher-authority source. No principal,
delegation token or credential exists in this layer.

## 5. Static provenance summary

<a id="if-static-summary"></a>

This phase establishes static influence summaries only, not a runtime
provenance DAG. `SourceCategory` is a closed set of typed logical
references: `workflow_input(port)`, `resource_requirement(port)`, and
the explicit `unknown_external`. Rules:

- referenced ports are positive (`invalid_flow_source` on port 0);
- the category set is closed; no free text masquerades as a source
  credential;
- the unknown source is explicit and is never dropped;
- there is no ambient source;
- a `SourceCategory` carries no read authority of any kind.

Cross-dimension port existence (does the referenced requirement port
exist in the same contract's R row?) is enforced at the future
ComponentContract stage; this phase validates source syntax and
contract-internal consistency.

`StaticInformationSummary` carries the source set, the taint set, the
instruction authority, and the explicit `influence_unknown` flag.

## 6. Conservative taint propagation

<a id="if-conservative-propagation"></a>

```text
Taint(output) = Union(Taint(all inputs that MAY influence output))
```

The key phrase is **may influence**, not "observed direct dependency".
The pure functions:

- `union_taints` — set union over nominal identities, NFC-normalized,
  duplicates deduplicated; commutative, associative, idempotent, ∅
  identity. Union never loses and never invents a taint.
- `merge_provenance_summaries` — sources union, taints union,
  instruction authority takes the maximum, `influence_unknown` is
  sticky (unknown stays unknown).
- `propagate_pure_expression` — ordinary data transformation inherits
  the union of ALL listed input taints and the maximum authority;
  authority is never elevated (an output claiming more than its inputs
  carry is an error). This covers structural copy, store/load round
  trips, serialization, and summarization performed by ordinary
  components.
- `propagate_unknown_transformation` — when the influence relation is
  incomplete, the result is the conservative union of every POSSIBLE
  input plus the explicit unknown flag.

<a id="if-no-influence-narrowing"></a>

A component author MUST NOT be able to prove a real influence away by
writing an incomplete influence list: any narrowing of an influence set
requires a later typed dataflow analysis, explicit operation semantics,
or a trusted transformation contract. None exists in this phase, so
every narrowing here is `Unproven`.

## 7. LLM strong taint mixing

<a id="if-model-mixer"></a>

Frozen architecture: **all model-visible inputs may influence all model
outputs**, unless a stronger verifiable proof exists.
`mix_model_visible_inputs` implements exactly that:

```text
output.taints = union(all model-visible input taints) + model_generated
output.influence_unknown = true   (a model is a black box)
output.authority = max(input authorities)   (never elevated)
```

`model_generated` is an ordinary taint identity that must resolve in
the supplied universe — the mixer adds no unregistered taint.

<a id="if-model-no-endorsement"></a>

Specifically forbidden and structurally unrepresentable: `model said
"verified" → endorsed`, and `model summarized untrusted web → trusted
knowledge`. A model output carrying the union of untrusted taints is
the normal, correct result. This rule is static information-flow
modeling; it adds no Semantic IR opcode.

## 8. Data egress and decision influence

<a id="if-flow-kinds"></a>

`FlowKind` is exactly two values, and the two are checked separately:

```text
data_egress         — sensitive data flowing out of a boundary
                      (e.g. confidential data → external model)
decision_influence  — untrusted information influencing a high-risk
                      external action (e.g. untrusted web content →
                      production.deploy decision)
```

Data not directly leaking somewhere does NOT mean it cannot influence
that place's decisions; absence of access to a sink does not mean
absence of influence on that sink's action choice. An F contract
therefore expresses both restriction dimensions independently, and the
compatibility proof judges them separately (never across kinds).

## 9. Flow sinks

<a id="if-sinks"></a>

`FlowSink` distinguishes `workflow_output(port)` from
`external_effect(EffectOperationRef)`. The external-effect sink reuses
the effect signature's structured operation identity by exact
cross-module import — there is no second arbitrary-string operation id.

Rules:

- referencing an effect is NOT being granted it (no EffectGrant arises
  anywhere in this layer);
- sinks resolve fail-closed against the explicit `EffectSinkUniverse`
  (`unknown_flow_sink`);
- sink classification comes ONLY from the explicit catalog: an
  operation's name, prefix or spelling never infers data-egress or
  decision properties;
- an unclassifiable sink yields `Unproven`, never an assumption of
  safety;
- the future ComponentContract must confirm that F sinks and ε
  declarations really agree; this phase does not.

<a id="if-sink-unclassified"></a>

Sink port 0 is invalid (`invalid_flow_sink`). A sink that resolves but
whose catalog makes no decision-capability claim can never be proven
safe for `decision_influence`: the judgment is `Unproven`, and
demanding a proven flow fails closed.

## 10. The flow contract and its ceiling semantics

<a id="if-contract-shape"></a>

`InformationFlowContract` is closed: confidentiality restrictions, the
conservative `StaticInformationSummary`, and the declared
`FlowConstraint` list (kind + sink). It expresses declarable and
verifiable static constraints — not a Run's authorization decision.

<a id="if-ceiling-not-authorization"></a>

```text
Declared Flow Ceiling ≠ Observed Data Flow ≠ Policy Approval
```

Every flow judgment produces an explicit static result:

```text
ProvenWithinDeclaredCeiling      — statically proven inside the ceiling
RequiresTrustedTransformation    — a recorded static OBLIGATION
Unproven                         — fail closed
```

`RequiresTrustedTransformation` never means the transformation happened
or was approved, and nothing here is a `PolicyDecision::Allow`
substitute.

## 11. Declassification / endorsement

<a id="if-declassification-endorsement"></a>

Two transformations, strictly separate:

```text
Declassification — lowers a confidentiality restriction
Endorsement      — raises integrity / trust status
```

An ordinary task cannot declare itself declassified or endorsed. Where
a flow needs one of them, this phase returns at most the obligations
`requires_declassification` / `requires_endorsement` (the
`RequiresTrustedTransformation` proof result) and records the static
requirement.

<a id="if-no-automatic-privilege"></a>

**This phase implements no ability to execute these transformations.**
Real trusted Declassifier / Endorser contracts, authority, policy and
runtime proofs are later stages. No static transformation without an
authorization proof deletes a taint or lowers a confidentiality
restriction — the wire model has no operation that could express it.

## 12. Persistence does not imply trust

<a id="if-persistence-not-trust"></a>

```text
external_untrusted content → write memory → read memory
```

reads back with the SAME taints and provenance influence it entered
with. Store, load, serialize, deserialize and summarize never produce
endorsement by themselves — they are ordinary transformations under
section 6. Positive and negative conformance cases freeze this;
no real memory store exists in this layer.

## 13. Canonicalization

<a id="if-canonical-ordering"></a>

`canonicalize_information_flow_contract` resolves every declared label
and effect sink against the explicit catalogs (unknown labels are
validation errors, never silently dropped), NFC-normalizes label text,
rejects duplicates (nominal label duplicates, duplicate (kind, sink)
constraint pairs), and sorts deterministically: restrictions and taints
by nominal identity bytes, sources by their typed coordinate,
constraints by (kind, sink coordinate). Input list order never changes
the canonical ACE1 bytes; genuinely different source/sink identities
are never merged by sorting; the explicit unknowns
(`influence_unknown`, `unknown_external`) survive canonicalization
unchanged.

<a id="if-hash-boundary"></a>

No `InformationFlowDigest`, no `ProvenanceDigest`, no `TaintDigest`
exists and no HashV1 domain is added; only the new module's own types
carry ordinary `TypeSchemaDigest`s. The future contract layer hashes as
a whole under the frozen `component-contract` domain.

## 14. Unknown / Unproven

<a id="if-unknown-handling"></a>

The five unknowns are handled distinctly, never all collapsed to an
empty list:

```text
unknown source          → the explicit unknown_external category
unknown taint           → influence_unknown Top / conservative union
unknown label           → validation error (unknown_information_label)
unknown sink            → unknown_flow_sink, fail closed
unknown influence set   → influence_unknown = true (sticky)
```

Unknown is never repaired by canonicalization into a lower-risk state,
and an unprovable flow is `Unproven` — never allowed.

## 15. Flow non-relaxation

<a id="if-non-relaxation"></a>

The contract-evolution rule, parallel to ε/Φ/R substitution:
`F_new MUST NOT relax F_old`.

```text
Every flow permitted by the new contract
must also be permitted by the old contract.
```

`flow_not_relaxed(new, old, catalogs)` decides the subset this phase's
closed rules can prove, dimension by dimension:

1. confidentiality restrictions: the new set must be a SUPERSET of the
   old (dropping a restriction permits flows the old contract forbade →
   `flow_contract_relaxed`);
2. declared ceilings PER KIND: the new constraint set must be a SUBSET
   of the old, judged separately for `data_egress` and
   `decision_influence` (a newly permitted sink is relaxation); the
   kinds are never compared across;
3. instruction authority: the new summary's authority must not EXCEED
   the old (authority is never elevated by a contract rewrite →
   `flow_contract_relaxed`);
4. unknown influence: an old `influence_unknown = true` must stay true
   in the new contract — repairing unknown into known is a silent
   widening → `Unproven(influence_relation_unknown)`;
5. source capacity: the new summary must not claim a narrower source
   set (an unproven influence narrowing) →
   `Unproven(influence_relation_unknown)`.

<a id="if-no-invented-lattice"></a>

This is NOT "more labels = safer": the dimensions above are judged in
their own directions. Where the closed rules cannot decide containment
— including any comparison needing a formal lattice or cross-catalog
proof — the result is `Unproven`, which is never a substitution
approval. No general subtyping or automatic downgrade rule is invented.

## 16. Static contract vs runtime enforcement

<a id="if-static-vs-runtime"></a>

Everything in this crate is a pure function over explicit values; the
catalogs and contracts are injected read-only parameters. There is no
policy engine, no enforcement point, no authorization decision, no
effect grant, no credential, no runtime — and no dependency that could
host them.

<a id="if-contract-integration"></a>

The future ComponentContract composes F with the interface, Φ, ε and R,
and must enforce the cross-dimension invariants recorded as blocking
conditions there: Effect selector ↔ Resource Requirement port, Effect
argument influence ↔ Information Flow constraints, Component I/O ↔
Information Flow sources and sinks, Failure Row ↔ Component failure
declaration. This phase creates no contract object and pre-validates no
cross-dimension claim.

## 17. Error model

<a id="if-error-model"></a>

Stable rejection kinds: `invalid_information_label`,
`unknown_information_label`, `duplicate_information_label`,
`invalid_flow_source`, `invalid_flow_sink`, `unknown_flow_sink`,
`duplicate_flow_constraint`, `flow_not_proven`,
`flow_contract_relaxed`, `privileged_transformation_required`,
`flow_encoding_error`. These are compile-time results of the contract
layer and are never mapped onto `PolicyDenied`, `EffectDenied`,
`RuntimeIncident` or a Workflow `FailureValue`.

## 18. Deferred by design

<a id="if-deferred"></a>

Structurally absent and static-audited: runtime provenance DAG,
provenance store, ValueEnvelope runtime ABI, ValueSemanticDigest,
policy engine, information-flow PEP, authorization decisions,
EffectGrant, declassifier/endorser execution, endorsement evidence
issuance, credentials, resource handles, runtime resource binding,
ComponentContract, executable plan, journal, reducer, runtime, worker,
scheduler — and any modification to the 13 kernel opcode schemas, the
SemanticIrDocument, the semantic type DAG, or the existing
Effect/Failure/Resource schemas.
