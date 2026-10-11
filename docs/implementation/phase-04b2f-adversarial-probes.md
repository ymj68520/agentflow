# Phase 4B.2F — Adversarial Probe Register

## Probe accounting

**78 discriminating cases were designed. 31 were executed against production
entry points** by constructing checked formal fixtures, changing actual
assertion/contract/context inputs, and calling the analysis/check APIs again.
30 reached RootExport; one unknown-Flow fixture was rejected at
ComponentValidation. The two P0 probes both reached RootExport and both
received a checked capability. No frozen Expected was edited. Probe source
hooks were temporary and removed after recording these results.

`Not run` means design only, never pass. Existing comparator mutations are
excluded because they mutate Expected/Actual projection data instead of
re-running the production checker on a changed proof input.

## Executed production probes

| ID | Input mutation from a valid fixture | Production stage | Actual verdict / checked capability |
| --- | --- | --- | --- |
| P01 | Change exact SemanticIrDigest pin | RootExport | Violation / no |
| P02 | Change top exported ComponentContractDigest pin | RootExport | Violation / no |
| P03 | Delete required input map | RootExport | Unproven / no |
| P04 | Delete output map | RootExport | Unproven / no |
| P05 | Change output type witness | RootExport | Unproven / no |
| P06 | Change root ResourceSlot mapping | RootExport | Unproven / no |
| P07 | Change ResourceTypeRef pin | RootExport | Unproven / no |
| P08 | Change complete EffectAtom qualifier | RootExport | Unproven / no |
| P09 | Change compiler semantics version | RootExport | Unproven / no |
| P10 | Change asserted relation canonical content | RootExport | Unproven / no |
| P11 | Delete one child execution mapping | RootExport | Unproven / no |
| P12 | Delete a Flow mapping | RootExport | Unproven / no |
| P13 | Delete Recovery mapping | RootExport | Unproven / no |
| P14 | Delete a Recovery usage profile | RootExport | Unproven / no |
| P15 | Narrow an unconditional action guard to input-equals | RootExport | Unproven / no |
| P16 | Delete a subflow dependency pin | RootExport | Unproven / no |
| P17 | Add an extra required root input absent from export | RootExport | Violation / no |
| P18 | Reuse one RunParameter for two exported inputs | RootExport | Violation / no |
| P19 | Remove a required WorkflowOutput | RootExport | Violation / no |
| P20 | Lower exported effect upper below the root upper | RootExport | Violation / no |
| P21 | Lower exported Forward usage ceiling below root usage | RootExport | Violation / no |
| P22 | Add known root Failure outside complete exported Phi | RootExport | Violation / no |
| P23 | Add root Flow permission absent from export | RootExport | Violation / no |
| P24 | Remove exported execution sandbox floor | RootExport | Violation / no |
| P25 | Unknown external Flow source fixture | ComponentValidation | Violation / no checker invoked |
| P26 | Frozen corpus effect-upper negative | RootExport | Violation / no |
| P27 | Frozen corpus recovery-usage negative | RootExport | Violation / no |
| P28 | Frozen corpus execution-security negative | RootExport | Violation / no |
| P29 | Frozen corpus action guard outside export | RootExport | Violation / no |
| P30 / F01 | Root `[0,10]`, unconditional, scope `{a=1,b=1}`; export E1 `{a=1}`, p1=0, `[0,10]`; E2 `{b=1}`, unconditional, `[0,5]` | RootExport | **Proven / checked capability issued — P0** |
| P31 / F02 | Derive with same-context subflow snapshot A/E=1; check with snapshot B/E=10; top root export C/E=1 | RootExport | **Proven / checked capability issued; body RequiresEvidence — P0** |

For P30 the full pipeline checked the source/export ComponentContracts, IR,
Phase 3C, resolution, root, RootRecovery and assertion. All nine dimensions
were Proven. For P31 the source root pin was
`2019eeb505c3d1e14fa086a89bd3868a6f1692224fbad44f3651fe493943afc5`, but the
assertion/current snapshot target export pin was
`1851278a5faa9a8b239abe89d4191d3f3735b83004124894b791304ba07fc661`; the top
root export remained upper 1. All nine dimensions nevertheless reported
Proven. The exact recipes and violated predicates are in
[false-Proven analysis](phase-04b2f-false-proven-analysis.md).

## Additional designed probes — not executed

