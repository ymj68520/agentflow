//! Import resolution against an explicit, immutable schema universe.
//!
//! ## What this module is
//!
//! A **resolver for exact specification identities**, not a package resolver:
//!
//! - input: an in-memory [`ResolvedSchemaUniverse`] (`SpecificationId` →
//!   compiled module) supplied by the caller;
//! - operation: check that every `import` names a specification present in
//!   that universe, and that every qualified reference names a type that
//!   specification actually declares;
//! - guarantee: the universe never changes during resolution, and nothing
//!   here reads the filesystem, the network, a registry, or a "latest"
//!   version (AF-AFIDL-0016).
//!
//! Version negotiation, package download, dependency solving, lock files and
//! yank/revoke handling are *not* implemented here; they belong to the future
//! Resolver / LockGraph stage.
//!
//! ## Identity
//!
//! An import names an exact `SpecificationId`, so the identity a qualified
//! reference lowers to is fully determined by the importing module's own
//! source. Resolution therefore adds *validity*, not identity: a module
//! canonicalizes to the same `CanonicalTypeSchema`s whether or not its
//! imported modules happen to be present. What resolution rejects is a
//! reference the author could not have meant — a missing specification, a
//! missing type, or a self/duplicate import.
//!
//! ## Cross-module recursion
//!
//! AFIDL1 rejects recursive named types (Phase 2A scoped restriction), and
//! imports must not become a way around that rule. The check that can only be
//! made with every module in hand — the global named-type dependency graph —
//! lives in [`compile_module_set`], which builds
//! `<SpecificationId>/<TypeName>` → referenced named types and rejects any
//! cycle, whether intra- or cross-module. A module compiled on its own against
//! an *already compiled* universe still has its own declarations checked
//! against those known dependencies; it cannot, on its own, prove that an
//! unknown future module will not refer back to it.

use crate::ast::{Decl, Module, TypeExpr, TypeExprKind};
use crate::canonical::CompiledSchema;
use crate::error::AfidlError;
use crate::span::SourceSpan;
use crate::specification::SpecificationId;
use std::collections::{BTreeMap, BTreeSet};

/// An explicit, immutable, in-memory set of already-compiled schema modules,
/// keyed by exact `SpecificationId`.
///
/// The universe performs no I/O and has no notion of versions: an entry is
/// found only by byte-exact specification identity (AF-AFIDL-0015,
/// AF-AFIDL-0017).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResolvedSchemaUniverse {
    modules: BTreeMap<String, CompiledSchema>,
}

impl ResolvedSchemaUniverse {
    /// An empty universe.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a compiled module under its own specification identity.
    ///
    /// # Errors
    /// [`AfidlError::DuplicateModuleSpecification`] when the same identity is
    /// registered twice, whichever order the two registrations arrive in.
    pub fn insert(&mut self, module: CompiledSchema) -> Result<(), AfidlError> {
        let id = module.specification.canonical();
        if self.modules.contains_key(&id) {
            return Err(AfidlError::DuplicateModuleSpecification {
                id,
                label: "universe".into(),
                span: SourceSpan::new(0, 0, 1, 1),
            });
        }
        self.modules.insert(id, module);
        Ok(())
    }

    /// Looks up a module by exact specification identity.
    pub fn get(&self, id: &str) -> Option<&CompiledSchema> {
        self.modules.get(id)
    }

    /// `true` when the universe holds a module with exactly this identity.
    pub fn contains(&self, id: &str) -> bool {
        self.modules.contains_key(id)
    }

    /// The registered specification identities, sorted.
    pub fn specification_ids(&self) -> impl Iterator<Item = &str> {
        self.modules.keys().map(String::as_str)
    }

    /// Number of registered modules.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// `true` when no module is registered.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }
}

/// A compiled module set: every participating module's canonical schemas,
/// keyed by full logical type identity `<SpecificationId>/<TypeName>`.
///
/// This is the artifact of [`compile_module_set`]: a validated, acyclic,
/// fully resolved schema universe.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CompiledModuleSet {
    modules: BTreeMap<String, CompiledSchema>,
}

