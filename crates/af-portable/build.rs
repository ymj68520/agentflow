use serde::Deserialize;
use std::env;
use std::fmt::Write as _;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize)]
struct SpecificationsFile {
    specifications: Vec<Specification>,
}

#[derive(Deserialize)]
struct Specification {
    specification_id: String,
    normative_source: Source,
}

#[derive(Deserialize)]
struct Source {
    logical_path: String,
}

#[derive(Deserialize)]
struct Materialization {
    set_rules: Vec<SetRule>,
}

#[derive(Deserialize)]
struct SetRule {
    owner: String,
    field: String,
    identity_key: String,
    element_type: String,
    #[serde(default)]
    collection: Option<String>,
}

fn main() {
    let manifest = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let workspace = manifest.join("../..");
    let registry_path =
        workspace.join("spec/conformance/g01-native-source-staging/analysis8-specifications.json");
    println!("cargo:rerun-if-changed={}", registry_path.display());
    let registry_bytes = fs::read(&registry_path).expect("read specification registry");
    let registry: SpecificationsFile =
        serde_json::from_slice(&registry_bytes).expect("parse closed specification registry");
    let mut entries = registry.specifications;
    entries.sort_by(|left, right| left.specification_id.cmp(&right.specification_id));

    let mut output = String::new();
    output.push_str("pub const SPECIFICATION_REGISTRY_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/conformance/g01-native-source-staging/analysis8-specifications.json\"));\n");
    output.push_str("pub const RULE_REGISTRY_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/conformance/g01-native-source-staging/analysis8-rules.json\"));\n");
    output.push_str("pub const HASH_DOMAIN_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/tables/hash-domains.json\"));\n");
    output.push_str("pub const ABI_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/registry/abi-numbers.json\"));\n");
    output.push_str("pub const RULE_AREA_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/tables/rule-areas.json\"));\n");
    output.push_str("pub const AFIDL_PRIMITIVE_TABLE_BYTES: &[u8] = include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../spec/tables/afidl-primitives.json\"));\n");
    output
        .push_str("pub static CURRENT_NORMATIVE_SOURCES: &[crate::support::CurrentSource] = &[\n");
    for entry in entries {
        let path = entry.normative_source.logical_path;
        println!("cargo:rerun-if-changed={}", workspace.join(&path).display());
        writeln!(
            output,
            "crate::support::CurrentSource {{ specification_id: {:?}, logical_path: {:?}, bytes: include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../\", {:?})) }},",
            entry.specification_id, path, path
        )
        .expect("write generated profile");
    }
    output.push_str("];\n");
    let inventory_path =
        workspace.join("spec/conformance/portable-proof-codec/materialization.json");
    println!("cargo:rerun-if-changed={}", inventory_path.display());
    let inventory: Materialization = serde_json::from_slice(
        &fs::read(inventory_path).expect("read registered set ordering inventory"),
    )
    .expect("parse set ordering inventory");
    let mut set_output =
        String::from("pub static REGISTERED_SET_RULES: &[crate::orders::SetRule] = &[\n");
    for rule in inventory.set_rules {
        let owner = rule
            .owner
            .strip_prefix("agentflow.portable.proof.inputs@1/")
            .expect("set rule belongs to Portable Proof Inputs");
        writeln!(
            set_output,
            "crate::orders::SetRule {{ owner: {:?}, field: {:?}, key: {:?}, element_type: {:?}, complete_collection: {} }},",
            owner, rule.field, rule.identity_key, rule.element_type, rule.collection.is_some(),
        )
        .expect("write set ordering rule");
    }
    set_output.push_str("];\n");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out_dir.join("current_profile_sources.rs"), output)
        .expect("write generated current profile");
    fs::write(out_dir.join("registered_set_rules.rs"), set_output)
        .expect("write generated set-order rules");
}
