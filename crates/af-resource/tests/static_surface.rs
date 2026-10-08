//! Static surface audit for the resource layer (Phase 4A.3).
//!
//! The Requirement Row is the static-need plane only: no binding, no
//! handle, no adapter, no grant, no credential, no runtime authority, no
//! cross-component merge, no new digest domain, and the frozen wire set
//! stays untouched. These audits fail the build the moment any of that
//! changes.

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
    let mut stack = vec![repo_root().join("crates/af-resource/src")];
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
        "af-resource must not use {what} (found {needle:?})"
    );
}

#[test]
fn no_authority_or_binding_symbol_exists() {
    // The requirement layer ends at static need: binding, handles,
    // adapters, grants, credentials, leases, physical resolution and any
    // object that would combine declaration, binding and authority are
    // structurally absent.
    let sources = crate_sources();
    for forbidden in [
        "ResourceBinding",
        "ResourceHandle",
        "ResourceAdapter",
        "ResourceGrant",
        "ResourceCredential",
        "LogicalResourceInstance",
        "CredentialBroker",
        "SecretLease",
        "EffectGrant",
        "PolicyEngine",
        "DeploymentRevision",
        "BindingSnapshot",
        "ResourceContext",
        "ResourceClient",
        "PhysicalResource",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred/forbidden authority symbol {forbidden}"),
        );
    }
}

#[test]
fn no_runtime_plane_or_digest_symbols_exist() {
    // Static resource errors are never mapped onto other semantic planes,
    // and no resource-requirement digest exists: the HashV1 domain
    // registry stays frozen.
    let sources = crate_sources();
    for forbidden in [
        "PolicyDenied",
        "ResourceUnavailable",
        "NoMatchingWorker",
        "CredentialExpired",
        "RuntimeIncident",
        "Quota",
        "UsageLedger",
        "InformationFlow",
        "RecoveryContract",
        "ExecutionRequirements",
        "ExecutablePlan",
        "Journal",
        "Reducer",
        "Scheduler",
        "Worker",
        "ComponentContract",
        "ResourceRequirementDigest",
        "ResourceRowDigest",
        "ResourceBindingDigest",
        "hash_v1",
        "DigestDomain",
    ] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the deferred/forbidden symbol {forbidden}"),
        );
    }
}

#[test]
fn the_dependency_set_is_exactly_the_static_layer() {
    // af-core ← af-idl / af-semantic ← af-resource; no af-ir, af-effect,
    // af-failure, af-spec, I/O, async runtime or network. NFC
    // normalization is the one extra pure dependency.
    let manifest = read("crates/af-resource/Cargo.toml");
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
            "unexpected af-resource dependency: {name}"
        );
    }
    for forbidden in ["af-ir", "af-effect", "af-failure", "af-spec"] {
        assert!(
            !manifest.contains(forbidden),
            "the resource layer must not depend on {forbidden}"
        );
    }
}

#[test]
fn no_cross_component_merge_operation_exists() {
    // Requirement rows are never merged by equal port numbers: no union
    // or merge function exists anywhere in the crate code.
    let sources = crate_sources();
    for forbidden in ["fn union", "fn merge", "fn aggregate"] {
        assert_absent(
            &sources,
            forbidden,
            &format!("the forbidden row-combination function {forbidden}"),
        );
    }
}

#[test]
fn the_effect_selector_linkage_is_recorded_and_the_module_is_clean() {
    // Read-only cross-dimension checks: the effect signature's resource
    // selector carries the future linkage coordinate
    // (requirement_port_no), the resource module declares the matching
    // port coordinate, and the normative spec records the integration
    // invariant — without this phase validating any effect port.
    let effect = read("spec/idl/effect-signature.afidl");
    assert!(
        effect.contains("requirement_port_no"),
        "the effect selector lost its linkage coordinate"
    );
    let resource = read("spec/idl/resource-requirement.afidl");
    assert!(
        resource.contains("1 port_no: uint32;"),
        "the requirement port coordinate moved"
    );
    let spec = read("spec/resource/resource-requirement.md");
    let spec_flattened = spec.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        spec_flattened.contains(
            "Every declared Effect resource selector must reference an \
             existing ResourceRequirement port"
        ),
        "the effect-selector linkage invariant is not recorded"
    );

    // The resource module itself declares no binding/handle/adapter/
    // grant/credential types (prose comments stripped; the audit sees
    // declarations only).
    let module_code: String = resource
        .lines()
        .map(|line| match line.find("//") {
            Some(index) => &line[..index],
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");
    for forbidden in [
        "ResourceBinding",
        "ResourceHandle",
        "ResourceAdapter",
        "ResourceGrant",
        "ResourceCredential",
    ] {
        assert!(
            !module_code.contains(forbidden),
            "the resource module must not declare {forbidden}"
        );
    }
}

#[test]
fn the_wire_freeze_holds() {
    // Exactly seven AFIDL modules; semantic-ir at its Phase 3B identity;
    // 14 HashV1 domains; the effect corpus stays at revision 2 and the
    // failure corpus at revision 1; X is an additive twelfth module.
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

    // The effect corpus revision stays at 2 (Phase 4A.1B), untouched.
    let effect_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/effect-signature/golden/effect-golden-pure-empty-row.json",
    ))
    .unwrap();
    assert_eq!(
        effect_golden["corpus_revision"].as_u64(),
        Some(2),
        "the effect corpus revision moved"
    );

    // The failure corpus revision stays at 1 (Phase 4A.2), untouched.
    let failure_golden: serde_json::Value = serde_json::from_str(&read(
        "spec/conformance/failure-row/golden/failure-golden-empty-failure-row.json",
    ))
    .unwrap();
    assert_eq!(
        failure_golden["corpus_revision"].as_u64(),
        Some(1),
        "the failure corpus revision moved"
    );
}
