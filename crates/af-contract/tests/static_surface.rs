use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .canonicalize()
        .expect("crate root")
}

#[test]
fn dependencies_and_production_source_are_static_only() {
    let manifest = std::fs::read_to_string(crate_root().join("Cargo.toml")).unwrap();
    for forbidden in ["af-ir", "af-spec", "tokio", "reqwest", "hyper"] {
        assert!(
            !manifest.contains(forbidden),
            "af-contract must not depend on {forbidden}"
        );
    }
    let mut stack = vec![crate_root().join("src")];
    let mut sources = String::new();
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
                sources.push_str(&std::fs::read_to_string(path).unwrap());
            }
        }
    }
    for forbidden in [
        "pub struct EffectGrant",
        "pub struct ResourceHandle",
        "pub struct UsageReceipt",
        "pub struct BudgetReservation",
        "pub struct Scheduler",
        "pub struct Worker",
        "pub struct RunManifest",
        "pub struct Journal",
        "std::fs::",
        "std::net::",
    ] {
        assert!(
            !sources.contains(forbidden),
            "static ComponentContract crate contains forbidden runtime surface {forbidden}"
        );
    }
}
