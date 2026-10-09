# Phase 4B.2C — Effect and Recovery accounting

## Independent formulas

Let `F` be the declared Forward occurrence upper for one scoped source, `M`
the product of logical repetition upper bounds attached to that occurrence,
`A` the K activation upper per Forward occurrence, `T` the K attempts upper
per activation, and `q` a per-attempt Recovery Usage meter upper. The
per-CompensationScope-node bounds are:

```text
local potential compensation Effect upper = F × M × A × T
local potential Recovery Usage upper       = q × local potential Effect upper
```

Both products use checked arithmetic. The lower bound is zero because there
is no EffectReceipt. For a matching action EffectAtom, the checked source ε
row must cover the sum of K action uppers (and the Forward upper too if the
complete atom is identical). This check is grouped by source node, exact
contract digest, and full action EffectAtom.

If a parent `RETRY`/`ITERATE` lies outside the CompScope node, its factor is
not in that node's child occurrence families. The parent independently
amplifies root ε and U and appends its family to the root occurrence
provenance. The child CompScope action summary remains local. A root-wide
conditional K aggregate is not currently materialized.

## Manual arithmetic checks

### One Forward

```text
F=1, M=1, A=2, T=3, q=1
local action upper = 1×1×2×3 = 6
local Recovery Usage upper = 1×6 = 6
```

Declaring action ε upper 5 is a Violation; upper 6 covers the action row if
the forward atom differs. The 55-case corpus tests many products and has an
underdeclared action-row negative. It does not contain this exact numeric
tuple as a distinct named vector.

### Retry nested inside CompScope

```text
COMPENSATION_SCOPE(RETRY(max attempts=3, INVOKE(F=1)))
A=2, T=2
local occurrence family M=3
local K action upper = 1×3×2×2 = 12
```

Here the inner Retry has already attached its family to the forward
occurrence seen by CompScope. Multiplying the local action by 3 again would
double count. Root U.recovery is the leaf U.recovery envelope ×3.

### Retry outside CompScope

```text
RETRY(max attempts=3, COMPENSATION_SCOPE(INVOKE(F=1)))
A=2, T=2
CompScope-local K action upper = 1×1×2×2 = 4
whole-root possible K actions = 4×3 = 12
```

The inner CompScope summary does not itself become 12; the parent Retry
family appears on the root occurrence and root ε/U envelopes. This is not
proof that the local summary is wrong; it is a missing formal coordinate
statement and root aggregation API. The audit probe using committed
`action-bound-06` observed local action 9 versus 27 for the opposite nesting
order, while both root ε action upper and root U.recovery were 54. The larger
root values came from the fixture's declared action row/U ceiling (18 per
leaf) multiplied by 3; they were conservative but did not make the K-local
field a whole-root total.

### Iterate nested with CompScope

```text
ITERATE(total count=4, COMPENSATION_SCOPE(INVOKE(F per iteration=2)))
A=1, T=2
per-iteration local K action upper = 2×1×2 = 4
whole-root possible K actions = 4×4 = 16
```

In the reverse nesting, CompScope sees the Iterate family and reports 16.
Neither orientation should multiply by `max_inflight`; it is concurrency,
not total count.

### Same full EffectAtom for Forward and action

If the exact full atom is both Forward and compensation action with finite
Forward upper `F>0` and `A×T>0`, the single E row bound must cover both
opportunities. The current local coverage rule requires at least
`F + F×A×T` after family scaling. A finite self-bound that supplies `F` from
the same row cannot be assumed sufficient; the check fails closed when it
does not cover. Unbounded is still explicit, not finite zero. No committed
55-case vector covers this same-atom shape; it is listed as P2 coverage work.

## Recovery Usage: no double addition, but scope caveat

`check_recovery_usage` validates each checked ComponentContract's K usage
profile and aggregate against `U.recovery`. At a parent Retry/Iterate,
`compose_usage` scales the child U envelope once per enclosing logical
occurrence. The CompScope `potential_recovery_usage` is a separate conditional
projection and is not added into `DerivedControlContractSummary.usage`; thus
the current code does not add K usage to U a second time. Missing meter
coverage is Unproven/Violation at leaf checking, and finite multiplication
overflow is a Violation.

The two nesting orders expose per-node versus whole-root K bounds (see above).
Root U remains conservatively scaled; however a consumer must not sum the
per-node potential Recovery Usage again on top of root U, nor mistake a local
CompScope amount for the root total. A typed scope coordinate/aggregation rule
is required before a future root-export checker can compare K and U once.

## Production path examined

1. `ComponentContract` validation checks exact K↔ε action identity and
   RecoveryUsageCoverage, then checked K usage against U.recovery.
2. `derive_effect_dimension` applies Iterate/Retry envelope algebra and adds
   one occurrence-family annotation per enclosing Iterate/Retry node.
3. `derive_usage_dimension` applies the corresponding U repetition algebra.
4. `derive_recovery_dimension` carries K sources per child; it does not
   aggregate or multiply their binding objects.
5. `derive_compensation_scope` enumerates exact K compensation bindings,
   matches forward and action ε occurrences, calculates local K upper and
   Recovery Usage, and checks action ε coverage.

The key risk is step 5 executing bottom-up before an outer repetition family
is available. Do not multiply a second time if a future root aggregator reads
an occurrence that already contains the same family.

## P1 guard-relation reproduction

The same CompScope input gives a reproducible P1 guard-relation case. Keep
the Forward Effect unconditional and set only the E row entry for the exact
compensation action atom to `input_equals(port=1,value=0)`. The action identity
in K still carries the same full atom but has no guard field; its activation
and attempt upper bounds remain positive. The exact EffectGuard rule says the
declared E action is possible only for input 0. The CompScope potential action
uses the Forward occurrence guard (unconditional), as required to conservatively
represent compensation after any applied Forward.

An isolated production fixture probe observed:

```text
root verdict                         = requires_evidence
declared epsilon action guard        = input_equals(port=1,value=0)
derived conditional action guard    = unconditional
derived action upper                 = 9
```

The ComponentContract checker accepts the action atom's presence and bound;
the CompScope checker checks `declared_upper.covers(required_upper)` but not
`guard_contained(K-derived-action-guard, declared-action-E-guard)`. The
potential action is wider than the declared E guard, so this is conservative
as a possibility sidecar and does not by itself demonstrate a false-negative
total. It does leave the E↔K guard relationship without a proof and makes the
root-export comparison rule ambiguous. The probe file was removed; no
production code or frozen Expected was changed.

The narrow follow-up is a formal decision about whether the action E guard is
an execution ceiling or whether K derives a wider sidecar. A future checker
must either prove guard containment or retain an explicit unresolved
obligation; this phase does not implement that rule.
