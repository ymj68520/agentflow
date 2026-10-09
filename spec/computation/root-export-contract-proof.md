# Root Export Contract Proof

Specification ID: `agentflow.computation.root-export-contract-proof.semantics@1`

This specification defines a pure, in-process proof that one exact checked
Semantic IR workflow root fits one exact checked exported ComponentContract.
The assertion is a compiler-boundary input, not a wire object, contract,
implementation attestation, publisher signature, grant, budget, or runtime
authorization. It introduces no digest domain, AFIDL type, ABI assignment,
Semantic IR field, or ComponentContract field.

## Exact assertion pins

<a id="exact-assertion-pins"></a>

The checker MUST recompute the SemanticIrDigest and ComponentContractDigest
from the exact checked inputs and compare every algorithm, domain, and digest
byte with the assertion. The derived root MUST name that same
SemanticIrDigest. A mismatch is a Violation and no dimension may be reused.
The assertion identity is the complete typed input tuple; it MUST NOT be a
Debug string, standalone digest, publisher signature, or wire identity.

## Proof-context manifest

<a id="proof-context-manifest"></a>

The checked export, immutable resolution snapshot, derived root and root
recovery summary MUST use the same pointer-identical StaticContractProofContext
object. The assertion manifest MUST also exactly match a complete typed
snapshot of compiled modules, resolved type schemas, complete effect catalogs,
recognized resource types and attributes, flow labels/taints/sinks, usage meter
descriptors, recovery operations, execution universes, all three relation
snapshot pins and canonical content, and resolver component/subflow pins.
Pointer identity is process-local and MUST NOT be presented as a durable
identity. A changed or incomplete content manifest is Unproven and requires a
fresh derivation.

## Compiler semantics identity

<a id="compiler-semantics-identity"></a>

CompilerSemanticsIdentity MUST include the checker analysis-semantics version,
the exact applicable SpecificationIds and complete normative source content,
the registry's exact normative revision/content, and the exact applicable
RuleId set and statements. Matching RuleIds alone are insufficient. Content
checksums are ordinary content checks only, not HashV1 domains or signatures.
A stale, incomplete, changed, or unverifiable identity is Unproven.

## Root input mapping

<a id="root-input-mapping"></a>

Each exported input port MUST map exactly once to one root RunParameterId and
its exact ValueSlotId. Positive v1 proof requires a bijection between the
exported port set and all required root run parameters, exact nominal
TypeSchemaRef pins, and exact producer, scope, and Phase 3C use-site evidence.
Equal port numbers or names are not mappings. Reusing one root input for two
independent exported inputs is a Violation. A complete known missing or extra
required input is a Violation; an incomplete ambiguous map is Unproven.

## Root output mapping

<a id="root-output-mapping"></a>

Each exported output port MUST map to one exact named WorkflowOutput and
ExpressionId with checked availability, producer/dominance, scope and exact
nominal TypeSchemaRef evidence on every possible control path. Names and port
numbers do not imply one another. A known missing output or incompatible type
is a Violation. Missing producer/type evidence is Unproven. An additional
unmapped root WorkflowOutput is Unproven because no additive-interface rule is
defined.

## Root resource mapping

<a id="root-resource-mapping"></a>

Every public resource coordinate used by the proof MUST preserve the explicit
chain from source Component resource port through the exact root ResourceSlotId
to one exported ResourceRequirement port. TypeSchemaRef, ResourceTypeRef,
attributes, selector restrictions, lexical visibility, aliases and effect
resource selectors MUST remain exact. A root slot reused for two distinct
export ports is Unproven without a public alias rule. Resource compatibility
proves neither a ResourceHandle nor authority or an EffectGrant.

## Failure containment

<a id="failure-containment"></a>

The exact nominal set of possible root business failures MUST be contained in
the complete exported Phi row. Unknown or incomplete root evidence is
Unproven. Timeout, cancellation, recovery blocking and race winner choice MUST
NOT be inserted into Phi. A known root failure missing from a complete export
row is a Violation.

## Effect containment

<a id="effect-containment"></a>

After complete typed resource and guard-coordinate mapping, every possible
root EffectAtom, full qualifier, scope, guard and occurrence bound MUST fit the
exported epsilon ceiling. Open/unknown rows, residuals without exact mapping,
and unsupported relations are Unproven. A known effect outside a complete
closed export ceiling is a Violation. Root K action possibility MUST fit both
root epsilon and exported epsilon. Root K action and recovery-usage projections
are coverage checks and MUST NOT be added to the already-derived root epsilon
or U envelopes a second time. A failed containment call alone is not a
counterexample; Violation requires a typed contradiction.

## Information-flow non-relaxation

<a id="information-flow-non-relaxation"></a>

Root Flow MUST preserve exact source, sink, FlowKind, confidentiality label,
taint, decision influence and EffectFlowPartition coordinates while proving
that it does not relax exported restrictions. Unknown external influence,
unknown AWAIT provenance, unresolved subflow body influence, and open
partitions MUST remain explicit and prevent Proven when they affect the
comparison. Recovery actions have no action-specific Flow field; no such
relation may be invented.

## Usage containment

<a id="usage-containment"></a>

Root Forward and Recovery Usage MUST each fit the export in exact phase, meter,
unit and bound coordinates. A missing meter or unit is not zero. Root
K-required Recovery Usage MUST separately fit root U.recovery and exported
U.recovery. It is not a second charge. A known exceeded finite ceiling or
checked arithmetic overflow is a Violation; unknown/open bounds are Unproven.
Static Usage MUST NOT create a budget reservation, receipt, ledger balance, or
settlement claim.

## Recovery compatibility

<a id="recovery-compatibility"></a>

Root K proof MUST retain the complete source contract digest, computation,
forward ordinal and full atom, RecoveryActionKind, full action atom, matching
RecoveryActionBinding and RecoveryUsageBinding, meter profile, root factors,
local/root coordinates, resource mapping and guard evidence. Root source
closure MUST compare full multisets, never only node IDs. The checker MUST use
the existing Recovery relation and MUST NOT invent an enum ordering. Where a
changed declaration's safety direction is not frozen, the result is
Unproven. Receipt, reconciliation result, provider honoring, action eligibility
and commit order remain runtime evidence.

## Execution compatibility

<a id="execution-compatibility"></a>

Every per-child ExecutionRequirements source MUST be mapped exactly and
compared using the explicit execution universes. Feasibility and security
preservation MUST be reported separately. Profiles MUST NOT be summed across
parallel children or interpreted as Worker placement/admission. Unknown
execution universes or action-specific X relations are Unproven.

## Subflow body separation

<a id="subflow-body-separation"></a>

Each CALL_SUBFLOW dependency MUST pin its exact call-site, target
SemanticIrDigest, target exported ComponentContractDigest when resolved, and
complete transitive target/export closure. Missing, stale, cyclic or
incomplete dependencies MUST NOT be silently dropped. A target static export
proof is not trusted implementation/body evidence. Without a separately
trusted body proof, body conformance is RequiresEvidence even when all nine
static dimensions are Proven.

## Static/runtime boundary

<a id="static-runtime-boundary"></a>

The strongest positive claim is that the exact checked root's static
declarations fit the exact checked export under the exact pinned compiler
semantics and proof context. It MUST NOT claim component implementation
conformance, publisher authentication, Worker admission, effects authorized,
runtime recovery safety, or runtime budget. Scope authority attenuation and
budget attenuation are Unproven because frozen ScopeBody declares neither
ceiling. No checker result creates a grant, handle, reservation, receipt,
dispatch, plan, journal, reducer, scheduler, or runtime state.
