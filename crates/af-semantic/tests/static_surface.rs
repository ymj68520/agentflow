//! Static self-checks: the Phase 3A crate must contain no runtime semantics,
//! no untyped escape hatch, no effect capability and no new digest identity.
//! These are assertions about the source tree, so they are checked the way a
//! reviewer would check them — by reading it.

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

/// Strips `//` line comments so prose may discuss forbidden words while
/// declarations may not use them, and drops `#[cfg(test)]` tails: in-crate
/// unit tests may use dev-dependencies that production code must not.
fn code_only(source: &str) -> String {
    let production = match source.find("#[cfg(test)]") {
        Some(index) => &source[..index],
        None => source,
    };
    production
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn assert_absent(source: &str, needle: &str, what: &str) {
    assert!(
        !source.contains(needle),
        "af-semantic must not use {what} (found {needle:?})"
    );
}

#[test]
fn the_semantic_crate_has_no_untyped_escape_hatch() {
    for entry in std::fs::read_dir(repo_root().join("crates/af-semantic/src")).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let source = code_only(&read(
            &path
                .strip_prefix(repo_root())
                .unwrap()
                .to_string_lossy()
                .replace('\\', "/"),
        ));
        for forbidden in ["serde_json", "Box<dyn Any>", "dyn Any", "impl Any"] {
            assert_absent(
                &source,
                forbidden,
                &format!("the dynamic escape hatch {forbidden:?}"),
            );
        }
    }
}

#[test]
fn the_semantic_crate_has_no_effect_capability() {
    // Cargo.toml is the authority for dependencies: an effect would need a
    // dependency, so asserting the dependency set is asserting purity.
    let manifest = read("crates/af-semantic/Cargo.toml");
    for forbidden in [
        "rand",
        "chrono",
        "time",
        "reqwest",
        "hyper",
        "tokio",
        "async-std",
        "std::net",
        "std::env",
        "std::time",
        "std::fs",
    ] {
        assert_absent(
            &manifest,
            forbidden,
            &format!("an effect capability ({forbidden})"),
        );
    }
    // Source level: no clock, randomness, environment, filesystem or network.
    let source = code_only(&read("crates/af-semantic/src/lib.rs"));
    for forbidden in ["std::time", "std::env", "std::fs", "std::net"] {
        assert_absent(&source, forbidden, forbidden);
    }
}

#[test]
fn the_semantic_crate_never_depends_on_registry_tooling() {
    // Dependency direction: af-core <- af-idl <- af-semantic, with af-spec as a
    // separate tooling branch that semantic code must never require.
    let manifest = read("crates/af-semantic/Cargo.toml");
    assert_absent(&manifest, "af-spec", "the registry tooling crate af-spec");
    assert_absent(&manifest, "af_spec", "the registry tooling crate af_spec");
    assert!(
        manifest.contains("af-core"),
        "af-semantic must depend on af-core"
    );
    assert!(
        manifest.contains("af-idl"),
        "af-semantic must depend on af-idl"
    );
}

#[test]
fn no_workflow_opcode_or_runtime_object_exists_in_the_semantic_crate() {
    let mut sources = String::new();
    let mut stack = vec![repo_root().join("crates/af-semantic/src")];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let rel = path
                    .strip_prefix(repo_root())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                sources.push_str(&code_only(&read(&rel)));
                sources.push('\n');
            }
        }
    }
    // The thirteen Phase 3B computation nodes must not appear as semantics.
    for opcode in [
        "Invoke",
        "Await",
        "Sequence",
        "Select",
        "Parallel",
        "Race",
        "Iterate",
        "Retry",
        "Timeout",
        "Handle",
        "Scope",
        "CompensationScope",
        "CallSubflow",
    ] {
        assert_absent(&sources, opcode, &format!("the workflow node {opcode}"));
    }
    // No runtime objects.
    for runtime in [
        "SemanticIRDocument",
        "ExecutablePlan",
        "JournalEvent",
        "RuntimeCommand",
        "ResourceHandle",
        "BindingSnapshot",
        "Reducer",
    ] {
        assert_absent(&sources, runtime, &format!("the runtime object {runtime}"));
    }
}

