# Independent D9 control model

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

`d9_control_model.py` resolves exact declared Bundle assets and caller/source associations; `d9_algebra.py` performs independent E/Phi/U calculations; `field_contract.py` validates typed semantic projections. No Rust checker, FFI, Actual file or runtime report is an Expected input. Existing Python ACE1/HashV1 and declaration recapture helpers are reused. CaseId binds the resulting value to its immutable input checksum; it does not select a canned report.

| Opcode | Independent rule | Normative attribution |
| --- | --- | --- |
| INVOKE | exact checked declaration and call-site mappings | AF-SEMANTIC-0111..0117 |
| AWAIT | unknown adapter dimensions; exact signal/correlation and external provenance | AF-SEMANTIC-0122 |
| CALL_SUBFLOW | exact target/export association; body RequiresEvidence | AF-SEMANTIC-0118..0121 |
| SEQUENCE | ordered accumulation; later positive effect lowers weaken | AF-SEMANTIC-0127 |
| SELECT | exclusive join; checked constant case; other source facts retained | AF-SEMANTIC-0128/0129 |
| PARALLEL | all branch uppers; fail-fast weakens lowers; no inflight multiplier | AF-SEMANTIC-0130/0131 |
| RACE | loser effects, Usage and K/X sources retained; no winner | AF-SEMANTIC-0132/0133 |
| ITERATE | [0,n] effect repetition and one Usage multiplier | AF-SEMANTIC-0143 |
| RETRY | logical multiplier once; physical attempts separately projected | AF-SEMANTIC-0144 |
| TIMEOUT | child facts retained; two control outcomes outside Phi | AF-SEMANTIC-0145 |
| HANDLE | exact residual Phi; protected + conditional exclusive handlers | AF-SEMANTIC-0146 |
| SCOPE | exact lexical resources; absent authority/budget Unproven; settlement RequiresEvidence | AF-SEMANTIC-0147/0148 |
| COMPENSATION_SCOPE | forward facts retained; empty closed action multiset, exact local scope | AF-SEMANTIC-0149..0151 |

Every node retains recursive children, leaf facts, interface, scope, all seven dimension values and their proofs, occurrences/residuals, aliases, adjustments, sequence dataflow, inherited obligations and same-context association. Root retains closure, resolver identity, all dimensions and full Recovery fields. Proof references use the registered internal coordinate vocabulary (computation token, pinned digest or exact schema identity). Fresh checked inputs discharge static input premises; this closed report profile declares no extra prerequisite-string instances. Remaining unknown/evidence obligations stay in typed verdict/availability/source fields; the prerequisites collection is still mandatory and compared exactly.

Outside this subset, active K actions, nonempty public output expansion, nonempty subflow interface/dependency expansion and incomplete child E/Phi/U require a separate model and raise OracleGap. They are not emptied or skipped. FOREACH finite/dynamic E/U algebra exists, while uninstantiated binding/payload branches remain inventoried Pending.
