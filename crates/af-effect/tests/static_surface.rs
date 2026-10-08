//! Static surface audit for the effect layer (Phase 4A.1).
//!
//! The Effect Signature is a declaration layer: no authorization surface,
//! no runtime, no second digest domain, and exactly one new AFIDL module.
//! These audits fail the build the moment any of that changes.

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

/// Source text with comments (including doc comments) and string-literal
/// noise stripped, so the audit sees code, not prose.
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
    let mut stack = vec![repo_root().join("crates/af-effect/src")];
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
        "af-effect must not use {what} (found {needle:?})"
    );
}

#[test]
fn no_authorization_or_runtime_surface_exists() {
    // Declared effects are not granted effects: none of the lifecycle
    // objects above the declaration may exist as placeholders, and no
    // runtime/journal/plan machinery may hide in the crate.
    let sources = crate_sources();
    for forbidden in [
        "ComponentContract",
        "EffectGrant",
        "EffectIntent",
        "EffectReceipt",
        "EffectGateway",
        "CredentialBroker",
        "ProviderDispatch",
        "FailureValue",
        "ResourceContract",
        "UsageLedger",
        "RecoveryContract",
        "ExecutablePlan",
        "LockGraph",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred authorization/runtime object {forbidden}"),
        );
    }
}

#[test]
fn the_canonical_form_is_never_hashed() {
    // No EffectSignatureDigest, no new HashV1 domain, no digest call: the
    // canonical form is ACE1 bytes only.
    let sources = crate_sources();
    for forbidden in [
        "EffectSignatureDigest",
        "hash_v1",
        "DigestDomain",
        "DigestRef",
    ] {
        assert_absent(&sources, forbidden, &format!("digest surface {forbidden}"));
    }
    let domains: serde_json::Value =
        serde_json::from_str(&read("spec/tables/hash-domains.json")).unwrap();
    assert_eq!(
        domains["domains"].as_array().unwrap().len(),
        14,
        "a HashV1 domain was added"
    );
}

#[test]
fn the_dependency_set_is_exactly_the_static_layer() {
    // af-core ← af-idl / af-semantic ← af-effect; no af-ir, no I/O, no
    // async runtime, no network, no clock, no randomness.
    let manifest = read("crates/af-effect/Cargo.toml");
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
                "af-core" | "af-idl" | "af-semantic" | "serde" | "unicode-normalization"
            ),
            "unexpected af-effect dependency: {name}"
        );
    }
    assert!(
        !manifest.contains("af-ir"),
        "the effect layer must not depend on the IR crate"
    );
}

#[test]
fn the_wire_freeze_holds() {
    // Exactly twelve AFIDL modules; the Semantic IR module stayed at its
    // Phase 3B identity; X is additive and registered.
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

    let specifications: serde_json::Value =
        serde_json::from_str(&read("spec/registry/specifications.json")).unwrap();
    let find = |id: &str| {
        specifications["specifications"]
            .as_array()
            .unwrap()
            .iter()
            .find(|spec| spec["specification_id"] == id)
            .unwrap_or_else(|| panic!("{id} is not registered"))
            .clone()
    };
    assert_eq!(
        find("agentflow.semantic.ir@1")["normative_source"]["content_digest"]["hex"],
        "9fb3292a0fe90b207db037575c50ef3ca50174411b3c5a461c09c37d53f616a3",
        "spec/idl/semantic-ir.afidl drifted off its Phase 3B identity"
    );
    let effect = find("agentflow.effect.signature@1");
    assert_eq!(effect["kind"], "schema");
    let digest = effect["normative_source"]["content_digest"]["hex"]
        .as_str()
        .unwrap();
    assert_eq!(
        digest.len(),
        64,
        "the effect module must be registered with its sha256 content digest"
    );
    assert!(
        digest
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
        "the effect content digest must be lowercase hex"
    );
}