impl CompiledModuleSet {
    /// A module by exact specification identity.
    pub fn module(&self, id: &str) -> Option<&CompiledSchema> {
        self.modules.get(id)
    }

    /// Every module, in specification-identity order.
    pub fn modules(&self) -> impl Iterator<Item = &CompiledSchema> {
        self.modules.values()
    }

    /// Number of modules in the set.
    pub fn len(&self) -> usize {
        self.modules.len()
    }

    /// `true` when the set is empty.
    pub fn is_empty(&self) -> bool {
        self.modules.is_empty()
    }

    /// A canonical named type by full logical identity.
    pub fn type_by_identity(
        &self,
        identity: &str,
    ) -> Option<&crate::canonical::CanonicalTypeSchema> {
        let (id, name) = identity.rsplit_once('/')?;
        self.modules.get(id)?.by_name(name)
    }

    /// Derives the finite ACE1 `Shape` of a named type, expanding named
    /// references across every module of the set (a qualified source reference
    /// lowers to a full logical identity, so expansion simply follows those
    /// identities into the declaring module).
    ///
    /// Termination is guaranteed by [`compile_module_set`]'s global
    /// named-type acyclicity validation: the reference graph this walk follows
    /// is exactly the graph that was proven acyclic.
    ///
    /// # Errors
    /// [`AfidlError::UnknownTypeReference`] when `identity` (or a reference
    /// inside its expansion) is not part of the set.
    pub fn shape_of(&self, identity: &str) -> Result<af_core::Shape, AfidlError> {
        let schema_type = self
            .type_by_identity(identity)
            .ok_or_else(|| unknown_identity(identity))?;
        let mut memo: BTreeMap<String, af_core::Shape> = BTreeMap::new();
        Self::body_shape(self, &schema_type.name, &schema_type.body, &mut memo)
    }

    fn body_shape(
        set: &Self,
        name: &str,
        body: &crate::canonical::CanonicalBody,
        memo: &mut BTreeMap<String, af_core::Shape>,
    ) -> Result<af_core::Shape, AfidlError> {
        use crate::canonical::CanonicalBody;
        Ok(match body {
            CanonicalBody::Alias(target) => Self::expr_shape(set, target, memo)?,
            CanonicalBody::Enum(values) => af_core::Shape::Enum(af_core::EnumShape::new(
                name,
                values.iter().map(|value| value.name.clone()),
            )),
            CanonicalBody::Record(fields) => {
                let mut members = Vec::with_capacity(fields.len());
                for field in fields {
                    members.push((
                        field.name.clone(),
                        Self::expr_shape(set, &field.type_expr, memo)?,
                    ));
                }
                af_core::Shape::Record(af_core::RecordShape::new(name, members))
            }
            CanonicalBody::Variant(cases) => {
                let mut members = Vec::with_capacity(cases.len());
                for case in cases {
                    members.push((
                        case.name.clone(),
                        Self::expr_shape(set, &case.type_expr, memo)?,
                    ));
                }
                af_core::Shape::Variant(af_core::VariantShape::new(name, members))
            }
        })
    }

    fn expr_shape(
        set: &Self,
        expr: &crate::canonical::CanonicalTypeExpr,
        memo: &mut BTreeMap<String, af_core::Shape>,
    ) -> Result<af_core::Shape, AfidlError> {
        use crate::canonical::CanonicalTypeExpr;
        Ok(match expr {
            CanonicalTypeExpr::Primitive(primitive) => primitive.shape(),
            CanonicalTypeExpr::NamedRef(target_identity) => {
                if let Some(shape) = memo.get(target_identity) {
                    return Ok(shape.clone());
                }
                let target = set
                    .type_by_identity(target_identity)
                    .ok_or_else(|| unknown_identity(target_identity))?;
                let shape = Self::body_shape(set, &target.name, &target.body, memo)?;
                memo.insert(target_identity.clone(), shape.clone());
                shape
            }
            CanonicalTypeExpr::List(inner) => {
                af_core::Shape::List(Box::new(Self::expr_shape(set, inner, memo)?))
            }
            CanonicalTypeExpr::MapString(inner) => {
                af_core::Shape::Map(Box::new(Self::expr_shape(set, inner, memo)?))
            }
            CanonicalTypeExpr::Option(inner) => {
                af_core::Shape::Option(Box::new(Self::expr_shape(set, inner, memo)?))
            }
        })
    }
}

