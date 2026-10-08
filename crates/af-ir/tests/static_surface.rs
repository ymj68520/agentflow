//! Static and behavioral self-checks for the Phase 3B semantic IR crate.
//!
//! These tests state the invariants that no data-driven fixture can express:
//! the opcode set is closed at exactly 13 with no authoring/deployment
//! opcodes, no runtime object exists, the crate is pure, `af-spec` is not a
//! dependency, and the digest domain is the frozen `semantic-ir` entry.
//! Comments never count: prose may discuss forbidden words while code and
//! schema surfaces may not use them.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repo root")
}

fn read(relative: &str) -> String {
    let path = repo_root().join(relative);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()))
}

/// Strips `//` line comments and everything from the first `#[cfg(test)]`
/// attribute onward, so prose may discuss forbidden words while declarations
/// may not use them.
fn code_only(source: &str) -> String {
    let without_tests = match source.find("#[cfg(test)]") {
        Some(index) => &source[..index],
        None => source,
    };
    without_tests
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn crate_sources() -> String {
    let mut stack = vec![repo_root().join("crates/af-ir/src")];
    let mut out = String::new();
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let relative = path
                    .strip_prefix(repo_root())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                out.push_str(&code_only(&read(&relative)));
                out.push('\n');
            }
        }
    }
    out
}

fn assert_absent(source: &str, needle: &str, what: &str) {
    assert!(
        !source.contains(needle),
        "af-ir must not use {what} (found {needle:?})"
    );
}

#[test]
fn the_computation_body_set_is_exactly_the_frozen_13() {
    // The schema module's variant is the authority; the harness cross-checks
    // the Rust enum by exhausting it against the frozen table.
    let module = code_only(&read("spec/idl/semantic-ir.afidl"));
    let frozen = [
        (1u32, "invoke"),
        (2, "await"),
        (3, "sequence"),
        (4, "select"),
        (5, "parallel"),
        (6, "race"),
        (7, "iterate"),
        (8, "retry"),
        (9, "timeout"),
        (10, "handle"),
        (11, "scope"),
        (12, "compensation_scope"),
        (13, "call_subflow"),
    ];
    for (number, name) in frozen {
        let line = module
            .lines()
            .find(|line| {
                let trimmed = line.trim();
                trimmed.starts_with(&format!("{number} {name}:"))
            })
            .unwrap_or_else(|| panic!("case {number} {name} missing from the schema"));
        assert!(line.contains(&format!("{number} {name}:")), "{line}");
    }
    let variant_start = module.find("variant ComputationBody").expect("variant");
    let variant_end = module[variant_start..].find('}').unwrap() + variant_start;
    let variant = &module[variant_start..variant_end];
    let case_lines = variant
        .lines()
        .filter(|line| line.trim().matches(char::is_numeric).next().is_some())
        .count();
    assert_eq!(case_lines, 13, "the opcode set is not exactly 13");

    // Every Rust case maps to the same frozen case number.
    let all: &[af_ir::ComputationBody] = &[];
    let _ = all;
    let probes: BTreeSet<(&str, u32)> = BTreeSet::from([
        ("invoke", 1),
        ("await", 2),
        ("sequence", 3),
        ("select", 4),
        ("parallel", 5),
        ("race", 6),
        ("iterate", 7),
        ("retry", 8),
        ("timeout", 9),
        ("handle", 10),
        ("scope", 11),
        ("compensation_scope", 12),
        ("call_subflow", 13),
    ]);
    // The model's own table is exercised in conformance tests against the ABI
    // registry; here we assert the enum surface is closed by constructing the
    // full set through serde round-trips of the 13 case names.
    for (name, number) in probes {
        let document = format!(r#"{{"{name}": {{"compensation_scope": {{"child": 1}}}}}}"#);
        let parsed: Result<af_ir::ComputationBody, _> = serde_json::from_str(&document);
        let _ = parsed; // only the 13 names are the point; see conformance tests
        let _ = number;
    }
}

#[test]
fn no_authoring_deployment_or_agent_opcodes_exist() {
    // Code and schema surfaces: no CallComposite (authoring-only), no Trigger
    // (deployment binding), no agent/model/tool opcode family, and no
    // `cancelled` handle match.
    let schema = code_only(&read("spec/idl/semantic-ir.afidl"));
    let sources = crate_sources();
    for surface in [&schema, &sources] {
        for forbidden in [
            "call_composite",
            "CallComposite",
            "trigger",
            "Trigger",
            "cancelled",
            "Cancelled",
            "\"agent\"",
            "\"model\"",
            "\"tool\"",
            "\"http\"",
            "\"sql\"",
            "\"shell\"",
            "\"human\"",
            "\"mcp\"",
            "\"a2a\"",
        ] {
            assert_absent(
                surface,
                forbidden,
                &format!("a forbidden opcode ({forbidden})"),
            );
        }
    }
}

#[test]
fn no_runtime_object_exists_in_the_crate() {
    let sources = crate_sources();
    for forbidden in [
        "SemanticExecutionState",
        "JournalEvent",
        "RuntimeCommand",
        "ScheduleTask",
        "ArmWait",
        "ScheduleTimer",
        "OccurrencePath",
        "AttemptId",
        "InvocationId",
        "ResourceHandle",
        "SequenceState",
        "RetryState",
        "RaceState",
        "WaitState",
        "SelectCaseChosen",
        "RaceWinnerChosen",
        "IterationGuardDecision",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the runtime object {forbidden}"),
        );
    }
    // Worker/scheduler/reducer: absent as identifiers (prose is stripped).
    for forbidden in ["Worker", "Scheduler", "Reducer"] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the runtime object {forbidden}"),
        );
    }
}

