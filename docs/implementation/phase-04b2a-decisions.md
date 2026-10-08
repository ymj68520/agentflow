# Phase 4B.2A Decisions

1. `INVOKE` is resolved only by the existing exact `component-contract`
   DigestRef in the IR contract table. Component identity comes from the
   checked contract, never from a Semantic IR node.
2. Snapshot assembly is explicit and immutable after build. A checked
   contract is accepted only when its exact proof-context object is the
   snapshot context; its semantic digest is recomputed before insertion.
3. Exact public `TypeSchemaRef` equality is the only Component public I/O
   mapping proof. A local primitive, container, or equal local type id is not
   an implicit public schema mapping.
4. Resource requirement ports and ResourceSlotIds are different
   coordinates. Attribute-bearing requirements remain Unproven because the
   current caller IR has no resource attribute descriptor. Aliases are
   explicit and never shrink Effect/Usage bounds.
5. A scoped Effect occurrence is keyed by caller SemanticIrDigest, node,
   source contract digest, and source entry ordinal. Its declared atom is
   never rewritten to the caller's slot coordinate.
6. Flow source/sink mappings are additional references; the original
   information-flow contract, unknown provenance, taints, restrictions,
   authority ceiling and exact partitions remain copied unchanged.
7. `CALL_SUBFLOW` uses the Phase 3D universe and node-scoped Phase 3D
   checker. A separately associated full checked export contract must match
   the derived interface. I/O/R alone never synthesizes other dimensions.
8. Export declaration and body conformance are separate. With no trusted
   complete body proof, the latter is `RequiresEvidence`.
9. Subflow dependency edges are derived from checked IR calls and stored as
   analysis metadata. Missing closure is Unproven; a cycle is a Violation;
   no recursive expansion is performed.
10. AWAIT records the exact signal pin and correlation, has no implicit
    timeout or Effect dispatch, preserves unknown external influence, and
    remains partial without an interaction-adapter contract.
11. ComponentContractDigest remains context-independent. Checked proof
    claims remain bound to the exact context object and cannot be reused
    under a changed context.
12. Only one leaf is resolved. Control-tree algebra, runtime authority,
    admission, plans, journals, reducers, and execution remain deferred; this
    phase does not start Phase 4B.2B.
