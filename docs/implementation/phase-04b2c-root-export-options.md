# Phase 4B.2C — Root Export proof options

Current Semantic IR has no root exported ComponentContract pin. Its
`workflow_outputs` are names plus expressions, root computation outputs use
local output-port/value-slot coordinates, resources use ResourceSlotIds, and
ComponentContract uses separate input/output/resource port namespaces. The
existing ComponentContract substitution checker compares two declared
contracts; it does not provide the missing root-coordinate map or prove a
workflow body conforms to an export.

## Option comparison

| Option | What it would mean | Strength | Cost / risk | Audit decision |
| --- | --- | --- | --- | --- |
| A. Explicit build-time export assertion | Non-wire, typed assertion references exact SemanticIrDigest and exported ComponentContractDigest plus complete I/O/R coordinate maps; checker derives root facts in an exact proof context and compares dimension-wise. | Can prove static envelope compatibility without changing the frozen Semantic IR; missing maps remain Unproven. | Needs a formal mapping schema, direction per dimension, stable proof assumptions, and separate body-conformance evidence. Current pointer-only context identity cannot be persisted. | **Recommend for a separate Phase 4B.2D design**, first as an in-process checker input. Persist only when a versioned release container can pin the assertion/assumptions. |
| B. New versioned Semantic IR export reference | A future IR version embeds an exact export pin and perhaps typed maps. | Canonical document can own the declared export relation. | Requires ABI/schema migration and still needs mapping semantics; violates the current phase's freeze if attempted now. | Do not select unless later work proves external assertion insufficient. No current IR edit. |
| C. Deferred export attestation | Publisher/release process states the body matches an export. | Can cover facts outside static IR, including implementation behavior, if evidence is trustworthy. | By itself it is not a compiler proof; publisher self-assertion cannot silently become Proven. | Useful only as a separately authenticated implementation attestation, not a replacement for A's static comparison. |

## Option A candidate shape (design only)

An eventual assertion should contain, at minimum:

```text
exact SemanticIrDigest
exact exported ComponentContractDigest
complete input mapping: RunParameter/ValueSlot ↔ Component input port
complete output mapping: WorkflowOutput/root output ↔ Component output port
complete resource mapping: ResourceSlotId ↔ ResourceRequirement port
exact nominal schema/type pins and mapping provenance
compiler semantics/version identity
exact static proof-context manifest or references to stable catalog identities
per-dimension result plus unresolved premises
```

This is a proposal, not a Rust type or wire shape added by Phase 4B.2C. Any
mapping must be typed, unique, complete where required, and immutable for the
assertion's lifetime. Reordering or remapping a port must cause re-evaluation.

## Proof directions to freeze before implementing a checker

| Dimension | Candidate static relation | Evidence required |
| --- | --- | --- |
| I | Every exported input has an exact typed root input source; body must not require a stronger unadvertised input precondition. | Explicit port/type map; decide whether equality or a formal variance relation applies. |
| O | Every exported output is produced with exact nominal type and a unique mapping from root workflow/computation output. | WorkflowOutput name/expression plus root value-slot producer chain. |
| R | Mapped root resource requirements must not exceed the exported requirement/capability relation. | ResourceSlot-to-port map, exact ResourceTypeRef and attribute proof. |
| E | Root possible Effect envelope is contained by the export's declared ceiling using full atom/guard identity. | Exact atom/resource mapping and checked occurrence bounds. |
| Phi | Every possible root Failure type is allowed by the export; no new type is hidden. | Exact TypeSchemaRef pins. |
| F | Root flow does not relax exported confidentiality/egress/decision constraints. | Explicit source/sink/EffectFlowPartition coordinate mappings; unknown provenance remains unknown. |
| K | Exported recovery obligations are not weakened; full action identities and dependency/coverage semantics remain exact. | Root K source mapping and root-total action count coordinate, including nested Retry/Iterate. |
| U | Root Forward/Recovery meter bounds do not exceed the export ceiling, with checked arithmetic. | Exact meter identity/phase mapping and no duplicate K aggregation. |
| X | Every child requirement fits the exported execution constraint; do not sum into a Worker. | Exact runtime/sandbox/trust/residency relation; placement/admission still separate. |

No compatibility direction should be inferred from matching field names. In
particular, root ResourceSlotIds are not Component resource ports, root Flow
source coordinates are not automatically exported Flow coordinates, and
per-child X is not a worker profile.

## Proof and trust boundary

An export assertion can prove static envelope containment only. It cannot
prove that component code conforms, that a publisher's binary matches the
contract, that a Grant exists, that a Worker is admitted, or that an Effect
was applied. CALL_SUBFLOW body conformance stays RequiresEvidence unless a
separately trusted evidence object is supplied.

The current proof-context pointer identity is process-local. A persistent
assertion needs exact stable identities for catalogs/relations and the compiler
semantics used; do not invent a new HashV1 domain or reuse a digest with a
different semantic payload. This audit selects Option A as the next design
direction, but does not start that phase or create a checker.
