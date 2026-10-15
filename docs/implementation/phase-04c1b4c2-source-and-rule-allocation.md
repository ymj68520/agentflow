# Proposed normative placement and rule allocation

**PROPOSED — NOT REGISTERED.** Existing registry shapes are unchanged.

## Actual registration mechanism inspected

`spec/registry/specifications.json` contains43 entries. `rules.json` contains404 descriptors across14 existing areas (SEMANTIC196, CONTRACT22); the SEMANTIC high-water mark is0196, not the total404. All descriptors are parsed by the existing checker; source integrity, duplicate IDs/anchors, canonical form and conformance resolution are validated at the gate. Inspection includes the two complete JSON instances, `registry-spec.md`, `specification-registry.afidl`, `rule-areas.json`, AFIDL1 SpecificationId grammar, `af-spec/src/{check,fs,model,abi,tables}.rs`, `af-portable/build.rs` and the current30 authoring generators.

SpecificationId grammar: lowercase namespace segments `[a-z][a-z0-9-]*` separated by dots, then positive semantic major `@N`. Rule grammar `AF-<registered AREA>-<0001..9999>`. IDs are permanent/non-reusable; no section/line-based IDs. Existing SEMANTIC area is sufficient; no new area or registry-schema kind. Each independently testable invariant gets its own rule.

Source refs require a real repo-relative forward-slash path, full plain SHA256 of actual file bytes, and an existing unique anchor. ArtifactFileDigest is tooling identity, NOT HashV1. Canonical registry JSON = sorted keys,2-space indent,LF,final newline, arrays by declared registry keys. Real conformance refs must already resolve; design D/M identifiers below are not registered CaseIds.

`tools/gen-registry-instances.py` authors registry/coverage/suite instances, not an authority for changing normative meanings. The other29 generators materialize frozen corpora/oracles. Future changes must update the generator's explicit descriptors as well as instances, then run spec-check and twice regenerate. `af-portable/build.rs` embeds ALL registered source bytes and the frozen set inventory; registering a source automatically changes compiled content support. Hard-coded43/16/84 assertions in support/root-export cannot silently be left behind.

## Three candidate entries

| Alias / proposed SpecificationId | Proposed logical path / kind | Ownership |
| --- | --- | --- |
| OS: agentflow.computation.proof-obligation.semantics@1 | spec/computation/proof-obligation-semantics.md / schema_language | Fact authority, derivation, prerequisite/reference/role and capability relation |
| RM: agentflow.computation.proof-report.model@1 | spec/tables/proof-report-model-v1.md / closed_table | Closed descriptor grammar, all semantic record fields/variants/ranks, encoding and order |
| PP2: agentflow.portable.proof.inputs.semantics@2 | spec/portable/portable-proof-inputs-v2.md / schema_language | Successor exact-profile admission and activation/isolation policy |

RM is a closed normative table, not `kind=schema`: the latter means an AFIDL module whose assignments must match ABI. OS defines the type relation and RM materializes its closed descriptors. Reject a giant single rule/source: identity algebra, representation and edition admission have different violation/compatibility boundaries.

Anchor feasibility correction: `af-spec/src/check.rs616–622` treats every non-AFIDL source as Markdown; a bare JSON file cannot host the eight RM rule anchors under the current mechanism. Therefore RM's proposed source is Markdown with explicit unique `<a id="model-..."></a>` anchors and exactly one normative fenced JSON descriptor table, preceded by heading `## Closed Descriptor Table`. The future model loader extracts that single designated block, rejects zero/multiple blocks and validates the closed descriptor grammar; the complete Markdown source bytes bind identity. This uses the existing closed_table kind/path/Markdown-anchor rules, requires no spec-check change, no new registered JSON source and no escaped-anchor trick inside JSON strings. The discarded .json placement is NOT the proposed path.

