# Independent E, Phi and U

Scope: the 22 existing D9-Control CaseIds only. D10 independent value comparison remains Pending. Historical CaseIds and Expected are unchanged. Starting HEAD: `5aa4ef7b865b9255a29c83c8375e13803d5e5ce1`.

Effect identities include operation, every qualifier member, selector port/scope, guard and bounds. Accumulation uses checked sums; SELECT joins/maxes with frozen guard/lower rules; SEQUENCE later lowers and RACE/fail-fast lowers weaken without erasing uppers. HANDLE adds protected work to the exclusive/no-handler envelope. ITERATE and RETRY multiply once with their distinct lower/count semantics; TIMEOUT retains work. Residual and Unknown states have separate tags.

Phi is an exact nominal TypeSchemaRef union, with exact HANDLE removal plus possible handler failures. Timeout, cancellation, race winner and RecoveryBlocked are not inserted into business Phi. Leaf/node identity and recursive paths retain failure provenance.

Usage is keyed by phase and complete meter identity; units/aggregation remain in the exact context recapture. Missing coordinates become Unknown under composition, distinct from explicit zero and known unbounded. Handler no-handler zeroes exist only at explicitly declared coordinates. Python integers are checked as exact U64 values; floats/bools/noncanonical numerals and overflow fail. Runtime reservations, costs and refunds are absent.
