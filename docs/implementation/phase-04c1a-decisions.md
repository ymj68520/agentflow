# Phase 4C.1A — Ten Machine-Model Decisions

All selected types and procedures are **DESIGN ONLY / NON-NORMATIVE /
NOT REGISTERED ABI / NOT IMPLEMENTED**. ACCEPTED below means a complete
design within the full-content **static RootExport** profile; it neither
registers a wire model nor closes an implementation/publication blocker.
The referenced schema documents supply the exact closed declarations.

## DECISION-4C1A-01 — Portable Bundle Ownership

- Status: **ACCEPTED**.
- Normative Inputs: C01/C02/C03; existing SemanticIrDocument,
  ComponentContract, CanonicalTypeSchema, ACE1 and their HashV1 domains.
- Alternatives: redundant content copies; checksum-only external locators;
  one complete inline typed asset/table Bundle.
- Selected Design: PortableProofInputBundle owns ten required fields:
  schema_version, root_assertion_id, schema_assets, ir_assets, contract_assets,
  structural_types, proof_contexts, resolver_dependency_manifests,
  compiler_semantics_manifests and root_export_assertions. A root AssertionEntry
  references the complete inline root IR/export and the exact scoped manifests.
- Rejected Alternatives: report-only/digest-only proofs, ambient retrieval,
  an opaque Rust memory image, or a parent pointer standing in for missing bytes.
- Exact Type / Identity: the closed records and aliases in
  [portable schema](phase-04c1a-portable-proof-schema.md); asset keys include
  both domain-bearing digest and full nominal SchemaRef. IDs are separate
  positive uint32 Bundle-local namespaces, not global digests.
- Validation Rules: resolve every typed reference; verify every content body;
  reject conflicting/duplicate assets and dangling/unreachable padding;
  retain explicitly inventoried unused members.
- Canonicalization: sets sort by full typed key or local numeric ID; no decode
  renumbering. Same typed packaging has unique ACE1 bytes; isomorphic local ID
  relabeling may change evidence bytes without changing expanded facts.
- Failure-closed Semantics: absent required bytes fails D2/D3; unavailable
  collections never become empty; v1 has no external transport fallback.
- Cross-language Risks: graph expansion, local ID width, partial asset lookup
  by digest alone and accidental deduplication. Explicit finite graphs and
  complete typed keys remove those choices.
- ABI Implications: candidate agentflow.portable.proof.inputs@1 and mechanical
  ordinal numbering only; no registration, digest value or new hash domain.
- Downstream Dependencies: 4C.1B registered schema/codec; B06 must own the whole
  Bundle, not an incomplete report or unregistered standalone proof digest.

## DECISION-4C1A-02 — Typed Assertion Schema

- Status: **ACCEPTED**.
- Normative Inputs: RootExportAssertion and every mapping/source field;
  root-export rules, Phase 3C coordinates and frozen P30/P31 behavior.
- Alternatives: integer-only port maps; untyped JSON witnesses; field-complete
  closed projections with separate scopes and typed source associations.
- Selected Design: project every current input/output/resource/effect/flow/
  recovery/execution/subflow field, compiler identity and context manifest.
  StructuralType witnesses use an acyclic node table; boxed target identities
  use optional AssertionIds expanded bottom-up.
- Rejected Alternatives: same PortNo implies same coordinate; omission of
  source node/ordinal/guard; trusted child-body verdicts; normalizing absent,
  empty and missing fields into one value.
- Exact Type / Identity: PortableRootExportAssertion and all linked records in
  [assertion projection](phase-04c1a-root-export-assertion-projection.md).
  Export coordinates are (exact export ContractPin, port); source coordinates
  retain caller/source pins, node, port/slot and zero-based uint64 ordinal.
- Validation Rules: D2 checks finite references; fresh derivation checks
  completeness, aliases, exact witnesses and source binding. Nested target
  assertion identity pins do not establish target body proof. Unknown guard/
  provenance relations remain Unproven at the existing checker.
- Canonicalization: exact current primary comparators plus full canonical-row
  byte tie-breakers; dependency slots, alias ports and occurrence-index sets
  are strictly ascending. Sequences retain their defined meaning.
