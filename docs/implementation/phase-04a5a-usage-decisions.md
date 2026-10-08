# Phase 4A.5A Decision Register — Static Usage Signature

Phase: 4A.5A. Wire authority: `spec/idl/usage-signature.afidl`; normative
semantics: `spec/usage/usage-signature.md`; Rust: `crates/af-usage`; frozen
corpus: `spec/conformance/usage-signature/` (independent oracle:
`tools/gen-usage-signature-corpus.py`).

## D-U1 Static-only boundary

Usage declarations are static upper bounds only. They are not estimates,
measurements, receipts, billing, reservations, settlement, quotas or runtime
authorization. Budget, Usage and Quota Ledgers remain independent future
authorities.

## D-U2 Meter identity and universe

`UsageMeterRef` is the exact `(Catalog SpecificationId, NFC MeterName)` pair.
`UsageMeterUniverse` is explicit, immutable and in-memory. Every recognized
meter has an exact unit identity and aggregation descriptor. There is no
discovery, latest fallback, registry lookup or pricing lookup.

## D-U3 Additive restriction

Only non-negative integer additive meters are supported. Different meters or
different units cannot be added or compared. Unsupported aggregation is
rejected; wall-clock latency is not silently treated as additive usage.

## D-U4 Phase accounting

`forward` and `recovery` are independent coordinates. Compensation adds
recovery usage alongside forward usage and never refunds forward usage.

## D-U5 Bound state separation

`finite(0)`, `finite(n)`, `unbounded` and missing are distinct. Missing is
not zero; unbounded is not `UINT64_MAX`; no lower bound is declared. The
derived analysis states are `KnownFinite`, `KnownUnbounded` and `Unknown`.

## D-U6 Canonicalization

Canonicalization resolves the explicit universe, NFC-normalizes names,
rejects duplicate `(phase, meter)` keys, sorts by phase number and complete
meter identity bytes, preserves explicit zero and unbounded values, and emits
ACE1 only. No Usage digest or HashV1 domain is added.

## D-U7 Algebra and proofs

Accumulation is checked addition; branch join is maximum; repetition is
checked multiplication; unknown propagates conservatively. `within_ceiling`
and `usage_not_increased` return `Proven`, `Unproven` or `Violation` without
collapsing those outcomes.

## D-U8 Control propagation

SEQUENCE/PARALLEL/RACE accumulate, SELECT joins, RETRY amplifies by attempts,
ITERATE requires a proven total count, TIMEOUT retains possible usage,
HANDLE includes protected and handler paths, and SCOPE does not erase usage.
`max_inflight` is never used as a total iteration count. INVOKE/CALL_SUBFLOW
remain unknown until a complete future Usage Contract is supplied.

## D-U9 Cost and runtime deferral

Currency, price snapshots, conversions, rounding, BudgetReservation,
Reserve/Settle, Usage Ledger, Budget Ledger, Quota Ledger, scheduler,
provider rate limits, receipts and runtime measurement are all deferred.

