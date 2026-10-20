# Windows x64 compatibility — Pending

Installed Rust targets observed: x86_64-unknown-linux-gnu and
aarch64-unknown-linux-gnu. x86_64-pc-windows-gnu is NOT installed. Neither the GNU
Windows linker nor LLVM Windows linkers were available on the observed PATH.
No target/toolchain/linker was installed. Cross-build/check: Not Executed.
No Windows build, launch, ABI equivalence or Windows Verified claim is made.

The locked workspace's dependencies were resolved offline; this is not a target
build. The lexical source audit found no direct std::os::windows/windows-sys/MSVC
binding in crate Rust code/manifests under its declared scope. This is not proof
that native/conditional dependencies or runtime behavior are Windows-compatible.
27 platform-sensitive source hits were recorded, mostly subprocess Python calls
and temporary-directory/process fixtures. A future target gate must review the
complete dependency graph and real compiler/linker diagnostics without dropping tests.

Research target: x86_64-pc-windows-gnu. GNU artifacts must never be described as
MSVC ABI-equivalent. If MSVC SDK/native dependencies emerge, report the exact
binding and retain a distinct release path; no Wine or Windows VM is authorized.

Architecture boundary for future adaptations: platform-independent core with
Linux and Windows adapters for paths, processes, temporary directories, permissions,
environment and native APIs. This phase does not invent adapters or change Rust
semantics. ACE1, HashV1, Canonical DAG and Source Identity remain unchanged.
