# T3 admission gates — no implementation acceptance

Start/source baseline: b121ab716bf4e87e33e7433b2b9dc3407699d327, clean. Production/frozen Git surface remains identical (crates/,spec/,Cargo.toml,Cargo.lock). New files are admission documentation, a read-only audit tool and its census result; no formal source/rule/Expected change.

Actual diagnostic: final standalone probe exit0; identity control Proven, name-mutated replacement Unproven, exact same checked context, AF-CONTRACT-0020, no supplied root/export and no typed capture. Initial probe compile exit101 due to omitted sha2/hex fixture dependencies is separately retained; only its ignored manifest was corrected. Actual source-audit exit2 means FORMAL_AMENDMENT_REQUIRED, not a passing T3 gate. First audit attempt failed its lexical inheritance count because an impl header was included; the filter was corrected, then the read-only audit completed with the explicit blocked result.

Fresh current targeted checks completed against unchanged production at b121ab7, with only this admission documentation/auditor added. Actual commands/results:

| Command | Actual result |
| --- | --- |
| cargo fmt --all -- --check | exit0 |
| cargo clippy -j2 --all-targets --all-features -- -D warnings | exit0 |
| cargo test -p af-contract --test component_contract_conformance compatibility_ -- --nocapture | exit0;3 test functions passed,18 filtered; includes recovery guard compatibility |
| cargo test -p af-computation --test edition2_staging | exit0;3 functions passed; native matrix/direct denial |
| cargo test -p af-portable --test edition2_staging | exit0;3 functions passed; actual Portable admission matrix included;80.50s |
| cargo run -p af-spec --bin spec-check -- check | exit0;46 specs,436 rules,435 reference-covered,1 exception,0 missing,775 ABI,17 AFIDL,0 cross-reference errors |
| python tools/audit-g01-t3-admission.py | exit2;expected explicit blocked admission, NOT a conformance pass |

Existing51 stable staging CaseIds are32 registration+10 native+9 Portable under those suites; supplemental35 direct declaration observations remain separately classified. All these are existing staging evidence, not new T3 seed conformance. No previous phase's logs are treated as fresh results. Native test duration20.15s;contract subset0.46s. No test was disabled or modified for this run.

Read-only audit was repeated with exit2 and byte-identical JSON SHA256 `98b1bd63c0b007d7053959a319ad1f9dd9554b2d46e585c637c291970a1832aa`;this is audit reproducibility, not the all-generator double-round gate. Toolchain freshly inspected:rustc1.92.0 commit ded5c06cf21d2b93bffd5d884aa6e96934ee4234,cargo1.92.0,Python3.14.2,Windows x86_64-pc-windows-msvc. All18 Markdown documents passed local-link/fence checks;ordered final report has exactly items1–40;machine census65/27 verified.

Not executed in this blocked phase: new T3 Expected/coverage/comparator/source-origin suites; fresh exact legacy512/D9 full-field/comparator regression; complete cargo test --all; all-generator original-directory double round. Prior successful C3 results remain historical evidence, not current execution claims. The generator inventory remains31 because the audit tool is not gen-*.py. These missing acceptance gates plus missing capture forbid Outcome A; no isolated generator substitute was run.

Registration remains46/436; mechanical435 reference-covered/1 exception/0 missing does not imply full G01 conformance. ABI775/AFIDL17/HashV1 domains14 unchanged. No full code-gate HEAD exists for a T3 implementation. Final documentation commit/clean-tree status is delivered separately.