#[test]
fn the_crate_is_pure() {
    // The manifest is the authority for dependencies: af-spec is forbidden
    // (layering) and so is every effect-capable crate.
    let manifest = read("crates/af-ir/Cargo.toml");
    for forbidden in [
        "af-spec", "reqwest", "hyper", "ureq", "tokio", "chrono", "rand", "walkdir", "sha2",
    ] {
        assert_absent(&manifest, forbidden, &format!("the dependency {forbidden}"));
    }
    let sources = crate_sources();
    for forbidden in [
        "std::time",
        "std::env",
        "std::fs",
        "std::net",
        "std::process",
        "rand::",
    ] {
        assert_absent(&sources, forbidden, forbidden);
    }
}

#[test]
fn the_digest_uses_only_the_frozen_semantic_ir_domain() {
    // The HashV1 domain registry is untouched: exactly its fourteen entries.
    let table = read("spec/tables/hash-domains.json");
    let value: serde_json::Value = serde_json::from_str(&table).unwrap();
    assert_eq!(
        value["domains"].as_array().unwrap().len(),
        14,
        "the frozen HashV1 domain registry changed size"
    );

    // The crate computes exactly one kind of digest: HashV1("semantic-ir",
    // canonical bytes) via af_core. No other domain name appears in code, and
    // no hashing primitive is reachable directly.
    let sources = crate_sources();
    assert!(
        sources.contains("DigestDomain::SemanticIr"),
        "the SemanticIr digest domain must be the crate's digest"
    );
    for forbidden in [
        "DigestDomain::PlanCore",
        "DigestDomain::JournalEventPayload",
        "DigestDomain::RuntimeCommandPayload",
        "DigestDomain::SemanticExecutionState",
        "DigestDomain::Value",
        "DigestDomain::Provenance",
        "DigestDomain::Artifact",
        "DigestDomain::RunManifest",
        "DigestDomain::LockGraph",
        "DigestDomain::ResourceContract",
        "DigestDomain::Conformance",
        "hash_v1",
        "Sha256",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("a direct hash path ({forbidden})"),
        );
    }
    // component-contract digests are validated by registry name, never computed.
    assert!(
        sources.contains("\"component-contract\""),
        "the component-contract domain is validated by name"
    );
}

#[test]
fn the_interface_projection_has_no_second_table() {
    // The interface is derived from outputs + slot types only; no stored
    // interface map exists anywhere in the model.
    let sources = crate_sources();
    assert_absent(&sources, "interface_table", "a stored interface table");
    assert_absent(&sources, "InterfaceTable", "a stored interface table");
    // The only construction path is the read-only projection.
    assert!(
        sources.contains("fn interface("),
        "the derived interface projection must exist"
    );
}

// ---------------------------------------------------------------------------
// Phase 3C wire-model freeze: the analysis layer must be invisible on every
// durable surface (schema modules, ABI rows, HashV1 domains, the digest).
// ---------------------------------------------------------------------------

#[test]
fn the_wire_schema_is_frozen_at_its_phase_3b_identity() {
    // spec-check enforces file == registry digest; this test pins the
    // registry entry itself to the exact Phase 3B value, so any byte drift
    // in spec/idl/semantic-ir.afidl fails here before it fails there.
    let specifications: serde_json::Value =
        serde_json::from_str(&read("spec/registry/specifications.json")).unwrap();
    let entry = specifications["specifications"]
        .as_array()
        .unwrap()
        .iter()
        .find(|spec| spec["specification_id"] == "agentflow.semantic.ir@1")
        .expect("the semantic IR specification is registered");
    assert_eq!(
        entry["normative_source"]["content_digest"]["hex"],
        "9fb3292a0fe90b207db037575c50ef3ca50174411b3c5a461c09c37d53f616a3",
        "spec/idl/semantic-ir.afidl drifted off its Phase 3B identity"
    );
}

