# Scope, CompensationScope, and Workflow Closure Static Algebra (Phase 4B.2B.3)

Status: frozen derived-only semantics for the existing `SCOPE@1` and
`COMPENSATION_SCOPE@1` opcodes and the workflow-root closure over all thirteen
Semantic IR computation opcodes. This specification adds no AFIDL field,
opcode, ABI assignment, digest domain, runtime identity, authorization,
receipt, journal, plan, or execution mechanism.

The analyzer consumes a checked Semantic IR document, Phase 3C dataflow
evidence, resolver-created leaf summaries, and the same immutable static proof
context used by `derive_control_contract`. Its outputs are in-memory analysis
summaries, never a `ComponentContract`, authorization decision, runtime
observation, or implementation-conformance attestation.

## 1. Frozen input boundary

The current formal Semantic IR definitions are decisive:

```text
ScopeBody             = { child: ComputationId, resources: Vec<ResourceSlotId> }
CompensationScopeBody = { child: ComputationId }
```

`SCOPE` owns exactly one child and declares the ResourceSlot set visible to that
child. It has no authority ceiling, budget ceiling, or detached-child list.
`COMPENSATION_SCOPE` owns exactly one forward child; it has no compensation
task list, handler body, receipt, or commit-order field. Compensation actions
are resolved only from the child's checked ComponentContract Recovery/K
bindings and matching Usage bindings. No action is inferred from an Effect
operation name.

The 13-node opcode set remains closed and unchanged: `INVOKE`, `AWAIT`,
`SEQUENCE`, `SELECT`, `PARALLEL`, `RACE`, `ITERATE`, `RETRY`, `TIMEOUT`,
`HANDLE`, `SCOPE`, `COMPENSATION_SCOPE`, and `CALL_SUBFLOW`.

## 2. Scope ownership, visibility, and propagation

<a id="scope-resource-attenuation"></a>

### 2.1 Checked ownership and no detach

`SCOPE` membership is the checked IR ownership edge, not an inferred runtime
group. The ordinary IR validator and Phase 3C analysis remain authoritative for
tree ownership, cycle/duplicate-parent rejection, child reachability, Value
visibility, and ResourceSlot lexical visibility. The derived analyzer does
not create an implicit detached child or silently omit an owned computation.
The owner-terminal constraint is represented separately as a lifecycle
obligation: every owned child must settle before the owner can be terminal.
Static ownership does not prove that a particular runtime child has settled;
that claim remains `RequiresEvidence`.

### 2.2 Resource attenuation

For each checked scope node, the analyzer reports the Phase 3C entry-resource
set at the scope, the exact sorted `ScopeBody.resources` set, and the child's
Phase 3C entry-resource set. Resource attenuation is `Proven` only when the
declared set is a subset of the parent-visible set and the child-visible set is
exactly the declared set. A violation is not converted to an empty resource
set. ResourceSlot visibility is not a `ResourceHandle`, binding, grant, or
proof of access authorization. Existing source component port, caller slot,
alias, requirement, and occurrence provenance remains intact.

### 2.3 Static facts and unresolved dimensions

<a id="scope-static-obligations"></a>

Scope composes its child's existing dimensions; it is not an implicit
compensation operator. Child Failure, Effect, Flow, Usage, Recovery/K, Resource,
Execution/X, typed interface, and occurrence facts are preserved under the
existing operator-specific algebras. It creates no rollback, reconciliation,
refund, or compensating Effect. Cancellation or owner termination does not
discount a possible child Effect or Usage.

The analyzer distinguishes declared static facts from absent runtime inputs:

| Dimension | Static result | Boundary |
| --- | --- | --- |
| Resource visibility | Exact Phase 3C set comparison | Does not mint a handle or permission |
| Authority attenuation | `Unproven` | No Scope authority ceiling, `RunAuthorityEnvelope`, or `DelegationGrant` is an input |
| Budget attenuation | `Unproven` | No Scope budget ceiling; Usage is not a reservation or ledger balance |
| Child settlement | `RequiresEvidence` | Ownership does not observe actual terminal state |
| Flow and confidentiality | Child evidence and unknowns preserved | Scope does not create trust or erase provenance |
| Execution/X | Per-child requirements retained | No Worker placement, admission, or aggregate capacity is inferred |

No absent authority or budget field is interpreted as unlimited or as a
successful check. `Forward` and `Recovery` Usage coordinates stay separate.

## 3. CompensationScope and conditional recovery

<a id="compensation-forward-history"></a>

### 3.1 Forward history and outcome

