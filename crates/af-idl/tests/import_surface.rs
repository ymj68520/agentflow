//! Static and behavioral self-checks for the Phase 2C import capability.
//!
//! These tests state the invariants that no data-driven fixture can express:
//! the resolver has no I/O capability, the import layer introduces no new
//! HashV1 domain, the canonical model gained no import member, and the
//! pre-existing `compile(source)` entry point is unchanged.

use af_idl::{AfidlError, CanonicalBody, CanonicalTypeExpr, compile, compile_with_universe};
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

/// Strips `//` line comments and everything from a `#[cfg(test)]` attribute
/// onward, so prose may discuss a forbidden word while declarations may not
/// use it.
fn code_only(source: &str) -> String {
    let without_tests = match source.find("#[cfg(test)]") {
        Some(index) => &source[..index],
        None => source,
    };
    without_tests
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
        "af-idl must not use {what} (found {needle:?})"
    );
}

#[test]
fn the_import_resolver_has_no_io_capability() {
    // The resolver is a pure function of source text plus an in-memory
    // universe. There is no filesystem, network, environment or clock
    // dependency anywhere in the crate - a "latest version" or registry
    // lookup is not merely discouraged, it is unrepresentable.
    let manifest = read("crates/af-idl/Cargo.toml");
    for forbidden in [
        "reqwest",
        "hyper",
        "ureq",
        "curl",
        "tokio",
        "async-std",
        "walkdir",
        "glob",
    ] {
        assert_absent(
            &manifest,
            forbidden,
            &format!("an I/O or network dependency ({forbidden})"),
        );
    }
    let mut sources = String::new();
    for module in [
        "resolve.rs",
        "parser.rs",
        "validate.rs",
        "canonical.rs",
        "lib.rs",
        "lexer.rs",
    ] {
        sources.push_str(&code_only(&read(&format!("crates/af-idl/src/{module}"))));
        sources.push('\n');
    }
    for forbidden in [
        "std::fs",
        "std::net",
        "std::env",
        "std::time",
        "std::process",
    ] {
        assert_absent(&sources, forbidden, forbidden);
    }
    // No registry concept may enter the resolver: it matches exact identities.
    for forbidden in ["latest", "version_range", "lock_graph", "LockGraph"] {
        assert_absent(
            &sources,
            forbidden,
            &format!("a resolver concept ({forbidden})"),
        );
    }
}

#[test]
fn imports_introduce_no_new_hash_domain() {
    // The frozen HashV1 domain registry is untouched by cross-module imports:
    // module identity never becomes a digest domain. Import resolution adds
    // validity, not identity, so no domain is needed for it.
    let table = read("spec/tables/hash-domains.json");
    let value: serde_json::Value = serde_json::from_str(&table).unwrap();
    assert_eq!(
        value["domains"].as_array().unwrap().len(),
        14,
        "the frozen HashV1 domain registry changed size"
    );
    for forbidden in ["schema-import", "schema-module", "schema-universe"] {
        assert_absent(
            &table,
            forbidden,
            &format!("the HashV1 domain {forbidden:?}"),
        );
    }
    // The crate computes no digest of its own beyond TypeSchemaDigest.
    let canonical = code_only(&read("crates/af-idl/src/canonical.rs"));
    assert!(
        canonical.contains("DigestDomain::TypeSchema"),
        "TypeSchemaDigest must remain the crate's only digest computation"
    );
    for forbidden in ["hash_v1", "Sha256"] {
        let resolver = code_only(&read("crates/af-idl/src/resolve.rs"));
        assert_absent(
            &resolver,
            forbidden,
            &format!("direct hashing in the resolver ({forbidden})"),
        );
    }
}

