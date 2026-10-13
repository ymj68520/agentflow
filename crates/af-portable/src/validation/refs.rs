use super::ReferenceValidatedPortableBundle;
use crate::codec::CanonicalPortableBundle;
use crate::error::{PortableProofError, ValidationStage};
use af_core::{DigestDomain, Value};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn validate_d2(
    canonical: CanonicalPortableBundle,
) -> Result<ReferenceValidatedPortableBundle, PortableProofError> {
    let graph_work_limit = canonical.limits.max_graph_work;
    let root = canonical.value();
    let assertions = table(root, "root_export_assertions")?;
    let contexts = table(root, "proof_contexts")?;
    let resolvers = table(root, "resolver_dependency_manifests")?;
    let compilers = table(root, "compiler_semantics_manifests")?;
    let structural = table(root, "structural_types")?;

    let assertion_ids = id_map(assertions, "$.root_export_assertions")?;
    let context_ids = id_map(contexts, "$.proof_contexts")?;
    let resolver_ids = id_map(resolvers, "$.resolver_dependency_manifests")?;
    let compiler_ids = id_map(compilers, "$.compiler_semantics_manifests")?;
    let structural_ids = id_map(structural, "$.structural_types")?;

    let root_id = u32_field(root, "root_assertion_id", "$.root_assertion_id")?;
    let root_assertion = assertion_ids.get(&root_id).ok_or_else(|| {
        err(
            "MissingAssertion",
            "$.root_assertion_id",
            "root AssertionId does not resolve",
        )
    })?;
    let assertion = field(root_assertion, "assertion", "$.root_assertion")?;
    let context_id = u32_field(
        field(assertion, "proof_context_manifest", "$")?,
        "context_id",
        "$.root_assertion.proof_context_manifest.context_id",
    )?;
    let resolver_id = u32_field(
        field(assertion, "proof_context_manifest", "$")?,
        "resolver_id",
        "$.root_assertion.proof_context_manifest.resolver_id",
    )?;
    let compiler_id = u32_field(
        assertion,
        "compiler_semantics_id",
        "$.root_assertion.compiler_semantics_id",
    )?;
    if !context_ids.contains_key(&context_id) {
        return Err(err(
            "MissingContext",
            "$.root_assertion.proof_context_manifest.context_id",
            "ContextId does not resolve",
        ));
    }
    let resolver_entry = resolver_ids.get(&resolver_id).ok_or_else(|| {
        err(
            "MissingResolver",
            "$.root_assertion.proof_context_manifest.resolver_id",
            "ResolverId does not resolve",
        )
    })?;
    let resolver = field(resolver_entry, "content", "$.resolver")?;
    if !compiler_ids.contains_key(&compiler_id) {
        return Err(err(
            "MissingCompilerManifest",
            "$.root_assertion.compiler_semantics_id",
            "CompilerManifestId does not resolve",
        ));
    }
    if u32_field(resolver, "context_id", "$.resolver.context_id")? != context_id {
        return Err(err(
            "ConflictingAssociation",
            "$.resolver.context_id",
            "resolver selects a different context",
        ));
    }

    let ir_ref = field(root_assertion, "ir_asset", "$.root_assertion.ir_asset")?;
    let contract_ref = field(
        root_assertion,
        "export_contract_asset",
        "$.root_assertion.export_contract_asset",
    )?;
    require_asset(root, "ir_assets", ir_ref)?;
    require_asset(root, "contract_assets", contract_ref)?;
    equal(
        field(assertion, "exact_semantic_ir_digest", "$")?,
        field(ir_ref, "digest", "$.root_assertion.ir_asset")?,
        "$.root_assertion.exact_semantic_ir_digest",
    )?;
    equal(
        field(assertion, "exact_exported_component_contract_digest", "$")?,
        field(
            contract_ref,
            "digest",
            "$.root_assertion.export_contract_asset",
        )?,
        "$.root_assertion.exact_exported_component_contract_digest",
    )?;
    check_digest_domain(
        field(ir_ref, "digest", "$")?,
        "semantic-ir",
        "$.root_assertion.ir_asset",
    )?;
    check_type_root(
        field(ir_ref, "type_schema", "$")?,
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
        "$.root_assertion.ir_asset",
    )?;
    check_digest_domain(
        field(contract_ref, "digest", "$")?,
        "component-contract",
        "$.root_assertion.export_contract_asset",
    )?;
    check_type_root(
        field(contract_ref, "type_schema", "$")?,
        "agentflow.component.contract@1",
        "ComponentContract",
        "$.root_assertion.export_contract_asset",
    )?;

    let caller = field(resolver, "caller_ir", "$.resolver.caller_ir")?;
    equal(caller, ir_ref, "$.resolver.caller_ir")?;
    check_typed_content_role(
        caller,
        "semantic-ir",
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
        "$.resolver.caller_ir",
    )?;
    require_asset(root, "ir_assets", caller)?;
    let components = list(resolver, "components", "$.resolver.components")?;
    if !components.iter().any(|value| value == contract_ref) {
        return Err(err(
            "MissingResolverComponent",
            "$.resolver.components",
            "root export contract is not inventoried by the resolver",
        ));
    }
    for (index, component) in components.iter().enumerate() {
        check_typed_content_role(
            component,
            "component-contract",
            "agentflow.component.contract@1",
            "ComponentContract",
            &format!("$.resolver.components[{index}]"),
        )?;
        require_asset(root, "contract_assets", component)?;
    }

    let schema_assets = table(root, "schema_assets")?;
    let ir_assets = table(root, "ir_assets")?;
    let contract_assets = table(root, "contract_assets")?;
    check_all_refs_and_asset_reachability(root, schema_assets, ir_assets, contract_assets)?;
    for (index, entry) in resolvers.iter().enumerate() {
        let path = format!("$.resolver_dependency_manifests[{index}].content");
        let content = field(entry, "content", &path)?;
        check_resolver_manifest_refs(root, content, &context_ids, schema_assets, &path)?;
    }

    let mut edges = BTreeMap::<u32, Vec<u32>>::new();
    for (id, entry) in &assertion_ids {
        let body = field(entry, "assertion", "$.root_export_assertions[].assertion")?;
        let mut children = Vec::new();
        for dependency in list(
            body,
            "subflow_dependencies",
            "$.assertion.subflow_dependencies",
        )? {
            if let Some(target) = option_u32(dependency, "target_root_assertion_id")? {
                if !assertion_ids.contains_key(&target) {
                    return Err(err(
                        "MissingAssertion",
                        "$.assertion.subflow_dependencies.target_root_assertion_id",
                        "target assertion ID does not resolve",
                    ));
                }
                children.push(target);
            }
        }
        edges.insert(*id, children);
    }
    let mut graph_work = 0usize;
    let reachable_assertions =
        check_assertion_graph(root_id, &edges, &mut graph_work, graph_work_limit)?;
    if reachable_assertions.len() != edges.len() {
        return Err(err(
            "UnreferencedAssertion",
            "$.root_export_assertions",
            "assertion table contains a row outside the root target graph",
        ));
    }
    let mut structural_references = BTreeSet::new();
    for id in edges.keys() {
        if let Some(entry) = assertion_ids.get(id) {
            collect_structural_refs(field(entry, "assertion", "$")?, &mut structural_references);
        }
    }
    check_structural_graph(
        &structural_ids,
        &structural_references,
        &mut graph_work,
        graph_work_limit,
    )?;
    if !context_ids.contains_key(&context_id) {
        return Err(err(
            "MissingContext",
            "$.root_assertion.proof_context_manifest.context_id",
            "ContextId does not resolve",
        ));
    }
    Ok(ReferenceValidatedPortableBundle { canonical })
}

