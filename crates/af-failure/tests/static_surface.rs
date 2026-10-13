//! Static surface audit for the failure layer (Phase 4A.2).
//!
//! The Failure Row is the typed-business-failure plane only: the five
//! other error planes are structurally outside it, no runtime exists, no
//! digest domain is added, and the frozen HANDLE matcher shape stays
//! untouched. These audits fail the build the moment any of that changes.

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

/// Source text with comments (including doc comments) stripped, so the
/// audit sees code, not prose.
fn code_only(source: &str) -> String {
    let source = match source.find("#[cfg(test)]") {
        Some(index) => &source[..index],
        None => source,
    };
    source
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn crate_sources() -> String {
    let mut stack = vec![repo_root().join("crates/af-failure/src")];
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
        "af-failure must not use {what} (found {needle:?})"
    );
}

#[test]
fn no_other_error_plane_exists_as_a_failure_type() {
    // TimedOut/Cancelled are Control Outcomes, RecoveryBlocked a Recovery
    // State, TaskAttemptLost an Infrastructure Fact, RuntimeIncident an
    // Incident Fact: none of them is a failure type, and no runtime
    // FailureValue payload exists in the static layer.
    let sources = crate_sources();
    for forbidden in [
        "TimedOut",
        "Cancelled",
        "RecoveryBlocked",
        "TaskAttemptLost",
        "RuntimeIncident",
        "FailureValue",
        "TerminalOutcome",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the non-failure plane symbol {forbidden}"),
        );
    }
}

#[test]
fn no_runtime_or_contract_placeholder_exists() {
    let sources = crate_sources();
    for forbidden in [
        "ComponentContract",
        "ResourceRequirementRow",
        "RecoveryContract",
        "InformationFlow",
        "ExecutionRequirements",
        "ExecutablePlan",
        "Journal",
        "Reducer",
        "Scheduler",
        "Worker",
        "FailureRowDigest",
        "hash_v1",
        "DigestDomain",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred/forbidden symbol {forbidden}"),
        );
    }
    // No error-message or error-code matching surface.
    for forbidden in ["error_code", "message_match", "class_name"] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the non-nominal matching symbol {forbidden}"),
        );
    }
}

#[test]
fn the_dependency_set_is_exactly_the_static_layer() {
    // af-core ← af-idl / af-semantic ← af-failure; no af-ir, af-effect,
    // af-spec, I/O, async runtime or network.
    let manifest = read("crates/af-failure/Cargo.toml");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .and_then(|rest| rest.split("[dev-dependencies]").next())
        .unwrap();
    for line in dependencies.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.split('=').next().unwrap().trim();
        assert!(
            matches!(name, "af-core" | "af-idl" | "af-semantic" | "serde"),
            "unexpected af-failure dependency: {name}"
        );
    }
    for forbidden in ["af-ir", "af-effect", "af-spec"] {
        assert!(
            !manifest.contains(forbidden),
            "the failure layer must not depend on {forbidden}"
        );
    }
}

#[test]
fn the_frozen_handle_matcher_shape_is_compatible_and_untouched() {
    // Read-only model check: HANDLE's failure matcher pins exactly a
    // core.TypeSchemaRef (the identity the failure algebra matches on),
    // timed_out is a payload-less unit case (no Failure<TimedOut>), and no
    // cancelled matcher exists. The HandleBody/HandleMatch AFIDL is not
    // modified by this phase.
    let ir = read("spec/idl/semantic-ir.afidl");
    let variant_start = ir.find("variant HandleMatch").expect("HandleMatch");
    let variant_end = variant_start + ir[variant_start..].find('}').unwrap();
    let variant = &ir[variant_start..variant_end];
    assert!(
        variant.contains("1 failure: FailureMatch;"),
        "the frozen failure matcher case moved"
    );
    assert!(
        variant.contains("2 timed_out: unit;"),
        "timed_out must stay a payload-less Control Outcome"
    );
    assert!(
        !variant.to_lowercase().contains("cancelled"),
        "no cancelled matcher may exist"
    );
    let matcher_start = ir.find("record FailureMatch").expect("FailureMatch");
    let matcher_end = matcher_start + ir[matcher_start..].find('}').unwrap();
    assert!(
        ir[matcher_start..matcher_end].contains("failure_type: core.TypeSchemaRef;"),
        "the failure matcher must pin an exact TypeSchemaRef"
    );

    // The failure-row module itself declares no control-outcome types
    // (prose comments stripped; the audit sees declarations only).
    let failure_module: String = read("spec/idl/failure-row.afidl")
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join(
            "
",
        );
    for forbidden in [
        "TimedOut",
        "Cancelled",
        "RecoveryBlocked",
        "TaskAttemptLost",
        "RuntimeIncident",
    ] {
        assert!(
            !failure_module.contains(forbidden),
            "the failure-row module must not declare {forbidden}"
        );
    }
}

#[test]
fn the_wire_freeze_holds() {
    // Phase 4B.1 adds four modules and Phase 4C.1B.1 adds the portable module;
    // semantic-ir stays at its Phase 3B identity and the 14 domains stay fixed.
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
        17,
        "the registered AFIDL set contains the 16 frozen modules plus Portable Proof Inputs: {modules:?}"
    );

    let specifications: serde_json::Value =
        serde_json::from_str(&read("spec/registry/specifications.json")).unwrap();
    let ir = specifications["specifications"]
        .as_array()
        .unwrap()
        .iter()
        .find(|spec| spec["specification_id"] == "agentflow.semantic.ir@1")
        .expect("the semantic IR specification is registered");
    assert_eq!(
        ir["normative_source"]["content_digest"]["hex"],
        "9fb3292a0fe90b207db037575c50ef3ca50174411b3c5a461c09c37d53f616a3",
        "spec/idl/semantic-ir.afidl drifted off its Phase 3B identity"
    );

    let domains: serde_json::Value =
        serde_json::from_str(&read("spec/tables/hash-domains.json")).unwrap();
    assert_eq!(
        domains["domains"].as_array().unwrap().len(),
        14,
        "a HashV1 domain was added"
    );

    // Effect corpus revision 3 adds the Phase 4B.0A identity regressions.
    let effect_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/effect-signature/golden/effect-golden-pure-empty-row.json",
    ))
    .unwrap();
    assert_eq!(
        effect_golden["corpus_revision"].as_u64(),
        Some(3),
        "the effect corpus revision moved"
    );
}
