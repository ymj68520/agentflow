# Phase 4B.2F — Coordinate Mapping Audit

## I — inputs

Source review confirmed full-set cardinality, exact exported contract digest
namespace, exact RunParameterId/ValueSlotId producer, exact nominal
TypeSchemaRef, unique public ports, and exact use-site lists. A use-site must
be a direct `ExpressionBody::ValueSlot` whose Phase 3C dependency set is
exactly that root slot; a transform cannot be inferred value-preserving.

Production probes: deleted mapping → Unproven; duplicate two-port source →
Violation; extra root RunParameter → Violation; stale IR pin → Violation.
No checked capability was issued for these cases. Unexecuted design vectors
remain for transformed expressions, cross-namespace numeric collisions,
invalid scope, missing producer, and port-to-two-parameters mapping.

## O — outputs

`prove_outputs` requires a unique export-port/WorkflowOutput mapping, exact
WorkflowOutput name and ExpressionId, exact nominal result schema, exact
producer witness, Phase 3C dependencies available at workflow exit, and a
complete root control interface. Root interface construction checks the
frozen branch/interface rules for the structured tree. Missing required
workflow output produced Violation; deleting only the assertion mapping or
changing its producer/type witness produced Unproven. An extra root output is
Unproven because there is no additive-interface rule.

Designed but not executed separately: SELECT/RACE branch omissions, HANDLE
handler mismatch, ITERATE carry producer unknown, out-of-scope producer, and
duplicate output target. No one-path-only positive was observed in the
executed probe set.

## R — resources and aliases

The checker reconstructs source component port → scoped ResourceSlotId → root
coordinate → export requirement port, verifies the source-coordinate/alias
list, exact TypeSchemaRef/ResourceTypeRef and catalog vocabulary, and applies
the existing non-strengthening relation. The ResourceSlot/ResourceType
assertion mutations both failed closed. Non-empty leaf attribute requirements
remain Unproven because the frozen Semantic IR carries no attribute evidence;
this is conservative, not a false proof.

No ResourceHandle, acquisition, grant, or authority is inferred. Designed but
not run as production probes: alias-list deletion, same numeric port in two
source contracts, selector narrowing, lexical resource leak, K action resource
omission, and one root slot mapped to two incompatible exported ports.

## Identity/canonicalization observations

The primary input/output/resource targets and effect/recovery/Flow/execution
source identities are checked for duplicates before dimension evaluation.
Input source reuse is independently rejected in I. The code preserves map rows
when sorting. No production counterexample showed distinct source nodes being
merged by equal ComponentContractDigest. The assertion identity itself is not
a signature.

No standalone RootExport digest exists. The actual stale subflow probe is
classified under proof-context/resolver invalidation rather than as a numeric
coordinate collision: root source coordinates carried the export-A pin while
the asserted direct subflow pin carried export B.
