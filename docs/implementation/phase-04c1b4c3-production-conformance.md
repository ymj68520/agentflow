# Real production conformance scope

Independent inputs/Expected:`spec/conformance/g01-edition2-staging`;authoring model in `tools/g01_registration_data.py`;direct rule dependencies separate;new generator reads normative source/AFIDL declarations and never invokes Rust or reads Actual.

| New stable case domain | Cases | Actual checks |
| --- | --- | --- |
| Registered rule descriptors |32|actual full row/anchor/statement/dependency/source registration and spec-check |
| Portable staging admission |9|real D0–D4 current-only API;none can return supported inputs |
| Native identity/header admission |10|complete native identity/error/resource/header tests;none can issue |
| Total new stable staged CaseIds |51|T1/T2 only,not full G01 semantics |
| Supplemental native direct checker suite |35 old declaration samples|Root denied or earlier component rejection;not35 new edition compatibility cases |

The separate native exact-descriptor test compares20 selected raw sources,138 rule rows,all46 complete source bindings,four complete tables,whole registries and four distinct state categories. Model-source inspection parses the registered machine block and verifies complete314 AFIDL declarations/core names;it is not a normalizer/encoder execution.

Conformance references for each of the32 new rules are real executed registration/staging responsibilities. spec-check's435 covered count is its actual reference-resolution statistic. The same rules' full derivation/canonical identity/evidence semantics are **NOT IMPLEMENTED** and cannot be claimed operationally covered;the nonactivatable staging boundary prevents a positive claim until T3–T8 close them.

Historical48 G01 design laws and16 migration laws retain Actual Status Not Executed. A related new staging mutation does not promote their original IDs to Executed. No Expected was generated from Rust Actual.

Pre-gate development failures were genuine test/lint repairs:missing public type re-export,an unreadable literal,and a test adapter initially passing the nested input instead of the complete declaration /not accounting for earlier component rejection. They were fixed before the code-gate commit;not masked by altering Expected or counted as passing runs.

Final command exits and exact code-gate HEAD are recorded in [gates](phase-04c1b4c3-gate-report.md). Fresh512-case legacy evidence is separate [here](phase-04c1b4c3-legacy-isolation.md).
