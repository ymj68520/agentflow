# Phase 4A.5A — Usage Meter / Static Usage Envelope / Usage Algebra

Status: complete (closed 2026-10-08) for the static U dimension only.

Artifacts:

| Artifact | Path |
|---|---|
| AFIDL module | `spec/idl/usage-signature.afidl` |
| Rust crate | `crates/af-usage` |
| Normative semantics | `spec/usage/usage-signature.md` |
| Independent corpus oracle | `tools/gen-usage-signature-corpus.py` |
| Conformance corpus | `spec/conformance/usage-signature/` |
| Decision register | `docs/implementation/phase-04a5a-usage-decisions.md` |

The implementation materializes explicit additive meter identity, unit and
aggregation validation, forward/recovery phase separation, finite/unbounded/
missing distinction, canonical ACE1, derived static envelopes, conservative
Usage Algebra, control propagation and static non-increase proofs.

It deliberately does not materialize measurement, usage evidence, pricing,
Budget/Usage/Quota Ledgers, reservation/settlement, scheduler state,
`ComponentContract`, runtime, receipts or any new digest domain.

The frozen corpus contains 34 cases: 7 golden, 7 negative, 13 algebra,
6 compatibility and 1 equivalence. The Usage AFIDL module has 2 aliases and
5 numbered types with 10 ABI rows. The AFIDL corpus is revision 10
(additive). The registry totals after this additive phase are 24
specifications, 255 normative rules, 310 active ABI rows, 9 AFIDL modules
and 57 suites; `spec-check` reports zero missing coverage.

