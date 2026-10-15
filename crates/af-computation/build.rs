use std::{env, fmt::Write as _, fs, path::PathBuf};
fn main() {
    let workspace = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("../..");
    let registry = workspace.join("spec/registry/specifications.json");
    println!("cargo:rerun-if-changed={}", registry.display());
    let json: serde_json::Value = serde_json::from_slice(&fs::read(registry).unwrap()).unwrap();
    let mut out = String::from("const REGISTERED_SOURCE_CONTENTS: &[(&str,&str,&[u8])] = &[\n");
    for row in json["specifications"].as_array().unwrap() {
        let id = row["specification_id"].as_str().unwrap();
        let path = row["normative_source"]["logical_path"].as_str().unwrap();
        assert!(!path.contains("..") && !path.contains('\\') && !path.starts_with('/'));
        println!("cargo:rerun-if-changed={}", workspace.join(path).display());
        writeln!(out,"({id:?},{path:?},include_bytes!(concat!(env!(\"CARGO_MANIFEST_DIR\"),\"/../../\",{path:?}))),").unwrap();
    }
    out.push_str("];\n");
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("registered_edition_sources.rs"),
        out,
    )
    .unwrap();
}
