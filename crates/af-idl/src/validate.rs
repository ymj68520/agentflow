//! AFIDL1 schema validation.
//!
//! Fail-closed checks over the parsed module, in order:
//!
//! 1. module header: import declarations (alias grammar is a parse-time
//!    concern, so validation covers duplicates and self-imports);
//! 2. duplicate type names (no silent merging, even for identical bodies);
//! 3. per-declaration structural rules: numbering `> 0` and unique, names
//!    unique, non-empty variants/enums;
//! 4. alias cycle detection (alias-to-alias references through any wrapper
//!    position);
//! 5. named-reference resolution — every `NamedRef` must resolve to a
//!    declaration in the same compilation unit and every `QualifiedNamedRef`
//!    must use a declared import alias (this is where unknown identifiers
//!    fail; the parser never accepts "maybe primitives" and a qualified
//!    reference is never silently reinterpreted as a local one);
//! 6. recursion detection (Phase 2A scoped restriction): any direct or
//!    indirect reference cycle through record/variant/alias expansion is
//!    rejected, because inline structural recursion cannot produce finite
//!    ACE1 values and cyclic canonical schema hashes are out of scope.
//!
//! Checks 1-6 are *universe-free*: they are decidable from the module's own
//! source. Whether an imported specification exists and really declares the
//! referenced type is a stronger question and belongs to
//! [`crate::resolve`], which takes an explicit `ResolvedSchemaUniverse`.

use crate::ast::{Decl, Module, TypeExpr, TypeExprKind};
use crate::error::AfidlError;
use crate::span::SourceSpan;
use std::collections::BTreeMap;

/// Validates a parsed module. On success the module is safe to canonicalize.
pub fn validate(module: &Module) -> Result<(), AfidlError> {
    check_module_header(module)?;

    let mut index: BTreeMap<&str, &Decl> = BTreeMap::new();
    for decl in &module.declarations {
        if index.insert(decl.name(), decl).is_some() {
            return Err(AfidlError::DuplicateTypeDefinition {
                name: decl.name().to_string(),
                span: decl.span(),
            });
        }
    }

    for decl in &module.declarations {
        check_declaration(decl)?;
    }

    check_alias_cycles(module)?;
    check_resolution_and_recursion(module, &index)?;
    Ok(())
}

/// Module header rules (AF-AFIDL-0018 .. AF-AFIDL-0021).
fn check_module_header(module: &Module) -> Result<(), AfidlError> {
    let this = module.schema.id.canonical();
    let mut aliases: Vec<&str> = Vec::new();
    let mut specifications: Vec<String> = Vec::new();
    for import in &module.imports {
        if aliases.contains(&import.alias.as_str()) {
            return Err(AfidlError::DuplicateImportAlias {
                alias: import.alias.clone(),
                span: import.span,
            });
        }
        aliases.push(import.alias.as_str());
        let id = import.id.canonical();
        if id == this {
            return Err(AfidlError::SelfImport {
                id,
                span: import.span,
            });
        }
        if specifications.contains(&id) {
            return Err(AfidlError::DuplicateImportSpecification {
                id,
                span: import.span,
            });
        }
        specifications.push(id);
    }
    Ok(())
}