- Failure-closed Semantics: malformed shape fails; valid negative witnesses
  reach formal checking, which may return Violation/Unproven. No decoding
  shortcut manufactures a mapping or reuses an old Proven report.
- Cross-language Risks: leaf-local versus root occurrence numbering, usize
  narrowing, equal integers in different namespaces and lossy recursive types.
  Explicit scopes, uint64 and finite typed tables are mandatory.
- ABI Implications: new candidate wrappers only; every existing imported
  nominal shape and number remains unchanged.
- Downstream Dependencies: 4C.1B inverse projections, native-width checks,
  portable P30/P31 production conformance; F04 guard relation remains OPEN/P2.

## DECISION-4C1A-03 — Proof Context Content Closure

- Status: **ACCEPTED**.
- Normative Inputs: every StaticContractProofContext field and nested universe;
  existing relation snapshot constructors and dimension semantics.
- Alternatives: labels/revisions/checksums only; copying one shared schema/effect
  universe into every scope; independent complete data inventories.
- Selected Design: PortableContextContent includes compiled modules, global
  TypeSchema membership, effects, independent resource schemas/descriptors,
  flow labels/taints/sinks, Usage, separate recovery effects/reconciliation,
  every execution sub-universe and three typed optional relation slots.
- Rejected Alternatives: implicit catalog defaults; dropping unused members;
  unioning schema scopes; treating static execution descriptors as a full
  RuntimeSemanticsProfile or catalog classification as origin approval.
- Exact Type / Identity: closed shapes and member keys in
  [context manifest](phase-04c1a-proof-context-manifest.md).
  CompleteCollection is a raw supplied-set declaration; Complete([]) and
  Unavailable are different. Resource scope pins resolve independently.
- Validation Rules: positive reconstruction requires complete inventories;
  capability exact_version is explicitly retained and registered first;
  region edges require listed endpoints; schema/name/pin conflicts fail.
  Relation bytes regenerate exactly from the existing typed constructor.
- Canonicalization: strict per-universe member orders; exact attribute pairs;
  raw source identity is not NFC-normalized. Host members outside the canonical
  NFC projection domain fail export rather than silently changing identity.
- Failure-closed Semantics: unknown catalogs and unsupported procedural versions
  yield no positive reconstruction. Required missing relations stay Unproven;
  Some(empty relation) never aliases None.
- Cross-language Risks: overwritten maps, constructor auto-registration,
  first-version behavior, different reconciliation/effect name grammars and
  confusing recognized resources with merely valid schemas.
- ABI Implications: candidate content records only; existing catalog schemas,
  snapshot pins and algorithms are unchanged.
- Downstream Dependencies: lossless read-only snapshot accessors and pure
  reconstruction bridges in 4C.1B; B05/B07 origin authentication/authorization.

## DECISION-4C1A-04 — Resolver Association Representation

- Status: **ACCEPTED**.
- Normative Inputs: StaticComputationResolutionSnapshot, SubflowIrUniverse,
  ResolverSnapshotIdentity and source-association soundness checks.
- Alternatives: sorted digest sets; process-local snapshot serialization;
  complete inventories with explicit source/target/export/edge rows.
- Selected Design: PortableResolverManifest owns caller/context, component
  inventory, ResolverTarget rows and SourceAssociation rows. Target IR content,
  export association and direct/transitive facts retain explicit optional states.
- Rejected Alternatives: equating swapped target/export pairs; inferring a
  missing association from a contract set; defaulting unknown dependencies to
  empty via the current manifest helper.
- Exact Type / Identity: [resolver schema](phase-04c1a-resolver-dependency-manifest.md);
  source key is (caller IrPin, ComputationId), target key is target IrPin, and
  transitive identity includes exact optional target export pin.
- Validation Rules: complete all source leaves in caller/target IR; derive and
  compare direct edges, transitive closure and export pairs; preserve unused
  declared inventory. Positive v1 requires the declared graph known/acyclic.
- Canonicalization: numeric scoped node order, full digest-key order, strict
  target/source uniqueness and tagged options. No graph pruning or dedup repair.
- Failure-closed Semantics: Some dangling assets fails D2; None content/edges
  prevents D8 success. D8 checks availability, declared cycles, actual edges,
  derived closure, then source associations. A declared cyclic negative case
  does not claim a materialized cryptographic hash-cycle IR.
