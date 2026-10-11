# Phase 4B.2F — Identity, Context, and Invalidation Audit

## RootExportAssertion identity

`RootExportAssertionIdentity` structurally wraps the full typed assertion
tuple. It includes SemanticIrDigest, top ComponentContractDigest,
CompilerSemanticsIdentity, ProofContextManifest, all typed maps and exact
subflow pins. There is no standalone RootExport digest, signature, or trust
claim. `canonicalized()` sorts map collections without removing mapping rows;
the checker separately rejects repeated targets/sources. Resource alias-port
members are treated as a set (sorted and deduplicated) because the derived
alias group is a set of component resource ports, not a list of occurrences.

The production probes changed IR/export pins, I/O/resource/Flow/Recovery/X
coordinates, compiler version and relation content. Every changed assertion
was rejected (Violation or Unproven), and no checked capability was issued.
The single P0 caused by a valid but incomplete relation across two effect rows
is recorded in the false-Proven report; it is not an identity collision.

Pure map reordering is not silently accepted as an unchecked identity: the
preflight compares the incoming assertion to its canonical form. Callers can
canonicalize first; duplicates remain and are rejected. A structural identity
does not authenticate who supplied it.

## ProofContextManifest

Source review confirms the manifest clones the compiled module set, schema
universe, effect/resource/Flow/Usage/Recovery/Execution universes, every
present relation pin plus canonical bytes, resolver component digest pins,
subflow target pins, direct dependencies and export pins. Preflight compares a
fresh capture with the assertion and independently checks pointer identity for
the checked export, snapshot and root context. The 31 probes exercised stale
assertion relation/compiler facts and a changed subflow export dependency.

The context content snapshot is useful, but it is not sufficient to bind the
supplied root derivation to the *same resolver snapshot*. AUDIT-F02 demonstrates
that same-context snapshots can differ in the subflow export attached to one
target digest. The root summary retains its old target export pin, while the
checker validates the assertion against the new snapshot. No equality check
connects those two values. Therefore the stale-proof invalidation requirement
is not met for resolver changes.

## CompilerSemanticsIdentity

Current identity equality includes the checker version, all 16 embedded
normative source byte arrays and checksums, the full specification registry
bytes, the full rules registry bytes, and RuleContentIdentity rows. A changed
source/registry identity fails closed; it is not a compiler binary attestation
or publisher signature.

The enumerated applied RuleId set is hard-coded as 22
`AF-CONTRACT-0001..0022` plus 62 `AF-SEMANTIC-0111..0172` (84 total). It omits
`AF-SEMANTIC-0001..0110`, including directly relevant type-pin, independent
namespace, SSA producer, WorkflowOutput, ownership, interface, Phase 3C
availability, scope visibility, and subflow interface rules. The full
`core-semantic-types.md` and `semantic-ir.md` bytes and the complete registry
bytes are embedded, so this omission did **not** permit stale content reuse in
the probe. But the claimed “complete applicable RuleId set” is not established
and the applied-rule provenance is incomplete. This is AUDIT-F03 (P2), not a
confirmed false-Proven by itself.

## Invalidation experiment status

| Mutation | Evidence | Result |
| --- | --- | --- |
| IR digest / top export digest | Actual assertion mutation, production RootExport call | Violation; no capability |
| Mapping, source/profile, relation manifest | Actual assertion mutations | Unproven or Violation; no capability |
| Compiler semantics version | Actual assertion mutation | Unproven; no capability |
| Relation content changed while pin retained | Actual manifest mutation | Unproven; no capability |
| Same RuleId, changed registered text | Source inspection: full rules registry bytes are equality-bound; no isolated mutation call was added | Mechanically invalidates the identity; not separately executed |
| Catalog/resource vocabulary/Flow/Usage/Recovery/X members changed in the actual context | A new exact context changes pointer identity; the checker cannot reuse the old checked export/root. No separate all-universe mutation matrix was executed | Fail-closed by pointer precondition; content-change coverage remains incomplete |
| Same target IR, changed target export association | Two real snapshots, same context, root derived under A and checked under B | **P0 false Proven**, AUDIT-F02 |

The equality check is effective only for the assertion/current context pair;
it does not close the missing root-summary/current-resolver link found in
AUDIT-F02.
