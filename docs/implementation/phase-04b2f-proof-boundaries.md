# Phase 4B.2F — Proof and Trust Boundaries

## Claims that remain distinct

The checker returns a static Root/Export compatibility capability only when
its report verdict is Proven. It separately records body conformance,
implementation conformance, authority/budget attenuation,
publisher authentication and runtime authority. The confirmed P0 probes show
that this separation is represented correctly while the epsilon/provenance
premises may still be wrong. Thus body/runtime separation is accepted as a
type boundary, not evidence that the static capability is sound.

Subflow target export resolution, target static envelope compatibility,
target body conformance and target implementation authentication are separate
facts. Missing body evidence remains RequiresEvidence. Static Usage is not a
budget; resource visibility is not a handle; X compatibility is not Worker
admission.

## Guard/control-path behavior

The f011eef change prevents typed guard countermodels when the overall root
tree contains SELECT, RACE or HANDLE. The result remains Unproven if the
containment primitive cannot prove the relation. It does not downgrade a
known bound/atom mismatch that is independently detected. The guard probe
returned Unproven, and the epsilon row stayed explicitly checked.

The implementation's root-global switch is wider than necessary: an unrelated
branch can suppress a typed countermodel elsewhere. This loses precision but
does not upgrade the relation to Proven. No false-Proven finding was attributed
to f011eef itself.

## Static subset is not release-ready

The two P0s invalidate any blanket assertion that a returned
`CheckedRootExportCompatibility` is safe as a Release Control. The only
currently justified positive claim is a report-shaped computation over the
specific supplied root summary and maps; the capability may be based on a
spliced effect ceiling or stale subflow export facts. Until fixed and
re-audited, do not use it as a gate for publication, deployment, approval,
budget, execution or recovery.

No publisher signature, binary attestation, trusted body evidence, grant,
reservation, receipt, dispatch, settlement or runtime state is created by these
APIs. This audit adds no such model.
