# Phase 4B.2B.2 — Advanced Control Contract Algebra

Status: implemented and gated. Starting baseline: `699d98525adfc8ccd2213527ae570c7309c222fe` (Phase 4B.2B.1C).

## Delivered

- Added derived-only propagation for the existing `ITERATE` (FOREACH/WHILE),
  `RETRY`, `TIMEOUT`, and `HANDLE` IR nodes. `SCOPE` and
  `COMPENSATION_SCOPE` remain unsupported/Unproven.
- ITERATE carries finite or unbounded conservative repetition bounds; the
  FOREACH `max_inflight` is retained only as concurrency. Exact checked carry
  mappings feed typed outputs; incomplete mappings do not become guessed
  interfaces.
- RETRY preserves the ordered delay schedule, separate physical-attempt
  bounds, conservative logical Effect/Usage amplification, and per-source K.
  Existing Recovery safety analysis is applied to exact leaf Effect/K facts;
  forbidden replay is a violation, while missing coverage and runtime-evidence
  requirements remain explicit. No retryable-failure policy is invented.
- TIMEOUT preserves all child dimensions and reports `child_completed` and
  `timed_out` separately; TimedOut is not converted to a Failure type, and no
  successful cancellation or zero-effect result is inferred.
- HANDLE matches exact pinned Failure types, keeps unmatched residual failures,
  and composes protected-child plus conditional handler Effect/Usage. Missing
  meter coordinates stay unknown; only the no-handler path receives zero for
  meters explicitly represented by handlers.
- Added nested effect-family provenance and Flow control-decision dependencies
  without adding runtime occurrence/attempt identities or wire fields.

## Differential evidence

`spec/conformance/computation-advanced-control/` contains 72 vectors generated
by the independent Python implementation in
`tools/gen-advanced-control-corpus.py`: 60 core cases (12 per control mode),
five typed-output/carry cases, one nested RETRY(ITERATE) case, and six boundary
cases including effect/usage overflow, unmatched HANDLE type, forbidden
replay, dynamic TIMEOUT, and unsupported SCOPE. Every vector is materialized
through checked Semantic IR, Phase 3C dataflow, production leaf resolution,
and `derive_control_contract`; the runner compares 1,413 declared projection
fields and checks same-snapshot repeat determinism. Ten semantic projection
mutations are rejected.

The prior 58-case Phase 4B.2B.1C suite remains green. Its historical
`unsupported-timeout-not-empty` CaseId is retained but its Expected now records
the supported TIMEOUT result (known child Effect); corpus revision is 5. No
other old expected case changed.

## Gates

- `cargo fmt --all -- --check` — pass.
- `cargo clippy --all-targets --all-features -- -D warnings` — pass.
- `cargo test --all` — pass, including the 72-case production differential,
  the 58-case regression suite, and the static-surface/wire-freeze tests.
- `cargo run -p af-spec --bin spec-check -- check` — pass: 39 specifications,
  353 rules, 352 covered, one intentional exception, 477 active ABI rows,
  16 AFIDL modules, zero cross-reference errors.
- The three relevant generators (58-case corpus, advanced-control corpus, and
  registry instances) were each run twice. Generated-state aggregate SHA-256
  was identical: `E685589FC4CE0F84749C3E5698B6AA3AED4F016FABA73027071CA07E426D3059`.

No AFIDL module, Semantic IR schema, ComponentContract field, ABI assignment,
HashV1 domain, or digest scope changed. Phase 4B.2B.3 was not started.
