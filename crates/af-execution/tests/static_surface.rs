//! Static boundary audit for Phase 4A.7 X.

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
        .map(|line| line.split_once("//").map_or(line, |(code, _)| code))
        .collect::<Vec<_>>()
        .join("\n")
}

fn sources() -> String {
    let directory = root().join("crates/af-execution/src");
    let mut source = String::new();
    for entry in std::fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        if path.extension().and_then(|x| x.to_str()) == Some("rs") {
            source.push_str(&code_only(&std::fs::read_to_string(path).unwrap()));
        }
    }
    source
}

#[test]
fn dependency_set_is_static_and_one_way() {
    let manifest = read("crates/af-execution/Cargo.toml");
    let dependencies = manifest
        .split("[dependencies]")
        .nth(1)
        .unwrap()
        .split("[dev-dependencies]")
        .next()
        .unwrap();
    for line in dependencies
        .lines()
        .map(str::trim)
        .filter(|x| !x.is_empty())
    {
        let name = line.split('=').next().unwrap().trim();
        assert!(
            matches!(
                name,
                "af-core" | "af-idl" | "af-semantic" | "serde" | "unicode-normalization"
            ),
            "unexpected af-execution dependency: {name}"
        );
    }
    for forbidden in [
        "af-ir",
        "af-spec",
        "af-effect",
        "af-failure",
        "af-resource",
        "af-usage",
    ] {
        assert!(
            !manifest.contains(forbidden),
            "X must not depend on {forbidden}"
        );
    }
}

#[test]
fn runtime_and_authority_surfaces_are_absent() {
    let source = sources();
    for forbidden in [
        "Worker",
        "WorkerRegistry",
        "Scheduler",
        "Admission",
        "Lease",
        "Credential",
        "Endpoint",
        "Reservation",
        "Capacity",
        "RuntimeState",
        "RuntimeCommand",
        "RuntimeEngine",
        "Plan",
        "Journal",
        "Reducer",
        "PolicyEngine",
        "ComponentContract",
        "ExecutionRequirementsDigest",
        "WorkerProfileDigest",
        "SandboxProofDigest",
        "hash_v1",
        "DigestDomain",
        "EffectGrant",
        "ResourceHandle",
    ] {
        assert!(
            !source.contains(forbidden),
            "X contains forbidden symbol {forbidden}"
        );
    }
}

#[test]
fn wire_surface_is_execution_only_and_digest_free() {
    let module = code_only(&read("spec/idl/execution-requirements.afidl"));
    assert!(module.contains("schema agentflow.execution.requirements @1;"));
    for type_name in [
        "ExecutionCapabilityRef",
        "CapabilityRequirement",
        "ComputeQuantityRequirement",
        "SandboxRequirement",
        "TrustDomainRef",
        "RegionResidencyRequirement",
        "RuntimeProfileRequirement",
        "ImplementationAbiRequirement",
        "PlacementPreference",
        "ExecutionRequirements",
    ] {
        assert!(
            module.contains(type_name),
            "missing X wire type {type_name}"
        );
    }
    for forbidden in [
        "WorkerId",
        "Lease",
        "Credential",
        "Endpoint",
        "heartbeat",
        "live_load",
        "ExecutionRequirementsDigest",
    ] {
        assert!(
            !module.contains(forbidden),
            "forbidden X wire member {forbidden}"
        );
    }
}
