# Phase 4B.2E.2 — Coordinate Mapping and Closure

## Namespaces

Port numbers are never mappings. Input, output, resource, semantic value,
computation, effect, flow, recovery, and execution identities remain distinct
until an explicit typed assertion row joins them. Mapping vectors are
canonicalized without deduplication, and the report preserves the exact rows.

## I — inputs

Every exported ComponentInputPort maps to one RunParameterId and its exact
ValueSlotId. Positive v1 proof is a bijection over the full export/root input
sets, requires the same exact nominal TypeSchemaRef, and retains all checked
leaf use sites. For a use-site proof the expression must directly preserve the
root ValueSlot; transformed expressions remain Unproven. Known root/export
cardinality mismatch is a Violation; an equal-size but incomplete map is
Unproven. Reusing one source for two independently callable public inputs is a
Violation.

## O — outputs

Each export output port maps to a named WorkflowOutputDecl, exact ExpressionId,
exact nominal result pin, and checked Phase 3C dependency/availability facts.
The control summary must have a complete all-path interface. A known missing
required output or incompatible actual result type is a Violation. An absent
or incorrect assertion witness is Unproven. Extra root outputs remain
Unproven—there is no frozen additive-interface rule.

## R — resources

Each public resource port maps to one root ResourceSlotId and exact
TypeSchemaRef/ResourceTypeRef. `RootResourceSourceCoordinate` retains the
source computation/contract, component resource port, slot, and source-local
alias group, preserving the full chain:

~~~text
Component resource port -> Root ResourceSlotId -> exported R port
~~~

Every source requirement and attribute is compared through the existing
resource non-strengthening relation in the explicitly mapped export
namespace. Scope visibility and the complete resource-type attribute
vocabulary are required. Reusing one root slot at multiple call sites for the
same public port is allowed; mapping it to distinct public ports is Unproven
without a public alias-equivalence rule. A resource proof is not a handle or
grant.

## epsilon and Flow coordinates

Every source Effect occurrence retains caller IR digest, computation,
contract digest, entry ordinal, full atom, guard, and mapped slot. The
ResourceSelector and each full qualifier are translated exactly. Guards move
through the source input expression, root parameter/value slot, and export
input port; equal numeric ports in different contracts are unrelated.

Every Flow source and sink retains its source contract, computation and
mapping ordinal, source/sink bindings, mapped root/export coordinates, and
EffectFlowPartition. Unknown external provenance and unsupported coordinate
translations remain Unproven. Recovery has no independent Flow or X wire
field: action-specific proofs are available only from exact source
EffectFlowPartition or execution facts; otherwise F/X remain Unproven.
