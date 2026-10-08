# Computation Leaf Contract Resolution

Specification ID: `agentflow.semantic.leaf-contract-resolution.semantics@1`

This specification defines derived, in-memory static analysis for the three
Semantic IR leaves `INVOKE`, `AWAIT`, and `CALL_SUBFLOW`. It adds no wire
object, AFIDL module, ABI assignment, digest domain, control-tree algebra, or
runtime authority.

## Exact resolution and proof context

<a id="leaf-resolution-exact-pin"></a>

An analyzer MUST resolve a component target only by the complete
`ContractRefEntry.digest` in an explicit resolution snapshot. It MUST NOT use
registry-latest, a name-only fallback, network access, wall-clock state, user
identity, or a mutable deployment pointer. The ComponentIdentity comes from
the resolved checked contract, never from the Semantic IR node.

<a id="leaf-resolution-checked-contract"></a>

`INVOKE` MUST resolve to a `CheckedComponentContract` whose digest is
recomputed from that checked canonical contract under the exact proof-context
object held by the snapshot. A missing target is `Unproven`; a mismatched
target pin fails closed. No unresolved `INVOKE` may be treated as pure.

<a id="leaf-resolution-context"></a>

A checked proof is valid only under the exact immutable proof context that
issued it. A context change requires revalidation; equivalent catalog pins do
not make a checked value portable to another context object.

<a id="leaf-resolution-digest-stability"></a>

`ComponentContractDigest` identifies the canonical semantic contract body,
not its proof context. Identical canonical bodies therefore retain the same
digest across proof contexts, while their checked proof results remain
context-bound.

## INVOKE ports and resources

<a id="leaf-resolution-public-port-types"></a>

Component input and output ports MUST map by exact port number and exact
public `TypeSchemaRef` (specification identity, type name, and digest).
Document-local `SemanticTypeId`, display names, primitive coincidence, and
structural similarity MUST NOT substitute for a public pin. If no exact
public mapping is available, the result is `Unproven`. Input scope and
dominance and output producer uniqueness MUST reuse Phase 3C analysis.

<a id="leaf-resolution-resource-ports"></a>

Each Component resource requirement port MUST map explicitly to a visible
Semantic IR `ResourceSlotId`; resource-port numbers and resource-slot ids are
distinct namespaces. Resource types MUST match by exact `TypeSchemaRef`.
Missing caller-side attribute evidence is `Unproven`, not an implicit
ambient-resource match. Multiple requirement ports mapped to one slot MUST
be recorded as an alias group and MUST NOT narrow Effect or Usage bounds.
Static mapping MUST NOT create a resource handle or grant.

## Scoped summaries

<a id="leaf-resolution-effect-occurrences"></a>

Each declared effect occurrence MUST retain its complete source contract,
EffectAtom (including resource selector and every qualifier), bounds and
guard, caller SemanticIrDigest, and computation node. A resource-port
translation is an additional mapping, not a rewrite of the declared atom.
Two call sites remain distinct occurrences even when their component digest
is equal. Aliases MUST NOT merge or reduce occurrences.

<a id="leaf-resolution-information-flow"></a>

Information-flow source and sink coordinates MUST map through exact input,
output, resource, and effect bindings. Unknown external origin, unknown
influence, taints, confidentiality restrictions, flow constraints,
effect-flow partitions, and may-carry instruction authority MUST be
preserved. Unknown or unmapped influence MUST NOT be dropped or upgraded to
effective instruction authority.

<a id="leaf-resolution-dimensions"></a>

The checked Failure, Effect, Resource, Information Flow, Usage, Recovery, and
Execution dimensions MUST be carried as declared per-invocation summaries.
Missing evidence MUST NOT default a dimension to an empty row. Empty Failure
does not guarantee success; missing Usage does not mean zero; KEYED recovery
does not prove provider idempotency; and compatible Execution requirements
do not admit a worker. This phase MUST NOT scale Usage by retries/iterations,
aggregate Execution requirements to workflow placement, or close a control
tree contract.

## CALL_SUBFLOW boundary

<a id="leaf-resolution-subflow-target"></a>

`CALL_SUBFLOW` MUST resolve its exact `semantic-ir` digest in the Phase 3D
`SubflowIrUniverse` and MUST reuse Phase 3D's interface checker for the
call-site port sets and types. The derived interface remains distinct from a
ComponentContract and is not itself a full contract.

<a id="leaf-resolution-subflow-export"></a>

A subflow export association MUST provide a complete checked
ComponentContract. Its exact input, output, and resource port sets and exact
public type pins MUST match the derived target interface. Derived I/O/R MUST
NOT be used to synthesize missing Failure, Effect, Flow, Usage, Recovery, or
Execution dimensions, nor may missing dimensions be filled with empty rows.

<a id="leaf-resolution-subflow-body-proof"></a>

Resolving an exported declaration and validating its call boundary MUST NOT
be reported as proof that the target body conforms to that declaration. If
no trusted, complete body-conformance proof is supplied, the body relation
MUST remain `RequiresEvidence`.

<a id="leaf-resolution-subflow-closure"></a>

The explicit exact-digest target dependency graph MUST fail closed on cycles.
A dependency absent from the supplied snapshot is `Unproven`. This rule
does not authorize recursive expansion, runtime child creation, or workflow
contract aggregation.

## AWAIT and phase boundary

<a id="leaf-resolution-await"></a>

`AWAIT` MUST preserve its exact signal schema pin and Phase 3C-validated
correlation expression. Without an explicit full interaction-adapter
contract, its result is partial and `Unproven`; external origin and unknown
influence remain explicit, and instruction authority is not inferred.
`AWAIT` has no implicit timeout, is not an Effect dispatch, and cancellation
or timeout is not a business Failure.

<a id="leaf-resolution-static-only"></a>

Leaf analysis is pure, deterministic, and operates only on explicit inputs.
It MUST NOT produce grants, handles, credentials, worker/child-run/attempt
admission, leases, reservations, receipts, recovery execution, plans,
journals, reducers, or runtime state. It resolves only one leaf; all
control-tree closure operators remain outside this phase.