fn check_declaration(decl: &Decl) -> Result<(), AfidlError> {
    match decl {
        Decl::Alias(_) => Ok(()),
        Decl::Enum(e) => {
            if e.values.is_empty() {
                return Err(AfidlError::EmptyEnum {
                    name: e.name.clone(),
                    span: e.span,
                });
            }
            let mut numbers = Vec::new();
            let mut names = Vec::new();
            for value in &e.values {
                if value.no == 0 {
                    return Err(AfidlError::InvalidEnumNumber {
                        no: value.no,
                        type_name: e.name.clone(),
                        span: value.span,
                    });
                }
                if numbers.contains(&value.no) {
                    return Err(AfidlError::DuplicateEnumNumber {
                        no: value.no,
                        type_name: e.name.clone(),
                        span: value.span,
                    });
                }
                if names.contains(&value.name) {
                    return Err(AfidlError::DuplicateEnumName {
                        name: value.name.clone(),
                        type_name: e.name.clone(),
                        span: value.span,
                    });
                }
                numbers.push(value.no);
                names.push(value.name.clone());
            }
            Ok(())
        }
        Decl::Record(r) => {
            let mut numbers = Vec::new();
            let mut names = Vec::new();
            for field in &r.fields {
                if field.no == 0 {
                    return Err(AfidlError::InvalidFieldNumber {
                        no: field.no,
                        type_name: r.name.clone(),
                        span: field.span,
                    });
                }
                if numbers.contains(&field.no) {
                    return Err(AfidlError::DuplicateFieldNumber {
                        no: field.no,
                        type_name: r.name.clone(),
                        span: field.span,
                    });
                }
                if names.contains(&field.name) {
                    return Err(AfidlError::DuplicateFieldName {
                        name: field.name.clone(),
                        type_name: r.name.clone(),
                        span: field.span,
                    });
                }
                numbers.push(field.no);
                names.push(field.name.clone());
            }
            Ok(())
        }
        Decl::Variant(v) => {
            if v.cases.is_empty() {
                return Err(AfidlError::EmptyVariant {
                    name: v.name.clone(),
                    span: v.span,
                });
            }
            let mut numbers = Vec::new();
            let mut names = Vec::new();
            for case in &v.cases {
                if case.no == 0 {
                    return Err(AfidlError::InvalidCaseNumber {
                        no: case.no,
                        type_name: v.name.clone(),
                        span: case.span,
                    });
                }
                if numbers.contains(&case.no) {
                    return Err(AfidlError::DuplicateCaseNumber {
                        no: case.no,
                        type_name: v.name.clone(),
                        span: case.span,
                    });
                }
                if names.contains(&case.name) {
                    return Err(AfidlError::DuplicateCaseName {
                        name: case.name.clone(),
                        type_name: v.name.clone(),
                        span: case.span,
                    });
                }
                numbers.push(case.no);
                names.push(case.name.clone());
            }
            Ok(())
        }
    }
}

/// Collects every named reference appearing anywhere inside a type expression.
fn collect_refs(expr: &TypeExpr, out: &mut Vec<(String, SourceSpan)>) {
    match &expr.kind {
        TypeExprKind::Primitive(_) | TypeExprKind::QualifiedNamedRef(_) => {}
        TypeExprKind::NamedRef(name) => out.push((name.clone(), expr.span)),
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => collect_refs(inner, out),
    }
}

/// Collects every *qualified* reference appearing anywhere inside a type
/// expression, paired with the source form that produced it.
fn collect_qualified_refs<'e>(expr: &'e TypeExpr, out: &mut Vec<&'e crate::ast::QualifiedRef>) {
    match &expr.kind {
        TypeExprKind::QualifiedNamedRef(reference) => out.push(reference),
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => collect_qualified_refs(inner, out),
        TypeExprKind::Primitive(_) | TypeExprKind::NamedRef(_) => {}
    }
}

/// Alias graph: alias name -> (alias targets declared in the module, span of
/// the alias declaration that closes a cycle would be reported against).
fn check_alias_cycles(module: &Module) -> Result<(), AfidlError> {
    let mut edges: BTreeMap<String, Vec<(String, SourceSpan)>> = BTreeMap::new();
    for decl in &module.declarations {
        if let Decl::Alias(alias) = decl {
            let mut refs = Vec::new();
            collect_refs(&alias.target, &mut refs);
            let entry = edges.entry(alias.name.clone()).or_default();
            for (referenced, _) in refs {
                if module.declarations.iter().any(|d| d.name() == referenced) {
                    entry.push((referenced, alias.span));
                }
            }
        }
    }

    for start in edges.keys().cloned().collect::<Vec<_>>() {
        let mut path: Vec<String> = vec![start.clone()];
        dfs_alias(&start, &edges, &mut path)?;
    }
    Ok(())
}

fn dfs_alias(
    current: &str,
    edges: &BTreeMap<String, Vec<(String, SourceSpan)>>,
    path: &mut Vec<String>,
) -> Result<(), AfidlError> {
    if let Some(targets) = edges.get(current) {
        for (target, closing_span) in targets.clone() {
            if path.contains(&target) {
                let mut cycle: Vec<String> = path.clone();
                cycle.push(target);
                return Err(AfidlError::AliasCycle {
                    cycle,
                    span: closing_span,
                });
            }
            path.push(target.clone());
            dfs_alias(&target, edges, path)?;
            path.pop();
        }
    }
    Ok(())
}