fn unknown_identity(identity: &str) -> AfidlError {
    AfidlError::UnknownTypeReference {
        name: identity.to_string(),
        span: SourceSpan::new(0, 0, 1, 1),
    }
}

/// Resolves one parsed-and-validated module's imports against `universe`.
///
/// Checks, in order:
///
/// 1. every imported `SpecificationId` exists in the universe;
/// 2. every qualified reference names the alias of an import whose
///    specification declares that type.
///
/// # Errors
/// [`AfidlError::UnknownImportSpecification`],
/// [`AfidlError::UnknownImportAlias`] and [`AfidlError::UnknownImportedType`].
pub fn resolve_against_universe(
    module: &Module,
    universe: &ResolvedSchemaUniverse,
) -> Result<(), AfidlError> {
    for import in &module.imports {
        let id = import.id.canonical();
        if !universe.contains(&id) {
            return Err(AfidlError::UnknownImportSpecification {
                id,
                span: import.span,
            });
        }
    }
    for decl in &module.declarations {
        for expr in decl.type_exprs() {
            resolve_qualified(module, universe, expr)?;
        }
    }
    Ok(())
}

fn resolve_qualified(
    module: &Module,
    universe: &ResolvedSchemaUniverse,
    expr: &TypeExpr,
) -> Result<(), AfidlError> {
    match &expr.kind {
        TypeExprKind::QualifiedNamedRef(reference) => {
            let Some(import) = module
                .imports
                .iter()
                .find(|import| import.alias == reference.import_alias)
            else {
                return Err(AfidlError::UnknownImportAlias {
                    alias: reference.import_alias.clone(),
                    span: reference.span,
                });
            };
            let id = import.id.canonical();
            let Some(target) = universe.get(&id) else {
                return Err(AfidlError::UnknownImportSpecification {
                    id,
                    span: import.span,
                });
            };
            if target.by_name(&reference.type_name).is_none() {
                return Err(AfidlError::UnknownImportedType {
                    specification: id,
                    type_name: reference.type_name.clone(),
                    span: reference.span,
                });
            }
            Ok(())
        }
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => resolve_qualified(module, universe, inner),
        TypeExprKind::Primitive(_) | TypeExprKind::NamedRef(_) => Ok(()),
    }
}

/// A name-only index of `<SpecificationId>` → declared type names.
///
/// Used to explain an unqualified reference that a module the author imported
/// happens to declare, and to answer "which specification declares this type"
/// without compiling anything. It is derived data: it carries no schema
/// content and never participates in identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeclaredTypeIndex {
    names: BTreeMap<String, BTreeSet<String>>,
}

impl DeclaredTypeIndex {
    /// An empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Builds the index of every module in `universe`.
    pub fn from_universe(universe: &ResolvedSchemaUniverse) -> Self {
        let mut index = Self::new();
        for id in universe.specification_ids() {
            let module = universe.get(id).expect("identity from the universe");
            index.names.insert(
                id.to_string(),
                module.type_names().map(ToString::to_string).collect(),
            );
        }
        index
    }

    /// Records the declared type names of one module.
    pub fn insert_module<I, S>(&mut self, id: &str, names: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let entry = self.names.entry(id.to_string()).or_default();
        for name in names {
            entry.insert(name.into());
        }
    }

    /// `true` when the module with this identity is present in the index (it
    /// may legitimately declare no types at all).
    pub fn declares_specification(&self, id: &str) -> bool {
        self.names.contains_key(id)
    }

    /// `true` when the module with this identity declares the type.
    pub fn declares(&self, id: &str, type_name: &str) -> bool {
        self.names
            .get(id)
            .is_some_and(|names| names.contains(type_name))
    }