`COMPENSATION_SCOPE` preserves the child's Forward Effect envelope, typed
Failure row, Flow, Resource, Usage, Recovery, Execution, and output interface.
Forward Effect identity includes the complete `EffectAtom` (operation,
resource selector, qualifier family/member/payload), guard, occurrence bounds,
source contract, and scoped occurrence identity. A possible compensating
action is a separate conditional new Effect; it is never subtracted from the
Forward envelope. A successful recovery action cannot rewrite the original
Forward outcome or refund Forward Usage.

`RecoveryBlocked` is a recovery outcome/obligation, not a business
`FailureTypeRef`; ordinary `HANDLE` does not consume or erase it. The analyzer
does not claim that a run actually entered or exited a blocked state.

### 3.2 Exact K identity and action envelope

<a id="compensation-exact-action-envelope"></a>

The analyzer reuses the frozen RecoveryActionIdentity:

```text
(forward EffectAtom, action kind, action EffectAtom)
```

Only a matching checked K binding and exact RecoveryUsageBinding can
contribute. A matching action has conservative potential upper bound:

```text
forward occurrence upper
× checked occurrence-family upper bounds
× activations-per-forward upper
× attempts-per-activation upper
```

Arithmetic is checked and fail-closed. The action lower bound is zero because
static declarations do not identify which concrete effects were committed or
eligible. A nonzero potential action therefore retains receipt/effect-state
`RequiresEvidence`; an absent or incomplete K/Usage source remains
`Unproven`/partial rather than becoming finite zero. Action source and forward
occurrence identity stay explicit.

For each source node, contract pin, and action EffectAtom, the summed K-derived
potential action upper (plus the Forward upper when the exact atom is also the
Forward atom) MUST fit the exact declared `epsilon` action-Effect occurrence
after applying the same enclosing occurrence-family bounds. Undercoverage is a
static `Violation`; checked sum/product overflow is fail-closed. This prevents a
known K action ceiling from exceeding the Effect row that claims to cover it.

When the exact full Forward and action atoms are equal, the one closed epsilon
entry is an aggregate ceiling, not two separately declared entries. If its
upper is `C` and the complete same-atom K binding set has checked multipliers
`m_i = activation_i.upper × attempts_i.upper`, the greatest Forward seed
compatible with the shared ceiling is `floor(C / (1 + sum(m_i)))`; each K
action contribution is that seed times its own `m_i`. A known Forward lower
bound greater than the inferred maximum is a `Violation`. If a different
Forward atom contributes to the same action atom and its count cannot be
isolated from the shared ceiling, the proof is `Unproven`. The analyzer MUST
not use the aggregate epsilon upper as the Forward seed and then multiply it
again as K, nor collapse the Forward and K occurrence into one event.

This is a conservative possibility envelope, not an instruction to execute
each action. It includes every resolved action source in the owned subtree,
including possible loser-branch Effects; it does not select a RACE winner or
assert that an Effect was applied.

### 3.3 Receipt, reconciliation, and order

<a id="compensation-receipt-usage-and-order"></a>

Static analysis observes no `EffectReceipt`. It cannot classify an occurrence
as `APPLIED`, `NOT_APPLIED`, `PARTIALLY_APPLIED`, or `UNKNOWN`, and does not
pretend the absence of a receipt means `NOT_APPLIED`. Unknown/partial effect
state requires runtime reconciliation before a safe concrete action can be
chosen. Reconciliation establishes effect reality; compensation is a new side
effect. Neither a provider call nor a recovery decision is performed here.

Static source order is not actual committed-effect order. If more than one
potential compensation event requires ordering, the runtime order remains
`RequiresEvidence` until committed receipt/journal order is available. The
static summary creates no sequence number, journal, or compensation command.
An at-most-one event may have a statically unambiguous order, but that does not
prove action eligibility or execution.

The derived action also retains the matching `epsilon` action-Effect occurrence
and its scoped ResourceSlot mapping, alongside the exact forward occurrence.
The Recovery/K model has no per-action Flow or Execution field: those
dimensions remain source-contract/per-child obligations and are not narrowed
or converted to an action-specific approval. No ResourceHandle or Worker
placement is inferred.

### 3.4 Recovery Usage

Recovery Usage is derived separately from Forward Usage, using the exact
per-attempt K usage and the potential action upper bound through the existing
checked Usage algebra. Missing meter coverage remains unresolved; overflow is
a violation. Compensation does not refund or decrease Forward Usage. No
reservation, actual usage receipt, or runtime budget state is produced.

## 4. Full checked control-tree closure

<a id="workflow-control-tree-closure"></a>