- Cross-language Risks: losing caller scope, target/export pair swaps, conflating
  None with Some([]), traversal-dependent closure and hidden target omission.
- ABI Implications: no wire serialization of existing pointer-bearing snapshot
  identity; candidate rows are raw input, not Checked resolution capability.
- Downstream Dependencies: 4C.1B fresh snapshot construction and differential;
  B03 verifies a complete static projection, not digest-set equality.

## DECISION-4C1A-05 — Canonical Encoding / Closed Decoding

- Status: **ACCEPTED**.
- Normative Inputs: unchanged ACE1, AFIDL1, HashV1 and imported exact primitives.
- Alternatives: ordinary JSON; tolerant normalization-on-read; strict
  schema-directed ACE1 with a closed staged Decoder.
- Selected Design: [encoding](phase-04c1a-canonical-encoding.md) fixes every
  scalar/container, canonical schema blob grammar and comparator.
  [Decoder](phase-04c1a-closed-decoder.md) fixes D0–D10 and earliest rejection.
- Rejected Alternatives: float/double for Int64/decimal; unknown-field ignore;
  missing Option sugar; last-write-wins; sorting malformed incoming sets.
- Exact Type / Identity: finite named candidate shapes plus imported nominal
  shapes. DigestRefs retain domain; plain file checksums stay distinct.
  No Bundle/Assertion/Context HashV1 domain is invented.
- Validation Rules: detect duplicate keys before map insertion; canonical byte
  re-encoding equality; exact widths/pins/graph closure; semantic-canonical
  IR/contract re-encoding equality after formal validation.
- Canonicalization: UTF-8 NFC keys/strings, exact minimal escapes, canonical
  decimal/integer strings and unpadded base64url; unordered sets sort by declared
  typed key, not locale or formatted numeric text.
- Failure-closed Semantics: unknown semantic variants/fields fail D0; all
  noncanonical byte forms fail D1; content/scope errors fail D2–D5. Local limits
  report ResourceLimit, never an empty context or implicit infinity.
- Cross-language Risks: >2^53 integers, arbitrary decimals, Unicode collisions,
  enum discriminants, byte encodings and recursive allocation before cycle checks.
- ABI Implications: only a deterministic numbering/materialization proposal;
  imported ABI/domain tables remain frozen until additive 4C.1B registration.
- Downstream Dependencies: two independently implemented codecs, exact schema
  expansion, canonical goldens and malformed-input production tests.

## DECISION-4C1A-06 — Compiler Semantics Content Tuple

- Status: **ACCEPTED**.
- Normative Inputs: full Specifications/Rules registries, four closed tables,
  all normative source rows, current CompilerSemanticsIdentity and AUDIT-F03.
- Alternatives: version-only identity; 84 RuleIds as a complete proof closure;
  full source/registry/role tuple with explicit attribution profile.
- Selected Design: [CompilerSemanticsManifest](phase-04c1a-compiler-semantics-identity.md)
  includes exact inline source bytes/checksums, both registries, all closed
  tables, ten role declarations, attribution and exact current static projection.
- Rejected Alternatives: a declared implemented flag; binary digest as semantic
  source identity; inventing lowering/runtime definitions from RootExport support.
- Exact Type / Identity: roles have exact version, normative SpecId set,
  required SchemaRef set and role dependencies. Initial static roles bind
  the exact sixteen current semantic sources and complete registered schema set.
- Validation Rules: exactly ten roles, closed acyclic role dependencies,
  exact registry/source/rule row matches; local verifier support establishes
  implementation status. Required static roles must actually be supported.
- Canonicalization: source bytes are opaque/raw; ordered unique source/role/
  schema/rule/edge sets retain every identity component.
- Failure-closed Semantics: content conflicts or unsupported roles/profiles stop
  checking. Profile A uses full-content revalidation with incomplete 84-rule
  attribution; Profile B closure is unimplemented; Profile C is a known
  unsupported unit selector, not a certificate format.
- Cross-language Risks: line-ending changes, counting rules instead of binding
  rows, label-only support, and converting a raw completeness claim into proof.
