# Findings and limits

No activation blocker remains for registration-only staging: registry admission and positive capability are safely separable. R2 does not claim full G01 soundness or full independent oracle closure.

- R2-01 RESOLVED: inherited support counts/rosters must not be mistaken for the active successor. Historical model remains intact inside inherited_registered_model; active canonical constraints explicitly append family/kind/dimension/clause/context roles and bind50/24/149.
- R2-02 RESOLVED: materialized native nominal IDs retain u32; logical scalar projections, fixed arrays and AppliedMultiplicity alias were audited against actual source, not guessed from similarly named R1 placeholders.
- R2-03 BOUNDARY: metadata shape probes validate shape, not Fact provenance, alias truth, Scope completeness, canonical value DAG, derivation or witness validity. Their unit success grants nothing.
- R2-04 BOUNDARY: reference coverage and source-derived schema Expected are not an independent full semantic oracle. R1 semantic48 stay NotExecuted.
- R2-05 RESOLVED: old Edition2 generator cannot consult enlarged current registry or rewrite old goldens; exact input pins and byte-verification preserve its original output identity.

Development diagnostics retained under ignored target: clippy findings (single-character binding/nonminimal boolean/semicolon) were corrected; extra EOF blank lines in new sources were fixed before final gates. The first full current run overlapped the source-constraint clarification: its compiled e919c57 identity no longer matched regenerated b85bf2a Portable inputs, so complete-successor-denied correctly returned InvalidIdentity instead of its intended same-edition UnsupportedSemantics, and cargo exited101. The complete failed log is retained as phase-04c1b4c4r2-current-tests-overlap-failed.log. This is not a final gate pass; the entire gate is repeated at an unchanged explicit code HEAD. No original-directory generator failure is hidden or spliced. Any observed gate failure remains in its own log.

F01/F02 CLOSED; F03/F04 OPEN/P2. G02 OPEN/P1; G03 OPEN/P2. B01/B02 full-conformance closure and B03–B07 remain BLOCKING. No unrelated comparator precision amendment is made.
