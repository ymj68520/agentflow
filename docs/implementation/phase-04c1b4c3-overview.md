# Phase 4C.1B.4C.3 — pre-registration separation review

Start HEAD: `abc91d533b4d62e91226722e1d1fcaa30bd00883`;initial worktree clean. This review precedes any registry edit.

## Separation decision

T1/T2 can be staged safely if the admission boundary lands before the additive registry transaction. `root_export.rs` has one private `CheckedRootExportCompatibility { report }` construction site;the public analyzer/checker and Portable D4 must all consult implementation-owned,distinct Registered/Supported/Activated states. Edition2 will be Registered/NotImplemented/NotAuthorized,with release NotAuthorized. A matching complete identity alone is not support. No input/configuration can enable it. Failure is typed admission metadata/error,not an implemented G01 normalizer or a static Violation.

The old checker currently recompiles whole registry bytes into `current_compiler_semantics_identity`;without this new guard,it could accept a freshly reconstructed identity after an additive registration. Therefore registration alone is unsafe. The guard and registration are one committed staging boundary;no intermediate registry-only commit is allowed. The substantive old proof algebra is not adapted in T1/T2.

Portable D0–D3 retains the closed Bundle schema. D4 validates exact edition content then refuses incomplete Edition2 support. The direct in-process constructor independently refuses unsupported/nonactivated editions. An unsupported legacy diagnostic report is explicitly tagged and cannot be a complete G01 view or mint Checked.

## Regression and generator feasibility

Historical82 RootExport-sensitive and201 Portable cases execute from an exact fresh archive of the start commit,with a separate build directory/toolchain record. The229 algebra cases remain in the staged build. Current tests must explicitly route old identity-sensitive assertions to that retained code,not relabel their Expected as Edition2.

Historical generators continue executing in the original directory against frozen Edition1 registry **input bytes** and unchanged old sources;their old output bytes must remain identical. New Edition2 data has a separate namespace/generator. No generator may silently read the new registry then overwrite old fixtures. The original-directory all-script double-round gate is mandatory;isolated runs never replace it.

Formal registry coverage resolution and executed staging conformance are separately reported. T1/T2 tests cover actual descriptors,source identity,admission and no-capability behavior;they do not execute the48+16 historical design laws or claim full G01 derivation/encoding conformance. T3–T8 remain unimplemented and blocked. No ABI/AFIDL/HashV1 changes are authorized.
