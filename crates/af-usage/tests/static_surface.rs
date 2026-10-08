//! Static-surface audit for the Usage layer (Phase 4A.5A).
//!
//! Usage declarations and proofs stay separate from measurement, billing,
//! ledgers, quotas, scheduling and runtime evidence.

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

fn code_only(source: &str) -> String {
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
    let directory = repo_root().join("crates/af-usage/src");
    let mut out = String::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push_str(&code_only(&std::fs::read_to_string(path).unwrap()));
            out.push('\n');
        }
    }
    out
}

#[test]
fn the_dependency_set_is_static_only() {
    let manifest = read("crates/af-usage/Cargo.toml");
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
            matches!(
                name,
                "af-core" | "af-idl" | "serde" | "unicode-normalization"
            ),
            "unexpected af-usage dependency: {name}"
        );
    }
    for forbidden in ["af-ir", "af-spec", "af-effect", "af-failure", "af-resource"] {
        assert!(
            !manifest.contains(forbidden),
            "Usage must not depend on {forbidden}"
        );
    }
}

#[test]
fn no_runtime_measurement_or_authority_surface_exists() {
    let sources = crate_sources();
    for forbidden in [
        "UsageReceipt",
        "UsageEvidence",
        "UsageLedger",
        "BudgetLedger",
        "QuotaLedger",
        "BudgetReservation",
        "Reserve",
        "Settle",
        "PricingApi",
        "PriceBook",
        "LatestPrice",
        "Provider",
        "Scheduler",
        "Worker",
        "PolicyEngine",
        "ComponentContract",
        "Runtime",
        "Journal",
        "Reducer",
        "UsageSignatureDigest",
        "hash_v1",
        "DigestDomain",
    ] {
        assert!(
            !sources.contains(forbidden),
            "af-usage must not contain deferred/runtime symbol {forbidden}"
        );
    }
}

#[test]
fn the_wire_module_is_usage_only_and_has_no_digest() {
    let module = read("spec/idl/usage-signature.afidl");
    assert!(module.contains("schema agentflow.usage.signature @1;"));
    for type_name in [
        "UsageMeterRef",
        "UsagePhase",
        "UsageUpperBound",
        "UsageLimit",
        "UsageSignature",
    ] {
        assert!(
            module.contains(type_name),
            "missing Usage wire type {type_name}"
        );
    }
    assert!(!code_only(&module).contains("UsageSignatureDigest"));
}
