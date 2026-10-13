# Phase 4C.0 — Gate Report

All publication models in this phase are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**.

## Baseline and scope

Start full HEAD: `92284a764c12550434fc3747e3369146d3a89309`.
Initial worktree was clean. Deliverable scope is the thirteen new
`docs/implementation/phase-04c0-*.md` reports.

No production Rust, AFIDL, ABI registry, Semantic IR, ComponentContract,
RootExport checker, HashV1 domain, formal Expected or runtime system is changed.
No temporary production probe is needed for architecture-only decision vectors.

## Recorded pre-commit verification

| Gate | Recorded result |
| --- | --- |
| cargo fmt --all -- --check | PASS, exit 0 |
| cargo clippy --all-targets --all-features -- -D warnings | PASS, exit 0 |
| cargo test --all | PASS, exit 0; all workspace tests including the 311-case production differential |
| cargo run -p af-spec --bin spec-check -- check | PASS, exit 0; 41 specifications, 380 rules, 379 covered, 1 intentional exception, 0 missing, 477 active ABI assignments, 16 AFIDL modules, 0 cross-reference errors |
| All tools/gen-*.py twice | PASS; all 24 generators returned exit 0 in both rounds |
| Initial versus round-one versus round-two content | PASS; SHA-256 compared across 1,199 tracked spec/crates/tools files, zero byte changes in either round |
| Frozen paths versus start HEAD | PASS; no diff under spec, crates or tools, including AFIDL/ABI/registry/Expected/digest goldens/source |
| 13 reports / 48 design vectors / seven core decisions | PASS; unique CaseIds, all nine vector columns, design markers and local links validated |

Formal production differential cases remain 58 Control, 72 Advanced Control,
55 Scope/Closure, 44 Root Recovery, 35 Root Export and 47 Root Export
Soundness: 311 in total. Phase 4C.0 vectors are design-only, Actual Not Executed,
and excluded from production pass totals.

Verified spec baseline is 41 specifications, 380 rules (379 covered, one
intentional exception, zero missing), 477 active ABI assignments, 16 AFIDL
modules and 14 registered HashV1 domains. Domain count was read directly from
the unchanged hash-domain table and its existing conformance passed.

## Commit boundary

The table records completed pre-commit commands against this source content.
All required gates are rerun after the containing final design commit.
The final handoff records the actual committed HEAD, post-commit outcomes and
worktree state; no self-referential digest of this report is invented.

The only allowed diff from the start HEAD is the thirteen design reports.
WorkflowVersion, PlanCore, LockGraph, Publication, DeploymentRevision and
runtime production work remain outside this commit.