- ABI Implications: schema registration itself changes the bound full registry
  tuple. Future implementation explicitly supports its post-registration tuple;
  it never edits an old saved label to refresh proof.
- Downstream Dependencies: F03 stays OPEN/P2; closure is mandatory before
  positive subproof cache, portable certificate, rule-complete release policy
  or proof-step attribution. B04/full runtime semantics remain ModelMissing.

## DECISION-4C1A-07 — Historical Semantics Support

- Status: **ACCEPTED**.
- Normative Inputs: exact current checker identity comparison, C04 and immutable
  semantics/source content, with no current historical dispatcher.
- Alternatives: major-version compatibility; implicit latest upgrade; local
  explicit exact current/historical supported profile entries.
- Selected Design: [version-support](phase-04c1a-version-support.md) defines
  VerifierSupportSnapshot outside the Bundle and classifies current supported,
  historical supported/unsupported, unknown and conflicting content.
- Rejected Alternatives: Bundle-selected binary execution; automatic upcast;
  label substitution; a supported semantic label overriding an unknown wire shape.
- Exact Type / Identity: SupportedProfileEntry binds schema version, operation,
  complete expected CompilerSemanticsManifest, required roles and actual local
  implementation statuses.
- Validation Rules: match the full expanded tuple, not SemVer. Historical
  success requires both a supported shape path and a conforming implemented
  verifier path. Only the exact path exercised can be advertised as supported.
- Canonicalization: unique entries ordered by canonical full entry identity;
  no tuple appears as both current and historical.
- Failure-closed Semantics: unsupported/unknown/conflicting tuple mints no
  capability. Explicit ModelMissing lower/runtime roles may accompany a
  static-only result but cannot satisfy a downstream operation.
- Cross-language Risks: version string ordering, silent historical migration,
  support-table shadowing and shape/version mismatches.
- ABI Implications: no new historical schema, dispatcher or upcaster is
  registered. Future UpcasterDefinition is an interface requirement only.
- Downstream Dependencies: 4C.1B may start with current-only support; historical
  paths need independent conformance/qualification. B05/B07 approve a build
  separately; conversion must be pure, deterministic and freshly checked.

## DECISION-4C1A-08 — Origin and Binary Trust Boundary

- Status: **ACCEPTED**.
- Normative Inputs: C04/C05/C06 and separate content/body/implementation/build/
  publisher evidence obligations.
- Alternatives: one verified bool; compiler executable checksum as semantic
  approval; separate typed raw claim/build/diagnostic interfaces.
- Selected Design: [origin model](phase-04c1a-origin-trust-boundary.md) keeps
  PortableProofSubmission sidecars outside static Bundle evidence identity.
  OriginSubject binds exact scope/content; CompilerBuildIdentity binds revision,
  recipe, implementation dependencies and raw binary artifact identity.
- Rejected Alternatives: OriginApproved=true; self-qualified verifier flag;
  trusted evidence inferred from a checksum or a report saying Proven.
- Exact Type / Identity: OriginClaim, RawArtifactIdentity,
  CompilerBinaryArtifactIdentity, BuildProvenanceEvidence and
  QualifiedVerifierIdentity are separate closed **claim** types.
- Validation Rules: resolve subjects and recompute claimed content/payload
  checksums. This establishes raw content consistency only, not issuer trust,
  qualification, signature validity or authorization.
- Canonicalization: sidecar sets sort by explicit full keys; origin subject
  checksums use expanded canonical content (or exact raw source bytes), not
  local IDs alone. Diagnostic presentation does not enter static checking.
- Failure-closed Semantics: static proof may be Proven under an exact untrusted
  context while origin/body/build remain RequiresEvidence. No sidecar changes
  the current checker verdict or selects executable producer code.
- Cross-language Risks: confusing raw SHA-256 with HashV1, checksum with
  authentication, or same source tuple with equivalent binary behavior.
- ABI Implications: interfaces only; no signature wire model, issuer registry,
  root-of-trust service or compiler loader is created.
- Downstream Dependencies: B05 trusted evidence/qualification and B07 release
  policy; B06 separately commits complete evidence when its model exists.

## DECISION-4C1A-09 — Cross-process Revalidation Pipeline

