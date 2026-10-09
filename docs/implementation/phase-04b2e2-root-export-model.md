# Phase 4B.2E.2 — RootExportAssertion Model

## Scope and claim

`af_computation::RootExportAssertion` is a compiler-boundary, in-process,
non-wire input. It is neither a `ComponentContract` nor a digest-bearing or
publishable artifact. The checker’s strongest positive claim is only:

> The exact checked root’s static declarations fit the exact checked export
> under the exact pinned compiler semantics and proof context.

It does not prove component implementation conformance, subflow body
conformance, publisher authenticity, Worker admission, effect authorization,
runtime recovery safety, or budget reservation. No new digest domain or
AFIDL/ABI field is introduced.

## Typed identity

The assertion contains exact SemanticIrDigest and ComponentContractDigest
pins, `CompilerSemanticsIdentity`, a content-complete `ProofContextManifest`,
typed I/O/resource/effect/flow/recovery/execution mappings, and exact subflow
dependencies. `RootExportAssertionIdentity` is a clone of the complete typed
tuple and compares through Rust structural equality. There is no assertion
digest, Debug-string identity, caller-supplied `compatible` flag, or proof
boolean.

`canonicalized()` orders vectors by target coordinate followed by exact source
identity and typed payload; it never removes duplicates. The checker rejects
duplicate targets/source identities and verifies every mapping against the
checked inputs. The report retains the complete assertion identity plus each
dimension’s exact typed coordinate witnesses.

## Compiler semantics identity

The identity binds:

- analysis semantics version `agentflow.root-export-checker@1`;
- exact SpecificationIds and the complete bytes/checksum of every applicable
  normative source;
- exact `specifications.json` and `rules.json` contents/revisions;
- the applicable AF-CONTRACT and AF-SEMANTIC RuleId set, with registered
  source, anchor, and statement content.

The checker recomputes content checksums and verifies source paths/digests and
RuleIds against the embedded current registries. A matching RuleId alone is
not accepted. These are ordinary SHA-256 content checks, not HashV1 domains or
signatures. A semantics/content change makes the old assertion Unproven and
requires fresh root derivation.

## Production API

`analyze_root_export_assertion` always returns an independent nine-dimension
report plus preflight diagnostics. `check_root_export_assertion` returns
`CheckedRootExportCompatibility` only when all nine static dimensions are
Proven. That capability is bound to the full report and claims static
compatibility only; body/runtime statuses remain separate.