| ID | Category | Mutation and expected safe behavior | Actual stage/verdict |
| --- | --- | --- | --- |
| D32 | Identity | Permute map rows then canonicalize; identity/semantics unchanged | Not run |
| D33 | Identity | Duplicate target row; Violation before proof | Not run |
| D34 | Identity | Same numeric port from two source contracts; keep namespaces distinct | Not run |
| D35 | Identity | Change one Flow partition while other maps stay fixed; re-evaluate F | Not run |
| D36 | Context | Same relation revision, altered relation content; old manifest rejected | Not run as a separate context object |
| D37 | Context | Same catalog label, changed member set; old root cannot be reused | Not run |
| D38 | Context | Change resource attribute vocabulary | Not run |
| D39 | Context | Change Flow taint/label relation | Not run |
| D40 | Context | Change Usage unit/aggregation semantics | Not run |
| D41 | Context | Change Recovery operation catalog | Not run |
| D42 | Context | Change execution trust-domain relation | Not run |
| D43 | Compiler | Same RuleId, changed normative short statement/source text | Not run; source equality logic reviewed |
| D44 | Input | Exact port type but incorrect use-site producer | Not run |
| D45 | Input | Invalid lexical scope for otherwise exact input | Not run |
| D46 | Input | Transform expression without exact identity proof | Not run |
| D47 | Output | SELECT case lacks an output producer | Not run |
| D48 | Output | RACE contestant lacks output on a possible terminal path | Not run |
| D49 | Output | HANDLE handler has incompatible output | Not run |
| D50 | Output | ITERATE carry type matches but producer is unknown | Not run |
| D51 | Resource | Drop one source alias coordinate | Not run |
| D52 | Resource | Similar display names but different ResourceTypeRef | Not run |
| D53 | Resource | Selector scope narrower than the actual effect selector | Not run |
| D54 | Resource | Cross-Scope resource leak | Not run |
| D55 | Resource | K action resource requirement omitted | Not run |
| D56 | Effect | Same full atom Forward=1 and K action=6; one ceiling must cover 7 | Not run |
| D57 | Effect | Two co-occurring same-atom K sources 4+6; ceiling must cover 10 | Not run |
| D58 | Effect | Open/unknown residual cannot be interpreted as empty | Not run as an additional audit probe |
| D59 | Effect | Guard mismatch with an unrelated SELECT in another subtree | Not run; source shows global fail-closed suppression |
| D60 | Recovery | Retry(3, CompScope) Local=4, Root=12 | Not run in this audit; arithmetic independently checked |
| D61 | Recovery | CompScope(Retry(3)) Local/Root=12; no second factor | Not run in this audit; arithmetic independently checked |
| D62 | Recovery | Iterate(4) outside CompScope Local=4, Root=16 | Not run in this audit; arithmetic independently checked |
| D63 | Recovery | Retry(3)×Iterate(4), Local=8, Root=96 | Not run in this audit; arithmetic independently checked |
| D64 | Recovery | RACE loser K source retained | Not run |
| D65 | Recovery | Missing K source/action/profile multiset member | Not run beyond mapping/profile deletions P13/P14 |
| D66 | Usage | Missing active K meter is Unproven, never zero | Not run |
| D67 | Usage | Root K upper arithmetic overflow | Not run |
| D68 | Flow | AWAIT external source → output remains unknown | Not run as a new audit mutation |
| D69 | Flow | AWAIT signal influences SELECT decision | Not run |
| D70 | Flow | Loop-carried taint reaches K action | Not run |
| D71 | Flow | EffectFlowPartition membership mismatch | Not run as a separate production input |
| D72 | X | Feasible profile but unsafe sandbox | Not run |
| D73 | X | Valid runtime ABI with wrong trust domain | Not run |
| D74 | X | Invalid residency region | Not run |
| D75 | X | Parallel children with distinct profiles are not summed into Worker X | Not run |
| D76 | X | K action lacks action-specific X evidence | Not run as an isolated probe |
| D77 | Subflow | Missing transitive target/cycle | Not run in this audit |
| D78 | Subflow | Target RootExport pin does not authenticate body | Static/body separation covered by the positive fixture; no extra mutation |

The overall set is 78 designs including the 31 actual probes. The unexecuted
cases are explicit audit follow-up coverage, not reported as passed.

## Metamorphic invariants

| Invariant | Result |
| --- | --- |
| Required assertion-map mutation cannot reuse a checked result | Observed: P03–P08, P11–P16 reject |
| Relation manifest content mutation invalidates assertion | Observed: P10 Unproven |
| Export epsilon ceiling tightened below root | Observed: P20 Violation |
| Export Usage ceiling tightened below root | Observed: P21 Violation |
| Additional root effect outside export cannot remain Proven | Ceiling variant P20 observed; separate added-atom shape not run |
| Additional root Recovery source is fully re-folded | Not run as a new metamorphic probe |
| Unknown influence cannot become empty | P25 early rejection; dedicated post-check unknown-propagation variant not run |
| Runtime/body evidence is not promoted | P31 has static Proven but body RequiresEvidence; P30 claims no runtime authority |
