# Phase 4A.5B Decision Register — Fixed Cost Projection

Phase: 4A.5B. Wire authority: `spec/idl/cost-projection.afidl`; normative
semantics: `spec/usage/cost-projection.md`; implementation: `crates/af-usage`;
independent oracle: `tools/gen-cost-projection-corpus.py`.

## D-C1 Currency identity

Currency is an exact catalog/code identity resolved through an explicit
`CurrencyUniverse`. Atomic unit and display scale are supplied descriptors;
code spelling never implies precision or FX.

## D-C2 Atomic money

Money is a non-negative `uint64` count of atomic units. No floating point,
negative amount, refund, credit, tax or implicit conversion is represented.

## D-C3 Price snapshot by value

`PriceTableSnapshot` contains the complete currency, atomic unit, pricing
model and normalized entries by value. No latest provider lookup or mutable
PriceBook pointer exists. Meter and unit validation reuses the existing Usage
Universe.

## D-C4 Fixed rational pricing

Only fixed linear `numerator / denominator` rates are supported. Denominator
zero, unsupported tiered/progressive models and unsupported rounding rules are
rejected. No approximation may be called exact.

## D-C5 Conservative rounding

The frozen rule is `ceil(rate numerator / denominator)` per meter unit before
multiplication. This safely covers per-request upward rounding without a
trusted billing-grouping contract.

## D-C6 Coverage and static states

Every explicit possible Usage coordinate needs a matching phase-specific or
phase-independent price entry. Missing price is Unproven, not free. Cost
projection states are finite, unbounded or unproven, with forward/recovery
coordinates retained.

## D-C7 Budget proof

Forward and recovery ceilings are checked independently. Static Cost Proof is
not reservation, admission, quota, payment approval or actual billing.

## D-C8 Cross-snapshot comparison

Cost non-increase requires equal currency and equal complete price snapshot
values. Cost non-increase and Usage non-increase are independent judgments.

## D-C9 Deferred runtime

PriceSnapshotDigest, CostProjectionDigest, BudgetDigest, ledgers, Reserve,
Settle, refunds, reconciliation, FX, provider pricing, Run Admission and
runtime billing remain deferred. No HashV1 domain is added.
