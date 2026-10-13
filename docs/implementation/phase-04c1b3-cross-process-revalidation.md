# Phase 4C.1B.3 — Cross-Process Revalidation

`producer_and_fresh_consumer_revalidate_across_process_boundary` launches the integration-test executable as two distinct child processes. The producer reads only a selected portable fixture, applies any declared mutation, validates D0–D1, and writes canonical ACE1 bytes plus PID/length evidence. It exits before the consumer starts.

The executable-backed harness is `crates/af-portable/tests/root_export_revalidation.rs`; the APIs under test are `PortableProofCodec` and `SupportedPortableStaticInputs::revalidate_root_export`.

The consumer receives only the immutable Bundle file path, then independently runs D0–D10 and writes its own PID and typed semantic projection. The parent checks producer/consumer/parent PIDs differ and compares the actual stage or static projection. No `StaticContractProofContext`, checked contract, checked IR, resolver snapshot, or D9 summary crosses the process boundary.

The six fresh-consumer cases cover a full positive, P30 split-row negative, P31 stale target-export declaration, changed context content, compiler semantics drift, and resolver source-association drift. The positive consumer independently issues a local capability; static-body/runtime evidence boundaries remain unchanged.
