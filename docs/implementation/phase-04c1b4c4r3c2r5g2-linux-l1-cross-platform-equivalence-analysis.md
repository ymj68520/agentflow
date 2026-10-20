# R5G2-LINUX-VERIFICATION-EQUIVALENCE — L1 adjudication

This is a review topic, **not a registered RuleId or a closed finding**. The
user's Linux-native decision is retained: no Windows VM, Wine or compatibility
layer is proposed. Primary outcome C concerns acceptance policy; dependency and
resource-execution prerequisites also remain incomplete (D). No production
semantic contradiction (B) was observed because no Rust gate was executed.

## Existing authority and the exact missing bridge

The authoritative inputs are the unchanged P0
`phase-04c1b4c4r3c2r5g2r4p0-historical-gate-contracts.json` groups1..8 and
`...-frozen-g3-gate-contract.json`. For each group,
`original_contract.ToolchainIdentity.host` records
`x86_64-pc-windows-msvc`; `BuildProfile` and `CommandAndFlags` retain the original
profiles, targets/commands and historical oracles. The inherited
`original_contract.PermittedEquivalence.currently_granted` is false. Its proposed
host/path/build-policy substitutions were awaiting separate approval, not
already operational equivalence. P0's environment-equivalence audit likewise
requires Windows x64/MSVC qualification and explicit path/resource approval.

The L1 instruction authorizes **additional Linux diagnostic execution**, not a
change to those original Windows gate requirements. Minimal counterexample
(hypothetical, not an actual test pass): a result at historical HEAD
abc91d533b4d62e91226722e1d1fcaa30bd00883, Debug2, all512 CaseIds, but built for
`x86_64-unknown-linux-gnu` cannot satisfy the frozen MSVC host/artifact condition
without an approved acceptance mapping. Even identical structured outputs do
not prove MSVC ABI/process/filesystem behavior. No current registered rule was
found granting that replacement. Original Windows history therefore remains
**0/8**, original frozen Windows G3 **Not Executed**.

## Semantic invariants versus environmental observations

Registered **AF-ACE1-0002**, source `spec/encoding/ace1-grammar.md`, anchor
`ace1-utf8-canonical-bytes`, defines exact UTF-8 canonical bytes; the grammar also
specifies machine numeric text and NFC strings. Registered **AF-DIGEST-0001**,
source `spec/digest/hash-v1.md`, anchor `digest-formula-exact-prefix`, fixes the
framed hash formula; **AF-DIGEST-0003** fixes SHA-256. These definitions do not
depend on a Windows machine-word layout. They support requiring the **same**
canonical bytes, ordering, domains and hashes on a Linux implementation. They
do not waive an acceptance gate's frozen toolchain or OS conditions.

The exact Git objects and all ten checkouts are verified. The nine historical
Git/raw manifests match the previously frozen source manifests; this includes
their exact Expected/Oracle bytes. Current Windows worktree raw bytes are a
different evidence universe: **222 paths differ from current Git/Linux raw
bytes, including193 JSON paths and33 .expected.json paths**. Streaming comparison
observes CRLF-to-LF correspondence for all222; no old file was rewritten. This
is not a general normalization permission and not proof that every raw-byte
gate or filesystem-dependent test is equivalent. Per-path raw hashes, lengths,
Git identity and correspondence evidence must remain separate in a future
profile. Canonical payloads themselves must never be rewritten to obtain PASS.

Pure algebra/codec/model expectations can be loaded unchanged from the frozen
Git snapshots, but their full Linux Rust Actual results do not yet exist.
Actual additional evidence here is only the110 standalone R3 Python static-data
vectors. P0/P1A pure data tools are potential cross-platform diagnostics; the
attempted combined suite stopped on an unavailable Python runtime dependency,
not on an AgentFlow semantic assertion. No portability conclusion is inferred
for their unexecuted105/37 cases.

## Platform-sensitive obligations

The census records exact snapshot/path/line, frozen Cargo declaration labels
and literal test functions. Absence of a marker is only a portable candidate,
not a transitive-dependency or cfg-expanded coverage proof. Slice `.windows()`
is not classified as a Windows API.

- `tools/run-production-input-graph-safe.py:13` requires `os.name=='nt'`;
  line23 loads kernel32 through WinDLL. This original protection harness cannot
  be run natively on Linux. A distinct qualified Linux protector is required;
  the old one is neither edited nor emulated.
- Rust integration files such as
  `crates/af-computation/tests/native_guard_staging.rs:170` and
  `.../production_input_graph.rs:716` invoke `python`. That alias exists on the
  server, but its3.12 interpreter differs from the Windows3.14 audit runtime.
  Interpreter identity, pipes/encoding/exit behavior must be bound explicitly.
- `crates/af-spec/src/tables.rs::rule_area_format_is_enforced` uses
  `std::env::temp_dir()` and writes fixture files. Future native runs must set
  an approved TMPDIR inside the dedicated project, and qualify path/case/sort,
  Unicode and process differences. No TMPDIR/test process was started here.
- Native-library/linker/conditional dependency closure, doc-test compilation,
  actual cfg/feature target rosters and process semantics remain unqualified.
  Linux1.91.1/LLVM21.1.2/GNU is not the cached Windows1.92/LLVM21.1.3/MSVC
  reference, and the latter's full binary trust chain was not established
  merely by its cached version report.

Windows Job committed-memory limits, Linux cgroup charged memory, RLIMIT_AS
virtual address space and available physical/commit metrics are different.
No numerical substitution is treated as equivalence. The Linux kernel's
[cgroup v2 documentation](https://www.kernel.org/doc/html/latest/admin-guide/cgroup-v2.html)
defines hierarchical CPU/memory control; the observed writable user-service
subtree is an indication, not a tested compiler-protector certificate. No
controller settings were changed, especially not limits on the shared user
service containing unrelated work.

## Required next decision

A separately reviewed Linux conformance/acceptance profile is needed if Linux
is to **replace**, rather than supplement, the Windows historical requirements.
Its version/registry/rule ownership must be adjudicated and approved explicitly;
L1 allocates none. It must keep all6337 historical declared CaseIds, frozen
G3's154/26/59, complete target/doc-test/oracle/denial scope and original debug
profiles. Unsupported Windows obligations remain visible, not deleted or
silently removed from the denominator.

Each side must first satisfy an independently frozen schema/CaseId/field/output
roster. Only then compare exact values, canonical bytes and hashes; identical
omissions on both platforms must fail. Raw artifacts retain native format and
toolchain provenance; Linux ELF is never relabelled MSVC/PE. Any permitted path
or nonsemantic difference needs an explicit, versioned field-level rule. The
next registration must state whether it creates additional evidence, an
approved alternative, or formally supersedes named Windows requirements. Until
then, those Windows gaps remain open regardless of Linux diagnostic successes.