#[test]
fn the_canonical_type_schema_gained_no_import_member() {
    // The canonical record is frozen at schema/name/body. An import member
    // would change every digest; its absence is the reason an alias rename is
    // non-semantic.
    let canonical = code_only(&read("crates/af-idl/src/canonical.rs"));
    let start = canonical
        .find("fn schema_shape")
        .expect("schema_shape is the canonical encoder");
    let shape = &canonical[start..];
    assert!(
        shape.contains(r#"("schema", Shape::String)"#)
            && shape.contains(r#"("name", Shape::String)"#)
            && shape.contains(r#"("body", Shape::Variant(body))"#),
        "the canonical TypeSchema record must still be schema/name/body"
    );
    assert_absent(
        shape,
        "imports",
        "an import member in the canonical TypeSchema record",
    );

    // Structurally: the canonical model has no import type at all.
    let ast = code_only(&read("crates/af-idl/src/canonical.rs"));
    assert_absent(
        &ast,
        "ImportDecl",
        "an import type in the canonical machine model",
    );
}

#[test]
fn the_pre_existing_compile_entry_point_is_unchanged() {
    // A module without imports must compile identically through both entry
    // points: the import capability is strictly additive.
    let source = "afidl 1;\n\
        schema agentflow.example@1;\n\
        record User { 1 id: string; 2 name: string; }\n";
    let plain = compile(source).unwrap();
    let with_empty_universe =
        compile_with_universe(source, &af_idl::ResolvedSchemaUniverse::new()).unwrap();
    assert_eq!(plain, with_empty_universe);
    assert_eq!(
        plain.by_name("User").unwrap().ace1_bytes().unwrap(),
        with_empty_universe
            .by_name("User")
            .unwrap()
            .ace1_bytes()
            .unwrap()
    );
}

#[test]
fn a_local_reference_is_never_satisfied_by_an_imported_type() {
    // No implicit import and no shadowing: the same source fails without the
    // universe, succeeds with a qualified reference, and never resolves bare
    // `Slot` against the imported module.
    let universe_source = "afidl 1;\n\
        schema agentflow.test.imported@1;\n\
        alias Slot = uint32;\n";
    let mut universe = af_idl::ResolvedSchemaUniverse::new();
    universe.insert(compile(universe_source).unwrap()).unwrap();

    let unqualified = "afidl 1;\n\
        schema agentflow.test.consumer@1;\n\
        import agentflow.test.imported@1 as dep;\n\
        record Bad { 1 slot: Slot; }\n";
    let error = compile_with_universe(unqualified, &universe).unwrap_err();
    assert_eq!(
        error.kind(),
        af_idl::error::kind::UNQUALIFIED_IMPORTED_REFERENCE,
        "an unqualified name must never resolve to an imported type"
    );
    let message = error.to_string();
    assert!(
        message.contains("dep.Slot"),
        "the diagnostic should name the qualified spelling, got: {message}"
    );

    let qualified = unqualified.replace("1 slot: Slot;", "1 slot: dep.Slot;");
    let compiled = compile_with_universe(&qualified, &universe).unwrap();
    let CanonicalBody::Record(fields) = &compiled.by_name("Bad").unwrap().body else {
        panic!("record expected");
    };
    assert_eq!(
        fields[0].type_expr,
        CanonicalTypeExpr::NamedRef("agentflow.test.imported@1/Slot".into())
    );

    // The universe-free path reports the plain unresolved reference: it has no
    // way to know the name exists elsewhere, and must not guess.
    let Err(AfidlError::UnknownTypeReference { .. }) = compile(unqualified) else {
        panic!("the local-only path must report an unresolved reference");
    };
}

/// The Phase 3A semantic core is a real cross-module import target: a module
/// that imports `agentflow.semantic.core@1` and refers to its types through
/// qualified references must compile, without the semantic core being modified
/// in any way.
#[test]
fn the_semantic_core_is_usable_as_an_import_target() {
    let core_source = read("spec/idl/semantic-core.afidl");
    let mut universe = af_idl::ResolvedSchemaUniverse::new();
    let core = compile(&core_source).expect("the semantic core must compile standalone");
    assert_eq!(core.specification.canonical(), "agentflow.semantic.core@1");
    assert_eq!(core.types.len(), 29, "the semantic core type set is frozen");
    universe.insert(core).unwrap();

    // The forward-looking shape Phase 3B needs: a machine schema that reuses
    // the semantic core's declarations by identity.
    let ir_source = "afidl 1;\n\
        schema agentflow.semantic.ir@1;\n\
        import agentflow.semantic.core@1 as core;\n\
        record SemanticIrDocument {\n\
            1 types: list<core.SemanticTypeNode>;\n\
            2 values: list<core.ValueSlot>;\n\
            3 expressions: list<core.ExpressionDef>;\n\
            4 resources: list<core.ResourceSlot>;\n\
        }\n";
    let ir =
        compile_with_universe(ir_source, &universe).expect("the semantic core must be importable");
    let document = ir.by_name("SemanticIrDocument").unwrap();
    let CanonicalBody::Record(fields) = &document.body else {
        panic!("record expected");
    };
    // Every reference lowers to the semantic core's full logical identity: the
    // local `core` alias is source spelling only.
    for (field, expected) in fields.iter().zip([
        "agentflow.semantic.core@1/SemanticTypeNode",
        "agentflow.semantic.core@1/ValueSlot",
        "agentflow.semantic.core@1/ExpressionDef",
        "agentflow.semantic.core@1/ResourceSlot",
    ]) {
        let CanonicalTypeExpr::List(inner) = &field.type_expr else {
            panic!("field {} must be a list", field.name);
        };
        assert_eq!(
            **inner,
            CanonicalTypeExpr::NamedRef(expected.into()),
            "field {} must lower to {expected}",
            field.name
        );
    }

    // A reference to a type the semantic core does not declare fails: the
    // universe is the authority, not the alias.
    let missing = ir_source.replace("core.ValueSlot", "core.Absent");
    assert_eq!(
        compile_with_universe(&missing, &universe)
            .unwrap_err()
            .kind(),
        af_idl::error::kind::UNKNOWN_IMPORTED_TYPE
    );
}
