# Phase 4B.2F.2 — Applied RuleId Closure Review

## Audited content binding

current_compiler_semantics_identity embeds 16 complete normative source byte
bundles, their content hashes and specification IDs, the complete
specifications.json, the complete rules.json, the parsed rule identities,
and the analysis-semantics version (crates/af-computation/src/root_export.rs:556-700).
The validator compares the complete identity against the current build,
requires the declared applicable count, recomputes embedded source hashes, and
checks registry rows/content (root_export.rs:703-760).

The 16 pinned source documents include Component Contract and relation
catalogs; Effect, Execution, Failure, Information Flow, Recovery, Resource,
Usage; Semantic IR and core types; control, advanced-control, leaf-resolution,
scope-compensation, and RootExport proof specifications.

The embedded registries represent all 41 specification rows and all 380
normative rules. Changing any registry byte, any pinned applicable source byte,
an applied RuleId/statement, or the analysis-semantics version makes the
assertion stale. Five independently mutated production inputs all yielded
compiler-semantics preflight Unproven, overall Unproven, and no capability.
spec-check separately validates registered source hashes.

## RuleId provenance gap

The implementation declares 84 applied IDs:
AF-CONTRACT-0001..0022 and AF-SEMANTIC-0111..0172
(root_export.rs:639-644). This is not the complete rule dependency closure.
Independent registry review found the omitted AF-SEMANTIC-0001..0110 rules are
110 rules sourced only from spec/semantic/core-semantic-types.md (24) and
spec/semantic/semantic-ir.md (86). Both complete normative documents are
among the 16 pinned sources.

The RootExport checker and transitive validation paths use the 194
Contract/Semantic rules represented by AF-CONTRACT-0001..0022 plus
AF-SEMANTIC-0001..0172; their source documents are present in the pinned
bundle. Other checker dependencies—Effect, Failure, Resource, Flow, Usage,
Recovery, and Execution—are also pinned. The audit did not establish a
machine-generated direct/transitive RuleId call-graph closure, and does not
claim that 84 IDs are complete.

## Finding and release-control scope

**AUDIT-F03 remains OPEN, P2: provenance completeness.** It is not a stale
content invalidation defect: exact full source/registry identity prevents
reuse against changed normative content, and changed content fails closed.
The gap therefore does not block the current static RootExport result as a
future Release Control input only if:

1. current compiler-semantics identity remains mandatory;
2. full source and registry content checks remain enabled;
3. the consumer treats applied RuleIds as attribution, not as a complete proof
   of dependency closure; and
4. future changes to checker dependencies add/update the pinned source bundle
   and semantics version through review.

If a later consumer requires auditable per-rule derivation provenance, F03
becomes blocking until the closure is built. The 84-ID list must never be
described as complete. This phase does not modify the Rule Registry.
