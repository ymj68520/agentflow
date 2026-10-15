# Typed identity and source separation

Authoritative proposed definitions are [draft D01–D04](phase-04c1b4c1-g01-normative-contract-draft.md). This document is design, not registered semantics.

Identity = `(edition, complete_scope, assertion_path, stage, kind, dimension, source, target, coordinate, rule, requirement, origin)`. Status, disposition, contradiction, prerequisites, references and observations are mandatory record values/links, not natural-language identity. Status changes cannot manufacture satisfaction; conflicting simultaneous values reject.

| Requested dimension | Identity decision |
| --- | --- |
| Kind | Closed draft23 substantive RootExport kinds + control_evidence24 + prerequisite25; no generic unknown-kind fallback |
| Stage | Always explicit; D6/D9/D10 predicates are distinct |
| Dimension | Explicit optional root/boundary dimension; I/O/Phi/E/R/F/U/K/X distinct |
| Rule | Exact normative specification/rule/anchor; full statement bytes bound in scope |
| Source IR | Required for root/node/occurrence/subflow subjects; full algorithm/domain/digest |
| Source contract | Required for component/action/execution source; actual resolved subflow export when known |
| Computation | Required whenever node-scoped; root-aggregate subject uses exact root node |
| Source ordinal | Required for effect/action/profile/flow occurrences where their frozen identity uses it |
| Target / mapping | Complete target and typed coordinate, not a name/port number alone |
| Subflow | Caller/call-site/target/export and full nested assertion edge path |
| Evidence requirement | Predicate clause plus evidence class, not merely Unproven/RequiresEvidence text |
| Context / Resolver | Complete typed content plus checked association premise, never pointers/counts/checksums alone |
| Independent duplicates | Declared origin path/index distinguishes equal-valued independent occurrences |

Whole-root obligations intentionally use a root subject; they are not assigned fake per-leaf coordinates. Exact per-source obligations require cause facts before aggregate reduction. Existing RuleIds, atom qualifiers, guard, action kind, usage meter/unit/phase, multiplicity factors and control paths keep their original nominal identities.

Examples (symbolic notation expands through the complete fixture environment in [vectors](phase-04c1b4c1-decision-vectors.md)):

```text
input_mapping(scope=S, source=(IR0,node2,C0), coordinate=(port1,param1,slot11), origin0)
!= input_mapping(scope=S, source=(IR0,node10,C0), coordinate=(port1,param1,slot11), origin0)

effect_containment(source=(IR0,node2,C0,entry0), full_atom=A, origin0)
!= effect_containment(source=(IR0,node2,C0,entry1), full_atom=A, origin1)
```

Different evidence class, RuleId, contract pin, target port, mapping target, nested call-site or context content similarly changes identity. Byte-equal explanations are not an equivalence proof. A parent observing the exact originating fact again keeps that identity and adds an observation edge; no new ordinal is taken from parent `push()` order. If an old report did not preserve the origin, no parser can invent it.

Scope snapshots are full typed value graphs under the exact compiler profile. Equality is content/association equality, not durable pointer identity. The proposed report-local transparent encoding is fully schema directed, preserves nominal scalar types and all equality-bearing fields, and rejects opaque/unregistered shapes. G02's getters and future shape materialization are implementation prerequisites, not implemented here. A canonical string includes its complete scope graph; a factored local ID alone is not a standalone identity.
