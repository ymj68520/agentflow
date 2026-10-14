# Source and closure derivation

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

The Oracle computes owned preorder and leaf preorder from the declared IR child edges, validates unique identities and accounts for every declared node. Exact resolver associations select declared checked assets by full pins. HashV1 is recomputed over each used contract/IR asset, schema content pins are independently checked, and relation plain SHA-256 checksums are recomputed. Same contract at two nodes yields two source instances.

Closure preserves all known occurrences, residuals and K sources; it is separate from root export compatibility. The latter stays NotExpressedInSemanticIr/Unproven. Context content is recaptured from declarations and compared with getter-based D5 recapture, while every D9 leaf/node must share its root's freshly rebuilt arena. No process pointer address is an identity.

The 22-case sources have no active K action. The Oracle checks this exact declared binding multiset before emitting an empty action/effect/Usage fold. AWAIT has no adapter K source and keeps Unknown with its source id. CompensationScope retains nearest local scope and forward facts; no action owner, count or usage profile is invented. The specialized 16-case nonempty K model remains Pending.