- Status: **ACCEPTED**.
- Normative Inputs: C01; existing contract/IR/dataflow/snapshot/Root/RootRecovery/
  RootExport production functions and private Checked capability boundaries.
- Alternatives: deserialize Checked wrappers; replay submitted report;
  reconstruct exact raw inputs and execute all existing validators afresh.
- Selected Design: fixed D0 decode → D1 canonical → D2 references → D3 content
  → D4 support → D5 context → D6 contracts → D7 IR/dataflow → D8 resolver
  → D9 joint Root/RootRecovery → D10 complete static checker.
- Rejected Alternatives: caching/restoring old process pointers; skipping static
  preflight; independent stale root summaries; using an old report as a certificate.
- Exact Type / Identity: [reconstruction](phase-04c1a-context-reconstruction.md)
  owns each new immutable ContextId arena; Checked contracts, snapshot, Root
  and recovery are fresh and use that same context pointer in the new process.
- Validation Rules: exact deterministic AFIDL projection recompiles schema
  bodies; compare every reconstructed scope and recaptured manifest; normalized
  semantic bytes must equal pinned assets before ordinal-based derivation.
- Canonicalization: compare expanded typed facts, not Rust object bytes/debug
  strings. Semantic, verdict, evidence-byte and presentation equalities are distinct.
- Failure-closed Semantics: any stage failure yields no final checked capability.
  Unknown formal implication remains Unproven; all five static preflight
  categories and nine dimensions stay mandatory. Body evidence stays separate.
- Cross-language Risks: hidden ambient catalogs, incompatible host lifetimes,
  pointer identity reuse, semantic normalization changing coordinates and
  different diagnostic rendering being mistaken for semantic divergence.
- ABI Implications: no serialized CheckedComponentContract, CheckedIrDocument,
  derived summary or CheckedRootExportCompatibility is permitted.
- Downstream Dependencies: 4C.1B bridges/production reconstruction and independent
  differential; sixty new vectors remain Not Executed in this design phase.

## DECISION-4C1A-10 — LockGraph / WorkflowVersion Interface

- Status: **ACCEPTED**, interface boundary only; B03–B07 remain BLOCKING.
- Normative Inputs: C02/C03/C05–C07 and the 4C.0 blocker ledger.
- Alternatives: rename resolver snapshot LockGraph; publish static report as
  WorkflowVersion; retain separate future models with exact projection obligations.
- Selected Design: B01 owns exact static associations; B03 later owns complete
  package/version/adapter/eligibility/semantic dependency locks. B06 future
  parent owns the entire inline Bundle plus independently typed evidence.
- Rejected Alternatives: declaring B03 closed by digest inventory; introducing
  PlanCore/WorkflowVersion/Publication or a runtime artifact in this phase.
- Exact Type / Identity: future LockGraph static projection must equal the
  complete PortableResolverManifest rows and typed pins. Future parent canonical
  value includes the whole Bundle; no parent type/domain/digest is assigned now.
- Validation Rules: a future verifier proves projection equality and complete
  parent binding; static checked capability alone is neither publish permission
  nor proof of executable body/Plan/runtime compatibility.
- Canonicalization: preserve target/export/source/optional edge associations and
  full inline bytes when embedding; no digest-set or report equality shortcut.
- Failure-closed Semantics: absent LockGraph, PlanCore, trusted evidence,
  WorkflowVersion or admission policy remains BLOCKING. No signed publication
  readiness is inferred from design closure.
- Cross-language Risks: adapter mappings or transitive metadata lost in a
  static projection; a parent embedding only convenient proof fragments.
- ABI Implications: no future parent/lock/plan schema or domain is registered;
  additive formalization requires its separately authorized phase.
- Downstream Dependencies: B03 LockGraph, B04 PlanCore, B05 evidence, B06
  WorkflowVersion, B07 admission; this phase stops before all implementation.

## Decision summary

Ten decisions are ACCEPTED for the explicitly bounded static input design.
None is IMPLEMENTED. No unresolved data meaning is hidden behind a placeholder
within that profile. Production bridges, historical implementations, rule-complete
attribution, lowering/runtime and authentication are separately named absent
capabilities, not defaults or implied acceptance.