PP2 preserves PP1's source and identity. Its explicit precedence clause supersedes only old current-profile policies0188/0190 for owner@2, not ACE1, transport fields, existing stable rule meanings or old-build support. Old0173–0187,0189,0191–0196 remain applicable reused constraints. PP2 must contain or explicitly incorporate those anchors with the exact PP1 source pin; it must not edit PP1 to pretend that old "only Profile A" included@2.

IDs and paths have zero collisions with present registry and files. Recheck immediately before future allocation; this document does not reserve numbers against another branch.

Proposed entry status at the final approved T1 snapshot is frozen, only after actual normative sources/descriptors exist (never pre-register unimplemented source placeholders). It is staged/non-released, not the active current profile. No draft->frozen registry-byte flip is deferred until after T7: freeze candidate bytes first, build/test that exact tuple, then T8 releases the same tested artifact. Any T1 text/descriptor/registry refinement invalidates T2–T7 candidate identities/evidence and requires rebuilding/retesting the affected transaction, not relabeling its Expected.

## Proposed descriptors

Each row is a complete allocation intent: prefix AF-SEMANTIC-, requirement_level `must`, OS/RM/PP2 identify the normative_source above. The anchor is exact and unique within that source. Dependencies are direct applicable premises, **not a complete transitive rule closure**. Conformance column names design laws to instantiate as new independent cases; positive and rejecting sides are mandatory. Migration: OS changes report/capability semantics; RM changes report bytes only, never existing semantic digests; PP2 changes admission/test routing.

