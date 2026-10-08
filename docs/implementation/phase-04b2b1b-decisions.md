# Phase 4B.2B.1B Decisions

These decisions govern test-only fixtures and comparison; they define no wire ABI or runtime model.

1. Keep CheckedComponentContract and snapshot gates strict. Open Effect Row
   cannot enter the control tree as a checked leaf; its CaseId now tests the
   actual ContractValidation rejection, while the prior open-row algebra
   claim is tested only through `af-effect`.
2. Upgrade the computation-control corpus from revision 2 to revision 3.
   Preserve revision-2 history in commit `1633604cf0e3ae27dc0d9e2592ce466c1c3ee82e`.
   Revision 3 reclassifies `proof/open-effect-row-remains-open` from a
   ControlDerivation open-row result to an expected ContractValidation
   rejection (`resources=Unproven`, AF-CONTRACT-0013, no checked wrapper).
   The former algebra expectation is retained in a lower-level
   `af-effect` test. Revision 3 also changes the exact-constant SELECT
   adjustment list from `[2,3]` to `[]`, and removes the synthetic
   `arithmetic` field from the overflow expected in favor of the production
   Unknown contributions plus Effect-envelope Violation.
3. Build a valid full contract for every pilot. For the RequiresEvidence
   vector, the test-only materializer supplies an explicitly empty known Flow
   contract so the exported component itself can be checked; Flow/Failure/
   Usage/Resource/Execution fields outside this case's evidence assertion are
   explicitly NotApplicable. The RequiresEvidence comes only from the real
   CALL_SUBFLOW body-conformance path.
4. Project all available typed semantic data in Rust, but compare only fields
   declared by each pilot schema. No missing Actual field is silently
   accepted for an asserted path.
5. Keep the guarded-lower vector at the formal EffectAlgebraValidation stage;
   it must not be counted as a SELECT `derive_control_contract` execution.
6. The typed-merge mismatch remains classified for IRValidation but is not
   yet executed. Do not count its intended rejection as a test result.
7. 39 CaseIds remain unmaterialized and one additional expected early
   rejection remains unexecuted. Phase 4B.2B.1C is the planned full-differential
   completion stage. No Phase 4B.2B.2 work is started.

