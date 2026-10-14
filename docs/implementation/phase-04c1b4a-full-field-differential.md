# Full-field D9 differential

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

Both implementations validate the closed schema, then compare every required typed visit. The Rust projection extension is under cfg(test); production serializers and checker behavior are unchanged. The data is analysis evidence, with no Checked constructor or serialization. Full child facts and dimension proof arrays are added using typed getters.

| CaseId | Stage | Required instances | Diagnostic instances |
| --- | --- | --- | --- |
| portable-root-export-revalidation/d9-opcode-invoke | D10 | 4844 | 88 |
| portable-root-export-revalidation/d9-opcode-await | D10 | 4509 | 72 |
| portable-root-export-revalidation/d9-opcode-sequence | D10 | 5758 | 137 |
| portable-root-export-revalidation/d9-opcode-select | D10 | 5767 | 137 |
| portable-root-export-revalidation/d9-opcode-parallel | D10 | 5759 | 137 |
| portable-root-export-revalidation/d9-opcode-race | D10 | 5794 | 141 |
| portable-root-export-revalidation/d9-opcode-handle | D10 | 5768 | 137 |
| portable-root-export-revalidation/d9-opcode-iterate | D10 | 5091 | 97 |
| portable-root-export-revalidation/d9-opcode-retry | D10 | 5117 | 101 |
| portable-root-export-revalidation/d9-opcode-timeout | D10 | 5104 | 99 |
| portable-root-export-revalidation/d9-opcode-scope | D10 | 5153 | 105 |
| portable-root-export-revalidation/d9-opcode-compensation_scope | D10 | 5188 | 109 |
| portable-root-export-revalidation/d9-opcode-call_subflow | D10 | 4976 | 102 |
| portable-root-export-revalidation/d9-opcode-sequence-reversed | D10 | 5758 | 137 |
| portable-root-export-revalidation/d9-opcode-parallel-fail-fast | D10 | 5795 | 141 |
| portable-root-export-revalidation/d9-opcode-race-three-contestants | D10 | 6473 | 181 |
| portable-root-export-revalidation/d9-opcode-retry-four-attempts | D10 | 5119 | 101 |
| portable-root-export-revalidation/d9-opcode-timeout-retry-nested | D10 | 5472 | 124 |
| portable-root-export-revalidation/d9-opcode-scope-compensation-nested | D10 | 5566 | 135 |
| portable-root-export-revalidation/d9-opcode-compensation-sequence | D10 | 6248 | 168 |
| portable-root-export-revalidation/d9-opcode-iterate-while-eight | D10 | 5091 | 97 |
| portable-root-export-revalidation/d9-opcode-select-two-exclusive-branches | D10 | 5767 | 137 |

| Aggregate | Count |
| --- | --- |
| Expected required | 120117 |
| Expected diagnostic | 2683 |
| Declared existing CaseIds | 22 |

The exact executed/compared/missing/unexpected counts are recorded in the gate evidence. D10 shape validation is separate and its independent values remain Pending.

| Oracle origin family | Instantiated visits |
| --- | --- |
| IndependentAlgebra | 31045 |
| ExactDeclarations | 5216 |
| ContentIdentity | 83856 |
| DiagnosticOnly | 2683 |

Origin accounting uses deterministic path families: complete context/pin/checksum identities, exact leaf declarations/source ids, and derived control/closure/proof facts. Diagnostic prose is counted separately.