| Rule | Source / exact anchor | Formal short statement | Direct dependencies | Required conformance |
| --- | --- | --- | --- | --- |
| 0197 | OS / g01-fact-authority | Only a complete validator-origin fact roster may derive a full obligation view. | CONTRACT0017;SEMANTIC0142 | D34,D40,M09 |
| 0198 | OS / g01-exact-scope | Scope retains complete compiler/context/resolver/root/assertion values and context-role partition. | 0160,0161,0182,0187 | D31,D39,D43,M06,M07 |
| 0199 | OS / g01-closed-predicate | Kind, clause, class, role and applicability must match the closed predicate roster. | 0197 | D29,D46,M15 |
| 0200 | OS / g01-subject-origin | Source, target, coordinate and declared occurrence origin must be complete and consistent. | 0141,0159 | D01–08,D16,D47 |
| 0201 | OS / g01-derive-nonproven | Each evaluated non-Proven fact yields exactly its disposition and typed obligation. | 0197–0200 | D17,D18,D21–23 |
| 0202 | OS / g01-blocked-not-evaluated | Failed core preflights block nine dimensions without inventing evaluations. | 0159–0161,0171;0201 | D19,D20,D32 |
| 0203 | OS / g01-five-plus-nine | Static proof eligibility requires five core checks and nine unique evaluated dimensions Proven. | 0159–0170 | D17–24,D28 |
| 0204 | OS / g01-independent-boundaries | Six root boundaries and each call-body requirement remain independent of static success. | 0171,0172,0148 | D24–28,D48,M11 |
| 0205 | OS / g01-child-execution-aspects | Each known child retains distinct feasibility/security and source-coverage predicates. | 0170 | D23,M13 |
| 0206 | OS / g01-prerequisite-identity | Prerequisites retain original predicate/stage/subject/origin and all consumer edges. | CONTRACT0001–0019;0199,0200 | D33,D35,D36,D46 |
| 0207 | OS / g01-prerequisite-consistency | A Proven predicate cannot have an unmet mandatory prerequisite or missing support. | CONTRACT0017;0206 | D34,D36 |
| 0208 | OS / g01-evidence-coordinate | References are complete typed evidence coordinates, not RuleIds or prose identities. | 0200 | D37,D39 |
| 0209 | OS / g01-primary-reference | Derived complete reference sets use the typed minimum except six root boundary primaries None. | 0208 | D41,M07 |
| 0210 | OS / g01-origin-copy-merge | Only equal originating facts merge observations; distinct origins stay distinct. | 0200,0201 | D09–11,D14,D42,M12 |
| 0211 | OS / g01-status-preservation | Status, disposition, links, contradiction and observations remain required alongside identity. | 0201,0206,0208 | D14,D18,D48 |
| 0212 | OS / g01-private-capability-gate | The private successor constructor requires proof eligibility and successful complete bounded normalization. | 0202–0207;0217–0220 | D24,D28,M08,M09,M11 |
| 0213 | OS / g01-no-report-authority | Text, ordering, emptiness or successful serialization cannot create proof or runtime authority. | 0172;0212 | D26,D40,D48 |
| 0214 | RM / model-closed-descriptors | Every semantic model type has exact closed field, width, option and variant descriptors. | ACE1-0001;0198–0200 | D29,D30,D44 |
| 0215 | RM / model-typed-order | Sets and identity views use the registered typed total order, preserving semantic sequences. | 0214 | D12,D13,D38,D45 |
| 0216 | RM / model-identity-ace1 | Canonical identity strings are schema-directed ACE1 of the complete identity and decode bijectively. | ACE1 rules;0214,0215 | D01–08,D30,D37–39,D44 |
| 0217 | RM / model-minimal-value-dag | Full scope is a finite minimal reachable typed value DAG with canonical bottom-up ranks. | 0181;0214 | D13,D40,M14 |
| 0218 | RM / model-no-partial-output | Output/work/graph limits fail explicitly without truncated successful reports. | 0193;0217 | D40,M08 |
| 0219 | RM / model-closed-decode | Unknown, missing, duplicate or noncanonical report fields/variants/identities are rejected without repair. | 0214–0217 | D15,D29,D30,D38,D41,D44 |
| 0220 | RM / model-independent-fullfield | Independent Expected compares every required typed field and canonical byte, not strings alone. | 0194;0211,0216–0219 | D01–48;M01–16 |
| 0221 | RM / model-shape-version | Model identity/rank/derivation changes require new shape/obligation/content edition. | 0214–0219 | D45,M15,M16 |
| 0222 | RM / model-diagnostic-separation | Reason is diagnostic only after all equality/decision-bearing content is structurally retained. | 0211,0214 | D39,M05–07 |
| 0223 | PP2 / edition-exact-tuple | The supported owner binds complete sources, registries, attribution, model and support policy. | 0186,0187;0221 | D31,M02–04 |
| 0224 | PP2 / edition-current-only | Owner@2 admits only its implementation-owned exact current edition without implicit old support. | 0191;0223 | M01,M10,M16 |
| 0225 | PP2 / edition-failure-taxonomy | Admission failures retain canonical/content/identity/support/resource categories, not static Violation. | 0176,0193;0224 | M01–04,M06–09,M15,M16 |
| 0226 | PP2 / edition-corpus-isolation | Legacy code/inputs/Expected and new-edition independent evidence are immutable separate domains. | 0194,0195;0224 | M01,M10,M11 |
| 0227 | PP2 / edition-atomic-activation | Registry, support, producers, lossless views and independent conformance activate only as one transaction. | 0212,0220,0223–0226 | M01–16 plus T7/T8 |
| 0228 | PP2 / edition-attribution-boundary | Applied-rule attribution is explicit and does not claim complete dependency closure. | 0189;CONTRACT0017 | D31,M04 and F03 negative claim |

Abbreviated rule numbers in dependencies mean AF-SEMANTIC unless explicitly CONTRACT/ACE1. "ACE1 rules" means the14 existing ACE1 descriptors, not an invented umbrella rule. Future exact descriptor rows must expand these references; no Rule dependency field is added to existing NormativeRuleDescriptor (the plan's dependency table is semantic documentation).

Conditional future registry delta: +3 specifications (46),+32 rules (436),+32 covered rules (435 covered,1 intentional exception,0 missing) **only after executable case refs exist**; ABI+0 (775),AFIDL+0 (17),HashV1+0 (14),rule areas+0. Current actual delta is zero. Never register these with fake design-case coverage or call the conditional count a present result.
