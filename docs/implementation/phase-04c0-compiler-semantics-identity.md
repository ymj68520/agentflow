# Phase 4C.0 — Compiler / Semantic Content Identity

All proposed portable identities and compatibility mechanisms are
**DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED ABI / NOT IMPLEMENTED**.
Decision C04 preserves F03/F04 and fixes future consumer requirements.

## Existing facts

The current identity (`crates/af-computation/src/root_export.rs:76,679`)
contains `agentflow.root-export-checker@1`, full Specification/Rule Registry
bytes, 16 full applicable normative source bundles/checksums and 84 applied
RuleId records. The checker compares the whole current identity and validates
embedded source/registry contents (`703`).

This binds semantic inputs, not the compiler executable. A version string
alone is insufficient. Changing checker semantics must bump its semantics
version; an authenticated build proves a particular binary was produced, not
that its implementation is correct.

## Publication identity roles

| Role | Exact proposed content | Trust/proof boundary |
| --- | --- | --- |
| Semantic source identity | Full normative source bytes, registries, schema/table/corpus release identities | Verifies which semantic rules/content are intended |
| Static analysis semantics | Exact checker family/version and applicable source roles | Defines Root derivation/comparison meaning |
| Plan lowering semantics | Exact lowering family/version, Plan schema and preservation rules | Defines executable behavior produced from IR |
| Runtime semantics profile | Exact formal supported profile content and version | Declares interpreter/handler obligations; no runtime admission |
| Per-rule attribution | Complete justified direct/transitive RuleId graph when required | Explains proof dependency; cannot be inferred from ID count |
| Compiler build identity | Exact binary/source/build recipe and implementation dependencies | Qualified build/provenance claim, separate from source semantic tuple |

The static, lowering and runtime content roles must be bound in the future
parent without conflating their version lines. Compiler implementation,
normative source, schema representation and corpus releases can change
independently only through explicit validation rules.

## Cross-compiler verification

| Change | Permitted future handling |
| --- | --- |
| New compiler binary, exactly same supported semantic content | Fresh option-A verification by a qualified build; retain its build provenance separately; no reuse of old checked wrapper |
| Normative/source/registry/checker semantics changed | Old assertion is stale for this verifier; reconstruct and reprove under the intended explicit semantics or reject |
| Lowering semantics changed while RootExport semantics equal | RootExport may still be Proven; Plan preservation and parent build identities must be reconsidered |
| Old immutable WorkflowVersion under archived semantics | Retain its identity; an approved historical verifier may revalidate under the exact archived tuple if current policy permits |
| New semantic release intended for old source program | Build new assertion/Plan/proof bundle; a changed semantic core gets a new version/content identity |
| Unknown semantic version or missing historical source | UnsupportedSemantics / MissingArtifact; no fallback to current defaults |

An explicit historical/new compatibility relation may permit decoding or
migration, but does not by itself reuse the old proof. No such production
compatibility/upcaster relation is introduced here. Current production
RootExport compares against the build's current exact identity; it does not
support arbitrary historical semantics through a supplied version label.

An editorial byte change also changes the current exact content identity.
Conservative revalidation is required unless a separately specified
compatibility model proves an allowed representation relation. The old artifact
is never overwritten.

## F03 ownership and hard trigger

AUDIT-F03 remains **OPEN / P2**. The 84 applied RuleIds are not a complete
transitive closure. Publication's baseline full-revalidation profile must
require exact complete semantic source/registry content and mark rule
attribution incomplete.

Complete Rule Dependency Closure becomes a **mandatory blocking deliverable
before** any of:

1. a certificate verifier whose rule steps discharge positive proof;
2. option-C cached/selectively rederived positive proof reuse;
3. a release policy/profile claiming or requiring complete per-rule provenance.

The proposed next context/compiler-model stage must specify whether its
release profile requires that closure. The selected basic full-revalidation
profile does not make it mandatory merely to use current static compatibility;
it must preserve the content binding and all downstream gates. Enabling a
rule-complete profile without closure is a blocking policy violation, not a
waived P2.

## F04 ownership

AUDIT-F04 remains **OPEN / P2**. The current root-global SELECT/RACE/HANDLE
guard diagnostic suppression affects only Violation precision, and unresolved
cases remain Unproven. A portable diagnostic may expose the exact occurrence,
guard operands, control-path families and missing implication obligation.
Those are witnesses to revalidate, not a new positive predicate.

Publication cannot reinterpret Unproven as “probably valid” or use an
unsupported selector Boolean solver to discharge it. A future path-sensitive
improvement needs formal implication/reachability rules and independent
soundness probes. No new false-Proven basis was found in this design phase;
the historical classifications are unchanged.

## Release consumer requirements

Consumers select authoritative supported semantic releases and qualified
checker builds; a publisher cannot supply its own root of trust. They check
full content availability and integrity, semantic support, required attribution
profile, and current build/evidence trust. Checker revalidation and attestation
verification remain separately recorded. The missing portable full
compiler/lowering/profile model remains BLOCKING for production Publish.
