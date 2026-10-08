# Static Usage Signature (agentflow.usage.signature@1)

Status: frozen for Phase 4A.5A. The machine schema authority is
`spec/idl/usage-signature.afidl`; the Rust implementation is
`crates/af-usage`; the independent corpus oracle is
`tools/gen-usage-signature-corpus.py`.

## 1. Static scope

<a id="usage-static-scope"></a>

The Usage Signature is the static U dimension of
`Computation<I, O, Φ, ε, R>`. It declares additive upper bounds. It is not
an estimate, measurement, runtime evidence, billing record, authorization,
reservation, settlement or quota decision.

```text
Usage declaration ≠ Usage estimate ≠ Usage evidence ≠ Actual billing
Run-local static ceiling ≠ Cross-run quota ≠ Tenant fairness ≠ Provider rate limit
```

The authoritative runtime objects remain separate:

```text
Budget Ledger → reservation/accounting authority
Usage Ledger  → measured consumption evidence
Quota Ledger  → cross-run governance
```

None is represented by this module, and none enters `SemanticIrDigest`.

## 2. Meter identity and explicit universe

<a id="usage-meter-identity"></a>

`UsageMeterRef` is the exact nominal pair `(Catalog SpecificationId, NFC
MeterName)`. Canonicalization resolves it against an explicit, immutable
`UsageMeterUniverse`; unknown identities, malformed catalogs and malformed
names fail closed. There is no registry lookup, latest fallback, discovery,
network access or automatic registration.

Each universe descriptor supplies an exact unit identity and aggregation
semantics. This phase recognizes only non-negative integer additive meters.

<a id="usage-additive-only"></a>

Different meter identities and different unit identities are never added or
compared as if they were the same quantity. Unsupported aggregation, such as
elapsed wall-clock latency treated as an additive consumption amount, is
rejected or remains unproven. Parallel elapsed time is not inferred by adding
task durations.

## 3. Forward and recovery accounting

<a id="usage-phase-separation"></a>

`UsagePhase` has exactly two values: `forward` and `recovery`. The identity
of a usage coordinate is `(phase, meter)`, so these are independent:

```text
(forward, model.output_tokens)
(recovery, model.output_tokens)
```

Recovery usage cannot overwrite or refund forward usage. Compensation success
does not erase a previously incurred forward upper bound.

## 4. Upper-bound states

<a id="usage-bound-states"></a>

`UsageUpperBound` has three semantic situations, two representable values and
one absence:

```text
finite(0)       explicit zero upper bound
finite(n)       finite upper bound
unbounded       explicitly no finite ceiling
missing meter   no declared proof; never finite(0)
```

`unbounded` is not `UINT64_MAX`. The wire declaration has no lower bound:
conditional execution must not be turned into a guaranteed positive lower
bound. The derived `StaticUsageEnvelope` distinguishes `KnownFinite(n)`,
`KnownUnbounded` and `Unknown`; a missing coordinate returns `Unknown`.

## 5. Canonical Usage Signature

<a id="usage-canonical"></a>

`canonicalize_usage_signature`:

- resolves every meter in the explicit universe;
- NFC-normalizes meter names;
- rejects duplicate `(phase, meter)` keys;
- orders limits by frozen phase number, then meter catalog and name bytes;
- preserves `finite(0)` and `unbounded` exactly;
- is idempotent and independent of source declaration order.

The canonical bytes are ACE1 only. No `UsageSignatureDigest`, `UsageDigest` or
new HashV1 domain exists.

## 6. Usage algebra

<a id="usage-accumulate"></a>

`accumulate(a, b)` is checked addition per identical `(phase, meter)` key:

```text
finite(a) + finite(b) = finite(a+b)
finite(a) + unbounded  = unbounded
Unknown + anything     = Unknown
```

An absent coordinate is unknown, not zero. Finite overflow is an explicit
`usage_arithmetic_overflow`; arithmetic never wraps or saturates.

