# Component Contract — v1

> Status: normative static component ABI. The contract is a semantic promise,
> not an implementation, runtime identity, capability grant, or authorization.

<a id="component-contract-closed-abi"></a>

## 1. Closed semantic body

The wire schema is `agentflow.component.contract@1` in
`spec/idl/component-contract.afidl`. Its `ComponentContract` record has
exactly thirteen fields, numbered 1 through 13: `component_identity`,
`inputs`, `outputs`, `failures`, `effects`, `resources`, `information_flow`,
`usage`, `recovery`, `execution`, `effect_flow_partitions`,
`recovery_action_bindings`, and `recovery_usage_bindings`. Its imports are
exact pins of the existing semantic-core, failure, effect, resource, flow,
usage, recovery, and execution schemas. No open extension or arbitrary
metadata field exists.

<a id="component-contract-identity-digest"></a>

`ComponentIdentity` is the exact pair `(catalog SpecificationId, name)` and
is part of the semantic body. The name is a non-empty NFC lowercase dotted key
whose segments contain only `[a-z0-9-]`. `ContractVersion` is a separate
publication coordinate and is not part of this record. Implementation/package
identity, artifact digest, price snapshot, diagnostics, evidence, and proof
reports are also outside it. Different identities therefore produce different
semantic content even when all other declarations match.

<a id="component-contract-canonical-digest"></a>

Canonicalization sorts input and output ports by number, flow partitions by
exact operation and kind, allowed effects by the complete Effect Atom key, and
recovery bindings by the complete recovery-action identity. Every nested
dimension uses its frozen canonicalizer. Duplicate set members are rejected;
source order is not semantic. The checked digest is
`HashV1("component-contract", ACE1(CanonicalComponentContract))`, using the
already registered HashV1 domain. Only a `CheckedComponentContract` produced
by the static validator may be passed to the trusted digest API. A raw or
merely canonical but unchecked value has no verified contract digest.

<a id="component-contract-ports"></a>

Input, output, and resource-requirement ports are independent namespaces.
Component I/O ports are 1-based, unique within their direction, required, and
typed by exact nominal `TypeSchemaRef` identity including schema digest.
`option<T>` is a value type, not an omitted port. Diagnostic names are not
part of this ABI. Internal structural alias normalization does not widen the
public ABI relation.

<a id="component-contract-effect-resource"></a>

Each declared effect selector points to an existing resource-requirement
port. Its exact operation and exact pinned resource type must be supported by
the explicit `EffectResourceCompatibilityCatalog`; selector keys map only to
the catalog's named resource attributes and scope values are exact NFC
literals. Qualifier profiles compare the complete typed qualifier list,
including family and member. A complete operation set distinguishes explicit
denial from missing evidence. Missing relation evidence and open/unknown
effects are `Unproven`, never allowed by default. This relation grants no
resource handle or effect authority.

<a id="component-contract-effect-selector-mapping"></a>

Every selector key maps through one exact relation row to a named resource
attribute; port-number equality alone is not a mapping.

<a id="component-contract-effect-resource-closure"></a>

An operation outside `complete_operations` has missing evidence; a covered
operation with no exact supported pair is explicitly denied.

<a id="component-contract-flow-partitions"></a>

An `EffectFlowPartition` refines an existing operation-level flow ceiling for
one exact `FlowKind`. Each allowed member is a complete `EffectAtom` identity
(operation, selector, all qualifiers), must occur in the declared closed
effect row, and must be within the corresponding legacy flow ceiling. A
partition cannot widen that ceiling. Missing exact coverage where a
selector-level proof is required, or an open/unknown effect residual, is
`Unproven`. Without a partition, the legacy operation-level ceiling applies
to every exact declared effect atom for that operation; unknown residuals are
not covered by that finite set.

<a id="component-contract-flow-ports"></a>

Workflow-input and resource-requirement flow sources resolve only in their
respective exact namespaces; workflow-output sinks resolve only against the
output namespace. Port numbers are not remapped. An unknown external source
or unknown influence cannot establish a positive flow proof. A may-carry
instruction-authority summary is not effective instruction authority.

<a id="component-contract-flow-unknown"></a>

Unknown external provenance and unknown influence require evidence or remain
unproven; a component author cannot turn them into a trusted positive result.

<a id="component-contract-recovery-bindings"></a>

Recovery action bindings use the complete forward-effect identity, an action
kind (`reconciliation` or `compensation`), and the complete declared action
effect identity. Each required action has exactly one binding, with no
conflicting extras. The action effect must occur in a closed declared effect
row. A compensation action's operation is the exact operation required by
the Recovery clause. The dependency relation among bound actions is acyclic.
An action declaration is not recovery evidence, and compensation does not
erase forward history.

<a id="component-contract-recovery-action-coverage"></a>

An action binding without the corresponding required recovery clause is an
extra action and is rejected; required reconciliation and compensation
actions are matched separately by full identity.

<a id="component-contract-recovery-usage"></a>

Each action has exactly one usage binding and an exhaustive rule in the
explicit `RecoveryUsageCoverageCatalog`. The rule identifies the complete
meter set; every meter coordinate must be present in the per-attempt recovery
profile, including an explicit finite zero when zero is intended. Missing
coverage is not zero. Activation, attempt, and meter bounds are combined with
checked arithmetic and conservatively accumulated against the recovery part
of `UsageSignature`. Missing, unbounded, or incomplete evidence cannot prove
a finite usage ceiling. No receipt, reservation, ledger, or price is part of
this contract.

<a id="component-contract-recovery-usage-checked"></a>

Forward occurrence, activation, attempt, and per-meter upper bounds use
checked multiplication and addition before comparison with `U.recovery`.

<a id="component-contract-cross-dimension"></a>

The checker reports schema validity and the independent dimensions
I/O, failure, effects, resources, information flow, usage, recovery, execution,
and cross-dimension consistency. Each result is `Proven`, `Violation`,
`Unproven`, or `RequiresEvidence`, with a rule, exact reference, reason, and
prerequisites. The overall result is not `Proven` if any required dimension
is not `Proven`. No `RequiresEvidence` result is promoted automatically.

<a id="component-contract-execution"></a>

Execution capability, trust-domain, region, runtime-profile, and sandbox
references resolve only against explicit execution universes.

<a id="component-contract-proof-composition"></a>

Composition preserves each dimension verdict: `Unproven` and
`RequiresEvidence` are not erased by other successful dimensions.

<a id="component-contract-static-authority-boundary"></a>

Static compatibility and catalog relations do not establish worker
attestation, execution placement, sandbox enforcement, policy authorization,
resource binding, or runtime authority. Whole Semantic IR traversal,
control-tree contract closure, scheduling, execution, and runtime evidence
are outside this specification.

<a id="component-contract-substitution"></a>

Substitution compares semantic declarations, not version numbers or
implementation digests. Version 1 proves public I/O substitution only for
exactly equal port-number sets and exact type pins; unsupported subtype or
alias cases are `Unproven`. Other dimensions use their established directional
compatibility relations and preserve separate proofs for feasibility and
security. Any `Unproven` required dimension prevents an overall positive
substitution result. Static compatibility is not behavioral certification.