fn check_resolution_and_recursion<'m>(
    module: &'m Module,
    index: &BTreeMap<&'m str, &'m Decl>,
) -> Result<(), AfidlError> {
    // Resolution: every reference must resolve inside the compilation unit,
    // and every qualified reference must use a declared import alias.
    for decl in &module.declarations {
        for expr in decl.type_exprs() {
            resolve_refs(expr, index)?;
            check_qualified_aliases(module, expr)?;
        }
    }
    // Recursion (scoped restriction): expand aliases and reject any cycle.
    for decl in &module.declarations {
        for expr in decl.type_exprs() {
            let mut path = vec![decl.name()];
            walk_recursion(decl.name(), expr, index, &mut path)?;
        }
    }
    Ok(())
}

/// A qualified reference's alias must exist in the module header; the target
/// type's existence is checked against the universe in [`crate::resolve`].
fn check_qualified_aliases(module: &Module, expr: &TypeExpr) -> Result<(), AfidlError> {
    let mut references = Vec::new();
    collect_qualified_refs(expr, &mut references);
    for reference in references {
        if !module
            .imports
            .iter()
            .any(|import| import.alias == reference.import_alias)
        {
            return Err(AfidlError::UnknownImportAlias {
                alias: reference.import_alias.clone(),
                span: reference.span,
            });
        }
    }
    Ok(())
}

fn resolve_refs<'m>(
    expr: &'m TypeExpr,
    index: &BTreeMap<&'m str, &'m Decl>,
) -> Result<(), AfidlError> {
    match &expr.kind {
        // Primitives need no resolution, and a qualified reference is resolved
        // against the import table (and, in the import-aware modes, against the
        // supplied universe): it is never silently reinterpreted as a local
        // reference (AF-AFIDL-0023).
        TypeExprKind::Primitive(_) | TypeExprKind::QualifiedNamedRef(_) => Ok(()),
        TypeExprKind::NamedRef(name) => {
            if index.contains_key(name.as_str()) {
                Ok(())
            } else {
                Err(AfidlError::UnknownTypeReference {
                    name: name.clone(),
                    span: expr.span,
                })
            }
        }
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => resolve_refs(inner, index),
    }
}

fn walk_recursion<'m>(
    start: &'m str,
    expr: &'m TypeExpr,
    index: &BTreeMap<&'m str, &'m Decl>,
    path: &mut Vec<&'m str>,
) -> Result<(), AfidlError> {
    match &expr.kind {
        // A qualified reference leaves the compilation unit: it is terminal
        // for this module's local recursion walk, exactly as a primitive is.
        // The cross-module type graph is checked by `compile_module_set`,
        // which sees every participating module at once.
        TypeExprKind::Primitive(_) | TypeExprKind::QualifiedNamedRef(_) => Ok(()),
        TypeExprKind::NamedRef(name) => {
            if name == start || path.contains(&name.as_str()) {
                return Err(AfidlError::RecursiveTypeNotSupported {
                    type_name: start.to_string(),
                    span: expr.span,
                });
            }
            let referenced = *index
                .get(name.as_str())
                .expect("resolved during validation");
            match referenced {
                Decl::Alias(alias) => {
                    path.push(name.as_str());
                    let result = walk_recursion(start, &alias.target, index, path);
                    path.pop();
                    result
                }
                Decl::Record(record) => {
                    path.push(name.as_str());
                    let mut result = Ok(());
                    for field in &record.fields {
                        result = walk_recursion(start, &field.type_expr, index, path);
                        if result.is_err() {
                            break;
                        }
                    }
                    path.pop();
                    result
                }
                Decl::Variant(variant) => {
                    path.push(name.as_str());
                    let mut result = Ok(());
                    for case in &variant.cases {
                        result = walk_recursion(start, &case.payload, index, path);
                        if result.is_err() {
                            break;
                        }
                    }
                    path.pop();
                    result
                }
                Decl::Enum(_) => Ok(()), // enums carry no payloads: terminal
            }
        }
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => walk_recursion(start, inner, index, path),
    }
}
