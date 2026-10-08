# Phase 4B.0B — Information Flow / Execution Relation Decision

> **DESIGN PROPOSAL** · **NON-NORMATIVE** · **NOT A FROZEN ABI**

## Decision record B0B-D8 — F↔X relation

**Status: ACCEPTED (design only).**

### Problem and existing frozen semantics

F uses nominal confidentiality labels and taints, an aggregate may-influence
summary, `FlowKind`, and sinks. X uses nominal `RegionRef`/`TrustDomainRef`,
closed sandbox levels, and separate execution/residency region sets. Equal
names, same provider, or a descriptor advertisement establish no relation.
The existing `RegionUniverse` compatibility pairs do not map F identities to
X requirements.

### Candidates and choice

* **A — infer from label/region/provider names.** Rejected: names are not
  ontology or compliance evidence.
* **B — add mapping strings/flags to frozen F or X.** Rejected: it changes
  old ABI and loses typed identity.
* **C — explicit typed, versioned relation snapshot injected into analysis.**
  Chosen. The ComponentContract carries F and X declarations; the relation
  catalog is an exact proof input and is not embedded into the contract body.

### Candidate typed relation shape

Exact candidate AFIDL shapes appear in the updated
[ABI proposal](phase-04b0-abi-proposal.md): confidentiality rules map a
nominal label to either a finite exact region-pair set or all pairs in a
closed region snapshot; taint/authority/sink rules use a typed trust-domain
policy (exact allowlist or all domains in a closed trust snapshot) and a
typed sandbox floor (`no_additional_minimum` or an explicit minimum). Trust
policy also has a distinct `no_additional_trust_requirement` case. The
catalog itself carries closed exact region and trust-domain identity sets;
`all_*_in_closed_*_snapshot` refers only to those sets. Empty allowlists are
explicit deny-all sets, not an omitted mapping. An X identity outside the
catalog's closed sets is unrecognized and Unproven.

This is a candidate additive schema
`agentflow.flow.execution.relation@1` (provisional/unregistered). Rule keys
use exact F identities; region and trust identities use exact catalog/name.
The catalog instance is immutable and versioned. Its proof pin includes the
exact catalog SpecificationId/revision and canonical-content checksum (plain
SHA-256, not HashV1). No latest pointer exists. The proof report records the
pin; the ComponentContract digest does not.

### Identity, region, trust, and authority rules

* Every nonempty F confidentiality label and taint must resolve exactly and
  have an exact rule. `InstructionAuthority` is matched by its closed enum
  member; no name-derived elevation or cross-catalog order exists.
* A label rule permits exact execution/residency pairs. X's finite execution
  and residency lists denote all allowed choices; their Cartesian product
  must be a subset of the label's allowed pairs. Every combination must pass,
  not merely one convenient pair. Empty X lists mean unrestricted in that
  dimension; `region_residency=None` is unrestricted in both dimensions. An
  unrestricted choice can be proved only against an explicitly complete
  closed region snapshot and a rule that covers its full pair set. Otherwise
  result is Unproven.
* `all_pairs_in_closed_region_snapshot` is an explicit closed policy over
  the catalog's exact region set, not an open wildcard. Adding a region
  changes the snapshot and requires re-evaluation.
* Taint, authority and sink rules apply their closed trust policy and sandbox
  floor to X. `exact_allowlist` compares full `(catalog, name)` identities;
  `all_domains_in_closed_trust_snapshot` is conditional on that exact
  catalog trust set. Missing X trust/sandbox when the rule requires it is Unproven; an
  explicit incompatible domain or weaker level is a Violation.
* A rule for one external effect sink uses the complete EffectAtom. The
  catalog has no operation-only sink key. A legacy coarse F operation sink
  expands to every matching exact atom in closed ε; each sink whose flow kind
  has an F/X obligation needs an exact rule. An operation-only mapping or an
  Open/Unknown ε is Unproven. `EffectFlowPartition` may separately narrow F's
  permission set, but no selector or qualifier is inferred from the operation
  name.
* Missing label, taint, authority, region, trust, or sink mapping is Unproven.
  A known exact mapping conflict is a Violation. A relation snapshot not
  supplied or not pinned is Unproven.

### Static verdicts and explicit evidence layers

| Layer | What it can establish | What it cannot establish |
| --- | --- | --- |
| Static relation catalog | Exact F/X mapping facts under one named immutable snapshot | That the catalog publisher is trusted or the mapping is legally authoritative |
| Static executor profile match | The supplied descriptor satisfies X and the mapped F constraints | That the descriptor describes a real/current worker |
| Trusted worker attestation | Evidence about worker identity/profile and measured properties | Runtime enforcement unless an enforcement proof is also present |
| Runtime enforcement | That placement/data handling obeyed policy during execution | A retroactive change to ComponentContract semantics |
| Policy authorization | A separate decision to allow this Run/action | Any grant inferred from static compatibility |

Static F/X `Proven` means only that the declared constraints are consistent
under exact relation/universe inputs. Missing required worker attestation is
`RequiresEvidence` in the separate admission report. Neither result means
Effect authorization, safe execution, or policy approval.

### Compatibility, digest, and migration

The ComponentContract digest includes exact F and X plus any exact
EffectFlowPartitions. It excludes the relation catalog and profile match
results. Comparisons require the same F/X relation snapshot; otherwise the
relation change has no defined monotonic order and compatibility is Unproven.
The existing `information-flow.afidl`, `execution-requirements.afidl`, their
TypeSchemaDigests, and all prior IDs remain unchanged. This is an additive
analysis relation, not a replacement F/X schema.

### Counterexamples and residual risk

| Input | Expected | Reason |
| --- | --- | --- |
| Exact confidential label maps to every pair in X's closed region set | Proven under the relation snapshot | All permitted placements satisfy the mapping |
| X permits one execution/residency pair outside the label rule | Violation | X's declared eligible set includes a prohibited pair |
| Label or taint has no rule in the supplied catalog | Unproven | No legal inference from names |
| Trust domain string resembles the publisher of an F label | Unproven | Provider/name similarity has no identity relation |
| Static match is Proven but no worker attestation is supplied | Static F/X Proven; admission RequiresEvidence | Static profile is not trusted-worker evidence |
| Same `data.write` operation has selectors 1 and 2, relation only maps selector 1 | Unproven for selector 2; selector 1 may be proven | Exact EffectAtom identity is required |

The relation catalog itself and publisher-provided declarations remain trust
assumptions. Authenticating the relation source and enforcing the result are
separate P3 obligations.
