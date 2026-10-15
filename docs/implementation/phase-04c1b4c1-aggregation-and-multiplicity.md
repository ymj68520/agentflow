# Aggregation, duplicates and provenance

Chosen formal model: **Set<complete Identity>** with a canonical sequence view. Not Set<String sentence>, not Multiset of sentences, not execution-ordered obligation list. [Draft D05/D10/D11](phase-04c1b4c1-g01-normative-contract-draft.md) is authoritative proposed text.

For observations of originating facts, let `id(f)` be the complete identity and `value(f)` its required status/contradiction/links. Quotient only explicit copies of the same origin:

```text
copies(x,y) iff id(x)=id(y) AND value(x)=value(y)
derive_set = union over unique originating unmet facts
observed_at(id) = union of exact report observation edges
```

Same identity with conflicting status is InvalidFacts, not max(Violation,RequiresEvidence,Unproven). That precedence remains the existing *overall proof verdict* rule and cannot erase per-source facts. Duplicate identities in submitted canonical output are rejected; the derivation's union does not license decoder repair.

Independent duplicate declarations retain distinct declared occurrence paths/source or mapping ordinals. Two uses of the same contract at node2/node10 remain two obligations. Equal atoms/profiles at different forward ordinals remain distinct; SEQUENCE order remains in their control paths. Retry/Iterate factors remain explicit existing logical bounds, not repeated list entries and not physical runtime attempts. There is no obligation cardinality → effect count or usage charge conversion.

Inherited child and parent report observations of the same originating fact have one identity with two observation edges. An independently evaluated parent coverage predicate is a different fact even if it uses the same RuleId and dimension. Never deduplicate by English reason, RuleId, node alone, digest alone or AtomKey alone.

Failed preflight: emit every failed core fact, maintain nine explicit blocker records with their exact cause keys; do not fabricate nine independently evaluated failures. The six evidence boundaries still exist. A Violation retains both its unmet predicate and its typed contradiction; unrelated unknown/evidence obligations are not removed. Proven evaluated facts emit no unresolved obligation, although their established prerequisites remain in the fact index.

This intentionally differs from current RootExport's two existential summary messages and current inherited vectors' insertion-order multiplicity. Migration requires origin/cause facts at the point of evaluation; a report-only postprocessor cannot recover lost distinctions. See [compatibility](phase-04c1b4c1-existing-rust-compatibility.md).