    /// The identity of a module declaring `type_name`, if any. Ties are broken
    /// by specification-identity order so the answer is deterministic.
    pub fn declaring_specification(&self, type_name: &str) -> Option<&str> {
        self.names
            .iter()
            .find(|(_, names)| names.contains(type_name))
            .map(|(id, _)| id.as_str())
    }
}

/// Upgrades an `unknown_type_reference` into a precise
/// [`AfidlError::UnqualifiedImportedReference`] when the unresolved local name
/// is actually declared by a module the author already imported.
///
/// The rule being explained is AF-AFIDL-0023: an external type is reachable
/// only through a qualified reference, so a bare `Bar` that only exists in an
/// imported module is a mistake, not an implicit import. The local-only
/// [`crate::compile`] path has no universe and therefore keeps reporting the
/// plain unresolved-reference error.
#[must_use]
pub fn explain_unqualified_reference(
    module: &Module,
    index: &DeclaredTypeIndex,
    error: &AfidlError,
) -> Option<AfidlError> {
    let AfidlError::UnknownTypeReference { name, span } = error else {
        return None;
    };
    for import in &module.imports {
        if index.declares(&import.id.canonical(), name) {
            return Some(AfidlError::UnqualifiedImportedReference {
                type_name: name.clone(),
                specification: import.id.canonical(),
                alias: import.alias.clone(),
                span: *span,
            });
        }
    }
    None
}

/// Compiles a set of AFIDL1 sources together and validates the *global* named
/// type dependency graph.
///
/// Each `(label, source)` pair is compiled: the label is used only in
/// diagnostics. Every module is parsed before anything is resolved, so a
/// module set is a set — the order of the pairs never matters and two modules
/// may legally import each other as long as their *type* dependency graph
/// stays acyclic (AF-AFIDL-0025: the module import graph is not required to be
/// acyclic; the named-type graph is).
///
/// # Errors
/// Any [`AfidlError`] from parsing, validation, import resolution, or
/// [`AfidlError::CrossModuleRecursiveType`] for a reference cycle that crosses
/// a module boundary.
pub fn compile_module_set(sources: &[(String, String)]) -> Result<CompiledModuleSet, AfidlError> {
    let mut parsed: Vec<(String, Module)> = Vec::with_capacity(sources.len());
    for (label, source) in sources {
        parsed.push((label.clone(), crate::parse(source)?));
    }

    // Duplicate schema families inside one set are rejected outright: the set
    // would otherwise not be a function of specification identity.
    let mut families: BTreeMap<String, String> = BTreeMap::new();
    for (label, module) in &parsed {
        let id = module.schema.id.canonical();
        if families.insert(id.clone(), label.clone()).is_some() {
            return Err(AfidlError::DuplicateModuleSpecification {
                id,
                label: label.clone(),
                span: SourceSpan::new(0, 0, 1, 1),
            });
        }
    }

    // Name-only view of the whole set: identities and declared type names are
    // known before any module is canonicalized, which is exactly what makes a
    // cyclic import graph resolvable.
    let mut index = DeclaredTypeIndex::new();
    for (_, module) in &parsed {
        index.insert_module(
            &module.schema.id.canonical(),
            module
                .declarations
                .iter()
                .map(|decl| decl.name().to_string()),
        );
    }

    for (_, module) in &parsed {
        if let Err(error) = crate::validate(module) {
            if let Some(upgraded) = explain_unqualified_reference(module, &index, &error) {
                return Err(upgraded);
            }
            return Err(error);
        }
        resolve_qualified_all(module, &index)?;
    }

    let mut set = CompiledModuleSet::default();
    for (_, module) in &parsed {
        let id = module.schema.id.canonical();
        set.modules.insert(id, crate::canonicalize(module));
    }

    check_global_recursion(&set)?;
    Ok(set)
}

