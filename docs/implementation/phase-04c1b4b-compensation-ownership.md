# Phase 4C.1B.4B — compensation ownership

Starting HEAD: `c5b49850d9ba0dce75255f152498468fe8e93762`. This phase targets only 16 original D9-Recovery cases; the 22 Control baseline is retained. Closure depends on the recorded committed gates. D10 independent values are Pending for all 78; B01/B02 Closure and B03–B07 remain BLOCKING. No later phase or runtime/publication capability is created.

Owned declaration edges determine nearest CompensationScope. Entering an inner scope resets local factors; Root factors remain. Outer scope does not re-register inner local actions. SourceAction walks preserve original ownership and ordered path. No scope ancestor means RootRecovery, not an invented scope. Retry(CompScope) and CompScope(Retry) differ in Local potential. Source/action/profile duplication is preserved; no actual recovery or committed-event order is inferred.
