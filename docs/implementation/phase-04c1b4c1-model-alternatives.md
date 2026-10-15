# Model alternatives and recommendation

Design only; registered semantics and executable coverage unchanged. See [draft](phase-04c1b4c1-g01-normative-contract-draft.md), [producer audit](phase-04c1b4c1-producer-inventory.md).

| Criterion | A typed identity + canonical string | B literal vocabulary | C typed equality + diagnostic old string |
| --- | --- | --- | --- |
| Source/ordinal/target injectivity | Explicit complete identity | Only after extending grammar beyond today's text | Possible in new typed field, absent in old text |
| Existing RequiredSemantic string | Preserved as lossless formal projection in new version | Preserved, but current sentences lose sources | Requires explicit classification migration |
| Independent derivation | Predicates/typed coordinates, not Rust push order | Easy to accidentally canonize Rust prose | Typed side feasible; correspondence must still be proven |
| Current compatibility | Requires adaptation, no false compatibility claim | Existing wording still insufficient; adding source fields is adaptation too | Requires adaptation plus justification for demotion |
| Cross-process / ordering | Closed codec and typed total order | Must invent equivalent structured grammar anyway | Only complete typed document is authoritative |
| Failure behavior | Missing origin/unknown kinds reject | Temptation to accept arbitrary text/fallback | Risk of silently ignoring required old facts |

**Choose A, not a contradictory A/C hybrid.** The canonical semantic string remains RequiredSemantic, with exact byte equality reflecting complete typed identity; reason/presentation is separate. Existing v1 strings are NOT reclassified DiagnosticOnly. B would either freeze lossy English permanently or become A with an undocumented parser. C could work under a different explicitly approved schema/classification migration, but is not chosen: the present requirement is exact semantic comparison without weakening the original field. There is no attempt to avoid necessary production changes.

Collection choice is Set of full identities plus a canonical sequence view. It is not a set of sentences. Origin occurrence is part of identity; independent repeated causes do not disappear. Copies of the same fact merge only their observation edges. Contradictory values for the same identity reject. SEQUENCE/control paths remain ordered input values, not collection-order side effects.

Soundness: obligations never establish the predicate they name; exact static gate uses five mandatory preflight plus nine dimensions, not emptiness. Losslessness: full identity fields, source occurrence and evidence coordinates are encoded; typed causes missing in current output trigger IncompleteFacts. Determinism: fixed typed ranks, ACE1 and canonical value-DAG assignment use no pointer/hash iteration/process state. Current behavior is incompatible at representation/provenance level; [minimal future deltas](phase-04c1b4c1-existing-rust-compatibility.md) are explicit.

Recommendation status: **FORMAL CONTRACT DESIGN ACCEPTED**, subject to completed design evidence and regression gates. This is not normative registration, deployed support, production G01 closure, G02/G03 repair or D10 Oracle execution.