/// Resolves one module's imports and qualified references against a name-only
/// index of a module set.
fn resolve_qualified_all(module: &Module, index: &DeclaredTypeIndex) -> Result<(), AfidlError> {
    for import in &module.imports {
        let id = import.id.canonical();
        if index.declares_specification(&id) {
            continue;
        }
        return Err(AfidlError::UnknownImportSpecification {
            id,
            span: import.span,
        });
    }
    for decl in &module.declarations {
        for expr in decl.type_exprs() {
            resolve_qualified_against(&module.imports, index, expr)?;
        }
    }
    Ok(())
}

fn resolve_qualified_against(
    imports: &[crate::ast::ImportDecl],
    index: &DeclaredTypeIndex,
    expr: &TypeExpr,
) -> Result<(), AfidlError> {
    match &expr.kind {
        TypeExprKind::QualifiedNamedRef(reference) => {
            let Some(import) = imports
                .iter()
                .find(|import| import.alias == reference.import_alias)
            else {
                return Err(AfidlError::UnknownImportAlias {
                    alias: reference.import_alias.clone(),
                    span: reference.span,
                });
            };
            let id = import.id.canonical();
            if !index.declares_specification(&id) {
                return Err(AfidlError::UnknownImportSpecification {
                    id,
                    span: import.span,
                });
            }
            if !index.declares(&id, &reference.type_name) {
                return Err(AfidlError::UnknownImportedType {
                    specification: id,
                    type_name: reference.type_name.clone(),
                    span: reference.span,
                });
            }
            Ok(())
        }
        TypeExprKind::List(inner)
        | TypeExprKind::MapString(inner)
        | TypeExprKind::Option(inner) => resolve_qualified_against(imports, index, inner),
        TypeExprKind::Primitive(_) | TypeExprKind::NamedRef(_) => Ok(()),
    }
}

/// Builds the global `<SpecificationId>/<TypeName>` → referenced named types
/// graph over every module and rejects any reference cycle.
///
/// This is the authoritative cross-module recursion check: the graph is built
/// from *canonical* type expressions, so it covers alias targets, record
/// fields, variant case payloads and every `list`/`map`/`option` wrapper, and
/// it spans modules because a `NamedRef` carries a full logical identity.
fn check_global_recursion(set: &CompiledModuleSet) -> Result<(), AfidlError> {
    let mut edges: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for module in set.modules() {
        for schema_type in &module.types {
            let identity = schema_type.full_identity();
            let mut targets = Vec::new();
            collect_expr_targets(&schema_type.body, &mut targets);
            edges.entry(identity).or_default().extend(targets);
        }
    }

    let mut done: BTreeSet<String> = BTreeSet::new();
    for start in edges.keys().cloned().collect::<Vec<_>>() {
        if done.contains(&start) {
            continue;
        }
        let mut path = vec![start.clone()];
        walk_global(&start, &edges, &mut path, &mut done)?;
    }
    Ok(())
}

fn collect_expr_targets(body: &crate::canonical::CanonicalBody, out: &mut Vec<String>) {
    use crate::canonical::{CanonicalBody, CanonicalTypeExpr};
    fn walk(expr: &CanonicalTypeExpr, out: &mut Vec<String>) {
        match expr {
            CanonicalTypeExpr::Primitive(_) => {}
            CanonicalTypeExpr::NamedRef(identity) => out.push(identity.clone()),
            CanonicalTypeExpr::List(inner)
            | CanonicalTypeExpr::MapString(inner)
            | CanonicalTypeExpr::Option(inner) => walk(inner, out),
        }
    }
    match body {
        CanonicalBody::Alias(target) => walk(target, out),
        CanonicalBody::Enum(_) => {}
        CanonicalBody::Record(fields) => {
            for field in fields {
                walk(&field.type_expr, out);
            }
        }
        CanonicalBody::Variant(cases) => {
            for case in cases {
                walk(&case.type_expr, out);
            }
        }
    }
}

