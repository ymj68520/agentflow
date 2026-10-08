//! Static surface audit for the information-flow layer (Phase 4A.4).
//!
//! The flow layer is the static-contract plane only: no runtime
//! provenance, no policy engine, no authorization, no grant, no
//! credential, no privileged-transformation execution, no digest domain,
//! and the frozen wire set stays untouched. These audits fail the build
//! the moment any of that changes.

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
    let mut stack = vec![repo_root().join("crates/af-flow/src")];
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
        "af-flow must not use {what} (found {needle:?})"
    );
}

#[test]
fn no_authorization_or_policy_symbol_exists() {
    // A declared flow ceiling is not a policy approval: no authorization
    // decision, no grant, no credential, no policy engine and no
    // component-contract object exists in this layer.
    let sources = crate_sources();
    for forbidden in [
        "PolicyEngine",
        "PolicyDecision",
        "EffectGrant",
        "EffectDenied",
        "PolicyDenied",
        "Credential",
        "ResourceHandle",
        "ResourceBinding",
        "ResourceAdapter",
        "ComponentContract",
        "ExecutablePlan",
        "Journal",
        "Reducer",
        "Scheduler",
        "Worker",
        "RuntimeIncident",
        "FailureValue",
        "trust_score",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred/forbidden authority symbol {forbidden}"),
        );
    }
}

#[test]
fn no_privileged_execution_or_runtime_provenance_exists() {
    // Declassification and endorsement are recorded obligations, never
    // executed; no provenance store, runtime DAG, runtime ABI or flow
    // digest exists anywhere in the crate.
    let sources = crate_sources();
    for forbidden in [
        "fn declassify",
        "fn endorse",
        "Declassifier",
        "Endorser",
        "ProvenanceStore",
        "ProvenanceDag",
        "ValueEnvelope",
        "ValueSemanticDigest",
        "InformationFlowDigest",
        "ProvenanceDigest",
        "TaintDigest",
        "hash_v1",
        "DigestDomain",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the forbidden execution/store symbol {forbidden}"),
        );
    }
}

#[test]
fn the_dependency_set_is_exactly_the_static_layer() {
    // af-core <- af-idl / af-semantic <- af-effect <- af-flow; the
    // one-directional effect dependency exists only to reuse the
    // structured EffectOperationRef for external-effect sinks. No af-ir,
    // af-failure, af-resource or af-spec dependency, no I/O, no clock,
    // no network.
    let manifest = read("crates/af-flow/Cargo.toml");
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
                "af-core"
                    | "af-idl"
                    | "af-semantic"
                    | "af-effect"
                    | "serde"
                    | "unicode-normalization"
            ),
            "unexpected af-flow dependency: {name}"
        );
    }
    for forbidden in ["af-ir", "af-failure", "af-resource", "af-spec"] {
        assert!(
            !manifest.contains(forbidden),
            "the flow layer must not depend on {forbidden}"
        );
    }
}

#[test]
fn the_effect_sink_reuse_is_exact_and_directional() {
    // The flow module imports the effect signature's structured
    // EffectOperationRef (never a second string operation id), and the
    // effect module itself is untouched by this phase.
    let flow = read("spec/idl/information-flow.afidl");
    assert!(
        flow.contains("import agentflow.effect.signature@1 as effect;"),
        "the flow module must reuse the effect identity by exact import"
    );
    assert!(
        flow.contains("2 external_effect: effect.EffectOperationRef;"),
        "the structured effect sink reference moved"
    );
    let effect = read("spec/idl/effect-signature.afidl");
    let start = effect
        .find("record EffectOperationRef")
        .expect("EffectOperationRef");
    let end = start + effect[start..].find('}').unwrap();
    assert!(
        effect[start..end].contains("1 catalog: CatalogId;"),
        "the frozen effect operation identity moved"
    );
}

#[test]
fn the_wire_freeze_holds() {
    // Exactly twelve AFIDL modules; semantic-ir at its Phase 3B identity;
    // 14 HashV1 domains; the effect, failure and resource corpora stay
    // at their frozen revisions.
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

    // The three earlier static-contract corpora stay at their frozen
    // revisions, untouched.
    let effect_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/effect-signature/golden/effect-golden-pure-empty-row.json",
    ))
    .unwrap();
    assert_eq!(
        effect_golden["corpus_revision"].as_u64(),
        Some(2),
        "the effect corpus revision moved"
    );
    let failure_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/failure-row/golden/failure-golden-empty-failure-row.json",
    ))
    .unwrap();
    assert_eq!(
        failure_golden["corpus_revision"].as_u64(),
        Some(1),
        "the failure corpus revision moved"
    );
    let resource_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/resource-requirement/golden/\
         resource-golden-empty-requirement-row.json",
    ))
    .unwrap();
    assert_eq!(
        resource_golden["corpus_revision"].as_u64(),
        Some(1),
        "the resource corpus revision moved"
    );
}
