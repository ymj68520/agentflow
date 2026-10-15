# Current Rust compatibility — accepted design, adaptation required

Implementation remains exactly at baseline293066e; producer facts are not normatively canonized. [Inventory](phase-04c1b4c1-producer-inventory.md) gives every direct/inherited/prerequisite/reference producer; [draft](phase-04c1b4c1-g01-normative-contract-draft.md) defines the independent candidate.

| Current producer / proposed predicate | Compatibility classification | Why / minimum future change |
| --- | --- | --- |
| U1 existential Unproven summary ↔ per-origin unmet predicates / blockers | Requires Production Change; Missing Source Identity | Add typed causes and evaluated/blocked marker before aggregation; no postprocessor can recover all nodes/ordinals from one sentence |
| U2 existential RequiresEvidence summary ↔ per-origin evidence requirements | Requires Production Change; Missing Source Identity | Preserve actual predicate, evidence class, source and target, not a body/runtime catch-all |
| U3 Scope authority ↔ root authority-boundary fact | Equivalent meaning, Requires Production Change in representation | Keep Unproven/root aggregate; do not invent an actual Scope node; encode typed complete identity |
| U4 Scope budget ↔ root budget-boundary fact | Equivalent meaning, Requires Production Change in representation | Same, distinct budget predicate |
| U5 combined implementation/body sentence ↔ independent body/implementation plus subflow-call facts | Unrepresentable losslessly in Current Vec<String> values | Split typed subjects; keep original statuses and source links |
| U6 no-grant prohibition ↔ boundary invariants plus runtime/publisher facts | Requires Production Change; Missing Formal Semantics in current vector | Preserve prohibition as invariant; derive runtime/publisher obligations from structured NotEstablished, not from grant object names |
| Returned body/implementation/scope/runtime/publisher statuses | Exact Match of status meanings | Add complete identity and role; no evidence created. Runtime evidence companion links to same authority fact |
| Five core preflights and appended body item | Exact gate meaning; Requires explicit Role projection | Do not require the post-gate body item to be Proven for static capability |
| `unproven_dimensions` after failed preflight | Requires evaluated/blocked metadata | Retain nine visible Unproven placeholders and blockers, not nine fabricated evaluated unknown relations |
| H1/H2/H3 inherited vectors | Missing Source Identity / Requires Production Change | Add original predicate/occurrence identity before copies; union observations only for the same complete originating fact |
| `control.rs::downgrade`4282–4296 and contextual proof transformations | Requires typed application-origin | A new parent/path-conditioned predicate is not a copy of the child's original statement; preserve its dependency and distinct context, not reason-based dedup |
| P1–P10 precondition descriptions | Missing Formal Semantics / Requires Production Change | Add predicate keys, applicability/status and supporting fact links; established premises may accompany Proven |
| P11 fallback | Missing Formal Semantics | Unknown future dimension cannot gain meaning from generic text; fail UnsupportedModel |
| R1 resolver text | Requires Production Change | Structured difference and exact old/current target/dependency/export contents; English remains explanation |
| R2–R7 port/node/digest/Debug/Display references | Equivalent With Canonicalization ONLY after typed scope reconstruction is proven; otherwise Missing Source Identity | Use full typed source/target/coordinate, never string parsing as sufficient authority |
| `set_dimension_at` winner selection | Requires Production Change in internal metadata | Typed cause/winner identity replaces reason-equality reference arbitration while preserving current verdict meaning |
| RootRecovery/Control/Leaf empty private prerequisite vectors | Exact current fact, not universal semantics | Future typed prerequisites may need additional facts; do not fill old vectors or edit old Expected |
| af-contract substitution proofs | Requires future typed dependency/reference projection where applicable | Existing exact-equality/changed-direction verdicts remain untouched; no artificial RootExport obligation collection today |
| af-portable getter-only serialization | Requires Projection Adaptation | Serialize complete typed obligation/prerequisite/reference view under supported edition; no new constructor or authority |

The new deterministic primary-reference policy can turn a current None into Some typed coordinate on non-boundary facts. That is an explicit new-version report-semantic change requiring adaptation, not evidence that current None was secretly the proposed canonical value. Full typed obligation values/statuses/links remain mandatory alongside the canonical identity-string list; comparing strings alone is not a full report check.

**Outcome B compatibility finding applies alongside an accepted Option-A design.** Existing values cannot satisfy the new full identity contract by sorting, deduplication or renaming a few strings. The physical `Vec<String>` container could carry complete canonical identities after a future change, but its current *values* cannot. Parsing the existing vector is provably many-to-one: multiple distinct failed sources produce the same U1; body and implementation collapse into U5; publisher lacks a dedicated string. No reversible v1→new identity upcast exists.

Minimal future production work (NOT implemented): add typed per-predicate origin/cause/evaluation/role facts where branches know coordinates; retain typed contradictions; add typed prerequisite applicability/discharge and references; normalize copies with explicit origin links; derive canonical identity Set and string projection; add bounded complete scope getters (G02); expose the new view with complete compiler-edition support. Do not change Effect/Recovery/Control algebra or verdict priority to match tests. Keep capability constructors private and static eligibility unchanged.

The exact old six sentences/order/multiplicity and all historical Expected stay frozen for old edition. A future implementation change is a formal report-semantic migration, not "test-format only". Required Source/Reference facts cannot be dropped to preserve superficial compatibility. No P0/false-Proven is alleged from this representation incompatibility. See [compiler migration](phase-04c1b4c1-compiler-version-impact.md).