fn walk_global(
    current: &str,
    edges: &BTreeMap<String, Vec<String>>,
    path: &mut Vec<String>,
    done: &mut BTreeSet<String>,
) -> Result<(), AfidlError> {
    if let Some(targets) = edges.get(current) {
        for target in targets {
            if path.iter().any(|entry| entry == target) {
                let mut cycle: Vec<String> = path.clone();
                cycle.push(target.clone());
                return Err(AfidlError::CrossModuleRecursiveType {
                    cycle,
                    span: SourceSpan::new(0, 0, 1, 1),
                });
            }
            // A reference to an identity outside the set (an imported module
            // that is not part of this compilation) is terminal here: its own
            // internals are the responsibility of the module that declared it.
            // An already-finished node needs no second visit (diamond
            // dependencies would otherwise re-walk shared subgraphs).
            if !edges.contains_key(target.as_str()) || done.contains(target) {
                continue;
            }
            path.push(target.clone());
            walk_global(target, edges, path, done)?;
            path.pop();
        }
    }
    done.insert(current.to_string());
    Ok(())
}

/// Declared type names of a module, used by callers that need the local type
/// universe (for example to reject an unqualified reference that is only
/// satisfied by an imported module).
pub fn local_type_names(module: &Module) -> BTreeSet<&str> {
    module.declarations.iter().map(Decl::name).collect()
}