`derive_workflow_contract_summary` resolves every reachable
`INVOKE`/`AWAIT`/`CALL_SUBFLOW` leaf from the supplied immutable resolution
snapshot, then invokes the existing bottom-up `derive_control_contract` over
the checked root. It is not a second analyzer. The checked computation walk
uses semantic child order and must account for every declared computation
exactly once; a repeated, missing, unresolved, or unaccounted node is an
explicit error, not an empty contribution.

The closure report records computation preorder, leaf preorder, and whether
all known scoped Effect occurrences, Effect residuals, and Recovery/K source
nodes match the resolved leaves. It reuses Phase 3C instead of replacing its
Value or Resource validation. All 13 opcode cases are classified by the
existing analyzer. Per-dimension unknowns and inherited `Violation`,
`RequiresEvidence`, or `Unproven` obligations remain attached to the derived
summary; successful traversal does not promote them.

<a id="workflow-root-export-boundary"></a>

The root summary retains typed root outputs and the same conservative
cross-dimension facts as its control nodes. It is not serialized as a
`CheckedComponentContract`. SemanticIrDocument currently has no root exported
ComponentContract pin or frozen cross-coordinate mapping; therefore root
export compatibility is explicitly `NotExpressedInSemanticIr`/`Unproven`.
No body-conformance assertion, compatibility result, contract digest, or
workflow digest is invented. Full structural closure means every node and
known source was accounted for; it does not mean every dimension is Proven or
the workflow is authorized or executable.

<a id="root-recovery-source-closure"></a>

## 4.1 Root-relative Recovery proof

The workflow-root Recovery analysis MUST begin with each checked leaf's exact
`RecoveryActionBinding`, matching `RecoveryUsageBinding`, Forward occurrence,
action Effect occurrence, source component pin, source computation node, and
full action identity. It MUST retain both `Reconciliation` and `Compensation`
actions. A source without a `COMPENSATION_SCOPE` ancestor is owned by the
typed `RootRecovery` coordinate. The nearest enclosing `COMPENSATION_SCOPE`
owns a local projection; an outer scope may expose an inner contribution but
MUST NOT register it as a second locally owned action.

The source-closure proof compares multisets, not only sets of computation
nodes. It separately reports source-node, action-binding, complete
Recovery-Usage-profile, and multiplicity-factor accounting. An action profile
includes the exact phase/meter/unit/bound tuple; two sources with the same
action atom remain distinct until after closure and aggregation. Missing
source-level facts remain in a partial/unknown projection and MUST NOT be
replaced with an empty K set or finite zero.

<a id="root-recovery-multiplicity"></a>

`LocalActionBound` is the potential upper for one activation of one exact
`COMPENSATION_SCOPE`. It includes only unique `ITERATE`/`RETRY` factors inside
that scope's child. `RootActionBound` is the potential upper over the complete
checked root control tree. These are distinct typed coordinates and MUST NOT
be implicitly converted. Each applied factor identity is the tuple
`(caller SemanticIrDigest, control ComputationId, control kind)` and retains
logical repetitions and, for `RETRY`, physical attempts separately. Logical
repetition upper is applied exactly once; physical attempts are not a second
multiplier. Duplicate or inconsistent factor identities fail closed.

<a id="root-recovery-operator-algebra"></a>

The root fold starts with factor-free leaf seeds and applies each checked
control node once. It MUST use the existing operator algebra:
The fold preserves component-validator-created K/epsilon guard verdicts and
records typed control-path coordinates separately; it MUST NOT synthesize a
conjunction or another unsupported EffectGuard to encode branch reachability.

| Operator | Root Recovery aggregation |
| --- | --- |
| `SEQUENCE` | Checked accumulation of all possible child contributions |
| `SELECT` | Exclusive join/max; an exact selected child is used only with checked selector evidence |
| `PARALLEL` | Accumulate all possible branches |
| `RACE` | Retain winner and loser possibilities; no static winner is chosen |
| `ITERATE` | Apply the total logical iteration upper once; `max_inflight` is not total work |
| `RETRY` | Apply maximum logical attempts once; keep physical attempts descriptive only |
| `TIMEOUT` | Preserve child Effect/K possibility |
| `HANDLE` | Protected contribution plus a conditional join of handlers |
| `SCOPE` | Preserve exact child source and mapping |
| `COMPENSATION_SCOPE` | Assign nearest local owner; add no occurrence multiplier |

Checked arithmetic overflow is a `Violation`; absent finite-factor or source
evidence remains `Unproven`/unbounded. Idempotency, reconciliation labels,
cancellation, timeout, and race-winner assumptions MUST NOT reduce a static
upper bound.

<a id="root-recovery-epsilon-coverage"></a>

