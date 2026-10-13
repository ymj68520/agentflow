# AUDIT-F01 — Same-Row Effect Containment Review

## Normative invariant

For every Root effect occurrence, one complete export occurrence must witness
containment. The witness binds full operation and qualifier identity, resource
requirement port, compatible selector scope, occurrence bounds, guard
containment, and one-use slot capacity. Facts from different export rows may
not be joined.

The frozen af_effect::contains relation is the authority. It canonicalizes
both operands, deterministically visits canonical entry order, and consumes
the first unused compatible export slot. claim_known requires the same
candidate to match the cover key, scope, bounds, and guard before setting that
slot to None (crates/af-effect/src/algebra.rs:114-173,186-211). Thus each
export slot is used at most once. Duplicate/invalid canonical operands fail
closed. Open/root residuals cannot be proved inside a closed export.

## Production proof-path audit

effect_containment_verdict
(crates/af-computation/src/root_export.rs:2699-2791) has only one Proven
return: the early return at 2712-2713 when the normative
af_effect::contains(root, export) returns Proven. All following fallback
branches are incapable of returning Proven.

The fallback:

1. Returns Unproven for Open or Unknown rows before matching (2707-2710).
2. Enumerates candidates only when operation, complete qualifiers, requirement
   port, and scope relation match (2722-2732).
3. Chooses one unused row only if that row also covers the full interval and
   guard (2741-2748); the row index is retained in claimed_export_slots.
4. Returns Unproven, not a synthesized proof, on slot contention (2750-2759).
5. Allows a local contradiction only when there is exactly one compatible row;
   multiple partial rows are Unproven (2761-2768).
6. Returns only Violation or Unproven from the remaining guard/bounds
   diagnostics, then unconditionally ends in Unproven (2769-2791).

Consequently there is no pair of independent any() checks whose bound from
E1 and guard from E2 can combine into success. No selected candidate identity
is discarded. The fast path preserves canonical complete-atom semantics;
the fallback cannot override its failure with another positive relation.

Root ε calls this relation at 2526-2533. Root K-to-root-ε and Root K-to-export-ε
both call the same function at 2561-2582. prove_recovery separately checks
action effect identity and complete row-local coverage; it does not upgrade
the ε verdict (root_export.rs:3531-3610 and following recovery mapping checks).
The K-required recovery usage is checked against the derived Root and Export
usage profiles rather than added to either envelope a second time.

## Slot-capacity result

The normative algebra consumes one slot per root entry. An independent
two-root/one-slot production fixture returned ε Unproven, overall Unproven,
no checked capability. A two-root/two-slot fixture with a valid one-to-one
canonical match remained Proven and issued the checked static capability.
Reversing candidate row order and adding an unrelated third row did not turn
the split-row P30 witness into Proven.

The matching algorithm is intentionally deterministic and conservative. A
valid but complex assignment that the normative matcher cannot establish may
be Unproven; it must not become false-Proven through fallback aggregation.

## Guard scope and open rows

root_has_guard_sensitive_path is a recursive, root-wide scan for SELECT,
RACE, and HANDLE (root_export.rs:2794-2801). If any such path exists, guard
countermodel diagnostics are disabled; this can suppress a decisive
Violation, but leaves the fallback Unproven. It is not used to select
positive candidates and cannot affect the sole normative Proven return.
Open/unknown residuals are not interpreted as empty or closed.

## Independent P30 reproduction

Formal input: root write with scope {a=1,b=1}, unconditional interval
[0,10]; E1 has scope {a=1}, guard p1 == Int64(0), interval [0,10]; E2 has
scope {b=1}, unconditional guard, interval [0,5]. At p1=1, E1 does not
contain the root guard and E2 does not contain the root upper bound. No single
row witnesses the root entry.

Production result after F.1: RootExport reached; ε Unproven, overall
Unproven, checked capability not issued. This matches the independent
expected safe result. The exact original red case is a reproduction, not
counted as a new semantic shape.

## F01 case disposition

The 20-case core matrix reached RootExport in 18 cases. R15 (one-branch RACE)
was rejected by checked IR validation (race_insufficient_branches); R17
(open/unknown residual) was rejected by ComponentValidation because an open
effect row is not a complete operation set. Both are correct earlier
rejections, not checker passes.

Among the 18 reached cases: split-row P30, reversed rows, unrelated third row,
and two roots competing for one slot were Unproven with no capability;
full same-row and two-root/two-slot positives were Proven with capability;
row-local bounds, typed-guard, qualifier, port, and scope contradictions were
Violation with no capability; SELECT/HANDLE/RACE-sensitive uncertain guards
were Unproven with no capability. Two additional co-occurring K-source
interactions and the Forward-plus-Recovery same-atom interaction were all
Violation, no capability. Exact case outcomes are recorded in
phase-04b2f2-adversarial-production-results.md.

## Conclusion

No cross-row positive path exists in the audited production call graph.
AUDIT-F01 is CLOSED. This is a static envelope result only; it does not
establish that a component body conforms to the claimed effect contract.
