# D10 proof authority, missing identity contract and coverage boundary

## ORACLE-D10-G01 — Required exact obligation values are underdetermined

Severity: P1, **formal conformance model**, not a proven production soundness defect.

The task requires independently computing every `RootExportProofReport.unresolved_obligations` value, with no Actual-to-Expected path. The frozen executable contract in `tools/oracle/field_contract.py` declares `RootExportReport.unresolved_obligations` as `arr(S)` with **exact** ordering. It is RequiredSemantic, not the DiagnosticOnly `reason` field. The Rust type is `Vec<String>`; that declaration establishes representation, not normative token identity.

The normative RootExport specification supplies substantive obligations:

- `root-export-contract-proof.md#subflow-body-separation`: exact target/export closure and body evidence separate from a target static export proof.
- `#static-runtime-boundary`: implementation, publisher and runtime claims are not established; Scope authority/budget ceilings are absent.
- `component-contract.md#component-contract-cross-dimension` and `#component-contract-proof-composition`: retain verdicts, exact references, reasons and prerequisites; do not erase non-positive verdicts.

These rules do **not** assign exact nominal tokens, an encoding for each obligation, conditions for flattening the structured report into this particular string vector, ordering, duplicate handling, or whether separate boundary fields also occur in the vector. A complete structured semantic proof can preserve every substantive obligation while using different exact spellings/order. For example, `implementation:requires_evidence` and `implementation_conformance:requires_evidence` illustrate two encodings of the same premise. Neither string is an Expected value or a claimed Rust output. Frozen normative rules select neither. A mere field-type declaration does not settle this choice.

The same concern must be reviewed for nonempty `DimensionProof.prerequisites` and the report's textual `reference` vocabulary. This audit does not infer those from checker branches or assert new gaps for all unvisited instances: the single mandatory unresolved-obligation array is already sufficient to prevent the requested exact full-report comparison. Empty lists are not chosen as an unsupported default.

The previous D9 subset had a reviewed, explicitly documented coordinate vocabulary and no additional prerequisite-string instances. That closed D9 profile does not prescribe D10's obligation vector. Its success cannot supply these missing values.

### Required resolution

A separately authorized formal conformance decision must specify stable obligation identities, mapping/aggregation rules and order/multiplicity, or a typed semantic comparison that preserves the same identities without relying on human text. This stage cannot modify frozen normative specifications or production semantics to make a newly chosen encoding authoritative. It also cannot silently demote required obligations to diagnostics, ignore them, sort away multiplicity or copy production strings.

`require_full_d10_expected()` therefore raises `OracleGap(ORACLE-D10-G01)` for every original input. The gap test protects this behavior. This is Outcome C, not an accepted incomplete positive proof. No D10 static verdict or capability value is filled from legacy Expected or observed Actual.

## ORACLE-D10-G02 — Legacy projection is not complete identity

Severity: P1, **projection/coverage**. Repairable with permitted lossless test-only getter work, so not by itself a reason to claim formal semantics are missing.

The real Rust `CompilerSemanticsIdentity` retains full registry bytes, rule-registry bytes,16 normative specification byte strings and84 rule records. However `root_export_assertion_json()` exports registry and normative `content_bytes` as lengths plus checksums. The corresponding Python schema uses `u64`. Checksum/length equality is not the requested byte-level comparison.

The real `ProofContextManifest` retains compiled modules, schema universe, effects/resources/flow/usage/recovery/execution universes, relation bytes and resolver dependencies. The current `ContextManifest` projection has module/schema/resource/flow/usage/recovery counts, omits execution universes, and emits relation content lengths/checksums. Counts cannot prove equality of descriptors with equal cardinality. Comparing D5 context separately does not establish that the actual **assertion manifest** contains the same complete typed content: the latter needs its own lossless read-only projection/association check.

No hash collision or production error is alleged. The issue is failure to observe all required content. A source-field-to-count binding must not be credited as full semantic content coverage. Production types remain unchanged; no current937-path inventory total is used to conceal the absent payload observations. Existing D9 coverage claims remain scoped as before; D10 content remains Pending.

Nested assertion identity must also eventually have finite acyclic expansion, bounded depth/work/mapping counts and explicit infrastructure rejection rather than a `projection_depth_limit` placeholder accepted as complete proof. No new recursion policy is introduced by this gap audit.

## ORACLE-D10-G03 — Two D9 implementation gaps

Severity: P2, **independent Oracle implementation**. Both original P31 cases have a nonempty CALL_SUBFLOW interface. Existing `Model` rejects that subset with `OracleGap("nonempty subflow interface needs independent Phase 3D interface expansion")`. The normative subflow/interface rules exist; this is not Outcome C's formal gap. The rejection and affected CaseIds remain in the additive inventory. No empty interface is substituted and no production Root is treated as Expected.

## Retained boundaries and incomplete targets

Assertion identities, compiler/context content, resolver closure, seven mapping families, preflight, nine dimension proofs/source facts/witnesses/countermodels, execution feasibility/security, obligations, overall verdict and capability issuance have **not** received independent D10 full-field closure here. Original38 D9 Expected remain byte-frozen, and their D10 values still await4D. Provenance8 and Cross-Process6 are not pulled into this subset.

New D10 comparator mutations:0/40 target. New real production-input mutation families:0/18 target (the20 historical registered input variants are not new mutation families). New semantic hand calculations:0/27 target. Original151 comparator mutations and74 hand calculations remain intact and are exercised only as regression. Seven Python integrity tests do not substitute for these acceptance targets.

P30 same-row/split-row and both P31 scenarios are retained as existing production regressions. This audit does not claim their full D10 fields have now been independently proven. F01/F02 stay CLOSED, F03/F04 stay OPEN/P2;84 applied rules is not transitive dependency closure, and no guard countermodel precision or global-suppression behavior is changed.

B01/B02 remain BLOCKING. Applicable full-field work for the original78-case corpus remains unclosed, not merely the52 CaseIds outside this subset: the current26 D10 proofs also remain open. This does not assert that every one of the78 reachesD10; earlier rejections among the other14 must receive their real stage/error comparison, never an invented D10 report. There is no4D admission. The complete current26 CaseId/pin/mapping inventory is `expected-full/d10-preconditions/manifest.json`; machine-readable findings are `coverage/d10-gap-ledger.json`.
