# Phase 4C.1A — Design and Frozen-Source Gate Report

All candidate models are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI /
NOT IMPLEMENTED**.

## Baseline and frozen scope

Start full HEAD: 89b4f3b41bde82a0bf15240b2b9ddb9013095c01.
The initial worktree was clean. The allowed change is fifteen new
docs/implementation/phase-04c1a-*.md documents only.

Frozen baseline: 41 specifications; 380 normative rules, 379 covered, one
intentional exception and zero missing; 477 active ABI assignments; 16 AFIDL
modules; 14 HashV1 domains. Historical production differential counts remain
58 Control / 72 Advanced Control / 55 Scope-Closure / 44 Root Recovery /
35 Root Export / 47 Root Export Soundness = **311**.

No production Rust, AFIDL, ABI assignment, hash domain, Semantic IR,
ComponentContract, checker, digest golden, formal Expected or generator is
changed. No temporary production probe or new executed fixture is added.

## Pre-commit verification ledger

This ledger records actual command outcomes completed before committing.

| Gate | Recorded result |
| --- | --- |
| cargo fmt --all -- --check | PASS, exit 0 |
| cargo clippy --all-targets --all-features -- -D warnings | PASS, exit 0 |
| cargo test --all, including existing 311 production differential cases | PASS, exit 0; all six formal corpus runners and P30/P31 regression passed |
| cargo run -p af-spec --bin spec-check -- check | PASS, exit 0; 41 specifications, 380 rules, 379 covered, one intentional exception, zero missing, 477 active assignments, 16 modules, zero cross-reference errors |
| All 24 tools/gen-*.py, two consecutive rounds | PASS; 24/24 exit 0 in each round |
| SHA-256 initial / round one / round two, all tracked spec/crates/tools files | PASS; 1,199 tracked files, zero byte changes in round one and round two |
| Frozen paths versus start HEAD | PASS; zero diff under spec/crates/tools, including all source, AFIDL, ABI, Expected and digest goldens |
| Fifteen documents / sixty vectors / ten decisions / closed local links | PASS; all four design markers, sixty unique eleven-field Not Executed vectors, all twelve decision attributes, balanced fences and local links |

Vector distribution is 12 schema/Decoder, 12 context, 10 resolver, 12 semantics,
6 origin and 8 cross-process. This satisfies all six minimum group counts.
The unchanged hash-domain table has fourteen entries and passed its existing
conformance. Corpus manifest counts were read independently as
58/72/55/44/35/47; their production runners executed inside cargo test --all.

All **sixty new design vectors have Actual Status = Not Executed**. They are
not production tests and do not increase the 311-case pass count.
Historical audit-design, root-reaching and full recorded-outcome counts are
not reclassified by this package.

## Commit boundary

The table records completed pre-commit commands on the containing content.
All required gates are rerun after the containing documentation commit.
The final handoff records actual final HEAD, post-commit outcomes and clean
worktree; this file does not invent a self-referential committed digest.

Only these fifteen design reports may differ from the start HEAD. MODEL-B01/B02
design acceptance is not implementation closure. B03–B07 remain BLOCKING;
F03/F04 remain OPEN/P2. Stop before all schema/Decoder or downstream production
implementation.
