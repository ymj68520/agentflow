# Actual substitution semantics and conservative evidence retention

**PROPOSED — NOT REGISTERED** for the proposed interpretation. Read-only production baseline: `crates/af-contract/src/compatibility.rs` lines42–376. Real signature order is `check_substitution(replacement, old)`, not old first. Both inputs are private-validator-issued CheckedComponentContracts; no analysis context parameter or caller IR exists.

ComponentIdentity is the complete closed `{catalog:String,name:String}` in Rust model.rs14–17 and AFIDL component-contract.afidl15–18 (field numbers1/2). Catalog must parse as an exact SpecificationId; name is nonempty NFC lowercase dotted segments of [a-z0-9-]. No publisher/version/implementation/digest/extra member is present. Checked canonical declarations are compared by exact values; no partial name, inferred version ordering, case-fold or digest equality substitutes for identity.

Three distinct relations: L=complete lineage equality; B=all13 ComponentContract fields equal; C=actual checked context object identity. B implies L, but L does not imply B or C. Same digest/content across separate contexts does not imply C. The contents include identity,I,O,Phi,epsilon,R,F,U,K,X,effect-flow partitions,action bindings,usage bindings; none is omitted from B.

## Frozen control flow

1. If !C, return12 Unproven rows immediately. No B/L/directional relation executed. No Rust Result rejection or formal D10 report exists here.
2. If C && B, return12 Proven rows by the whole-body exact-equality shortcut. Do not claim twelve directional functions executed.
3. Otherwise initialize **all12 rows Unproven**, evaluate the concrete candidate relations in source order, and call set. set only replaces a row when rank(candidate)>rank(current), with P0<U1<RE2<V3. Thus P candidates DO NOT promote an initial U. Candidate U also does not replace a reason. Cross-dimension is recomputed after every set from the entire current array, including its previous cross row. Because non-cross updates only increase rank, final cross equals the worst of the final11 non-cross rows; the initial U floor remains. No current candidate returns RE.

Consequences: on changed-body path final per-row judgments are U or V, never P/RE; exact-body path is the sole all-P shortcut. Different lineage is U, not V. Same lineage with an added output (legal checked input) has local L=P but **final identity=U**. Modeling these as different PredicateIds is necessary; a single key with P and U would be a false conflict. The amendment captures both before status arbitration and preserves final reports exactly. It does not fix the conservative no-promotion behavior.

Execution has its own nested rule: `combined` returns U if either aspect U, otherwise V if either V, otherwise P. Outer contract row folding uses V>RE>U>P. Do not replace the inner rule with the outer rank: (feasibility=V,security=U) gives local execution=U but separate feasibility row V and cross V.

Fresh R1 diagnostic (not design-vector execution) confirms same-body P; different name/catalog U; same-lineage added-output body identity U; added required input gives I/O V and identity U; reversing that pair gives I/O U; equal-body contracts from different contexts give12 U. Final probe log `target/phase-04c1b4c4r1-probe-final.log`, exit0. An initial proposed unknown-influence fixture could not become Checked:standalone check_contract returned information-flow RequiresEvidence (the validator also used by Portable D6,but no Portable pipeline was invoked here),so no substitution executed for that fixture;failed probe log retained separately,never counted as capture or positive comparison. Final legal content mutation uses an added output instead.
