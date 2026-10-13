use std::path::{Path, PathBuf};

fn rust_sources(root: &Path) -> Vec<PathBuf> {
    let mut pending = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(path) = pending.pop() {
        for entry in std::fs::read_dir(path).expect("read crate source directory") {
            let entry = entry.expect("read source entry");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn portable_static_revalidation_is_closed_and_stops_at_d10() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let sources = rust_sources(&root.join("src"));
    assert!(!sources.is_empty());
    let source = sources
        .iter()
        .map(|path| std::fs::read_to_string(path).unwrap())
        .collect::<String>();

    for required in [
        "check_contract",
        "SemanticIrDocument",
        "analyze_dataflow",
        "StaticComputationResolutionSnapshotBuilder",
        "derive_workflow_contract_summary",
        "check_root_export_assertion",
        "CheckedRootExportCompatibility",
    ] {
        assert!(
            source.contains(required),
            "portable static reconstruction is missing required D6-D8 path: {required}"
        );
    }

    for forbidden in [
        "std::fs",
        "std::net",
        "std::process",
        "SystemTime",
        "Instant",
        "current_dir()",
        "PublisherAuthorized",
        "TrustedOrigin",
        "root-export-assertion",
        "portable-bundle",
    ] {
        assert!(
            !source.contains(forbidden),
            "portable runtime surface contains forbidden capability/dependency: {forbidden}"
        );
    }

    let manifest = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
    for forbidden in ["reqwest", "hyper", "tokio", "ureq", "chrono"] {
        assert!(
            !manifest.contains(forbidden),
            "portable crate added ambient/network dependency {forbidden}"
        );
    }
}
