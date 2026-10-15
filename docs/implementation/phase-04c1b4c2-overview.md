# Phase 4C.1B.4C.2 — registration and migration plan

Status: PLANNING ONLY. All candidate rules below are **PROPOSED — NOT REGISTERED**. All proposed code entry points are **NOT IMPLEMENTED**. Nothing here activates a supported compiler profile.

Baseline: `8b4e3c64ee1d84fc38b6faa9b0b1257deb18873d`; initial worktree clean. The accepted [D01–D16 design](phase-04c1b4c1-g01-normative-contract-draft.md) and Option A are inputs, not newly registered authority. Old documents, findings, registries, production code, generators and Expected are frozen. Planning does not close G01/G02/G03 or B01/B02.

## Decisions made here

1. Three separate proposed sources: obligation semantics, a closed non-wire report-model descriptor table, and Portable semantics edition2. Allocate32 candidate SEMANTIC rules0197–0228, not draft clause numbers.
2. Report-local semantic model, no new AFIDL module, ABI assignment or HashV1 domain. Stable bytes require approved model descriptors, ordering and edition, not merely an ACE1 encoder.
3. **Normalization is an explicit additional capability-creation prerequisite in the successor.** This is an availability restriction, not a restatement of old five-plus-nine sufficiency. Proof verdicts remain independent; body is not a sixth mandatory preflight.
4. Proposed owner `agentflow.root-export-checker@2`, obligation edition `agentflow.g01.obligation@1`, report shape `agentflow.g01.report-shape@1`. Exact content, not these labels alone, determines support.
5. Current-only admission; old positives are unsupported by the new build, not automatically upgraded. Preserve an executable old-code test environment and all512 historical inputs/Expected.
6. T1–T7 are implementation-branch checkpoints; only T8 atomically activates a new current profile after all required adaptation/oracles. G02 precedes G01 full-field conformance; G03 remains independently blocking.

## Document map

| Topic | Plan |
| --- | --- |
| Normative allocation and actual registry mechanics | [sources/rules](phase-04c1b4c2-source-and-rule-allocation.md) |
| Testable clause review and amendments | [D01–D16](phase-04c1b4c2-normative-clause-mapping.md) |
| Closed types, context closure, ranks | [schema](phase-04c1b4c2-typed-schema-materialization.md) |
| ACE1, DAG and resource/size feasibility | [encoding](phase-04c1b4c2-ace1-registration-review.md) |
| Private constructor and error contract | [capability](phase-04c1b4c2-capability-gate-decision.md) |
| Every producer and P/R/ROLE adaptation | [producer map](phase-04c1b4c2-producer-adaptation-map.md) |
| Exact new profile tuple | [edition](phase-04c1b4c2-compiler-edition.md) |
| Current-only error/admission table | [migration](phase-04c1b4c2-legacy-version-migration.md) |
| Old512 / new evidence isolation | [regression](phase-04c1b4c2-regression-isolation.md) |
| Lossless getter coverage and G03 | [G02/G03](phase-04c1b4c2-g02-dependencies.md) |
| T0–T8 and future authorization | [transaction](phase-04c1b4c2-activation-transaction.md) |
| 48+16 design-vector execution contracts | [vectors](phase-04c1b4c2-conformance-vectors.md) |
| REG-01–12 | [decisions](phase-04c1b4c2-decisions.md) |
| Approval and implementation gates | [readiness](phase-04c1b4c2-readiness.md) |
| Fresh validation evidence | [gate report](phase-04c1b4c2-gate-report.md) |
| Ordered35-item handoff | [final report](phase-04c1b4c2-final-report.md) |

This plan authorizes no implementation. A separate human approval must name the atomic transaction, candidate allocations and availability change. No push, runtime, publication, workflow version, LockGraph or PlanCore work is part of it.
