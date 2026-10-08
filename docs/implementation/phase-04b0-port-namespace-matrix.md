# Phase 4B.0 — Port Namespace Matrix

All current coordinates use positive integer encodings in their respective
models, but the integer is meaningful only inside its declared namespace.
The same integer never establishes identity across rows or directions.

| Coordinate | Owner / direction | Current representation | Relation to other coordinates | Required check / disposition |
| --- | --- | --- | --- | --- |
| Component input port | One ComponentContract, I namespace | **Not defined**; `InvokeBody.inputs[].port_no` reserves the call-site slot | Never an output or R port | Future `ComponentInputPort`; 1-based, unique, closed; `InputBinding.port_no` resolves here |
| Component output port | One ComponentContract, O namespace | **Not defined**; `ComputationOutputBinding.port_no` is the caller's declared computation output | Never an input, branch ordinal or R port | Future `ComponentOutputPort`; 1-based, unique, closed; INVOKE output mapping resolves here |
| Component resource requirement port | One ComponentContract, R namespace | `ResourceRequirement.port_no` | May be referenced by ε selector and F resource source; not a ResourceSlotId or resource instance | Existence check plus explicit type/operation compatibility proof |
| Subflow derived input port | One exact pinned SemanticIrDocument, derived I namespace | `SubflowInterface.inputs[].port_no` | Derived by RunParameterId ascending; not a globally stable ID | `CALL_SUBFLOW.inputs[].port_no` maps exactly; no local RunParameterId leakage |
| Subflow derived output port | One exact pinned SemanticIrDocument, derived O namespace | `SubflowInterface.outputs[].port_no` | Derived by NFC output name bytes; not WorkflowOutput name itself | Caller computation outputs map by port; no implicit renumbering |
| Subflow derived resource port | One exact pinned SemanticIrDocument, derived R namespace | `SubflowInterface.resources[].port_no` | Derived by ResourceSlotId ascending; `source_resource_slot` is diagnostic only | `CALL_SUBFLOW.resources[].port_no` maps to a caller ResourceSlot; exact TypeSchemaRef check |
| `InvokeBody.inputs[].port_no` | Call-site I binding namespace | `InputBinding.port_no` | Intentionally names the target Component input when contract is resolved | Exact declared port set and structural type check are not implemented for Component |
| `ComputationDef.outputs[].port_no` | Computation-local O interface namespace | `ComputationOutputBinding.port_no` | Parent controls use the child computation's O namespace | `CheckedIrDocument::interface` derives port→SemanticTypeId; Component target check deferred |
| `ResourceUseBinding.port_no` | Call-site R binding namespace | `ResourceUseBinding.port_no` | Names target Component/Subflow R port, not local `resource_slot` | Exact contract R port set and type mapping; CALL_SUBFLOW is implemented, Component is deferred |
| `EffectAtom.resource_selector.requirement_port_no` | ε selector's R reference | `ResourceSelector.requirement_port_no` | Intended to reference the same contract's R namespace | Positive number validated; existence in R row is explicitly deferred. Must not infer compatibility from number alone |
| `FlowSource.workflow_input(port)` | F source reference to I | `SourceCategory::WorkflowInput(u32)` | Should resolve against Component I; current F canonicalizer checks positive syntax only | Exact I existence check required at contract assembly |
| `FlowSource.resource_requirement(port)` | F source reference to R | `SourceCategory::ResourceRequirement(u32)` | Should resolve against R; not ResourceSlotId | Exact R existence check required at assembly |
| `FlowSink.workflow_output(port)` | F sink reference to O | `FlowSink::WorkflowOutput(u32)` | Should resolve against Component O; not output name or computation-local intermediate | Exact O existence check required at assembly |
| `FlowSink.external_effect(operation)` | F sink reference to ε | `FlowSink::ExternalEffect(EffectOperationRef)` | Frozen coarse sink remains operation-level; B0B adds a closed exact EffectAtom partition refinement | Coarse sink means all matching atoms; selector-specific narrowing requires the new partition or is Unproven |
| Recovery clause `effect` | K reference to ε | `EffectRecoveryClause.effect: EffectAtom` | Exact operation + selector + complete typed qualifiers | Phase 4B.0A closes the qualifier-member collision; compare by full AtomKey |
| Recovery reconciliation action | K action reference to ε | `ReconciliationRequirement::{None,Required}` | Old K flag is preserved; B0B adds exact `(forward Atom, kind, action Atom)` binding | B0B design accepted; future checker must prove the full action EffectAtom is in closed ε |
| Recovery compensation action | K action reference to ε | `CompensationRequirement::Required(EffectOperationRef)` | Old operation pin is preserved and must equal the new full action atom's operation | B0B design accepted; selector and qualifier must match an exact ε entry |
| `EffectGuard.input_equals(port)` | ε guard reference to I | `EffectGuardInputEquals.port_no` | Intended component input port | Positive syntax exists; exact I existence and type/constant check must be done in contract pass |
| Parallel branch ordinal | Control namespace | `BranchOrdinal` | Not a port; distinct coordinate even if same `u32` | Current IR validates exact branch identities and mappings |
| Semantic local IDs | Source document-local namespaces | `RunParameterId`, `ResourceSlotId`, `SemanticTypeId`, `ValueSlotId`, `ExpressionId` | Not external Component port IDs | Translate only through a derived interface/map; never copy numeric values as public identity |

