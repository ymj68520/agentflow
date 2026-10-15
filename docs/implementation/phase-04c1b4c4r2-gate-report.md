# R2 gate report

Code-gate HEAD: `635616cea0916429161a5a353dc9a4090834589d`. Starting HEAD: `51239a0045d7a6fb274bb24cdc8130f2346bf085` (clean). The final commit adds verification receipts/documents only; its exact full HEAD and clean status are recorded at delivery rather than a self-referential Git hash in this file.

## Required command sets

Executed independently per source tree: cargo fmt --all -- --check; cargo clippy -j2 --all-targets --all-features -- -D warnings; cargo test --all -j2; cargo run -p af-spec --bin spec-check -- check. Toolchain: rustc1.92.0,cargo1.92.0,Python3.14.2,Windows x86_64-pc-windows-msvc.

Edition1 exact abc91d533b4d62e91226722e1d1fcaa30bd00883: all four commands exited0.512 historical stable CaseIds,0 failed/0 skipped;382 reported cargo test passes in87 result groups (including actual child-process test invocations),0 ignored. Historical cross-process child invocations deliberately filter other test functions (aggregate144 filtered);this is not144 skipped manifest cases or ignored tests. Spec-check43/404/403/1/0,ABI775/AFIDL17/xref0. The1732 exact source blobs match before/after. Full logs: target/phase-04c1b4c4r2-edition1-tests.log and -edition1-spec.log. Exact tree/build separation is in legacy-isolation.md.

Edition2 exact 51239a0045d7a6fb274bb24cdc8130f2346bf085: all four commands exited0.51 original staging CaseIds (32+10+9),0 failed/0 skipped;335 cargo test functions in71 result groups,0 ignored.35 supplemental direct observations are separate. Spec-check46/436/435/1/0,ABI775/AFIDL17/xref0.1811 exact source blobs match before/after. Logs: target/phase-04c1b4c4r2-edition2-tests.log and -edition2-spec.log.

Current successor full code-gate run at635616cea0916429161a5a353dc9a4090834589d completed: all four commands exited0.336 reported test passes in71 groups,0 failed/ignored/filtered.173 independent registration/shape/admission CaseIds passed, not typed capture;35 supplemental direct root invocations remain separate. Native6 functions completed30.10s;Portable1 function with10 inputs completed99.51s. Spec-check50/452/451/1/0,ABI775/AFIDL17/xref0. Current logs: target/phase-04c1b4c4r2-{clippy-final,current-tests,current-spec}.log. A prior overlapping-source test failed101 and is preserved separately; it is not used as the final code gate. Fresh diagnostic reference probe exited0 and confirmed all frozen source behavior branches without creating trusted facts.

## Original-directory generator receipt

Required exact cwd E:/Projects/agent; current actual inventory32 tools/gen-*.py. Complete Round1 then complete Round2, every script exits0, every nonignored path and SHA256 equals the original baseline, including all preexisting implementation/fixtures/docs at round start. The ignored helper records full maps plus added/removed/changed lists and timings. No isolated directory substitute, retries inside a round, swallowed failure or splicing is allowed.

Registration-docs pair completed with process exit0:32/32 scripts per round;1866 nonignored files;baseline=Round1=Round2,0 added/removed/changed. Round1:190.43s;Round2:166.89s. Its complete original-directory maps/log are preserved under target/phase-04c1b4c4r2-registration-generators*. These are separate complete rounds,not spliced. No generator I/O failure occurred in this phase.

The [tracked audit receipt](phase-04c1b4c4r2-gate-receipt.json) binds all four source hashes,registry hashes,exact source HEADs,test/spec logs,old byte audit and that complete pair. The receipt reader initially rejected historical child-process filtered-test output as if it were ignored tests;its read-only assertion was corrected to distinguish actual ignored0 from deliberate child filtering. No underlying test or Expected changed. This administrative receipt failure is not a generator or cargo failure.

Only gate/final receipt documentation and the new audit JSON are finalized after that pair;production/normative/fixture/generator inputs remain untouched. A further post-commit complete consecutive pair runs with the FINAL documents/receipt present,without ANY nonignored file edits. Its actual delivery status/full HEAD are reported to the user and its fresh maps/log remain target/phase-04c1b4c4r2-generators*. This avoids a self-referential receipt hash while verifying final tracked bytes as well. The two complete pairs are independent,not combined into one pass.

## Evidence meaning and stopping rule

Formal exact identities and source SHA values are in registration-manifest.md and compiler fixtures, not labels alone. Current-only denial is documented in capability-denial.md; registered shape success is not CompleteAndValid. R1 semantic48 remain NotExecuted; D9 historical38 remain separate; D10 full-field incomplete. F01/F02 CLOSED,F03/F04 OPEN/P2,G02 OPEN/P1,G03 OPEN/P2,B01/B02/B03–B07 BLOCKING.

Outcome A — COMPLETE / REGISTERED STAGING: the current full gate,frozen-row/byte audit and registration-docs original-directory double round have actually passed. Delivery additionally requires the post-commit final-tree pair and fast fmt/clippy/spec/row-byte/documentation checks. T3-REG-01 is FORMAL BINDING REGISTERED only;capture/normalizer/complete identity remain NOT IMPLEMENTED and positive successor proof NOT ACTIVATED. Stop before R3/T4–T8. No push.
