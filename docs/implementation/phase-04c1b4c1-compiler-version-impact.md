# Compiler semantics identity and migration decision

Design only. Current43/404/403/1/0, ABI775, AFIDL17, HashV1 domains14 and all exact-profile bytes remain unchanged. [Draft D15–D16](phase-04c1b4c1-g01-normative-contract-draft.md) defines the proposed version boundary.

## Complete content, not a label

RootExport currently binds analysis version plus both registry byte strings,16 applicable normative sources and84 applied rule records (`root_export.rs:56–81,571–702`). Portable D4 compares its complete implementation-owned manifest (`af-portable/src/support.rs`; normative `ppi-current-profile`/`ppi-attribution`/`ppi-no-upcast`), including all43 source rows, both registries, four closed tables, roles/dependencies, schema requirements and byte associations. Checksums are ordinary SHA-256; they cannot replace content identity or become an ObligationDigest.

Registering obligation kinds/identity/derivation/order/encoding changes normative content. Adding a new source changes registry bytes and membership; adding RuleIds changes rule registry/applied rows; changing static analysis support changes roles and dependencies. Even unchanged AFIDL/IR/ComponentContract bytes do not make a new compiler profile byte-equal to the old one. Merely changing `agentflow.root-export-checker@1` or updating Expected cannot cure that mismatch.

## Three alternatives

| Strategy | Benefit | Cost / danger | Decision |
| --- | --- | --- | --- |
| New Analysis Semantics Version | Explicit report-contract break and exact content boundary | New token alone insufficient; all sources/registries/rules/roles must agree | Required part of recommended migration; proposed successor `agentflow.root-export-checker@2` is NOT allocated here |
| Explicit historical exact-version support | One binary could continue verifying old512/old portable bytes | Requires separate immutable profile tables, version-dispatch validators/checkers/projections and independent Expected; cannot substitute new rules into old proofs | Not recommended for this first migration; no historical dispatcher is authorized/implemented |
| Fail-closed current-only migration | Small supported semantics surface, matches existing fail-closed trust model | Old exact manifests legitimately reject; legacy/new test editions must be partitioned explicitly | Recommended with a newly identified analysis/obligation edition |

The chosen strategy is **newly versioned analysis + current-only exact support**, not all three at once. Keep legacy code/artifacts/Expected available at the historical commit; a new current-only build rejects old unsupported manifests, never silently upcasts them. Reproducibility is by the proper historical verifier, not arbitrary label replacement.

## Proposed registration transaction (future authorization required)

Prefer a separately registered proof-obligation semantics source (candidate logical name `agentflow.computation.proof-obligation.semantics@1`, UNREGISTERED) rather than pretending these new semantic fields already have canonical tokens. Its content becomes applicable to the new RootExport checker: the currently16-source static set must expand to include it, or an explicitly approved equivalent versioned source arrangement must be selected. The current84 applied rows cannot be left unchanged if new normative rules govern the report.

The16 local draft clauses require formal RuleId allocation/review, not reusing existing meanings. Let N be the approved newly applicable rules; future registry/rule counts and static applied rows must be explicitly recomputed, not asserted to remain43/404/16/84. The16 draft labels do not pre-allocate N or official IDs. Existing row meanings/old corpus are preserved, not overwritten to disguise a break.

Portable's current normative text explicitly says checker@1,16 static sources,84 rows and current-only support. A new support edition therefore ALSO requires an authorized version/revision decision for that Portable support specification and source-role mapping. Adding a new obligation specification without updating the applicable support contract would be inconsistent. Whether registry policy requires new major SpecificationIds or explicit source revisions must be resolved in the registration transaction; no implicit edit of any of the current43 rows is allowed by this design task. This is registration allocation/governance, not an unresolved obligation identity rule.

Atomic future steps: approve exact source/RuleId/version allocations and regression-edition policy; register exact source/registry/table content; implement typed report facts/projections and supported profile; materialize independent NEW-edition fixtures/Expected; validate all complete byte associations and fail-closed negatives; run legacy verifier on frozen legacy corpus and new verifier on new edition; independent adversarial audit before closure. No partially updated manifest is supported.

## Affected artifacts, including unchanged numeric D9 semantics

| Historical surface | Impact if new edition is activated | Preservation requirement |
| --- | --- | --- |
| Portable codec60, static reconstruction63, RootExport78 =201 CaseIds | Embedded exact compiler manifests/profile support may reject at D4 in a new current-only build | Keep old bytes and old Expected; new edition separately identified |
| RootExport26 and other52 original registration entries | All include an applicable version/stage contract; early rejects still get actual stage/error, never invented D10 | No old CaseId reclassification or success rewrite |
| D9 Control22 / Recovery16 / new24 precondition sidecars | Underlying arithmetic may remain identical, but input canonical byte length/SHA and compiler scope change with a new bundle | Preserve old sidecars; derive new-edition Expected independently, not patch hashes |
| RootExport35 / soundness47 | Compiler identity and obligation/reference/precondition report projections may change | Review exact per-case fields; keep old fixtures for legacy verifier |
| Other control/scope/recovery corpora in historical512 | No automatic substantive algebra change is authorized | Re-run and prove unchanged domains; do not assume all fields change or all fields stay equal |

**Compatibility constraint:** a current-only successor cannot simultaneously claim it accepts every old exact-profile portable case unchanged. Future authorization must approve version-partitioned regression or choose the more costly historical support alternative. This phase's512-case pass is on the unchanged current verifier, not proof of a nonexistent successor. Finding G01-MIG/P1 records this activation dependency. Registration design can begin from this accepted draft, but activating production support is blocked until the migration transaction and test-edition policy are approved.

No new public wire schema/domain is necessary for the in-process/report-local proposal. If future publication/transmission makes this report a public wire object, separate AFIDL/ABI and transport authorization is required; this task does not grant it. Existing plan_lowering/runtime_semantics_profile roles remain model_missing, profiles B/C unsupported, F03/F04 open and B01/B02/B03–B07 blocking.
