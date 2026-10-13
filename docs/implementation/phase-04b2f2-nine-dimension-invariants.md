# Phase 4B.2F.2 — Nine-Dimension and Recovery Invariants

## Preflight gate

RootExport requires every preflight item to be Proven before it evaluates
dimensions (crates/af-computation/src/root_export.rs:793-807). If any
precondition is Unproven or Violation, all nine dimensions are materialized
as Unproven; a violation still makes overall static compatibility Violation
(808-827). Checked capability construction requires overall Proven only
(882-897).

The preflight includes canonical/unique typed assertion mappings, recomputed
exact IR and top-export pins, exact proof-context pointer plus structural
resolver identity and full manifest equality, current compiler-semantics
identity, and direct/transitive subflow dependency pins (900-979). No
dimension-specific success can override a failed preflight.

## Dimension matrix

| Dimension | Static relation and re-audit invariant |
| --- | --- |
| I | Exact typed input maps; duplicate source/target identities are rejected. A duplicate-source probe yielded I Violation, no capability. |
| O | Exact typed output and producer mappings; stale target interface content invalidates the complete Resolver identity before this dimension. |
| Φ | Complete Failure row containment; changing the target Failure row invalidates old-root provenance. |
| ε | Normative same-row effect containment, including operation/qualifier/selector/guard/bounds and one-use slots. |
| R | Exact resource port/type/scope mapping and resolver context; changed resource attributes invalidate the old root. |
| F | Information-flow source/authority/taint constraints; changed Flow content invalidates the old root. |
| U | Forward and recovery usage profiles are derived and compared without adding a second K sidecar. Changed Usage content invalidates old provenance. |
| K | Recovery source/action/profile multiset and source closure are checked; K effect requirements are independently contained in Root ε and Export ε. |
| X | Execution requirements are checked; changed execution content invalidates old provenance even if another dimension is unchanged. |

The identity mismatch is dimension-independent: if a changed resolver fact
happens not to change a particular numeric envelope, preflight still blocks all
nine dimensions. This is enforced at the single preflight gate rather than by
expecting each dimension to notice every source mutation.

## K, ε, and U accounting

The root K ε-coverage requirement is derived in RootRecovery and checked twice
as a containment obligation: against already-derived Root ε and against Export
ε (root_export.rs:2551-2590). It is not merged into the Root ε envelope a
second time. K recovery usage profiles are source-bound and then compared
against RootRecoverySummary.recovery_usage and exported recovery usage in the
recovery proof (root_export.rs:3531-3610 and subsequent profile checks).
The aggregate Recovery contract is not counted as an extra action.

Independent mutations covering one action, two co-occurring source nodes,
and Forward plus Recovery on the same atom all failed with no capability when
their required bounds/guards were not satisfied. Exact same-row and
exact-snapshot positives still returned Proven.

## Capability boundary

CheckedRootExportCompatibility has a private report field, no public
constructor, and is created only inside check_root_export_assertion after
the overall static verdict is Proven (root_export.rs:542-553,882-897).
RootExportProofReport is a report value; callers cannot convert it into the
checked capability by setting its public verdict field.

Even a valid capability claims only static compatibility. Body and
implementation conformance remain RequiresEvidence; publisher authentication
and runtime authority remain NotEstablished (root_export.rs:865-877).
