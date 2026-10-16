# R3-F1 — 30 ordered delivery items

This report is accepted only with the successful final runtime ledger at
`target/phase-04c1b4c4r3f1-final-runtime-results.json`, the two complete original-
directory generator rounds, and the identical final nonignored tree. Unfinished
execution outcomes are not prefilled. The actual final hash is in that ledger
and the handoff; the report cannot contain its own commit hash.

1. **HEADs:** start `be04c05e07d3a72a7c795d289e689281bceaa65f`; complete production/
   conformance gate `325b1b465fc00b62b5c753788927f7c779bd0176`; evidence-only commit follows.
2. **Original IDs:** `r3-ng/primitive-bytes-0`, `r3-ng/primitive-bytes-1`, retained
   without removal/skip/reclassification. See `original-two.json`.
3. **Root cause/chain:** source base64url String was copied as canonical String;
   full contract/effect/row/entry/guard/constant/typed-encoder call chain is in
   `overview.md`. Both cases traverse the same incorrect branch.
4. **Bytes source:** canonical unpadded base64url, strict decode to raw u8 data.
   `"AQ" -> [1]`, not `[65,81]`; malformed text never means empty or String.
5. **Production diff:** only `af-effect/src/canonical.rs`; one correct Bytes
   branch plus necessary fallible private helper propagation; public API unchanged.
6. **Nominal distinction:** exact typed Bytes/String payloads and full Guard
   wire tags differ. Identical scalar spelling under different shapes does not
   erase the schema's nominal identity or require a new grammar.
7. **Binary edges:**18 low-level vectors include empty/one/multi/zero/non-UTF8/
   UTF8-as-Bytes/URL alphabet/block lengths and786431/786432/786433 raw-byte sizes.
8. **Canonical roundtrip:** full content retained; deterministic repeat encoding,
   schema-directed typed decode and identical reencode; no source-text `.as_bytes()`.
9. **Native positive/negative:** genuine contained Bytes contracts obtain Checked;
   typed countermodels/type mismatch/invalid bytes retain genuine rejection.
10. **Countermodel:** Bytes assignment tags/values, all14 witness and3 assignment
    members retain exact owner/meaning. Original two cases still yield Native V.
11. **Before/after53:** fresh pre-fix gate exits1 at51/53; post-fix unchanged
    default Oracle exits0 at53/53, and immutable frozen Expected is also compared.
12. **Special mutations:**40 F1-only laws, including equal-scalar Bytes/String,
    changed raw/assignment bytes and both-sided required-field deletion. Exact
    imported14/3 field rosters come from frozen model descriptors. Not full T3.
13. **Original51:** complete Actual values, including diagnostic reasons, remain
    equal to the frozen pre-fix capture, separately from semantic Expected equality.
14. **Registration889:** rerun in current Cargo, not relabeled as Capture cases.
15. **New cases:**62 input IDs =26 lowering +36 Native pipeline;40 separate mutation
    IDs. Six new Rust test groups; two old contradiction regressions adapted to
    require fixed behavior. Eighteen binary vectors are not added as62 new IDs.
16. **Edition1:**512 original CaseIds at exact `abc91d533b4d62e91226722e1d1fcaa30bd00883`.
17. **Edition2:**51 original CaseIds at exact `51239a0045d7a6fb274bb24cdc8130f2346bf085`.
18. **Edition3:**173 original CaseIds at exact `4d3bb59031795d31ecfccf29e3d9f2d89a7d9ce8`.
    Each uses its own original source/Registry/Expected; actual exits/pass records
    and all5,445 exact blob comparisons are retained in the runtime ledger.
19. **Capability:** all existing native analyzer/direct/private/alternate/Portable
    denial tests rerun. Default D0 limits unchanged; explicit bounded D4 denial
    remains separate. New local Native P does not activate RootExport.
20. **G01/G02/G03:** G01 incomplete; G02 OPEN/P1; G03 OPEN/P2. No repair here.
21. **F01–F04:** F01/F02 CLOSED; F03/F04 OPEN/P2. Unchanged.
22. **B01–B07:** original B01/B02 and B03–B07 remain BLOCKING. Do not confuse
    those IDs with this distinct `R3-NG-B01` production defect.
23. **Finding:** on complete successful gates only:
    `R3-NG-B01 CLOSED — PRODUCTION LOWERING FIX VERIFIED`.
24. **Freeze:**1985 original paths raw-byte equal; two reviewed exceptions.
    All54 sources/468 rules/467 covered/1 exception/0 missing,775 ABI,17 AFIDL,
    14 domains, model@3, Analysis@4, old Expected, original Oracle, D9/D10 gaps
    unchanged. Native97 imports/90 live declarations/19 snapshots remain exact.
25. **Quality:** fresh fmt/clippy/all-Cargo/spec-check actual exits in gate index
    and runtime ledger. Independent Native gate is reported separately.
26. **Generators:** actual34 scripts, original `E:/Projects/agent`, two consecutive
    complete rounds. Full raw path/hash maps must equal baseline and final tree,
    with0 additions/removals/changes; timings/counts recorded after execution.
27. **Development failures:** exact raw log bytes in `development-failures.json`:
    field-vector ordering assertion, wrong ProofReport API assumptions, test-side
    null insertion, zero-Native-P/Checked conflation, and String-new Clippy lint.
    None was hidden by changing old Expected or the original semantic Oracle.
28. **F1 Outcome A:** accepted only when final ledger confirms every repair,
    current/historical/semantic/denial/freeze/generator gate. Otherwise finding
    remains OPEN and stage is incomplete; no partial acceptance is claimed.
29. **R3 eligibility:** successful F1 permits requesting R3 resumption only.
    R3 stays incomplete, R3-NG full seed conformance pending, T4 NOT ELIGIBLE.
30. **Commits/tree/push:** classified production, conformance and evidence commits;
    actual final full HEAD and clean status are checked after the generator pair.
    No push, automatic R3 resumption, T4–T8, G02/G03 or later work is performed.