/// The specification identity a module declares.
pub fn module_identity(module: &Module) -> SpecificationId {
    module.schema.id.clone()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile;

    const CORE: &str = "afidl 1;\n\
        schema agentflow.core@1;\n\
        alias ValueSlotId = uint32;\n\
        record Ref { 1 slot: ValueSlotId; }\n";

    const USER: &str = "afidl 1;\n\
        schema agentflow.user@1;\n\
        import agentflow.core@1 as core;\n\
        record Holder { 1 reference: core.Ref; 2 name: string; }\n";

    fn universe_of(sources: &[&str]) -> ResolvedSchemaUniverse {
        let mut universe = ResolvedSchemaUniverse::new();
        for source in sources {
            universe.insert(compile(source).unwrap()).unwrap();
        }
        universe
    }

    #[test]
    fn qualified_references_resolve_against_the_universe() {
        let universe = universe_of(&[CORE]);
        let compiled = crate::compile_with_universe(USER, &universe).unwrap();
        let holder = compiled.by_name("Holder").unwrap();
        let crate::canonical::CanonicalBody::Record(fields) = &holder.body else {
            panic!("record expected");
        };
        assert_eq!(
            fields[0].type_expr,
            crate::canonical::CanonicalTypeExpr::NamedRef("agentflow.core@1/Ref".into())
        );
    }

    #[test]
    fn missing_specification_and_missing_type_are_distinct_errors() {
        let empty = ResolvedSchemaUniverse::new();
        assert_eq!(
            crate::compile_with_universe(USER, &empty)
                .unwrap_err()
                .kind(),
            crate::error::kind::UNKNOWN_IMPORT_SPECIFICATION
        );

        let other =
            universe_of(&["afidl 1;\n schema agentflow.core@1;\n record Other { 1 x: string; }\n"]);
        assert_eq!(
            crate::compile_with_universe(USER, &other)
                .unwrap_err()
                .kind(),
            crate::error::kind::UNKNOWN_IMPORTED_TYPE
        );
    }

    #[test]
    fn alias_rename_does_not_change_the_digest() {
        let universe = universe_of(&[CORE]);
        let renamed = USER
            .replace("as core;", "as semantic;")
            .replace("core.Ref", "semantic.Ref");
        let a = crate::compile_with_universe(USER, &universe).unwrap();
        let b = crate::compile_with_universe(&renamed, &universe).unwrap();
        assert_eq!(a.digest("Holder"), b.digest("Holder"));
    }

    #[test]
    fn import_order_does_not_change_the_digest() {
        let universe = universe_of(&[CORE]);
        let two = "afidl 1;\n\
            schema agentflow.user@1;\n\
            import agentflow.core@1 as core;\n\
            record Holder { 1 reference: core.Ref; }\n";
        let reordered = "afidl 1;\n\
            schema agentflow.user@1;\n\
            import agentflow.core@1 as semantic;\n\
            record Holder { 1 reference: semantic.Ref; }\n";
        let a = crate::compile_with_universe(two, &universe).unwrap();
        let b = crate::compile_with_universe(reordered, &universe).unwrap();
        assert_eq!(a.digest("Holder"), b.digest("Holder"));
    }

    #[test]
    fn unused_import_does_not_change_the_digest() {
        let universe = universe_of(&[CORE]);
        let plain = "afidl 1;\nschema agentflow.user@1;\nrecord Holder { 1 name: string; }\n";
        let with_import = "afidl 1;\nschema agentflow.user@1;\n\
            import agentflow.core@1 as core;\n\
            record Holder { 1 name: string; }\n";
        let a = crate::compile_with_universe(plain, &universe).unwrap();
        let b = crate::compile_with_universe(with_import, &universe).unwrap();
        assert_eq!(a.digest("Holder"), b.digest("Holder"));
        assert_eq!(
            a.by_name("Holder").unwrap().ace1_bytes().unwrap(),
            b.by_name("Holder").unwrap().ace1_bytes().unwrap()
        );
    }

    #[test]
    fn compile_module_set_resolves_multi_module_sets() {
        let set = compile_module_set(&[
            ("user.afidl".into(), USER.into()),
            ("core.afidl".into(), CORE.into()),
        ])
        .unwrap();
        assert_eq!(set.len(), 2);
        assert_eq!(
            set.type_by_identity("agentflow.core@1/Ref").unwrap().name,
            "Ref"
        );
    }

    #[test]
    fn cross_module_cycles_are_rejected() {
        let a = "afidl 1;\nschema agentflow.a@1;\nimport agentflow.b@1 as b;\n\
                 record A { 1 other: b.B; }\n";
        let b = "afidl 1;\nschema agentflow.b@1;\nimport agentflow.a@1 as a;\n\
                 record B { 1 other: a.A; }\n";
        let error =
            compile_module_set(&[("a".into(), a.into()), ("b".into(), b.into())]).unwrap_err();
        assert_eq!(
            error.kind(),
            crate::error::kind::CROSS_MODULE_RECURSIVE_TYPE
        );
    }

    #[test]
    fn a_cycle_through_a_wrapper_is_still_a_cycle() {
        // The reference is wrapped in list/option/map: expansion is still
        // infinite, so the cycle must be rejected exactly as for a direct one.
        let a = "afidl 1;\nschema agentflow.a@1;\nimport agentflow.b@1 as b;\n\
                 record A { 1 items: list<option<b.B>>; }\n";
        let b = "afidl 1;\nschema agentflow.b@1;\nimport agentflow.a@1 as a;\n\
                 record B { 1 back: map<string,a.A>; }\n";
        let error =
            compile_module_set(&[("a".into(), a.into()), ("b".into(), b.into())]).unwrap_err();
        assert_eq!(
            error.kind(),
            crate::error::kind::CROSS_MODULE_RECURSIVE_TYPE
        );
    }

    #[test]
    fn three_module_cycles_are_rejected() {
        let a = "afidl 1;\nschema agentflow.a@1;\nimport agentflow.b@1 as b;\n\
                 record A { 1 next: b.B; }\n";
        let b = "afidl 1;\nschema agentflow.b@1;\nimport agentflow.c@1 as c;\n\
                 record B { 1 next: c.C; }\n";
        let c = "afidl 1;\nschema agentflow.c@1;\nimport agentflow.a@1 as a;\n\
                 record C { 1 next: a.A; }\n";
        let error = compile_module_set(&[
            ("a".into(), a.into()),
            ("b".into(), b.into()),
            ("c".into(), c.into()),
        ])
        .unwrap_err();
        assert_eq!(
            error.kind(),
            crate::error::kind::CROSS_MODULE_RECURSIVE_TYPE
        );
    }

    #[test]
    fn mutual_imports_without_a_type_cycle_are_legal() {
        // A imports B and B imports A, but no type depends on the other's type.
        let a = "afidl 1;\nschema agentflow.a@1;\nimport agentflow.b@1 as b;\n\
                 record A { 1 name: string; }\n";
        let b = "afidl 1;\nschema agentflow.b@1;\nimport agentflow.a@1 as a;\n\
                 record B { 1 name: string; }\n";
        let set = compile_module_set(&[("a".into(), a.into()), ("b".into(), b.into())]).unwrap();
        assert_eq!(set.len(), 2);
    }

    #[test]
    fn duplicate_specifications_in_a_set_are_rejected() {
        let error = compile_module_set(&[("one".into(), CORE.into()), ("two".into(), CORE.into())])
            .unwrap_err();
        assert_eq!(
            error.kind(),
            crate::error::kind::DUPLICATE_MODULE_SPECIFICATION
        );
    }

    #[test]
    fn universe_rejects_duplicate_insertions() {
        let mut universe = ResolvedSchemaUniverse::new();
        universe.insert(compile(CORE).unwrap()).unwrap();
        assert_eq!(
            universe.insert(compile(CORE).unwrap()).unwrap_err().kind(),
            crate::error::kind::DUPLICATE_MODULE_SPECIFICATION
        );
    }

    #[test]
    fn unqualified_reference_to_a_local_type_is_still_local() {
        // A local declaration named the same as an imported type is the local
        // one: imports never shadow or merge into the local namespace.
        let universe = universe_of(&[CORE]);
        let module = "afidl 1;\nschema agentflow.user@1;\n\
            import agentflow.core@1 as core;\n\
            record Ref { 1 local: string; }\n\
            record Holder { 1 reference: Ref; }\n";
        let compiled = crate::compile_with_universe(module, &universe).unwrap();
        let holder = compiled.by_name("Holder").unwrap();
        let crate::canonical::CanonicalBody::Record(fields) = &holder.body else {
            panic!("record expected");
        };
        assert_eq!(
            fields[0].type_expr,
            crate::canonical::CanonicalTypeExpr::NamedRef("agentflow.user@1/Ref".into())
        );
    }

    #[test]
    fn local_only_modules_compile_identically_with_and_without_a_universe() {
        let universe = universe_of(&[CORE]);
        let a = compile(CORE).unwrap();
        let b = crate::compile_with_universe(CORE, &universe).unwrap();
        assert_eq!(a, b);
    }

    #[test]
    fn module_set_shapes_expand_across_modules() {
        let set = compile_module_set(&[
            ("core.afidl".into(), CORE.into()),
            ("user.afidl".into(), USER.into()),
        ])
        .unwrap();
        let shape = set.shape_of("agentflow.user@1/Holder").unwrap();
        shape.validate().expect("cross-module shape is valid");
        // Holder.reference: core.Ref expands into the core module's fields.
        let af_core::Shape::Record(record) = &shape else {
            panic!("record shape expected");
        };
        let reference = record
            .fields
            .iter()
            .find(|field| field.name == "reference")
            .map(|field| &field.shape)
            .unwrap();
        assert_eq!(
            *reference,
            set.shape_of("agentflow.core@1/Ref").unwrap(),
            "the qualified reference must expand into the imported module's shape"
        );
        // A value of the cross-module type encodes against the derived shape.
        let value = af_core::Value::Record(vec![
            (
                "reference".into(),
                af_core::Value::Record(vec![("slot".into(), af_core::Value::UInt32(7))]),
            ),
            ("name".into(), af_core::Value::String("n".into())),
        ]);
        let bytes = af_core::encode(&shape, &value).unwrap();
        assert_eq!(
            std::str::from_utf8(&bytes).unwrap(),
            r#"{"name":"n","reference":{"slot":"7"}}"#
        );
    }

    #[test]
    fn module_set_shape_rejects_foreign_identities() {
        let set = compile_module_set(&[("core.afidl".into(), CORE.into())]).unwrap();
        assert_eq!(
            set.shape_of("agentflow.user@1/Holder").unwrap_err().kind(),
            crate::error::kind::UNKNOWN_TYPE_REFERENCE
        );
        assert_eq!(
            set.shape_of("agentflow.core@1/Missing").unwrap_err().kind(),
            crate::error::kind::UNKNOWN_TYPE_REFERENCE
        );
    }
}
