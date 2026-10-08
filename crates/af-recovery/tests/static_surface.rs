//! Static surface audit for Recovery Contract K.

use std::path::{Path, PathBuf};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .unwrap()
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(root().join(relative)).unwrap()
}

fn code_only(source: &str) -> String {
    source
        .lines()
        .map(|line| line.split("//").next().unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

fn sources() -> String {
    let mut out = String::new();
    for entry in std::fs::read_dir(root().join("crates/af-recovery/src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push_str(&code_only(&std::fs::read_to_string(path).unwrap()));
        }
    }
    out
}

#[test]
fn dependency_set_is_static_only() {
    let manifest = read("crates/af-recovery/Cargo.toml");
    let deps = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    for line in deps.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let name = line.split('=').next().unwrap().trim();
        assert!(matches!(
            name,
            "af-core" | "af-idl" | "af-semantic" | "af-effect" | "serde" | "unicode-normalization"
        ));
    }
    for forbidden in ["af-ir", "af-spec", "database", "network"] {
        assert!(!manifest.contains(forbidden));
    }
}

#[test]
fn no_runtime_receipt_grant_or_execution_surface_exists() {
    let source = sources();
    for forbidden in [
        "EffectReceipt",
        "EffectIntent",
        "EffectGrant",
        "ProviderDispatchRecord",
        "ReconciliationExecution",
        "CompensationExecution",
        "IdempotencyKey",
        "RecoveryJournal",
        "RetryController",
        "ComponentContract",
        "ExecutablePlan",
        "Runtime",
        "Scheduler",
        "Worker",
        "UsageLedger",
        "BudgetLedger",
        "QuotaLedger",
        "RecoveryContractDigest",
        "hash_v1",
        "DigestDomain",
    ] {
        assert!(
            !source.contains(forbidden),
            "recovery contains deferred symbol {forbidden}"
        );
    }
}

#[test]
fn wire_module_has_only_static_recovery_objects() {
    let module = read("spec/idl/recovery-contract.afidl");
    assert!(module.contains("schema agentflow.recovery.contract @1;"));
    for type_name in [
        "ReplaySemantics",
        "IdempotencySemantics",
        "KeyStabilityRequirement",
        "ReconciliationRequirement",
        "CompensationRequirement",
        "UnknownEffectHandling",
        "EffectRecoveryClause",
        "RecoveryContract",
    ] {
        assert!(module.contains(type_name));
    }
    assert!(!code_only(&module).contains("RecoveryContractDigest"));
}
