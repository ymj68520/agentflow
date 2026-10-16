# Native subject and admission lifecycle

NativeRecoveryGuardSubject has owner_compiler:CompleteCompiler,declaration:CanonicalNativeContractDeclaration,context:ContractProofContextContents,context_roles:NativeGuardContextPartition. CanonicalNativeContractDeclaration contains the complete thirteen-field native ComponentContract and exact canonical bytes. No Checked-only contract pin or synthetic resolver/IR/node is required.

Raw declaration,canonical validated declaration,native evaluation,diagnostic output and whole-contract Checked/rejected are distinct. Canonical failure is genuine earlier rejection,not empty capture. Canonical-valid typed-countermodel negatives can be observed but never fabricate Checked values. If historical raw validation provenance is unavailable,a future authorized adapter must fail or explicitly validate the supplied canonical declaration afresh under its actual context;it cannot reconstruct nonexistent raw history. This registration does not implement that adapter.

