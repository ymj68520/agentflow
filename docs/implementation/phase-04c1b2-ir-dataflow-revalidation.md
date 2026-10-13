# Phase 4C.1B.2 — Fresh Semantic IR and Phase 3C

D7 decodes caller and target IR assets against the D5 module set and exact
SemanticIrDocument root schema. It invokes `SemanticIrDocument::check` against
the reconstructed global TypeSchemaUniverse, followed by
`CheckedIrDocument::analyze_dataflow` for every checked document.

Only after both passes succeed does D7 recompute canonical Semantic IR bytes
and SemanticIrDigest. The bytes and digest must equal the original asset and
pin exactly; normalization is never written back to the portable input.
Target insertion into `SubflowIrUniverse` independently recomputes the same
digest. The formal `check_subflow_interfaces` call consumes the already-fresh
Phase 3C result for each checked IR; it does not substitute for, or claim to
rerun, dataflow analysis.

The portable diagnostics are not evidence. All proof facts in the output
projection are produced from fresh in-process validators.