## Frozen namespace rules

1. I, O and R are independent namespaces. The same `port_no = 1` in I and R
   names different coordinates.
2. Component I/O ports and Resource Requirement ports are contract-local.
   Two components' port 1 values are not the same external resource or value.
3. The three Subflow derived namespaces are also independent. Their ordinal
   mapping is deterministic for one exact target digest but derived, not a
   global ABI identity.
4. `RunParameterId`, `WorkflowOutputDecl.name` and `ResourceSlotId` remain
   source-local keys. They must never be exposed as Component port identity.
5. Explicit mapping records are required whenever a caller-local coordinate
   is connected to a target contract coordinate. The mapping, not numeric
   coincidence, is authoritative.

## Proposed Component port identity

For the first contract schema, define the public port identity as
`(ComponentIdentity, direction, port_no)`. Input, output and resource
requirement numbers are each positive, unique and never rebound/reused within
the component's published identity line. Renumbering or reusing a number is a
contract change requiring a new digest and an explicit compatibility result.
Human-readable diagnostic names are carried by a separate closed publisher
documentation projection and are excluded from semantic canonical bytes.

If later requirements need ports to survive renumbering, add a distinct stable
port key and an explicit old→new mapping in a reviewed ABI revision. Do not
silently make names the stable key: names are diagnostic and may be edited.

## Phase 4B.0B decisions

The proposed I/O fields are closed lists of `{port_no, TypeSchemaRef}`. Every
port is required; `option<T>` only makes its value nullable. Public comparison
uses the full TypeSchemaRef pin exactly, never a local SemanticTypeId or an
alias-expanded target. A changed port set/pin is Unproven for general
substitution; a known missing required input/output is a Violation. There is
no default, implicit rename, or output-width-subtyping rule in v1.

The Phase 4B.0B cross-dimension fields use the same namespace ownership:

| Binding | Exact identity | Required lookup |
| --- | --- | --- |
| EffectFlowPartition | `(EffectOperationRef, FlowKind)` then a complete set of full EffectAtom keys | Each allowed atom resolves in ε; the partition refines an existing operation-level F sink |
| RecoveryActionIdentity | `(forward EffectAtom, action kind, action EffectAtom)` | Both atoms resolve exactly in closed ε; K prescribes exactly the matching action kind |
| RecoveryUsageBinding | Exact RecoveryActionIdentity | Must have one complete meter-coverage rule and explicit recovery-phase U limits |
| EffectResource rule | `(EffectOperationRef, ResourceTypeRef)` | Exact R port maps to ResourceTypeRef; relation checks selector vocabulary separately |
| F/X relation rule | Exact label/taint/authority and exact output/effect sink | Resolve against explicit versioned relation snapshot and exact X Region/Trust identities |

None of these relations use the integer port value outside its owner
namespace. A ComponentIdentity change creates new port identities; equal
numbers and names do not map old to new ports.

## Which mappings are semantic ABI

The Component I/O and R port declarations themselves belong in the future
ComponentContract canonical body. The references already present inside ε and
F (selector R port, workflow-input I port, resource-source R port,
workflow-output O port) remain their authoritative coordinates; the
ComponentContract checker resolves them against the three port tables and
does not duplicate the mapping into a second table.

Phase 3D's RunParameterId→Subflow-I, workflow-output-name→Subflow-O and
ResourceSlotId→Subflow-R projections are Compiler-derived analysis. They stay
out of ComponentContract bytes and SemanticIrDigest. `InputBinding`,
`ComputationOutputBinding` and `ResourceUseBinding` are already Semantic IR
ABI fields and keep their declared call-site numbers. An old→new port mapping
needed to prove a cross-version rename/re-number is a compatibility proof
input or attestation evidence, not part of either contract's semantic body.

## Required assembly mapping

```text
Semantic IR RunParameterId ──derived order──> Subflow I port
Semantic IR WorkflowOutput ──NFC name order──> Subflow O port
Semantic IR ResourceSlotId ──ascending order─> Subflow R port

Invoke InputBinding.port_no ─────────────────> Component I port
ComputationOutputBinding.port_no ────────────> Component O port
ResourceUseBinding.port_no ─────────────────> Component R port

EffectAtom.selector.requirement_port_no ────> Component R port
F workflow_input source ────────────────────> Component I port
F resource_requirement source ──────────────> Component R port
F workflow_output sink ─────────────────────> Component O port
```

Every arrow requires lookup in the correct namespace and a typed mapping.
Missing target, wrong namespace or duplicate/rebound coordinate is a
Violation. Missing mapping evidence is Unproven. Neither state can be repaired
by equal integers or matching diagnostic names.

## Related audit documents

[Audit report](phase-04b0-preassembly-audit.md) ·
[Field matrix](phase-04b0-field-matrix.md) ·
[Compatibility](phase-04b0-compatibility-matrix.md) ·
[Cross-dimension obligations](phase-04b0-cross-dimension-obligations.md) ·
[Counterexamples](phase-04b0-counterexamples.md) ·
[ABI proposal](phase-04b0-abi-proposal.md) ·
[Decisions](phase-04b0-decisions.md) ·
[Type/port policy](phase-04b0b-type-and-port-policy.md) ·
[Readiness report](phase-04b0b-readiness-report.md)