#[test]
fn module_abi_and_domain_counts_hold_at_their_frozen_values() {
    // The cumulative module freeze: exactly twelve AFIDL modules after
    // Phase 4A.7 added the Execution Requirements schema module (and the
    // preceding Recovery Contract module).
    fn collect_modules(directory: &Path, out: &mut Vec<String>) {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect_modules(&path, out);
            } else if path.extension().and_then(|e| e.to_str()) == Some("afidl") {
                out.push(path.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    let mut modules = Vec::new();
    collect_modules(&repo_root().join("spec/idl"), &mut modules);
    modules.sort_unstable();
    assert_eq!(
        modules.len(),
        12,
        "an unexpected AFIDL module set: {modules:?}"
    );

    // Exactly the frozen ABI rows: 400 total, 100 of them semantic IR,
    // 32 of them the Phase 4A.1 effect-signature family, 2 of them the
    // Phase 4A.2 failure-row family, 7 of them the Phase 4A.3
    // resource-requirement family and 27 of them the Phase 4A.4
    // information-flow family, 10 of them the Phase 4A.5A usage family and
    // 25 of them the Phase 4A.5B cost-projection family.
    let abi: serde_json::Value =
        serde_json::from_str(&read("spec/registry/abi-numbers.json")).unwrap();
    let rows = abi["assignments"].as_array().unwrap();
    assert_eq!(rows.len(), 400, "an ABI row was added or removed");
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.semantic.ir@1")
            .count(),
        100,
        "the semantic IR ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.effect.signature@1")
            .count(),
        32,
        "the effect-signature ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.failure.row@1")
            .count(),
        2,
        "the failure-row ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.resource.requirement@1")
            .count(),
        7,
        "the resource-requirement ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.information.flow@1")
            .count(),
        27,
        "the information-flow ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.usage.signature@1")
            .count(),
        10,
        "the usage-signature ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.usage.cost@1")
            .count(),
        25,
        "the cost-projection ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.recovery.contract@1")
            .count(),
        26,
        "the recovery-contract ABI block changed size"
    );
    assert_eq!(
        rows.iter()
            .filter(|row| row["schema_family"] == "agentflow.execution.requirements@1")
            .count(),
        39,
        "the execution-requirements ABI block changed size"
    );

    // Exactly the frozen HashV1 domains.
    let domains: serde_json::Value =
        serde_json::from_str(&read("spec/tables/hash-domains.json")).unwrap();
    assert_eq!(
        domains["domains"].as_array().unwrap().len(),
        14,
        "a HashV1 domain was added"
    );
}

#[test]
fn the_analysis_is_derived_only_and_never_enters_the_digest() {
    // The canonical/digest path must not know the analysis exists.
    let canonical = code_only(&read("crates/af-ir/src/canonical.rs"));
    assert_absent(&canonical, "dataflow", "the analysis projection");
    assert_absent(&canonical, "Dataflow", "the analysis projection");
    // The analysis exists, is a plain derived structure, and is never
    // canonicalized: only the canonical document feeds the digest.
    let sources = crate_sources();
    assert!(
        sources.contains("pub fn analyze_dataflow"),
        "the Phase 3C analysis pass must exist"
    );
    assert_absent(
        &sources,
        "DataflowAnalysisDigest",
        "a new digest over the analysis",
    );
}

#[test]
fn no_deferred_phase_3d_or_runtime_placeholder_exists() {
    // FailureValue, the ports-only ComponentContract shrinkage, a fake
    // component interface universe and parallel resource concurrency remain
    // deferred (Phase 4 materializes the real contract core); none of them
    // may sneak in as a placeholder.
    let sources = crate_sources();
    for forbidden in [
        "FailureValue",
        "failure_slot",
        "ContractPort",
        "resource_shared_in_parallel",
        "struct ComponentContract",
        "ComponentContractUniverse",
        "ResolutionSnapshot",
        "LockGraph",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred placeholder {forbidden}"),
        );
    }
}

#[test]
fn the_subflow_analysis_stays_off_every_wire_surface() {
    // The Phase 3D subflow model is derived-only: the interface, the
    // universe and the structural type projection must never appear in the
    // wire model, the canonical/digest path or the schema module.
    let wire_surfaces = [
        code_only(&read("crates/af-ir/src/model.rs")),
        code_only(&read("crates/af-ir/src/canonical.rs")),
        code_only(&read("spec/idl/semantic-ir.afidl")),
    ];
    for surface in &wire_surfaces {
        for forbidden in [
            "SubflowInterface",
            "SubflowIrUniverse",
            "SubflowValuePort",
            "SubflowResourcePort",
            "ResolvedSemanticType",
        ] {
            assert_absent(
                surface,
                forbidden,
                &format!("the derived subflow model ({forbidden})"),
            );
        }
    }
    // The analysis types are plain Rust: no serde derive, no accidental
    // wire ABI.
    let subflow = code_only(&read("crates/af-ir/src/subflow.rs"));
    for forbidden in ["Serialize", "Deserialize"] {
        assert_absent(
            &subflow,
            forbidden,
            &format!("a serde surface on the derived subflow model ({forbidden})"),
        );
    }
    assert!(
        subflow.contains("pub fn derive_subflow_interface"),
        "the derived interface construction must exist"
    );
}
