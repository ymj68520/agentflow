# Phase 4C.0 — Cross-Process Static Proof Revalidation

All prospective records and entry points are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**. Decision C01 selects full revalidation.

## Existing capability

`CheckedRootExportCompatibility` is a Rust wrapper with a private report field
(`crates/af-computation/src/root_export.rs:545`). Only
`check_root_export_assertion` creates it after the fresh aggregate static
verdict is Proven (`884`). A cloned/serialized report, a signature over a report,
or a saved address cannot reconstruct that capability.

The checker requires exact IR/export pins, canonical typed mappings, current
compiler semantic content, exact context pointer/content, resolver identity
and complete subflow pins (`root_export.rs:900`). A failed static preflight
withholds all nine dimension proofs. Body evidence is recorded separately
after the static gate; a release consumer must not treat that separate
RequiresEvidence as already satisfied.

## Strategy comparison

| Strategy | Proof performed and trusted inputs | Benefits | Additional soundness obligations | Decision |
| --- | --- | --- | --- | --- |
| A. Full re-execution | Rebuild checked inputs, target closure, Root and RootRecovery; execute all nine dimensions | Uses the audited production relation; clear change invalidation | Portable decoding, full context reconstruction, qualified verifier and immutable input closure | ACCEPTED baseline |
| B. Portable certificate | Independent verifier checks derivation witnesses without all computation | Potentially cheap independent replay of proof rules | New certificate schema, rule closure, complete verifier semantics, authenticity if trusting issuer claims, per-step context binding and differential audit | BLOCKING; not a renamed ProofReport |
| C. Selective re-derivation plus attestations | Validate pins, reuse selected proven subresults, rederive affected parts | Could reduce repeated work | Sound complete cache keys, dependency graph, trusted subproof predicate and invalidation, semantics-equivalence proof for omitted derivations | Deferred; BLOCKING for positive subproof reuse |

Full revalidation still trusts the checker implementation and approved content
origins. It does not by itself prove a binary behaves as declared. A
cryptographically authenticated certificate can still contain a false claim;
signature verification and proof verification remain separate.

## Fresh verification sequence

The following is a future publication procedure, not an existing API:

1. Parse the versioned parent and inline inputs using closed schemas. Enforce
   canonical order, typed digest domains, duplicate/conflict rejection, exact
   presence/absence, and bounded resource use.
2. Obtain exact IR, export, contract/schema/catalog/relation and target IR
   contents from pinned artifacts. Check bytes against the appropriate existing
   canonical encodings/digests. A locator is only a retrieval hint; it may not
   select content or substitute a newer version.
3. Validate full normative source and registry contents and the static checker
   semantics. Obtain an approved verifier build; verify build provenance
   separately when policy requires it.
4. Reconstruct all explicit universes and relation snapshots. Verify catalog
   origin authorization against consumer-controlled policy; equality alone
   does not make caller-supplied catalogs trusted.
5. Allocate one immutable local StaticContractProofContext. Check every contract
   under this same object; validate target IR and Phase 3C, populate the target
   universe, and build the exact resolver associations.
6. Derive dependencies from the actual target IR documents, compare direct
   edges/transitive closure with the portable manifest and LockGraph, and reject
   missing nodes, cycles or association conflicts.
7. Check caller IR/dataflow and freshly derive Root plus RootRecovery from that
   snapshot. Never load a saved derived summary as trusted input.
8. Reconstruct the typed assertion from the parent inputs, require every
   redundant root/export/context/association field to agree, and call the
   existing formal RootExport check.
9. Retain the fresh preflight and all nine verdicts. Require the static
   prerequisite subset and all nine dimensions Proven. Evaluate body,
   implementation, Plan and publication policy requirements independently.
10. Produce a new verification result bound to the exact parent, verifier
    semantics/build and release policy inputs. The pointer remains local.

## Context reconstruction requirements

Current `ProofContextManifest` contains cloned Rust universe objects
(`root_export.rs:106`); `capture` is an in-process snapshot, not a stable
encoder/decoder. The future schema must define the following:

| Context family | Required content and validation |
| --- | --- |
| Compiled modules and named types | Exact AFIDL/schema identities and canonical bodies/imports; recompute TypeSchemaDigest, reject duplicate logical names with divergent bodies before insertion |
| Effect catalogs | Complete operation sets with exact catalog identity; complete qualifiers and selector relations |
| Resource types | Exact named schema pins, attribute vocabulary and descriptors |
| Flow catalogs | Complete confidentiality/taint/sink/decision-capable sets |
| Usage | Exact phase/meter/unit/aggregation descriptors; absent coordinate never becomes zero |
| Recovery | Complete operation/reconciliation sets and exact action/profile identities |
| Execution | Capability/version/attribute/trust/region/runtime-profile universes; declared constraints, not current capacity |
| Relations | Preserve each optional relation, full canonical relation bytes and typed pin; recompute checksum and schema identity |
| Resolver inputs | Actual component/target/export contents, source and call-site associations, direct edges, optional exports and complete reachable closure |

Roundtrip obligation: after decoding and rebuilding a supported portable
manifest, reprojecting and canonically encoding the rebuilt context must
produce exactly the original manifest content. Independent cross-language
goldens must establish this; Rust Debug or incidental Serde layout is
insufficient.

The new process's pointer may differ from the compiler's pointer. All fresh
checked values in that new process must use the one reconstructed local
context object. Cross-process equality is content validation plus reconstruction;
it is never equality of saved addresses.

Current manifest capture maps absent dependency lookup to an empty vector
(`root_export.rs:150`). The fixed resolver identity preserves Option separately.
A portable schema must preserve Unknown/Unavailable versus Known(empty), not
copy that helper as a durable encoding. No required target with unknown
dependencies can pass publication. Empty known dependencies remain valid.

## Reproducibility and invalidation

The normative verifier inputs are exact immutable content. Network, clock,
current defaults, ambient configuration and registry-latest do not enter the
static derivation. Trust-policy freshness may use a separately supplied
verification time/revocation snapshot; its facts belong to the release decision,
not to semantic derivation.

Any change in IR/export, typed map, semantic source/registry, universe/relation,
component pin, target/export association or direct/transitive edge requires new
checked inputs, root derivation and all dimensions. The existing checker may
report Violation for an exact pin mismatch or Unproven for stale context. A
publisher cannot “refresh” a saved report by replacing its identity fields.

The entire current universe is bound. Pruning unused context entries is not
authorized proof reuse: construct a fresh minimal input set and rederive a new
assertion if a future schema explicitly permits that projection.

## Limited reuse under option A

Content-addressed immutable byte retrieval, parsers and canonical schema
compilation may be cached after independently validating their exact keys and
content. Cached reports and derived positive subproofs are diagnostic hints
only. Option C needs a separately accepted verifier/cache model before such
subproofs can discharge a mandatory gate. Body attestation reuse follows its
own exact subject and freshness rules.

Current cross-process decoder/revalidation production entry point: absent.
Future conceptual entry point F-REVALIDATE is defined only in the decision
vectors. This design preserves current checker APIs.
