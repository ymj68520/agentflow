# Phase 4C.0 — PlanCore / LockGraph Dependencies

All prospective objects and verifiers are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C03 fixes the dependency order.

## Current versus missing

The domain table already registers `semantic-ir`, `component-contract`,
`plan-core`, `lock-graph` and `artifact`. Only the first two have relevant
implemented program/contract models here. `StaticComputationResolutionSnapshot`
and its structural identity are in-memory proof inputs, not a LockGraph.
Neither generic hash support nor architecture section 24.2 provides an implemented
PlanCore or LockGraph codec.

## Resolution and LockGraph

LockGraph must be the immutable semantic dependency graph used by rebuilding
and lowering. It fixes source and call-site edge identities, package origin,
exact package/contract/model-facing semantic content, target SemanticIrDigest
and its associated export pin, compatible adapter contracts, resolver semantics
and eligibility inputs. The graph is not a set of contract digest values.

Required mapping example:

~~~text
(caller IR, call_site A, target IR A) -> exported contract E1
(caller IR, call_site B, target IR B) -> exported contract E2
~~~

Swapping E1/E2 changes the graph even when the digest set is unchanged.
Every direct edge is independently derived from exact IR/package content;
transitive closure must include every reachable semantic node. Duplicates that
conflict, unavailable target associations, unknown dependency sets and cycles
block publication. Known empty closure is distinct from absent evidence.

Fresh resolution and locked rebuilding are separate operations. Fresh
resolution uses a pinned registry/eligibility/trust snapshot. Locked rebuilding
retrieves only exact locked content and never selects latest or substitutes
a version. Yank affects fresh selection; revoke affects trust/eligibility and
may block use without altering retained immutable content.

Implementation SBOM dependencies, live Worker capacity and endpoint instances
are outside semantic LockGraph. Semantically relevant adapter contracts,
model-facing prompt/tool metadata and declared selection policies remain
semantic dependencies even if their eventual implementation is deploy-bound.
A semantically relevant binary fixed during compilation must be exactly pinned;
it cannot hide in an untracked implementation dependency.

## Relation to current resolver identity

A future LockGraph projection reconstructs the current exact component pins
and each target/dependency/export association. The portable proof manifest owns
the complete proof input projection; LockGraph owns broader semantic package
closure. The verifier must prove that projection matches, including source
namespaces and optional fields.

Current `ResolverSnapshotIdentity` is never serialized as a stable wire
identity. The future LockGraph schema defines its own canonical graph encoding,
then fresh snapshot construction produces a new process-local identity.
Additional proof-context entries cannot be dropped while reusing an old proof;
if the schema permits a minimal graph/context projection, generate a new
assertion and Root from it.

## RootExport and lowering order

1. Resolve publish-time semantic parameters and normalize final IR.
2. Validate canonical IR/contracts and close dependencies.
3. Derive Root and RootRecovery; prove RootExport against that exact closure.
4. Lower the exact IR plus LockGraph under explicit lowering semantics into
   PlanCore.
5. Verify Plan semantics preservation independently from RootExport.
6. Revalidate the final pinned static inputs at Publish and verify all Plan,
   evidence and artifact obligations.

RootExport can run before PlanCore. It compares declarations in IR; it cannot
certify what lowering produces. If lowering changes the IR, pins, tables or
semantic parameter values, the earlier proof is invalidated. Conservative
analysis and executable semantics have different purposes; a plan-envelope
containment check alone is not behavioral preservation.

## Plan preservation obligations

The future Plan verifier requires a formal IR/Plan transition relation and
runtime semantics profile. It must establish the specified trace relation
under declared inputs/nondeterminism:

| Area | Required preservation |
| --- | --- |
| Control | All 13 opcode meanings, ownership, branch/loop selection, logical retry, timeout, cancellation and terminal outcomes |
| Values / interface | Exact I/O types, producers, scoped dataflow, ordering and carry values |
| Effects / resources | No new effect possibility, selector/qualifier widening, hidden resource, or scope leakage |
| Failure / Recovery | Business failures remain distinct from termination; K source/action/profile identities, owner and multiplicity preserved |
| Usage | Forward/Recovery bounds and meter/unit meanings preserved; K obligations not added twice |
| Flow / security | Source/sink/partition restrictions and required sandbox/trust/residency constraints preserved |
| Obligations | Required authorization/runtime checks remain at the proper boundary; no erased safety obligation |
| Dependencies | Every call resolves to exact locked semantic target/export/adapter; no ambient registry lookup |
| Semantics versions | Plan opcodes, lowering, profile and schema meanings match the versioned baseline |

An optimization may change internal mechanics only under a proven appropriate
observational equivalence/refinement relation. It may not invent additional
allowed public traces, race winners, recovery behavior or external commands.
This phase does not claim that such a relation has been formally encoded.

Compiler identity for publication must bind the lowering semantics and exact
Plan schema/profile as well as the RootExport semantics. Current
`CompilerSemanticsIdentity` lacks those future dependencies.

## Proposed engineering order

~~~mermaid
flowchart TD
  C["Portable context + compiler semantic identity models"] --> L["LockGraph schema + exact resolution projection"]
  L --> P["PlanCore schema + lowering model"]
  R["Existing RootExport static checker"] --> V["Fresh static revalidation"]
  C --> V
  L --> V
  P --> Q["Plan preservation verifier + independent vectors"]
  Q --> A["Versioned parent artifact / WorkflowVersion schema"]
  V --> A
  E["Evidence predicates + release policy model"] --> A
  A --> G["Future Publish implementation"]
~~~

Without PlanCore/LockGraph, an internal compile result is permitted. Actual
WorkflowVersion publication is blocked. Plan/LockGraph design, exact schemas,
canonical encoding and independent conformance must precede materialization
of a production version type.