The root K action projection is a proof input, not a new epsilon row. The
analyzer MUST compare the aggregate K action potential against the already
derived root epsilon envelope using full atom and qualifier identity, source
mapping, guard, upper, and closed/open state. When Forward and K action use the
same exact atom, the comparison requirement contains the Forward opportunity
once plus the K opportunity; it MUST NOT compare K alone or add K a second time
to the declared epsilon envelope. A known insufficient closed ceiling is a
`Violation`; an unknown/open residual or unresolved resource/guard mapping is
`Unproven`.

<a id="root-recovery-usage-coverage"></a>

Root K-required Recovery Usage is derived from the exact per-attempt profile
and root action upper, then composed once with the existing checked Usage
algebra. The result MUST be compared at the exact `(Recovery phase, meter,
unit)` coordinate against the already derived root `U.recovery`. It is a
coverage projection, not an extra charge: the analyzer MUST NOT add it to root
U or mix it with Forward Usage. Missing active-source meter coverage is not
zero; unknown units are not compatible; checked overflow is a `Violation`.
A fully closed leaf with no K action is an empty contribution and need not
invent zero entries for every meter. This exception applies only when the
exact source/action multiset proves the leaf has no K action; it never fills a
missing coordinate for an active action profile.

<a id="root-recovery-proof-boundary"></a>

Guard compatibility, control-path evidence, root epsilon/usage coverage, and
action eligibility are separate proof obligations. A static guard proof does
not prove that a receipt is `APPLIED`, that reconciliation succeeded, that an
action is eligible now, or that runtime dispatch is authorized. Positive
action potential therefore keeps its `RequiresEvidence` obligation. The root
Recovery summary is not a RootExportAssertion checker; export maps and
export-specific F/I/O/R/epsilon/K/U/X comparisons remain outside this phase.
It creates no plan, journal, reducer, scheduler, worker, grant, reservation,
receipt, command, or execution result.

## 5. Status vocabulary

* **Static Proven Facts:** checked ownership/tree order; Phase 3C resource
  visibility; exact known Effect/Usage algebra; exact source and K bindings;
  successful leaf/source coverage accounting.
* **Conservative Upper Bounds:** child Effects and Usage; all possible
  conditional compensation Effects and Recovery Usage, with checked
  occurrence-family, activation, and attempt multiplication.
* **RequiresEvidence:** child settlement; nonzero compensation eligibility;
  multi-event actual compensation ordering; unresolved runtime outcomes.
* **Unproven:** authority/budget attenuation without explicit ceilings;
  missing/unknown sources; root export compatibility without an export pin and
  mapping.
* **Runtime-Only Obligations:** grant evaluation, budget reservation, resource
  acquisition, receipts and reconciliation, actual applied/partial/unknown
  classification, cancellation/settlement, compensation dispatch, actual
  commit order, and actual recovery outcome.

## 6. Stable rule identifiers

- `AF-SEMANTIC-0147` — [scope-resource-attenuation](#scope-resource-attenuation).
- `AF-SEMANTIC-0148` — [scope-static-obligations](#scope-static-obligations).
- `AF-SEMANTIC-0149` — [compensation-forward-history](#compensation-forward-history).
- `AF-SEMANTIC-0150` — [compensation-exact-action-envelope](#compensation-exact-action-envelope).
- `AF-SEMANTIC-0151` — [compensation-receipt-usage-and-order](#compensation-receipt-usage-and-order).
- `AF-SEMANTIC-0152` — [workflow-control-tree-closure](#workflow-control-tree-closure).
- `AF-SEMANTIC-0153` — [workflow-root-export-boundary](#workflow-root-export-boundary).
- `AF-SEMANTIC-0154` — [root-recovery-source-closure](#root-recovery-source-closure).
- `AF-SEMANTIC-0155` — [root-recovery-multiplicity](#root-recovery-multiplicity).
- `AF-SEMANTIC-0156` — [root-recovery-operator-algebra](#root-recovery-operator-algebra).
- `AF-SEMANTIC-0157` — [root-recovery-epsilon-coverage](#root-recovery-epsilon-coverage).
- `AF-SEMANTIC-0158` — [root-recovery-usage-coverage](#root-recovery-usage-coverage).

Rules 0147–0153 are covered by the independent production differential suite
`SUITE-COMPUTATION-SCOPE-CLOSURE-PYTHON-ORACLE`. Rules 0154–0158 are covered by
`SUITE-COMPUTATION-ROOT-RECOVERY-PYTHON-ORACLE`; exact CaseIds are in the rule
registry.

This phase adds derived-only Rust types and tests. It changes no AFIDL module,
ABI row, existing opcode, ACE1 encoding, HashV1 domain, digest semantics, or
runtime subsystem.
