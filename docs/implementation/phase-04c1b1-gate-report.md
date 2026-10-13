# Phase 4C.1B.1 — Gate Report

All changes are scoped to the registered portable schema, D0-D4 codec and
conformance. This report records measured pre-commit and post-commit gates,
the old-row comparison, and the final clean-tree check.

Start HEAD: efc2a8d90a65432916a8dbf559510e2936f0d146.
Initial tree was clean. The pre-allocation snapshot is
[baseline.json](../../spec/conformance/portable-proof-codec/baseline.json).

## Registration and implementation

Schema materialization: 108 named types, including 17 PpiAux types, 298 new
ABI assignments. The registered schema corpus includes 108 canonical
TypeSchema byte strings and digests. All 477 old assignment records compare
exactly with the pre-registration snapshot.

| Property | Recorded result |
| --- | --- |
| Specifications / Rules / covered | 43 / 404 / 403, one intentional exception, zero missing |
| Active ABI / AFIDL modules / HashV1 domains | 775 / 17 / 14 |
| Portable production corpus | 60 unique CaseIds across seven D0-D4 groups |
| Comparator mutations | 17 separate projection-comparator fault injections |
| Legacy production differentials | 311 across six existing suites |
| Registered collection order rules | 61 typed rules, including CompleteCollection expansion |
| D5-D10 | Not implemented by this phase |

## Required gates

| Gate | Pre-commit | Post-commit |
| --- | --- | --- |
| cargo fmt --all -- --check | Pass | Pass |
| cargo clippy --all-targets --all-features -- -D warnings | Pass | Pass |
| cargo test --all, 311 historical cases | Pass | Pass |
| Portable Python/Rust corpus, full execution/comparison | Pass: 60/60 | Pass: 60/60 |
| Comparator and malformed-input mutations | Pass: 17/17 comparator; 60/60 inputs | Pass: 17/17 comparator; 60/60 inputs |
| cargo run -p af-spec --bin spec-check -- check | Pass: 43/404/403/1/0; 775 ABI; 17 modules; 14 domains | Pass: same counts; zero errors |
| All tools/gen-*.py, two rounds | Pass: 25 generators; 48 visible files, 0 differences | Pass: 25 generators; clean after each round |
| Old ABI rows, registry/source bytes and old goldens | Pass: 477/477 ABI; 380/380 Rules; 41/41 Specs; 960 frozen files unchanged | Pass: same exact comparisons |
| Worktree clean | Staged for commit | Pass |

Generator inventory is 25 scripts from the current repository. The three
pre-existing AFIDL golden files retain all prior cases (8 import, 52 schema,
52 TypeSchema digest); only one portable schema case and digest set are added.
The suite registry retains all 95 prior entries and adds one. All other
baseline frozen file hashes, including prior Semantic IR, ComponentContract,
RootExport, P30/P31 and HashV1 artifacts, are unchanged.

The permitted scope is additive registered AFIDL, normative sources, ABI/rule
rows, codec, independent Python oracle, conformance fixtures and the eleven
phase documents. No new HashV1 domain or Bundle digest is allowed. Stop after
the final documentation commit and its gates; D5-D10 and B03-B07 remain
blocking.
