# Phase 4B.2E.1 — Multiplicity and Occurrence Accounting

## Identity and coordinates

One applied repetition factor is identified by the exact tuple
`(caller SemanticIrDigest, control ComputationId, control kind)`. Only
`ITERATE` and `RETRY` produce factors. The factor stores the logical
repetition `OccurrenceBounds` and, for Retry, the physical attempt bounds as
separate metadata. Duplicate identities on one path, inconsistent factor
provenance, and checked multiplication overflow fail closed.

`LocalActionBound` carries the nearest `COMPENSATION_SCOPE` node and only the
repetition factors inside that scope's child. `RootActionBound` carries the
root SemanticIrDigest and every factor on the checked root path. There is no
implicit cast between the coordinates. The root action walk is leaf-seeded;
neither an already-local-scaled bound nor an already-root-scaled bound is
fed back through the same Repeat node.

## Factor application

For leaf Forward seed `F`, activation upper `A`, attempt upper `T`, local
factor set `L`, and root factor set `R`:

```text
unscoped base       = F × A × T
LocalActionBound    = base × each unique factor in L
RootActionBound     = base × each unique factor in R
```

Every product uses checked arithmetic. A known unbounded input remains
unbounded; missing factor/source evidence remains Unproven rather than
becoming one. Retry's physical attempt maximum is not a second root multiplier
because its logical repetition upper already represents the conservative
effect/usage attempts.

The root fold passes the original factor-free seed through existing
`af_effect::control` / `af_usage::control` operators. The K02–K05 tests assert
outer Retry, inner Retry, outer Iterate, and Retry×Iterate in both coordinate
systems, including exact applied factor identities. K19 verifies duplicate
computation identities are rejected in the checked IR stage, before they can
mint a second authenticated factor record. K25 verifies a U64 usage overflow
is rejected, never wrapped or saturated into Proven.

## Branch and usage treatment

Sequence sums possible children; Select joins mutually exclusive branches;
Parallel and Race accumulate all possible branches; Iterate and Retry apply
one logical factor; Timeout preserves the child; Handle composes protected
and conditional handlers; Scope preserves the child; CompensationScope
assigns local ownership but adds no count. The fold uses the existing operator
helpers instead of a generic sum.

A completely closed leaf with no K action contributes no K-required Usage
coordinate. Such a proven-empty contribution can be omitted before composing
active profiles; this does not treat a missing meter on an active K source as
zero. An active unknown meter, unknown unit, unresolved profile, or unknown
action count stays unknown. K Usage is compared with root `U.recovery`; it is
never added to that already-derived ceiling.
