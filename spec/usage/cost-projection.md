# Static Cost Projection (agentflow.usage.cost@1)

Status: frozen for Phase 4A.5B. The machine schema authority is
`spec/idl/cost-projection.afidl`; the implementation extends
`crates/af-usage`; the independent corpus oracle is
`tools/gen-cost-projection-corpus.py`.

## 1. Usage and Cost separation

<a id="cost-static-scope"></a>

Cost Projection consumes the static Usage Envelope from Phase 4A.5A. It
does not create a second Usage algebra and does not measure runtime use.

```text
Usage quantity ≠ Cost projection ≠ Actual bill
Static cost ceiling ≠ Run Budget Reservation
Run Budget ≠ Tenant Quota ≠ Provider Rate Limit
```

No price API, account balance, reservation, settlement, receipt, ledger,
quota or provider runtime exists in this phase.

## 2. Currency and atomic money

<a id="cost-currency-identity"></a>

`CurrencyRef` is an exact `(catalog SpecificationId, code)` identity resolved
by an explicit `CurrencyUniverse`. The universe supplies the atomic monetary
unit and display scale. Currency names never imply precision, exchange rates
or conversion rules.

Different currencies and different atomic units cannot be added, compared or
silently converted. There is no FX lookup.

<a id="cost-money-atomic"></a>

`MoneyAmount.atomic_units` is a non-negative `uint64`. It is the authoritative
internal representation; display scale is presentation metadata only. No
`f32`, `f64`, negative refund, credit, tax or discount is represented.

## 3. Fixed price snapshot

<a id="cost-price-snapshot"></a>

`PriceTableSnapshot` is complete by value: currency, atomic unit, fixed-linear
pricing model and normalized `PriceEntry` values. It is not a pointer to a
mutable PriceBook, a provider version string or a latest-price lookup.

Each entry identifies an existing Usage meter, its exact Usage unit, an
optional phase applicability, an exact rational rate and a rounding rule.
Meter and unit mismatches fail closed.

<a id="cost-price-rate"></a>

`PriceRate = numerator / denominator` with `denominator > 0` and both values
`uint64`. Only non-negative fixed-linear prices are supported. Tiered,
progressive, minimum-charge, subscription, tax, credit, surge, spot and
region-dependent pricing are unsupported and cannot be approximated as an
exact projection.

## 4. Deterministic safe rounding

<a id="cost-rounding"></a>

Phase 4A.5B supports exactly `per_meter_unit_ceil`:

```text
unit_upper = ceil(rate_numerator / rate_denominator)
cost_upper = usage_upper × unit_upper
```

The division is exact integer ceiling division. Multiplication and money
addition are checked. `aggregate_once_ceil` is not accepted because it may
underestimate a billing model that rounds each request independently:

```text
ceil(1/3) + ceil(1/3) = 2
ceil((1+1)/3) = 1       // unsafe lower result
```

The rule is deliberately conservative and may be wider than a future,
trusted billing-grouping contract.

## 5. Meter and price coverage

<a id="cost-meter-coverage"></a>

For every explicit Usage `(phase, meter)` coordinate with a possible
non-zero or unknown static bound, a matching phase-specific or explicit
phase-independent `PriceEntry` is required. Missing price coverage is
`Unproven`, never free.

The coverage proof establishes only the declared meter/unit/model scope. It
does not prove that a real provider will charge no other fees.

## 6. Static projection states

<a id="cost-projection-states"></a>

The non-wire `StaticCostProjection` retains `forward`, `recovery` and a
derived `total`, all in one Currency. Each bound is:

```text
KnownFinite(MoneyAmount)
KnownUnbounded
Unproven(reason)
```

The state table is frozen:

| Usage | Price | Result |
|---|---|---|
| finite(0) | valid fixed rate | finite(0) |
| finite(n) | positive rate | checked finite upper |
| finite(n) | explicit zero rate | finite(0) |
| unbounded | positive rate | unbounded |
| unbounded | explicit zero rate | finite(0), under fixed-linear no-extra-fee assumptions |
| missing/unknown | any | Unproven |
| any possible usage | missing price | Unproven |

Neither missing usage nor missing price is converted into zero.

## 7. Static Budget Ceiling

<a id="cost-static-budget"></a>

`StaticCostCeiling` carries one Currency and independent forward/recovery
atomic ceilings. `check_static_budget` proves both independently:

```text
projected_forward <= declared_forward
projected_recovery <= declared_recovery
```

Forward surplus cannot subsidize recovery overspend. `Proven`, `Violation`
and `Unproven` remain distinct; a static proof is not `BudgetReserved`,
`RunAdmitted`, `QuotaGranted` or `PaymentApproved`.

## 8. Cost non-increase and price snapshots

<a id="cost-non-increase"></a>

Cost non-increase is independent from Usage non-increase. Comparing numeric
costs under different currencies or different price snapshots is
`Unproven`; a lower number under a changed price cannot be attributed to a
component implementation. The same-price scenario requires equal complete
snapshot values and equal Currency identity.

## 9. Control and runtime boundary

<a id="cost-control-propagation"></a>

Cost uses the already-propagated Usage Envelope. RACE losers, RETRY attempts,
FOREACH total-count limits, HANDLE paths, recovery usage and TIMEOUT possible
usage must already be represented by Phase 4A.5A. INVOKE and CALL_SUBFLOW
without a complete Usage Contract remain unproven; Cost does not invent zero.

<a id="cost-no-runtime"></a>

This phase introduces no PriceSnapshotDigest, CostProjectionDigest or
BudgetDigest and no HashV1 domain. It introduces no UsageReceipt, Actual
Billing Receipt, Budget/Usage/Quota Ledger, Reserve, Settle, Refund,
Reconciliation, Provider Pricing API, FX service, Run Admission,
ComponentContract, Plan, Journal, Reducer, Runtime, Scheduler or Worker.