fn id_map<'a>(
    rows: &'a [Value],
    path: &str,
) -> Result<BTreeMap<u32, &'a Value>, PortableProofError> {
    let mut result = BTreeMap::new();
    for (index, row) in rows.iter().enumerate() {
        let item_path = format!("{path}[{index}]");
        let id = u32_field(row, "id", &item_path)?;
        if id == 0 {
            return Err(err(
                "InvalidLocalId",
                &item_path,
                "Bundle-local IDs are positive",
            ));
        }
        if result.insert(id, row).is_some() {
            return Err(err(
                "DuplicateIdentity",
                &item_path,
                "duplicate local table ID",
            ));
        }
    }
    Ok(result)
}

fn require_asset(
    root: &Value,
    table_name: &str,
    reference: &Value,
) -> Result<(), PortableProofError> {
    let identity = content_identity(reference)?;
    let rows = table(root, table_name)?;
    for row in rows {
        let candidate = field(row, "reference", "$.asset")?;
        if content_identity(candidate)? == identity {
            return Ok(());
        }
    }
    Err(err(
        "MissingReferencedContent",
        "$",
        "TypedContentRef does not resolve to an inline asset",
    ))
}

fn check_all_refs_and_asset_reachability(
    root: &Value,
    schemas: &[Value],
    irs: &[Value],
    contracts: &[Value],
) -> Result<(), PortableProofError> {
    let schema_rows = schemas
        .iter()
        .map(|row| field(row, "reference", "$"))
        .collect::<Result<Vec<_>, _>>()?;
    let ir_rows = irs
        .iter()
        .map(|row| field(row, "reference", "$"))
        .collect::<Result<Vec<_>, _>>()?;
    let contract_rows = contracts
        .iter()
        .map(|row| field(row, "reference", "$"))
        .collect::<Result<Vec<_>, _>>()?;
    let schema_ids = schema_rows
        .iter()
        .map(|row| schema_identity(row))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let ir_ids = ir_rows
        .iter()
        .map(|row| content_identity(row))
        .collect::<Result<BTreeSet<_>, _>>()?;
    let contract_ids = contract_rows
        .iter()
        .map(|row| content_identity(row))
        .collect::<Result<BTreeSet<_>, _>>()?;
    check_unique_asset_identities(schemas, "SchemaAsset")?;
    check_unique_asset_identities(irs, "IrAsset")?;
    check_unique_asset_identities(contracts, "ContractAsset")?;
    for row in irs {
        check_typed_content_role(
            field(row, "reference", "$.ir_assets")?,
            "semantic-ir",
            "agentflow.semantic.ir@1",
            "SemanticIrDocument",
            "$.ir_assets.reference",
        )?;
    }
    for row in contracts {
        check_typed_content_role(
            field(row, "reference", "$.contract_assets")?,
            "component-contract",
            "agentflow.component.contract@1",
            "ComponentContract",
            "$.contract_assets.reference",
        )?;
    }
    let mut found_schema = BTreeSet::new();
    let mut found_content = BTreeSet::new();
    collect_refs(root, &mut found_schema, &mut found_content)?;
    for key in &found_schema {
        if !schema_ids.contains(key) {
            return Err(err(
                "MissingReferencedContent",
                "$",
                "SchemaRef has no inline SchemaAsset",
            ));
        }
    }
    for key in &found_content {
        if !ir_ids.contains(key) && !contract_ids.contains(key) {
            return Err(err(
                "MissingReferencedContent",
                "$",
                "TypedContentRef has no inline IR or contract asset",
            ));
        }
    }
    if schema_ids.iter().any(|key| !found_schema.contains(key))
        || ir_ids.iter().any(|key| !found_content.contains(key))
        || contract_ids.iter().any(|key| !found_content.contains(key))
    {
        return Err(err(
            "UnreferencedAsset",
            "$",
            "an asset is not reachable from root evidence or an explicit complete inventory",
        ));
    }
    Ok(())
}

