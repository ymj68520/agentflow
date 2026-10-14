# Phase 4C.1B.4B — overview

Starting HEAD: `c5b49850d9ba0dce75255f152498468fe8e93762`. This phase targets only 16 original D9-Recovery cases; the 22 Control baseline is retained. Closure depends on the recorded committed gates. D10 independent values are Pending for all 78; B01/B02 Closure and B03–B07 remain BLOCKING. No later phase or runtime/publication capability is created.

Verification pending — no Outcome A claim.

The declaration-driven RecoveryModel subclasses the independent 4A Control model; it recalculates every affected Node, E/Phi/R/F/U/X/K fact, scope, alias, proof and closure. Expected never reads Rust reports or target outputs. The test-only getter is unchanged; runners are under tests/support, without weakening the original static scan.

| CaseId | Stage | Sources | Local uppers | Root uppers | Required visits | Diagnostic visits |
| --- | --- | --- | --- | --- | --- | --- |
| portable-root-export-revalidation/d9-recovery-outer-retry-1 | D10 | [3] | [{'finite': 1}] | [{'finite': 1}] | 9468 | 150 |
| portable-root-export-revalidation/d9-recovery-outer-retry-2 | D10 | [3] | [{'finite': 1}] | [{'finite': 2}] | 9501 | 154 |
| portable-root-export-revalidation/d9-recovery-outer-retry-3 | D10 | [3] | [{'finite': 1}] | [{'finite': 3}] | 9502 | 154 |
| portable-root-export-revalidation/d9-recovery-outer-retry-4 | D10 | [3] | [{'finite': 1}] | [{'finite': 4}] | 9503 | 154 |
| portable-root-export-revalidation/d9-recovery-outer-iterate-1 | D10 | [3] | [{'finite': 1}] | [{'finite': 1}] | 9414 | 146 |
| portable-root-export-revalidation/d9-recovery-outer-iterate-2 | D10 | [3] | [{'finite': 1}] | [{'finite': 2}] | 9414 | 146 |
| portable-root-export-revalidation/d9-recovery-outer-iterate-3 | D10 | [3] | [{'finite': 1}] | [{'finite': 3}] | 9414 | 146 |
| portable-root-export-revalidation/d9-recovery-outer-iterate-4 | D10 | [3] | [{'finite': 1}] | [{'finite': 4}] | 9414 | 146 |
| portable-root-export-revalidation/d9-recovery-retry-2-times-iterate-2 | D10 | [4] | [{'finite': 1}] | [{'finite': 4}] | 10360 | 190 |
| portable-root-export-revalidation/d9-recovery-retry-2-times-iterate-4 | D10 | [4] | [{'finite': 1}] | [{'finite': 8}] | 10360 | 190 |
| portable-root-export-revalidation/d9-recovery-retry-3-times-iterate-2 | D10 | [4] | [{'finite': 1}] | [{'finite': 6}] | 10361 | 190 |
| portable-root-export-revalidation/d9-recovery-retry-3-times-iterate-4 | D10 | [4] | [{'finite': 1}] | [{'finite': 12}] | 10361 | 190 |
| portable-root-export-revalidation/d9-recovery-local-retry-inside-compensation-scope | D10 | [3] | [{'finite': 3}] | [{'finite': 3}] | 9639 | 155 |
| portable-root-export-revalidation/d9-recovery-two-sequence-actions | D10 | [3, 5] | [{'finite': 1}, {'finite': 1}] | [{'finite': 1}, {'finite': 1}] | 12724 | 235 |
| portable-root-export-revalidation/d9-recovery-race-retains-loser-action | D10 | [3, 5] | [{'finite': 1}, {'finite': 1}] | [{'finite': 1}, {'finite': 1}] | 12768 | 239 |
| portable-root-export-revalidation/d9-recovery-parallel-keeps-both-actions | D10 | [3, 5] | [{'finite': 1}, {'finite': 1}] | [{'finite': 1}, {'finite': 1}] | 12721 | 235 |

