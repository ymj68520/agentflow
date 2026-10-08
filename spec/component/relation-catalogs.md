# Component Relation Catalogs — v1

> Status: normative static analysis inputs. All three catalogs are explicit,
> closed, versioned snapshots; none grants execution authority.

<a id="component-relation-catalogs-snapshots"></a>

Each relation catalog has an exact schema identity, opaque non-empty NFC
revision token, canonical ACE1 content, and a plain SHA-256 checksum over
those canonical bytes. The checksum is not a HashV1 digest or a signature.
Proof output pins all four values. A proof is valid only for the exact
catalog content used to derive it; changed content requires revalidation.
There is no latest fallback, lookup, discovery, or mutable-name cache key.
Comparisons using different catalog pins are `Unproven` unless an explicit
evolution relation is supplied.

<a id="effect-resource-catalog-closed-coverage"></a>

`agentflow.effect.resource.compatibility@1` records a complete set of
effect operations and exact supported operation/resource-type pairs. A
missing pair for a covered operation is a closed denial; an operation absent
from `complete_operations` is missing evidence. A pair lists exact selector
key to resource-attribute-key mappings, requiredness, exact finite values or
the explicit opaque-NFC-string rule, and complete allowed qualifier
profiles. Scope matching is literal; prefix, wildcard, provider-family, and
port-number coincidences have no meaning.

<a id="recovery-usage-catalog-exhaustive"></a>

`agentflow.recovery.usage.coverage@1` imports the exact component action
identity and maps each action to its exhaustive complete meter set. A
missing action rule is not an empty set. An explicit empty set is meaningful
only when the action truly declares no metered usage. The separate Usage
Meter Universe resolves exact meter identities, units, and supported
additive aggregation.

<a id="flow-execution-catalog-exact-relations"></a>

`agentflow.flow.execution.relation@1` relates exact confidentiality labels
to execution/residency region pairs, exact taints and instruction-authority
levels to trust-domain and sandbox policies, and exact flow-kind/sink tuples
to trust-domain and sandbox policies. Its closed region and trust-domain
snapshots define the only meaning of the corresponding “all in snapshot”
policy cases. Sink identity is a workflow-output port or a full exact effect
atom; operation-only sinks are forbidden. Empty or absent requirements do
not manufacture a positive relation. Exact static consistency is not
attestation, authorization, or enforcement.
