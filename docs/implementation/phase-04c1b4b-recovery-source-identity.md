# Phase 4C.1B.4B — recovery source identity

Starting HEAD: `c5b49850d9ba0dce75255f152498468fe8e93762`. This phase targets only 16 original D9-Recovery cases; the 22 Control baseline is retained. Closure depends on the recorded committed gates. D10 independent values are Pending for all 78; B01/B02 Closure and B03–B07 remain BLOCKING. No later phase or runtime/publication capability is created.

Identity is `(caller SemanticIrDigest, source ComputationId, exact ComponentContractDigest, forward entry ordinal, complete RecoveryActionIdentity)`. Action identity includes full Forward atom, action kind and full action atom, including every qualifier/selector. Equal pins or atoms at distinct call sites remain separate. Ordinals are declaration-derived positions in canonical effect rows, not action-array indexes. Input identity binds exact bytes/checksum; resolver associations are recaptured from the exact declared caller.

| CaseId | Input SHA256 | Root node | Source nodes |
| --- | --- | --- | --- |
| portable-root-export-revalidation/d9-recovery-outer-retry-1 | 8d13268ef50b23a3a098397a94e9c3e5960072cd0217299378efd483e486ede7 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-retry-2 | fa8a4192b6a3d3efa56aa9e5308fafe761d3cc0ed7b89f6c3f44642f45610481 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-retry-3 | 0cd50d75c2db0d01e25088b7204d3adf723e171392185c06cb0caf4c1f8f5c26 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-retry-4 | 95926620d76d330f0c09a16f9db63b1d350e440b44151ad9eef79bbab7d53e44 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-iterate-1 | 99587870cde231c04e849af4d2390563ffe1b980c725e7ca69515ad539c21342 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-iterate-2 | 468989a02ce404bce049348718b1e7524b17bd03b4c8549f8d8f509ef03256a3 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-iterate-3 | e41403b8151c74025a78c34ebe05a83b7175b89244495477286276df587e3386 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-outer-iterate-4 | a8d3d22fc5a21b704da99ab539495cdc46f783e4faf17636f71da4a350591bb4 | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-retry-2-times-iterate-2 | 6047a07660267947431fcb3734cb68639ac44e7e9484c275df296afad8bf527b | 1 | [4] |
| portable-root-export-revalidation/d9-recovery-retry-2-times-iterate-4 | aa42bdc7adf25a6289c6a6e228842ffee4562d0980851f84ccc8a27fde0d503d | 1 | [4] |
| portable-root-export-revalidation/d9-recovery-retry-3-times-iterate-2 | 8776661704b8de07f744ccaebc8d49c0c9731526a57ea931e066858a0f06e2e2 | 1 | [4] |
| portable-root-export-revalidation/d9-recovery-retry-3-times-iterate-4 | 51cbcbf06cb4790eae829bbab40f35d405f060db8960f54d3cf08934fa48773b | 1 | [4] |
| portable-root-export-revalidation/d9-recovery-local-retry-inside-compensation-scope | 605f6df85947bec424ad865e2744d26c49c1a4d282427106b3460919f49c2f4b | 1 | [3] |
| portable-root-export-revalidation/d9-recovery-two-sequence-actions | f38aa2200d603921006a5bd379779d6de826340bd5cde6a51ff546ab7a4af120 | 1 | [3, 5] |
| portable-root-export-revalidation/d9-recovery-race-retains-loser-action | efe7989f995aae22df72799018d55e50cfa0affe3a1bfcf9aea9c36da937f5dc | 1 | [3, 5] |
| portable-root-export-revalidation/d9-recovery-parallel-keeps-both-actions | 385c96d2a97a79d649ddc4755e6fbb393cef7719256848caf461ab9055ecc6aa | 1 | [3, 5] |

