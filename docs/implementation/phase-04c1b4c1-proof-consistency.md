# Structured proof consistency and trust boundaries

The full relation and errors are in [draft D05–D14](phase-04c1b4c1-g01-normative-contract-draft.md). This design proposes a view, not a new proof solver or evidence authority.

```text
complete checked declarations + independent frozen relations
    -> structured predicate facts / typed prerequisites / references
    -> obligation identity Set + blocker/contradiction indexes
    -> canonical sequence/string projection + separate human reasons
```

Completeness laws apply to facts, not current sentences. Every required evaluated non-Proven predicate contributes an exact obligation; every emitted obligation must point back to that predicate. Unknown source/request/evidence cannot disappear into `[]`. An aggregate with explicitly root-scoped meaning may remain root-scoped; a required per-source analysis whose causes were discarded fails IncompleteFacts rather than guessing.

Seven required derivation decisions:

1. Proven evaluated static dimensions produce no unresolved obligation of their own; established prerequisites remain in the fact index.
2. Unproven records missing static proof; RequiresEvidence records a distinct evidence-dependent disposition and normative requirement class. Neither is promoted; neither is just a new synonym string.
3. Violation retains a remediation obligation AND a typed contradiction. It is not recast as missing evidence, and other dimensions remain available. A failed containment call without a typed counterexample remains Unproven under frozen rules.
4. Body and implementation are independent predicates. Root and each applicable subflow call-site body requirements are source-scoped; two observations of the same call-site fact share identity, different calls do not.
5. Failed core preflight causes9 explicit blocked-dimension records. It does not establish9 independently evaluated unknown/negative relations. Failed core facts themselves remain obligations.
6. Runtime/publisher/Scope/body/implementation boundaries remain listed even when static compatibility fails or is Proven. No-string is not an all-proofs-Proven certificate.
7. Same originating fact seen in several report fields shares identity and observation links; different source/ordinal/rule/coordinate/requirement is a different identity. Conflicting duplicates reject.

The role roster recognizes exactly five root mandatory preflight statements and exactly nine static dimensions. The post-gate subflow body item carried in current `preflight` is an independent boundary, not a sixth mandatory-static precondition. The proposed eligibility predicate remains complete/valid report + five Proven mandatory preflights + nine evaluated Proven dimensions. Actual private capability creation remains in production; no collection renderer gains that ability. A full static-only positive report still has six root boundary obligations.

Context association is semantic evidence: preserve a canonical partition including analysis_context and checked export/resolver/root/root-K role bindings plus full context contents. It is not a process address. Four mutually equal role contexts still fail if distinct from the analysis argument. Root/current resolver identities and root source IR remain separately named so stale-root/snapshot combinations cannot share identity merely because their node sets match. Canonical cross-process equality relies on content and association facts, never pointer integers, PID, UUID or clock.

Unknown kind/token/profile, missing evidence coordinate, unresolved graph reference/cycle, inconsistent prerequisite status, incomplete roster or resource limit yields an explicit model/infrastructure failure; it is not a semantic Violation or a truncated success. No new verdict enum is introduced. Scope authority/budget have no ceiling input; body/implementation/runtime/publisher evidence cannot be supplied by a self-asserted flag. No EffectGrant, ResourceHandle, reservation, receipt, worker admission, release authorization or runtime state is constructed.

Soundness argument: the view only records already established status/coordinates and cannot strengthen them; complete predicates are not identified by prose; the static gate is computed separately. Losslessness argument: full identity/evidence/contradiction/prerequisite/source links are retained; incomplete production summaries are rejected as sources, not silently repaired. Determinism argument: pure finite typed relation plus fixed canonical order/codec; proof-observation copies have explicit origin association. These are design arguments, not an executed implementation conformance claim.

Counterexamples and positive examples are [declarative vectors](phase-04c1b4c1-decision-vectors.md), all Not Executed. F04's root-global sensitive-path suppression remains unchanged; the design does not invent a guard countermodel or assert higher precision.
