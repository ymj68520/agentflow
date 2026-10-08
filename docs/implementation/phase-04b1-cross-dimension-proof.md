# Phase 4B.1 — Cross-Dimension Static Proof

The static validator keeps independent dimension verdicts and composes them
fail-closed. It checks: nominal I/O pins and port namespaces; F references to
I/O/R ports; complete ε atom identities and exact operation/resource
relations; operation-level F ceilings refined by exact EffectFlowPartitions;
K clauses linked to exact forward effects and declared recovery action
effects; per-action usage coverage and checked multiplicity arithmetic; and
F/X constraints against the exact flow/execution relation snapshot.

The explicit proof context is read-only. Every relation snapshot pin contains
its exact schema identity, opaque revision, and SHA-256 checksum of canonical
ACE1 bytes. A changed pin means the old result is not reusable. The pin is
evidence of byte identity, not publisher authenticity.

`Proven`, `Violation`, `Unproven`, and `RequiresEvidence` remain distinct at
each dimension and in the aggregate. This is a static constraint proof only;
there is no trusted-worker attestation, placement, sandbox enforcement,
policy authorization, or execution permission.

Substitution reuses the existing directional static proofs for failure,
closed effect containment, resource non-strengthening, flow non-relaxation,
usage non-increase, recovery non-weakening, and the independent execution
feasibility/security directions. Public I/O remains exact-set/exact-pin only.
Any changed effect-flow partition or recovery action/usage binding outside its
implemented directional proof remains `Unproven`.
