# T3-REG-01 — formal binding for standalone substitution facts

Status: **PROPOSED REQUEST ONLY; NOT REGISTERED, NOT IMPLEMENTED.** This document allocates no SpecificationId, RuleId, predicate case, shape or obligation edition.

## Reproducible production subject

`crates/af-contract/src/compatibility.rs::check_substitution` accepts two checked ComponentContracts and compares exact context object identity. Its `identity` dimension applies `replacement.component_identity == old.component_identity`, governed by AF-CONTRACT-0020. This compares component lineage (catalog/name), not IR/export pins. The function requires no caller Semantic IR, computation node, export assertion, or RootExport proof.

Fresh diagnostic: old component `agentflow.components@1/acme.transform`; replacement either identical or `agentflow.components@1/acme.other`; all other declarations are identical empty valid declarations, checked under the same actual context object. Existing production checker returns `identity=Proven` for the identical control and `identity=Unproven` for the renamed replacement. There is no Typed Fact Capture in this probe. No verdict is inferred from a reason or relabelled as a G01 fact.

## Registered constraints

Authoritative RM `spec/tables/proof-report-model-v1.md` has SHA256 `e23ca47c05efe9f7ec22177011982ac47ca0cdc8642173de56a90e6e29a93099`. Its `semantic_constraints` defines exactly eight PredicateFamily cases and the exact `closed_predicates` rosters. None declares a standalone substitution/lineage-comparison case or a binding of that subject to an existing case. The contract-validation Scope does allow `old_checked`, but a Scope variant does not itself register a PredicateId or Requirement binding.

AF-SEMANTIC-0199 requires kind/clause/class/role/applicability to match the closed predicate roster. AF-SEMANTIC-0214 requires closed model descriptors. AF-SEMANTIC-0221 requires edition review when model identity/ranks/derivation change. The substantive AF-CONTRACT-0020 rule does not silently add a new G01 predicate family/case to that roster.

The source has all original data. The blocker is **not** G02's pending cross-process Scope projection and **not** a need for T4 canonical encoding. An unscoped seed still needs the formally supported local predicate/role/requirement. A Rust-only case would assign unregistered semantic meaning.

## Why apparent shortcuts are not accepted

- `root_preflight/exact_pin` checks assertion IR/export/root pin association, not catalog/name lineage of two standalone contracts.
- `control/root-export-compatibility` is the existing control-node/root export boundary predicate, not arbitrary contract substitution. A report carrier or dimension string cannot change that meaning.
- `leaf/type-compatibility` concerns a checked call site's mapped I/O types. No leaf exists in this legal standalone input.
- A `prerequisite` case such as `required_dimensions` does not name lineage equality; its successful presence cannot stand for an unrelated comparison.
- Borrowed Scope, a full raw input, a diagnostic string, or an audit ProducerSiteId does not repair the missing predicate binding.

This conclusion is a semantic source review, not an inference merely from the absence of a spelling. The mechanical audit reports spellings and exact model bytes for reproducibility, not a proof of semantic equivalence or its impossibility under future amendments.

## Requested decision before implementation resumes

Approve a normative design/registration amendment that explicitly binds the standalone substitution producers, including `identity`, context association, the twelve dimension rows and their concrete comparisons. Specify family/cases, Kind/Requirement, role, both compared subjects, Source/Target/Coordinate, declaration/application origin, evaluation/status, dependencies, references and typed contradiction duties. Review the impact on model/report/obligation/analysis editions and full content identities under AF-SEMANTIC-0221; do not mutate a frozen source in place without that decision.

Alternatively, explicitly amend T3's authorized inventory to exclude those producers, with a documented remaining blocker. That changes the requested coverage and cannot be inferred by this implementation agent. If a previously registered exact binding is intended, identify its source/anchor and complete mapping for review instead; a new source is not required merely because this request exists.

No choice is preselected, no edition number is invented, and no registration is changed. Approval of T3 alone is not approval of this separate normative change. Until resolved: capture paused, no trusted empty collection, no Canonical ObligationIdentity, no positive Checked, no T4/T8 entry.

## Diagnostic evidence

Retained ignored probe: `target/phase-04c1b4c4-probe/`; execution:

```powershell
cargo run --offline --manifest-path target/phase-04c1b4c4-probe/Cargo.toml --target-dir target --quiet
```

Fresh successful result in `target/phase-04c1b4c4-substitution-probe-final.log`: same checked context=true; control identity=proven; renamed identity=unproven; RuleId=AF-CONTRACT-0020; root export supplied=false; capture executed=false; exit0. This is an admission diagnostic, not one of the proposed72 T3 cases. An initial probe compilation failed because two test-fixture dependencies were omitted; that exit101 log is retained separately, then the isolated probe manifest was corrected with sha2/hex and rerun successfully. No production dependency or Cargo.lock changed.