<a id="usage-branch-join"></a>

`branch_join(a, b)` is the conservative maximum of mutually exclusive
branches:

```text
join(finite(a), finite(b)) = finite(max(a,b))
join(unbounded, anything)  = unbounded
join(Unknown, anything)    = Unknown
```

It is commutative, associative and idempotent over the defined coordinate
semantics. A coordinate absent from one branch is not a zero branch.

<a id="usage-repeat"></a>

`repeat(n, U)` uses checked multiplication. `repeat(0, U)` yields explicit
zero for coordinates present in U; it does not create zero declarations for
meters absent from U. A count that is not proven finite cannot yield a finite
static bound.

## 7. Static ceiling proof

<a id="usage-static-proof"></a>

`within_ceiling(observed_static_envelope, declared_ceiling)` returns exactly:

```text
Proven       every consulted coordinate is explicitly bounded by the ceiling
Unproven     a meter, unit, bound or observation is missing/unknown
Violation    a finite or unbounded observed upper exceeds the ceiling
```

The observed envelope is a static analysis result, not measured runtime usage.
An empty declaration and an empty observed projection do not prove zero for
an unspecified meter.

## 8. Conservative control propagation

<a id="usage-control-propagation"></a>

Static propagation is pure and independent of `af-ir` runtime state:

| Control | Static Usage rule |
|---|---|
| SEQUENCE | accumulate all children |
| SELECT | branch join / maximum |
| PARALLEL | accumulate all branches |
| RACE | accumulate all contestants, including losers |
| ITERATE | repeat by a proven total count only |
| RETRY | amplify by maximum possible attempts |
| TIMEOUT | retain potentially incurred usage |
| HANDLE | conservatively include protected child and possible handlers |
| SCOPE | does not exempt usage |
| COMPENSATION_SCOPE | retain forward and recovery coordinates separately |
| INVOKE / CALL_SUBFLOW | unknown until a complete Usage Contract is supplied |
| AWAIT | never invent all-meter zero usage |

<a id="usage-retry-race-iterate"></a>

`RETRY` keeps logical usage and physical attempt count distinct. Without a
later recovery/idempotency proof, no retry narrowing or refund is inferred.
`RACE` retains usage from a loser because cancellation can occur after work.
`FOREACH` `max_inflight` is a concurrency bound, not a total iteration bound;
it cannot be used to produce a finite total Usage upper bound.

## 9. Usage non-increase

<a id="usage-non-increase"></a>

`usage_not_increased(new, old)` compares the same phase, exact meter identity,
unit and additive aggregation:

```text
new finite <= old finite       → Proven
new finite, old unbounded      → Proven
new unbounded, old finite      → Violation
new finite > old finite        → Violation
missing or mismatched identity → Unproven
```

`Unproven` is never compatibility approval. This is only the U-dimension
comparison; full ComponentContract substitution remains deferred.

## 10. Cost and budget boundary

<a id="usage-cost-boundary"></a>

Usage quantity is not monetary cost. This phase freezes no currency, pricing
snapshot, unit conversion, rounding policy or provider rate validity. A token
count cannot be converted to USD without a separate frozen Cost Projection.

<a id="usage-budget-boundary"></a>

A static ceiling proof is not a `BudgetReservation` and does not perform
`Reserve → Execute → Settle`. No Budget Ledger, Usage Ledger, Quota Ledger,
quota service, tenant fairness policy or scheduler state exists here.

## 11. Runtime boundary and deferred work

<a id="usage-no-runtime"></a>

The Usage AFIDL module and `af-usage` are static only. They contain no usage
receipt, runtime measurement, pricing API, ledger, reservation, settlement,
quota, scheduler, worker, policy engine, credential, `ComponentContract`,
Plan, Journal, Reducer or runtime object. No new HashV1 domain is introduced.

Future stages may compose U with F, K, X, ε and R, and may connect static
declarations to independent runtime ledgers. They must not reinterpret this
declaration as runtime evidence.