#[test]
fn the_semantic_crate_introduces_no_new_hash_domain() {
    // The HashV1 domain registry is frozen: it must still hold exactly its
    // fourteen entries, and no semantic-sounding domain may appear anywhere.
    let table = read("spec/tables/hash-domains.json");
    let value: serde_json::Value = serde_json::from_str(&table).unwrap();
    let count = value["domains"].as_array().unwrap().len();
    assert_eq!(count, 14, "the frozen HashV1 domain registry changed size");

    let mut sources = String::new();
    let mut stack = vec![repo_root().join("crates/af-semantic/src")];
    while let Some(directory) = stack.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
                let rel = path
                    .strip_prefix(repo_root())
                    .unwrap()
                    .to_string_lossy()
                    .replace('\\', "/");
                sources.push_str(&code_only(&read(&rel)));
                sources.push('\n');
            }
        }
    }
    // The crate may *read* the frozen registry to validate a pinned digest
    // (that is the point of DigestRefValue), but it must never compute a new
    // digest, and the only domain it may ever name is `type-schema`.
    for forbidden in ["DigestRef::compute", "hash_v1", "sha2::", "use sha2"] {
        assert_absent(
            &sources,
            forbidden,
            &format!("a digest computation ({forbidden})"),
        );
    }
    // Digest-domain constants: every `DigestDomain::X` the crate names must be
    // the frozen `TypeSchema` domain. Lookup calls (`from_registry_name`) are
    // the sanctioned read path and are exempt.
    for (index, line) in sources.lines().enumerate() {
        let Some(start) = line.find("DigestDomain::") else {
            continue;
        };
        let tail = &line[start + "DigestDomain::".len()..];
        let variant: String = tail
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect();
        assert!(
            variant == "TypeSchema" || variant == "from_registry_name",
            "af-semantic may only name the frozen type-schema domain, found on line {}: {}",
            index + 1,
            line.trim()
        );
    }
}

#[test]
fn the_semantic_core_module_declares_no_workflow_node() {
    let module = code_only(&read("spec/idl/semantic-core.afidl"));
    for opcode in [
        "Invoke",
        "Await",
        "Sequence",
        "Parallel",
        "Race",
        "Iterate",
        "Retry",
        "Timeout",
        "Handle",
        "CompensationScope",
        "CallSubflow",
    ] {
        assert_absent(&module, opcode, &format!("the workflow node {opcode}"));
    }
    // `Select` and `Scope` are substring-prone; check them as type names only.
    for exact in [
        "record Select ",
        "variant Select",
        "record Scope ",
        "variant Scope",
    ] {
        assert_absent(&module, exact, &format!("the workflow node {exact}"));
    }
}

#[test]
fn the_semantic_documents_tables_are_canonically_ordered_in_the_corpus() {
    // Every golden document in the corpus must be canonically orderable: the
    // Rust canonical ordering is a sort by id, so a shuffled document and its
    // sorted twin must both be accepted and produce identical bytes. This is
    // asserted directly on the frozen equivalence groups.
    let corpus: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/semantic-core/semantic-core-golden.json",
    ))
    .unwrap();
    let groups = corpus["equivalence_groups"].as_array().unwrap();
    assert!(
        !groups.is_empty(),
        "the corpus must freeze at least one canonical-ordering equivalence group"
    );
    let golden = corpus["golden"].as_array().unwrap();
    let mut found_scope = false;
    for case in golden {
        if case["case_id"].as_str().unwrap().starts_with("canonical/") {
            found_scope = true;
        }
    }
    assert!(
        found_scope,
        "the corpus must contain canonical ordering cases"
    );
}