fn collect_refs(
    value: &Value,
    schemas: &mut BTreeSet<Vec<u8>>,
    contents: &mut BTreeSet<Vec<u8>>,
) -> Result<(), PortableProofError> {
    match value {
        Value::Record(fields) => {
            if fields.iter().any(|(name, _)| name == "schema_digest")
                && fields.iter().any(|(name, _)| name == "specification_id")
                && fields.iter().any(|(name, _)| name == "type_name")
            {
                schemas.insert(schema_identity(value)?);
            }
            if fields.iter().any(|(name, _)| name == "type_schema")
                && fields.iter().any(|(name, _)| name == "digest")
            {
                contents.insert(content_identity(value)?);
            }
            for (name, child) in fields {
                if matches!(
                    name.as_str(),
                    "schema_assets" | "ir_assets" | "contract_assets"
                ) {
                    continue;
                }
                collect_refs(child, schemas, contents)?;
            }
        }
        Value::Variant { value, .. } | Value::Option(Some(value)) => {
            collect_refs(value, schemas, contents)?;
        }
        Value::List(items) => {
            for item in items {
                collect_refs(item, schemas, contents)?;
            }
        }
        Value::Map(items) => {
            for (_, item) in items {
                collect_refs(item, schemas, contents)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn check_structural_graph(
    nodes: &BTreeMap<u32, &Value>,
    roots: &BTreeSet<u32>,
    work: &mut usize,
    work_limit: usize,
) -> Result<(), PortableProofError> {
    let mut referenced = roots.clone();
    let mut edges = BTreeMap::new();
    for (id, node) in nodes {
        charge_graph_work(work, work_limit, "$.structural_types")?;
        let (case, value) = variant(field(node, "body", "$.structural_types[].body")?)?;
        if matches!(case, "list" | "map_string" | "option") {
            let target = value_u32(value).ok_or_else(|| {
                err(
                    "InvalidNominalReference",
                    "$.structural_types",
                    "structural edge ID is not uint32",
                )
            })?;
            if !nodes.contains_key(&target) {
                return Err(err(
                    "MissingReferencedContent",
                    "$.structural_types",
                    "structural edge has no node",
                ));
            }
            referenced.insert(target);
            edges.insert(*id, target);
            charge_graph_work(work, work_limit, "$.structural_types")?;
        }
    }
    let mut done = BTreeSet::new();
    for root in referenced.iter().filter(|id| nodes.contains_key(id)) {
        let mut active = BTreeSet::new();
        let mut stack = vec![(*root, false)];
        while let Some((node, exiting)) = stack.pop() {
            charge_graph_work(work, work_limit, "$.structural_types")?;
            if exiting {
                active.remove(&node);
                done.insert(node);
                continue;
            }
            if done.contains(&node) {
                continue;
            }
            if !active.insert(node) {
                return Err(err(
                    "ReferenceCycle",
                    "$.structural_types",
                    "structural witness graph is cyclic",
                ));
            }
            stack.push((node, true));
            if let Some(child) = edges.get(&node) {
                charge_graph_work(work, work_limit, "$.structural_types")?;
                if active.contains(child) {
                    return Err(err(
                        "ReferenceCycle",
                        "$.structural_types",
                        "structural witness graph is cyclic",
                    ));
                }
                if !done.contains(child) {
                    stack.push((*child, false));
                }
            }
        }
    }
    if nodes.keys().any(|id| !referenced.contains(id)) {
        return Err(err(
            "UnreferencedAsset",
            "$.structural_types",
            "structural type table contains padding",
        ));
    }
    Ok(())
}

fn collect_structural_refs(value: &Value, result: &mut BTreeSet<u32>) {
    match value {
        Value::Record(fields) => {
            for (name, item) in fields {
                if matches!(name.as_str(), "root_type" | "root_result_type") {
                    if let Some(id) = value_u32(item) {
                        result.insert(id);
                    }
                }
                collect_structural_refs(item, result);
            }
        }
        Value::Variant { value, .. } | Value::Option(Some(value)) => {
            collect_structural_refs(value, result);
        }
        Value::List(items) => {
            for item in items {
                collect_structural_refs(item, result);
            }
        }
        Value::Map(items) => {
            for (_, item) in items {
                collect_structural_refs(item, result);
            }
        }
        _ => {}
    }
}

fn check_assertion_graph(
    root: u32,
    edges: &BTreeMap<u32, Vec<u32>>,
    work: &mut usize,
    work_limit: usize,
) -> Result<BTreeSet<u32>, PortableProofError> {
    let mut active = BTreeSet::new();
    let mut done = BTreeSet::new();
    let mut stack = vec![(root, false)];
    while let Some((node, exit)) = stack.pop() {
        charge_graph_work(work, work_limit, "$.root_export_assertions")?;
        if exit {
            active.remove(&node);
            done.insert(node);
            continue;
        }
        if done.contains(&node) {
            continue;
        }
        if !active.insert(node) {
            return Err(err(
                "ReferenceCycle",
                "$.root_export_assertions",
                "assertion target graph is cyclic",
            ));
        }
        stack.push((node, true));
        for child in edges.get(&node).into_iter().flatten().rev() {
            charge_graph_work(work, work_limit, "$.root_export_assertions")?;
            if active.contains(child) {
                return Err(err(
                    "ReferenceCycle",
                    "$.root_export_assertions",
                    "assertion target graph is cyclic",
                ));
            }
            if !done.contains(child) {
                stack.push((*child, false));
            }
        }
    }
    Ok(done)
}

fn charge_graph_work(work: &mut usize, limit: usize, path: &str) -> Result<(), PortableProofError> {
    *work = work.saturating_add(1);
    if *work > limit {
        return Err(PortableProofError::new(
            ValidationStage::D2,
            "ResourceLimit",
            path,
            "graph validation work limit exceeded",
        ));
    }
    Ok(())
}

fn check_digest_domain(
    value: &Value,
    expected: &str,
    path: &str,
) -> Result<(), PortableProofError> {
    let algorithm = text(field(value, "algorithm", path)?, path)?;
    let domain = text(field(value, "domain", path)?, path)?;
    let hex = text(field(value, "hex", path)?, path)?;
    if algorithm != "sha256"
        || !lower_hex64(hex)
        || DigestDomain::from_registry_name(domain).is_err()
    {
        return Err(err(
            "WrongDigestDomain",
            path,
            "invalid registered DigestRef",
        ));
    }
    if domain != expected {
        return Err(err(
            "WrongDigestDomain",
            path,
            "DigestRef domain does not match its typed role",
        ));
    }
    Ok(())
}
fn equal(left: &Value, right: &Value, path: &str) -> Result<(), PortableProofError> {
    if left == right {
        Ok(())
    } else {
        Err(err(
            "ConflictingAssociation",
            path,
            "typed root assertion pin differs from its asset reference",
        ))
    }
}

fn check_type_root(
    value: &Value,
    family: &str,
    name: &str,
    path: &str,
) -> Result<(), PortableProofError> {
    if text(field(value, "specification_id", path)?, path)? != family
        || text(field(value, "type_name", path)?, path)? != name
    {
        return Err(err(
            "InvalidNominalReference",
            path,
            "typed asset has the wrong nominal root SchemaRef",
        ));
    }
    check_digest_domain(field(value, "schema_digest", path)?, "type-schema", path)
}

fn check_unique_asset_identities(rows: &[Value], label: &str) -> Result<(), PortableProofError> {
    let mut identities = BTreeSet::new();
    for (index, row) in rows.iter().enumerate() {
        let reference = field(row, "reference", "$.assets")?;
        let identity = if label == "SchemaAsset" {
            schema_identity(reference)?
        } else {
            content_identity(reference)?
        };
        if !identities.insert(identity) {
            return Err(err(
                "DuplicateIdentity",
                &format!("$.{label}[{index}].reference"),
                "asset content identity is duplicated",
            ));
        }
    }
    Ok(())
}

fn check_typed_content_role(
    value: &Value,
    domain: &str,
    schema: &str,
    name: &str,
    path: &str,
) -> Result<(), PortableProofError> {
    check_digest_domain(field(value, "digest", path)?, domain, path)?;
    check_type_root(field(value, "type_schema", path)?, schema, name, path)
}

fn check_resolver_manifest_refs(
    root: &Value,
    resolver: &Value,
    contexts: &BTreeMap<u32, &Value>,
    schema_assets: &[Value],
    path: &str,
) -> Result<(), PortableProofError> {
    let context_id = u32_field(resolver, "context_id", path)?;
    if !contexts.contains_key(&context_id) {
        return Err(err(
            "MissingContext",
            path,
            "resolver ContextId does not resolve",
        ));
    }
    let caller = field(resolver, "caller_ir", path)?;
    check_typed_content_role(
        caller,
        "semantic-ir",
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
        path,
    )?;
    require_asset(root, "ir_assets", caller)?;

    for component in list(resolver, "components", path)? {
        check_typed_content_role(
            component,
            "component-contract",
            "agentflow.component.contract@1",
            "ComponentContract",
            path,
        )?;
        require_asset(root, "contract_assets", component)?;
    }

    let targets = list(resolver, "subflow_targets", path)?;
    let mut target_keys = BTreeSet::new();
    for target in targets {
        let target_digest = field(target, "target_semantic_ir_digest", path)?;
        check_digest_domain(target_digest, "semantic-ir", path)?;
        if !target_keys.insert(digest_identity(target_digest)?) {
            return Err(err(
                "DuplicateIdentity",
                path,
                "resolver target digest identity is duplicated",
            ));
        }
        if let Some(target_content) = option_value(target, "target_ir_content", path)? {
            check_typed_content_role(
                target_content,
                "semantic-ir",
                "agentflow.semantic.ir@1",
                "SemanticIrDocument",
                path,
            )?;
            equal(target_digest, field(target_content, "digest", path)?, path)?;
            require_asset(root, "ir_assets", target_content)?;
        }
        if let Some(contract) = option_value(target, "exported_contract", path)? {
            check_typed_content_role(
                contract,
                "component-contract",
                "agentflow.component.contract@1",
                "ComponentContract",
                path,
            )?;
            require_asset(root, "contract_assets", contract)?;
        }
        if let Some(dependencies) = option_list(target, "direct_dependencies", path)? {
            for dependency in dependencies {
                check_digest_domain(dependency, "semantic-ir", path)?;
            }
        }
        if let Some(dependencies) = option_list(target, "transitive_dependencies", path)? {
            let mut transitive_keys = BTreeSet::new();
            for dependency in dependencies {
                let digest = field(dependency, "semantic_ir_digest", path)?;
                check_digest_domain(digest, "semantic-ir", path)?;
                let mut key = digest_identity(digest)?;
                if let Some(contract) = option_digest(dependency, "exported_contract_digest", path)?
                {
                    check_digest_domain(contract, "component-contract", path)?;
                    key.extend(digest_identity(contract)?);
                }
                if !transitive_keys.insert(key) {
                    return Err(err(
                        "DuplicateIdentity",
                        path,
                        "transitive resolver target identity is duplicated",
                    ));
                }
            }
        }
    }

    let mut source_keys = BTreeSet::new();
    for association in list(resolver, "source_associations", path)? {
        let source_caller = field(association, "caller_ir", path)?;
        check_typed_content_role(
            source_caller,
            "semantic-ir",
            "agentflow.semantic.ir@1",
            "SemanticIrDocument",
            path,
        )?;
        require_asset(root, "ir_assets", source_caller)?;
        equal(caller, source_caller, path)?;
        let source_node = u32_field(association, "source_node", path)?;
        if source_node == 0 {
            return Err(err(
                "InvalidLocalId",
                path,
                "source ComputationId must be positive",
            ));
        }
        let mut source_key = content_identity(source_caller)?;
        source_key.extend(source_node.to_be_bytes());
        if !source_keys.insert(source_key) {
            return Err(err(
                "DuplicateIdentity",
                path,
                "caller/source-node association is duplicated",
            ));
        }
        let (kind, target) = variant(field(association, "target", path)?)?;
        match kind {
            "invoke" => {
                check_typed_content_role(
                    target,
                    "component-contract",
                    "agentflow.component.contract@1",
                    "ComponentContract",
                    path,
                )?;
                require_asset(root, "contract_assets", target)?;
            }
            "call_subflow" => {
                let digest = field(target, "target_semantic_ir_digest", path)?;
                check_digest_domain(digest, "semantic-ir", path)?;
                let exported = option_digest(target, "exported_contract_digest", path)?;
                if let Some(contract) = exported {
                    check_digest_domain(contract, "component-contract", path)?;
                }
                let mut found = false;
                for candidate in targets {
                    if resolver_target_matches(candidate, digest, exported)? {
                        found = true;
                        break;
                    }
                }
                if !found {
                    return Err(err(
                        "MissingResolverTarget",
                        path,
                        "source subflow association has no exact target/export row",
                    ));
                }
            }
            "await_signal" => {
                let key = schema_identity(target)?;
                if !schema_assets.iter().any(|asset| {
                    field(asset, "reference", path)
                        .and_then(schema_identity)
                        .is_ok_and(|candidate| candidate == key)
                }) {
                    return Err(err(
                        "MissingReferencedContent",
                        path,
                        "await-signal SchemaRef has no inline SchemaAsset",
                    ));
                }
            }
            _ => {
                return Err(err(
                    "UnknownVariant",
                    path,
                    "unknown SourceAssociationTarget case",
                ));
            }
        }
    }
    Ok(())
}

fn resolver_target_matches(
    target: &Value,
    digest: &Value,
    exported: Option<&Value>,
) -> Result<bool, PortableProofError> {
    if field(target, "target_semantic_ir_digest", "$.resolver_target")? != digest {
        return Ok(false);
    }
    match (
        option_value(target, "exported_contract", "$.resolver_target")?,
        exported,
    ) {
        (None, None) => Ok(true),
        (Some(content), Some(digest)) => {
            Ok(field(content, "digest", "$.resolver_target.exported_contract")? == digest)
        }
        _ => Ok(false),
    }
}

fn digest_identity(value: &Value) -> Result<Vec<u8>, PortableProofError> {
    let mut key = Vec::new();
    for name in ["algorithm", "domain", "hex"] {
        push_part(
            &mut key,
            text(field(value, name, "$.DigestRef")?, "$.DigestRef")?.as_bytes(),
        );
    }
    Ok(key)
}

fn option_value<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<Option<&'a Value>, PortableProofError> {
    match field(value, name, path)? {
        Value::Option(None) => Ok(None),
        Value::Option(Some(inner)) => Ok(Some(inner)),
        _ => Err(err("InvalidOption", path, "field is not a tagged Option")),
    }
}

fn option_list<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<Option<&'a [Value]>, PortableProofError> {
    match option_value(value, name, path)? {
        None => Ok(None),
        Some(Value::List(items)) => Ok(Some(items)),
        Some(_) => Err(err(
            "InvalidTypedShape",
            path,
            "Some dependency value is not a list",
        )),
    }
}

fn option_digest<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<Option<&'a Value>, PortableProofError> {
    option_value(value, name, path)
}

fn schema_identity(value: &Value) -> Result<Vec<u8>, PortableProofError> {
    check_digest_domain(
        field(value, "schema_digest", "$.SchemaRef")?,
        "type-schema",
        "$.SchemaRef",
    )?;
    let mut result = Vec::new();
    for field_name in ["specification_id", "type_name"] {
        push_part(
            &mut result,
            text(field(value, field_name, "$.SchemaRef")?, "$.SchemaRef")?.as_bytes(),
        );
    }
    let digest = field(value, "schema_digest", "$.SchemaRef")?;
    for name in ["algorithm", "domain", "hex"] {
        push_part(
            &mut result,
            text(field(digest, name, "$.SchemaRef")?, "$.SchemaRef")?.as_bytes(),
        );
    }
    Ok(result)
}
fn content_identity(value: &Value) -> Result<Vec<u8>, PortableProofError> {
    let mut out = Vec::new();
    let digest = field(value, "digest", "$.TypedContentRef")?;
    let domain = text(
        field(digest, "domain", "$.TypedContentRef.digest")?,
        "$.TypedContentRef.digest.domain",
    )?;
    check_digest_domain(digest, domain, "$.TypedContentRef.digest")?;
    for name in ["algorithm", "domain", "hex"] {
        push_part(
            &mut out,
            text(
                field(digest, name, "$.TypedContentRef")?,
                "$.TypedContentRef",
            )?
            .as_bytes(),
        );
    }
    push_part(
        &mut out,
        &schema_identity(field(value, "type_schema", "$.TypedContentRef")?)?,
    );
    Ok(out)
}
fn push_part(target: &mut Vec<u8>, bytes: &[u8]) {
    target.extend(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    target.extend(bytes);
}
fn lower_hex64(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn value_u32(value: &Value) -> Option<u32> {
    match value {
        Value::UInt32(number) => Some(*number),
        _ => None,
    }
}
fn u32_field(value: &Value, name: &str, path: &str) -> Result<u32, PortableProofError> {
    value_u32(field(value, name, path)?)
        .ok_or_else(|| err("InvalidLocalId", path, "expected uint32"))
}
fn option_u32(value: &Value, name: &str) -> Result<Option<u32>, PortableProofError> {
    match field(value, name, "$")? {
        Value::Option(None) => Ok(None),
        Value::Option(Some(inner)) => value_u32(inner)
            .map(Some)
            .ok_or_else(|| err("InvalidLocalId", "$", "option ID has wrong width")),
        _ => Err(err("InvalidOption", "$", "expected tagged option")),
    }
}
fn text<'a>(value: &'a Value, path: &str) -> Result<&'a str, PortableProofError> {
    match value {
        Value::String(s) | Value::Enum(s) => Ok(s),
        _ => Err(err("InvalidTypedShape", path, "expected text")),
    }
}
fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(err("InvalidTypedShape", path, "expected record"));
    };
    fields
        .iter()
        .find(|(key, _)| key == name)
        .map(|(_, v)| v)
        .ok_or_else(|| err("MissingField", path, "required field missing"))
}
fn table<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(err("InvalidTypedShape", "$", "expected Bundle record"));
    };
    match fields.iter().find(|(key, _)| key == name).map(|(_, v)| v) {
        Some(Value::List(values)) => Ok(values),
        _ => Err(err("InvalidTypedShape", "$", "Bundle table must be list")),
    }
}
fn list<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a [Value], PortableProofError> {
    let child = field(value, name, path)?;
    match child {
        Value::List(values) => Ok(values),
        _ => Err(err("InvalidTypedShape", path, "expected list")),
    }
}
fn variant(value: &Value) -> Result<(&str, &Value), PortableProofError> {
    match value {
        Value::Variant { case, value } => Ok((case, value)),
        _ => Err(err("InvalidTypedShape", "$", "expected variant")),
    }
}
fn err(family: &'static str, path: &str, detail: &str) -> PortableProofError {
    PortableProofError::new(ValidationStage::D2, family, path, detail)
}
