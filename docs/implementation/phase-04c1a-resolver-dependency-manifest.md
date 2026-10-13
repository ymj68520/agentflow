# Phase 4C.1A — Exact Resolver Association Schema

All candidate types/rules are **DESIGN ONLY / NON-NORMATIVE / NOT REGISTERED
ABI / NOT IMPLEMENTED**. This is the current static resolver's portable input
projection, not MODEL-B03 LockGraph.

## Complete records

~~~text
PortableResolverManifest = record {
  caller_ir: TypedContentRef;
  context_id: ContextId;
  components: set<TypedContentRef, TypedContentRefKey>;
  subflow_targets: set<ResolverTarget, target_digest_key>;
  source_associations: set<SourceAssociation, (caller_digest,source_node)>;
}
ResolverTarget = record {
  target_semantic_ir_digest: IrPin;
  target_ir_content: option<TypedContentRef>;
  exported_contract: option<TypedContentRef>;
  direct_dependencies: option<set<IrPin, DigestKey>>;
  transitive_dependencies: option<set<SubflowTargetIdentity, target_digest_key>>;
}
SourceAssociation = record {
  caller_ir: TypedContentRef;
  source_node: ComputationId;
  target: SourceAssociationTarget;
}
SourceAssociationTarget = variant {
  invoke: TypedContentRef;
  call_subflow: ResolvedSubflowAssociation;
  await_signal: SchemaRef;
}
ResolvedSubflowAssociation = record {
  target_semantic_ir_digest: IrPin;
  exported_contract_digest: option<ContractPin>;
}
~~~

caller_ir and target content use the IR root type/domain; components and
exported_contract use the contract root type/domain. Source nodes are uint32
positive and scoped by their caller IR, not by contract or target name.

Every field is required. None in direct/transitive dependencies means
unavailable/unknown; Some([]) means a known empty set. None target_ir_content
retains an incomplete input declaration and cannot construct a positive
snapshot. None exported_contract is exact association absence, not an empty
contract. Missing referenced asset rows fail D2.

## Derived versus declared facts

| Fact | Reconstruction rule |
| --- | --- |
| Component inventory | Every exact listed contract checked under context_id; duplicate pins rejected before builder insertion |
| Target inventory | Every Some target IR checked in the same schema/dataflow scope and inserted via SubflowIrUniverse |
| Direct dependencies | Derived from canonical target CALL_SUBFLOW contract references; compare exactly with Some declared set |
| Transitive dependencies | DFS on all declared target edges; complete reachable set excluding the starting target, with each target's exact optional export |
| Target/export pair | Explicit input association; fresh builder recomputes contract digest and retains target key |
| INVOKE source association | Derive source contract pin from that caller's exact ContractRef; require matching component inventory |
| CALL_SUBFLOW source association | Derive target pin and current target export association; compare both |
| AWAIT source association | Retain exact signal TypeSchemaRef; does not become a component or trusted external provenance |

Source associations cover all leaf nodes in the root and declared target
documents. Extra or missing source rows, incorrect computation kind, ambiguous
coordinates or a stale contract pin are rejected; no same-name fallback is
allowed. Their full caller/node keys preserve different occurrences of a
shared contract.

Declared inventory can include targets/components not reachable from the root:
the current resolver identity includes all of them. Their content and known
dependency facts are retained. This v1 positive profile requires the full
declared target graph to be known and acyclic; an unsupported unused graph is
not silently pruned to recover a capability.

## Association identity

~~~text
(target A, export E1), (target B, export E2)
is different from
(target A, export E2), (target B, export E1)
even if sorted export digest sets are identical.
~~~

The portable canonical rows bind these pairs. Current identity reconstruction
uses the generated ResolverSnapshotIdentity, with direct_dependencies
Some(empty) retained and optional exported_contract_digest preserved.
Caller SemanticIrDigest is independently checked in the assertion/root/source
rows. The manifest never provides a trusted resolver identity object.

## Errors and stage distinctions

| Condition | Earliest specified stage / outcome |
| --- | --- |
| Identical duplicate target/source row | D1 DuplicateSetMember |
| Distinct rows with duplicate target/source identity or conflicting export | D2 DuplicateIdentity / ConflictingAssociation |
| Out-of-width/malformed scalar IDs | D0–D1 typed/canonical rejection |
| Wrong digest domain or zero/dangling typed IDs | D2 reference rejection |
| Referenced Some content absent from asset table | D2 MissingReferencedContent |
| Target content pin disagrees with canonical bytes | D3 ContentIdentityMismatch |
| None direct/transitive facts or target content | D8 IncompleteResolverInputs; Unproven, no snapshot/capability |
| Declared edge set differs from checked target IR | D8 DependencyAssociationMismatch |
| Dependency cycle or unreachable referenced target | D8 UnsupportedDependencyCycle / MissingDependencyTarget |
| Stale INVOKE pin or wrong source kind | D8 SourceAssociationMismatch |
| Complete valid inputs | Fresh snapshot; new root/RootRecovery; normal RootExport verdict |

Do not use the current ProofContextManifest helper's unwrap_or_default as a
wire rule. A positive target has Some direct facts after deriving them; its
empty set is supported only because the actual checked document has no edges.

## Interface to LockGraph and parent artifact

B01 owns exact IR/contract/scoped association facts sufficient for static
reconstruction. B03 additionally owns package origin, version constraints,
adapter and semantic metadata closure, resolver eligibility/yank/revoke and
locked rebuilding. Those are absent from this manifest and remain BLOCKING.

A future LockGraph verifier must prove its static projection equals this
manifest by complete rows and typed references. It cannot assert equality by
matching contract digest sets. A future WorkflowVersion parent must commit
the complete inline Bundle, including these associations. No current
ResolverSnapshotIdentity serialization is a LockGraph digest or stable ABI.
