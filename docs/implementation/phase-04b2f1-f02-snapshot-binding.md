# Phase 4B.2F.1 — F02 Resolver Snapshot Binding

## Design choice: Option B

The implementation binds a content-complete, process-local resolver identity
to `DerivedWorkflowContractSummary`. `StaticComputationResolutionSnapshot`
creates the identity; callers cannot fill in or attach it. The identity
contains the sorted checked ComponentContract digests and, for every subflow
target, its SemanticIrDigest, optional direct-dependency vector, and optional
associated exported ComponentContract digest. The snapshot builder already
recomputes checked-contract digests under the exact proof-context object.

`derive_workflow_contract_summary` stores that identity alongside the root,
closure, and `RootRecoverySummary`. RootRecovery is derived in the same
operation from that root and context and cannot be detached or separately
supplied. RootExport preflight now requires both pointer-identical proof
context and structural equality between the stored root resolver identity and
the current snapshot's generated identity. Existing exact IR/export pins,
assertion dependency checks, and the content-complete manifest remain
independent checks.

If any component pin, target IR pin, direct dependency, or target-export
association differs, preflight is `Unproven`; all nine dimensions are
reported `Unproven` and the checked static capability is withheld. The
checker does not continue using stale root or recovery summaries.
For a resolver-identity mismatch the proof-context diagnostic retains a
reference naming the differing component-pin set, target dependency closure,
missing target, or exact old/current exported-contract pin; P31 asserts the
target-export mismatch reference.

## A/B tradeoff record

**Soundness.** Option A would freshly derive all root dimensions and
RootRecovery inside the checker from the supplied checked IR, exact snapshot,
and context. Option B proves the supplied summary was derived from exactly the
same structural resolver content and still checks the same IR digest and
proof-context pointer. Both close the reproduced gap; B does not trust a
caller-supplied identity because the summary field is private and assigned
only by root derivation.

**Scope and API impact.** A would move ownership of root derivation into
RootExport checking or require a fresh-vs-supplied summary comparison, with
changes across the checker API and callers. B adds a non-wire identity type,
one private summary field/getter, and one preflight condition. Existing
checker signatures and assertion mappings remain unchanged.

**Validation boundary and failure modes.** A would recompute the complete
root and recovery proof at check time. B validates exact structural snapshot
content while separately retaining existing exact-context pointer and
assertion-manifest checks. A mismatch, absent association, or changed
dependency fails closed as `Unproven`; no digest collision claim is made
beyond the existing SHA-256 ComponentContract/SemanticIr identity model.

**Performance.** A duplicates the full control, leaf, and recovery
derivation during checking. B stores a small sorted collection of pins and
compares it at preflight; no envelope arithmetic is repeated.

**Conformance.** Old RootExport cases continue to use structurally equal
snapshots and retain their expected output. New P31 and six resolver
soundness cases explicitly compare Root(A), assertion/Snapshot(B), and top
export(C) pins.

No resolver identity is serialized, hashed into a new domain, added to AFIDL,
or accepted from `RootExportAssertion`. It is process-local provenance only.
