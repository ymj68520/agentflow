# Phase 4C.1B.4C.1 — G01 obligation formal-contract design

Start/full-code regression HEAD: `293066e865e25dad695cd2d774f558b70eac43f3`, initially clean. Authorized changes are design documents and an **unregistered** normative draft only. Production, registered normative sources/registries, ABI/AFIDL/domains and every existing Expected stay unchanged.

Recommended model: **Option A — complete typed obligation identity with mandatory lossless canonical string projection**. Collection = Set of full identities, canonically ordered for encoding. Independent source occurrences remain separate; inherited copies share origin identity with observation links. Prerequisites/references remain required semantic facts. No Actual→Expected path or string demotion is used.

Read in this order:

1. [Producer inventory](phase-04c1b4c1-producer-inventory.md): six direct string writers, three inherited-obligation constructors, eleven prerequisite arms/twenty literal occurrences, all reference forms and source-factory census.
2. [Gap analysis](phase-04c1b4c1-normative-gap-analysis.md) and [alternatives](phase-04c1b4c1-model-alternatives.md).
3. [Typed identity](phase-04c1b4c1-typed-obligation-identity.md), [encoding/order](phase-04c1b4c1-encoding-and-ordering.md), [aggregation](phase-04c1b4c1-aggregation-and-multiplicity.md), [prerequisite/reference semantics](phase-04c1b4c1-prerequisite-reference-semantics.md).
4. [Proof consistency](phase-04c1b4c1-proof-consistency.md), [Rust compatibility](phase-04c1b4c1-existing-rust-compatibility.md), [compiler impact](phase-04c1b4c1-compiler-version-impact.md).
5. [Independent design vectors](phase-04c1b4c1-decision-vectors.md), [normative draft](phase-04c1b4c1-g01-normative-contract-draft.md), [decisions](phase-04c1b4c1-decisions.md).
6. [Readiness](phase-04c1b4c1-readiness.md), [gate report](phase-04c1b4c1-gate-report.md), [38-item delivery](phase-04c1b4c1-final-report.md).

The model is intended to reach **Outcome A — G01 Formal Contract Design Accepted**, with explicit **Outcome B current-behavior incompatibility/adaptation required**. This does not register a specification, implement a codec/Checker/Oracle, close G01 in production, repair G02/G03, complete the26-case D10 proof, or enter4D. Actual final gates and accepted design evidence are recorded in the gate/delivery reports.

Current baseline remains: historical512 CaseIds; D9 Control22/Recovery16 specialized38 closed; current D10 group26 with24 independent D9 comparisons/two P31 Oracle gaps, D10 full-field0. G02 OPEN/P1, G03 OPEN/P2, F01/F02 CLOSED, F03/F04 OPEN/P2; B01/B02 and B03–B07 conformance/publication dependencies BLOCKING.
