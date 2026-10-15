use super::ContentValidatedPortableBundle;
use crate::error::{PortableProofError, ValidationStage};
use crate::schema::{compiled_modules, type_schema_shape};
use crate::validation::ReferenceValidatedPortableBundle;
use af_core::{DigestDomain, DigestRef, Value, decode, encode};
use af_idl::{CanonicalBody, CanonicalTypeExpr, CanonicalTypeSchema, Primitive};
use af_spec::model::{RulesFile, SpecificationsFile};
use serde::de::DeserializeSeed;
use sha2::{Digest as _, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::sync::{Mutex, OnceLock};

static VALID_SCHEMA_TABLES: OnceLock<Mutex<HashSet<[u8; 32]>>> = OnceLock::new();
static VALID_CONTENT_TABLES: OnceLock<Mutex<HashSet<[u8; 32]>>> = OnceLock::new();

pub(crate) fn validate_d3(
    referenced: ReferenceValidatedPortableBundle,
) -> Result<ContentValidatedPortableBundle, PortableProofError> {
    let root = referenced.canonical.value();
    let compiled = compiled_modules()?;
    let schema_shape = type_schema_shape();
    let schema_assets = table(root, "schema_assets")?;
    let schema_key = asset_table_cache_key("schema_assets", schema_assets, true)?;
    if !cache_contains(&VALID_SCHEMA_TABLES, &schema_key) {
        for (index, asset) in schema_assets.iter().enumerate() {
            let path = format!("$.schema_assets[{index}]");
            let reference = field(asset, "reference", &path)?;
            let bytes = bytes_field(asset, "canonical_content", &path)?;
            let value = decode(schema_shape, bytes)
                .map_err(|e| d3("InvalidTypeSchema", &path, &e.to_string()))?;
            let schema = parse_schema(&value, &path)?;
            check_schema_body(&schema, &path)?;
            let encoded = schema
                .ace1_bytes()
                .map_err(|e| d3("InvalidTypeSchema", &path, &e.to_string()))?;
            if encoded != bytes {
                return Err(d3(
                    "NoncanonicalTypeSchema",
                    &path,
                    "canonical TypeSchema bytes differ",
                ));
            }
            let digest = DigestRef::compute(DigestDomain::TypeSchema, bytes);
            check_schema_pin(reference, &schema.schema, &schema.name, &digest, &path)?;
            if let Some(registered) =
                compiled.type_by_identity(&format!("{}/{}", schema.schema, schema.name))
            {
                if registered.ace1_bytes().ok().as_deref() != Some(bytes) {
                    return Err(d3(
                        "RegisteredSchemaMismatch",
                        &path,
                        "registered schema differs from its exact AFIDL source",
                    ));
                }
            }
        }
        cache_insert(&VALID_SCHEMA_TABLES, schema_key);
    }
    check_asset_content(
        root,
        "ir_assets",
        DigestDomain::SemanticIr,
        "agentflow.semantic.ir@1",
        "SemanticIrDocument",
        compiled,
    )?;
    check_asset_content(
        root,
        "contract_assets",
        DigestDomain::ComponentContract,
        "agentflow.component.contract@1",
        "ComponentContract",
        compiled,
    )?;
    for entry in table(root, "compiler_semantics_manifests")? {
        check_compiler_content(field(entry, "content", "$.compiler")?, "$.compiler")?;
    }
    check_relation_checksums(root)?;
    Ok(ContentValidatedPortableBundle { referenced })
}

fn check_asset_content(
    root: &Value,
    table_name: &str,
    domain: DigestDomain,
    family: &str,
    type_name: &str,
    compiled: &af_idl::CompiledModuleSet,
) -> Result<(), PortableProofError> {
    let shape = compiled
        .shape_of(&format!("{family}/{type_name}"))
        .map_err(|e| d3("UnsupportedCurrentSchema", "$.assets", &e.to_string()))?;
    let assets = table(root, table_name)?;
    let cache_key = asset_table_cache_key(table_name, assets, false)?;
    if cache_contains(&VALID_CONTENT_TABLES, &cache_key) {
        return Ok(());
    }
    for (index, asset) in assets.iter().enumerate() {
        let path = format!("$.{table_name}[{index}]");
        let reference = field(asset, "reference", &path)?;
        let content = bytes_field(asset, "canonical_content", &path)?;
        let value = decode(&shape, content)
            .map_err(|e| d3("InvalidCanonicalAsset", &path, &e.to_string()))?;
        let canonical = encode(&shape, &value)
            .map_err(|e| d3("InvalidCanonicalAsset", &path, &e.to_string()))?;
        if canonical != content {
            return Err(d3(
                "ContentIdentityMismatch",
                &path,
                "asset bytes are not canonical ACE1",
            ));
        }
        let digest = DigestRef::compute(domain, content);
        let digest_ref = field(reference, "digest", &path)?;
        if text_field(digest_ref, "algorithm", &path)? != "sha256"
            || text_field(digest_ref, "domain", &path)? != domain.as_str()
            || text_field(digest_ref, "hex", &path)? != digest.hex()
        {
            return Err(d3(
                "ContentIdentityMismatch",
                &path,
                "existing HashV1 digest does not match canonical asset bytes",
            ));
        }
        let type_ref = field(reference, "type_schema", &path)?;
        let registered = compiled
            .type_by_identity(&format!("{family}/{type_name}"))
            .ok_or_else(|| d3("UnsupportedCurrentSchema", &path, "root schema is missing"))?;
        let type_digest = registered
            .digest()
            .map_err(|error| d3("UnsupportedCurrentSchema", &path, &error.to_string()))?;
        check_schema_pin(type_ref, family, type_name, &type_digest, &path)?;
    }
    cache_insert(&VALID_CONTENT_TABLES, cache_key);
    Ok(())
}

fn asset_table_cache_key(
    name: &str,
    rows: &[Value],
    schema_refs: bool,
) -> Result<[u8; 32], PortableProofError> {
    let mut digest = Sha256::new();
    digest.update(b"agentflow.portable.d3.asset-table.v1\0");
    digest.update(name.as_bytes());
    let mut entries = Vec::with_capacity(rows.len());
    for row in rows {
        let reference = field(row, "reference", "$.asset")?;
        let identity = cache_reference_identity(reference, schema_refs)?;
        let content = bytes_field(row, "canonical_content", "$.asset")?;
        let mut entry = Vec::with_capacity(identity.len() + content.len() + 16);
        entry.extend(
            u64::try_from(identity.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        entry.extend(identity);
        entry.extend(
            u64::try_from(content.len())
                .unwrap_or(u64::MAX)
                .to_be_bytes(),
        );
        entry.extend(content);
        entries.push(entry);
    }
    entries.sort();
    for entry in entries {
        digest.update(u64::try_from(entry.len()).unwrap_or(u64::MAX).to_be_bytes());
        digest.update(entry);
    }
    Ok(digest.finalize().into())
}

fn cache_reference_identity(
    value: &Value,
    schema_ref: bool,
) -> Result<Vec<u8>, PortableProofError> {
    let mut identity = Vec::new();
    let digest = if schema_ref {
        field(value, "schema_digest", "$.asset.reference")?
    } else {
        field(value, "digest", "$.asset.reference")?
    };
    let fields: &[&str] = if schema_ref {
        &["specification_id", "type_name"]
    } else {
        &[]
    };
    for name in fields {
        cache_key_part(
            &mut identity,
            text_field(value, name, "$.asset.reference")?.as_bytes(),
        );
    }
    for name in ["algorithm", "domain", "hex"] {
        cache_key_part(
            &mut identity,
            text_field(digest, name, "$.asset.reference.digest")?.as_bytes(),
        );
    }
    if !schema_ref {
        let schema = field(value, "type_schema", "$.asset.reference")?;
        for name in ["specification_id", "type_name"] {
            cache_key_part(
                &mut identity,
                text_field(schema, name, "$.asset.reference.type_schema")?.as_bytes(),
            );
        }
        let schema_digest = field(schema, "schema_digest", "$.asset.reference.type_schema")?;
        for name in ["algorithm", "domain", "hex"] {
            cache_key_part(
                &mut identity,
                text_field(
                    schema_digest,
                    name,
                    "$.asset.reference.type_schema.schema_digest",
                )?
                .as_bytes(),
            );
        }
    }
    Ok(identity)
}

fn cache_key_part(out: &mut Vec<u8>, bytes: &[u8]) {
    out.extend(u64::try_from(bytes.len()).unwrap_or(u64::MAX).to_be_bytes());
    out.extend(bytes);
}

fn cache_contains(cache: &OnceLock<Mutex<HashSet<[u8; 32]>>>, key: &[u8; 32]) -> bool {
    cache
        .get_or_init(|| Mutex::new(HashSet::new()))
        .lock()
        .map(|entries| entries.contains(key))
        .unwrap_or(false)
}

fn cache_insert(cache: &OnceLock<Mutex<HashSet<[u8; 32]>>>, key: [u8; 32]) {
    if let Ok(mut entries) = cache.get_or_init(|| Mutex::new(HashSet::new())).lock() {
        entries.insert(key);
    }
}

fn check_compiler_content(value: &Value, path: &str) -> Result<(), PortableProofError> {
    let registries = field(value, "registries", path)?;
    let (spec_checksum, spec_bytes) = raw_blob(registries, "specifications", path)?;
    let (rule_checksum, rule_bytes) = raw_blob(registries, "rules", path)?;
    if plain_sha256(spec_bytes) != spec_checksum || plain_sha256(rule_bytes) != rule_checksum {
        return Err(d3(
            "RegistryContentMismatch",
            path,
            "registry checksum mismatch",
        ));
    }
    let specs: SpecificationsFile = strict_json(spec_bytes, path)?;
    let rules: RulesFile = strict_json(rule_bytes, path)?;
    let mut registry_specs = BTreeMap::new();
    for entry in &specs.specifications {
        if registry_specs
            .insert(entry.specification_id.as_str(), entry)
            .is_some()
        {
            return Err(d3(
                "RegistryContentMismatch",
                path,
                "duplicate SpecificationId",
            ));
        }
    }
    let sources = list(value, "normative_sources", path)?;
    let mut seen = BTreeSet::new();
    for source in sources {
        let id = text_field(source, "specification_id", path)?;
        let logical_path = text_field(source, "logical_path", path)?;
        if !seen.insert(id) {
            return Err(d3(
                "SourceContentMismatch",
                path,
                "duplicate normative source",
            ));
        }
        let registered = registry_specs.get(id).ok_or_else(|| {
            d3(
                "SourceContentMismatch",
                path,
                "source ID is absent from registry",
            )
        })?;
        if registered.normative_source.logical_path != logical_path {
            return Err(d3(
                "SourceContentMismatch",
                path,
                "logical path differs from exact registry row",
            ));
        }
        let bytes = bytes_field(source, "content", path)?;
        let checksum = checksum_field(field(source, "content_sha256", path)?, path)?;
        if checksum != plain_sha256(bytes)
            || checksum != registered.normative_source.content_digest.hex
        {
            return Err(d3(
                "SourceContentMismatch",
                path,
                "source bytes disagree with plain SHA-256 or registry",
            ));
        }
    }
    if seen.len() != specs.specifications.len() {
        return Err(d3(
            "IncompleteContent",
            path,
            "not all registry sources are inline",
        ));
    }

    let tables = field(value, "closed_tables", path)?;
    for name in [
        "hash_domains",
        "abi_numbers",
        "rule_areas",
        "afidl_primitives",
    ] {
        let (checksum, bytes) = raw_blob(tables, name, path)?;
        if checksum != plain_sha256(bytes) {
            return Err(d3(
                "RegistryContentMismatch",
                path,
                "closed-table checksum mismatch",
            ));
        }
        let _: serde_json::Value = strict_json(bytes, path)?;
    }
    let _roles = list(value, "semantic_roles", path)?;
    let projection = field(value, "static_identity_projection", path)?;
    let applied = list(projection, "applied_rules", path)?;
    let attribution = field(value, "attribution", path)?;
    let (profile, profile_payload) = variant(attribution, path)?;
    if profile == "full_content_revalidation"
        && list(profile_payload, "listed_rules", path)? != applied
    {
        return Err(d3(
            "CompilerRuleProjectionMismatch",
            path,
            "Profile A list differs from applied_rules",
        ));
    }
    let by_id = rules
        .rules
        .iter()
        .map(|row| (row.rule_id.as_str(), row))
        .collect::<BTreeMap<_, _>>();
    for rule in applied {
        let id = text_field(rule, "rule_id", path)?;
        let row = by_id.get(id).ok_or_else(|| {
            d3(
                "CompilerRuleProjectionMismatch",
                path,
                "applied RuleId absent from full registry",
            )
        })?;
        if text_field(rule, "normative_source", path)? != row.normative_source
            || text_field(rule, "source_anchor", path)? != row.source_anchor
            || text_field(rule, "short_statement", path)? != row.short_statement
        {
            return Err(d3(
                "CompilerRuleProjectionMismatch",
                path,
                "applied rule identity differs from registry row",
            ));
        }
    }
    Ok(())
}

fn check_relation_checksums(root: &Value) -> Result<(), PortableProofError> {
    for context in table(root, "proof_contexts")? {
        let content = field(context, "content", "$.proof_contexts")?;
        let relations = field(content, "relation_snapshots", "$.proof_contexts.content")?;
        for name in ["effect_resource", "recovery_usage", "flow_execution"] {
            let Some(relation) = option_field(relations, name)? else {
                continue;
            };
            let header = field(relation, "header", "$.relation")?;
            let bytes = bytes_field(relation, "canonical_content", "$.relation")?;
            if checksum_field(
                field(header, "canonical_content_checksum", "$.relation.header")?,
                "$.relation.header",
            )? != plain_sha256(bytes)
            {
                return Err(d3(
                    "ContentIdentityMismatch",
                    "$.relation.header.canonical_content_checksum",
                    "relation content checksum mismatch",
                ));
            }
        }
    }
    Ok(())
}
fn option_field<'a>(value: &'a Value, name: &str) -> Result<Option<&'a Value>, PortableProofError> {
    let candidate = field(value, name, "$.relation_snapshots")?;
    match candidate {
        Value::Option(None) => Ok(None),
        Value::Option(Some(inner)) => Ok(Some(inner)),
        _ => Err(d3(
            "InvalidOption",
            "$.relation_snapshots",
            "relation slot must be explicitly tagged",
        )),
    }
}

pub(crate) fn parse_schema(
    value: &Value,
    path: &str,
) -> Result<CanonicalTypeSchema, PortableProofError> {
    let schema = text_field(value, "schema", path)?.to_owned();
    let name = text_field(value, "name", path)?.to_owned();
    let (kind, payload) = variant(field(value, "body", path)?, path)?;
    let body = match kind {
        "alias" => CanonicalBody::Alias(parse_expr(payload, path)?),
        "enum" => CanonicalBody::Enum(
            list(payload, "values", path)?
                .iter()
                .map(|row| {
                    Ok(af_idl::CanonicalEnumValue {
                        no: u32_field(row, "no", path)?,
                        name: text_field(row, "name", path)?.to_owned(),
                    })
                })
                .collect::<Result<_, PortableProofError>>()?,
        ),
        "record" => CanonicalBody::Record(
            list(payload, "fields", path)?
                .iter()
                .map(|row| {
                    Ok(af_idl::CanonicalField {
                        no: u32_field(row, "no", path)?,
                        name: text_field(row, "name", path)?.to_owned(),
                        type_expr: parse_expr(field(row, "type", path)?, path)?,
                    })
                })
                .collect::<Result<_, PortableProofError>>()?,
        ),
        "variant" => CanonicalBody::Variant(
            list(payload, "cases", path)?
                .iter()
                .map(|row| {
                    Ok(af_idl::CanonicalCase {
                        no: u32_field(row, "no", path)?,
                        name: text_field(row, "name", path)?.to_owned(),
                        type_expr: parse_expr(field(row, "type", path)?, path)?,
                    })
                })
                .collect::<Result<_, PortableProofError>>()?,
        ),
        _ => {
            return Err(d3(
                "InvalidTypeSchema",
                path,
                "unknown canonical TypeBody case",
            ));
        }
    };
    Ok(CanonicalTypeSchema { schema, name, body })
}

fn parse_expr(value: &Value, path: &str) -> Result<CanonicalTypeExpr, PortableProofError> {
    let (kind, payload) = variant(value, path)?;
    match kind {
        "primitive" => Primitive::from_keyword(text_content(payload, path)?)
            .map(CanonicalTypeExpr::Primitive)
            .ok_or_else(|| d3("InvalidTypeSchema", path, "unknown machine primitive")),
        "ref" => Ok(CanonicalTypeExpr::NamedRef(
            text_content(payload, path)?.to_owned(),
        )),
        "list" => Ok(CanonicalTypeExpr::List(Box::new(parse_expr(
            payload, path,
        )?))),
        "map_string" => Ok(CanonicalTypeExpr::MapString(Box::new(parse_expr(
            payload, path,
        )?))),
        "option" => Ok(CanonicalTypeExpr::Option(Box::new(parse_expr(
            payload, path,
        )?))),
        _ => Err(d3(
            "InvalidTypeSchema",
            path,
            "unknown canonical TypeExpr case",
        )),
    }
}
fn text_content<'a>(value: &'a Value, path: &str) -> Result<&'a str, PortableProofError> {
    match value {
        Value::String(s) => Ok(s),
        _ => Err(d3("InvalidTypeSchema", path, "expected TypeExpr text")),
    }
}

fn check_schema_body(schema: &CanonicalTypeSchema, path: &str) -> Result<(), PortableProofError> {
    if !valid_spec(schema.schema.as_str()) || !valid_type(schema.name.as_str()) {
        return Err(d3(
            "InvalidTypeSchema",
            path,
            "invalid SpecificationId or TypeName",
        ));
    }
    match &schema.body {
        CanonicalBody::Alias(_) => Ok(()),
        CanonicalBody::Enum(rows) => {
            if rows.is_empty() {
                return Err(d3("InvalidTypeSchema", path, "empty enum"));
            }
            let mut numbers = BTreeSet::new();
            let mut names = BTreeSet::new();
            for row in rows {
                if row.no == 0
                    || !numbers.insert(row.no)
                    || !names.insert(row.name.as_str())
                    || !valid_lower(row.name.as_str())
                {
                    return Err(d3("InvalidTypeSchema", path, "invalid enum member"));
                }
            }
            if rows.windows(2).any(|pair| pair[0].no >= pair[1].no) {
                return Err(d3(
                    "NoncanonicalTypeSchema",
                    path,
                    "enum numbers are unsorted",
                ));
            }
            Ok(())
        }
        CanonicalBody::Record(rows) => {
            let mut numbers = BTreeSet::new();
            let mut names = BTreeSet::new();
            for row in rows {
                if row.no == 0
                    || !numbers.insert(row.no)
                    || !names.insert(row.name.as_str())
                    || !valid_lower(row.name.as_str())
                {
                    return Err(d3("InvalidTypeSchema", path, "invalid field member"));
                }
                check_expr(&row.type_expr)?;
            }
            if rows.windows(2).any(|pair| pair[0].no >= pair[1].no) {
                return Err(d3(
                    "NoncanonicalTypeSchema",
                    path,
                    "field numbers are unsorted",
                ));
            }
            Ok(())
        }
        CanonicalBody::Variant(rows) => {
            if rows.is_empty() {
                return Err(d3("InvalidTypeSchema", path, "empty variant"));
            }
            let mut numbers = BTreeSet::new();
            let mut names = BTreeSet::new();
            for row in rows {
                if row.no == 0
                    || !numbers.insert(row.no)
                    || !names.insert(row.name.as_str())
                    || !valid_lower(row.name.as_str())
                {
                    return Err(d3("InvalidTypeSchema", path, "invalid variant member"));
                }
                check_expr(&row.type_expr)?;
            }
            if rows.windows(2).any(|pair| pair[0].no >= pair[1].no) {
                return Err(d3(
                    "NoncanonicalTypeSchema",
                    path,
                    "case numbers are unsorted",
                ));
            }
            Ok(())
        }
    }
}
fn check_expr(expr: &CanonicalTypeExpr) -> Result<(), PortableProofError> {
    match expr {
        CanonicalTypeExpr::Primitive(_) | CanonicalTypeExpr::NamedRef(_) => Ok(()),
        CanonicalTypeExpr::List(v)
        | CanonicalTypeExpr::MapString(v)
        | CanonicalTypeExpr::Option(v) => check_expr(v),
    }
}
fn check_schema_pin(
    reference: &Value,
    schema: &str,
    name: &str,
    digest: &DigestRef,
    path: &str,
) -> Result<(), PortableProofError> {
    if text_field(reference, "specification_id", path)? != schema
        || text_field(reference, "type_name", path)? != name
    {
        return Err(d3(
            "InvalidNominalReference",
            path,
            "SchemaRef names a different canonical type",
        ));
    }
    let pin = field(reference, "schema_digest", path)?;
    if text_field(pin, "algorithm", path)? != "sha256"
        || text_field(pin, "domain", path)? != "type-schema"
        || text_field(pin, "hex", path)? != digest.hex()
    {
        return Err(d3(
            "ContentIdentityMismatch",
            path,
            "SchemaRef pin differs from TypeSchemaDigest",
        ));
    }
    Ok(())
}
fn raw_blob<'a>(
    parent: &'a Value,
    name: &str,
    path: &str,
) -> Result<(String, &'a [u8]), PortableProofError> {
    let blob = field(parent, name, path)?;
    let bytes = bytes_field(blob, "content", path)?;
    let digest = field(blob, "content_sha256", path)?;
    Ok((checksum_field(digest, path)?, bytes))
}
fn checksum_field(value: &Value, path: &str) -> Result<String, PortableProofError> {
    if text_field(value, "algorithm", path)? != "sha256" {
        return Err(d3(
            "InvalidChecksum",
            path,
            "plain checksum algorithm must be sha256",
        ));
    }
    let hex = text_field(value, "hex", path)?;
    if !lowerhex(hex) {
        return Err(d3(
            "InvalidChecksum",
            path,
            "checksum must be lowercase SHA-256 hex",
        ));
    }
    Ok(hex.into())
}
fn plain_sha256(value: &[u8]) -> String {
    hex::encode(Sha256::digest(value))
}
fn strict_json<T: serde::de::DeserializeOwned>(
    bytes: &[u8],
    path: &str,
) -> Result<T, PortableProofError> {
    reject_dups(bytes, path)?;
    let mut d = serde_json::Deserializer::from_slice(bytes);
    let value =
        T::deserialize(&mut d).map_err(|e| d3("InvalidRegistryContent", path, &e.to_string()))?;
    d.end()
        .map_err(|e| d3("InvalidRegistryContent", path, &e.to_string()))?;
    Ok(value)
}
fn reject_dups(bytes: &[u8], path: &str) -> Result<(), PortableProofError> {
    let mut d = serde_json::Deserializer::from_slice(bytes);
    DuplicateSeed
        .deserialize(&mut d)
        .map_err(|e| d3("InvalidRegistryContent", path, &e.to_string()))?;
    d.end()
        .map_err(|e| d3("InvalidRegistryContent", path, &e.to_string()))
}
struct DuplicateSeed;
struct DuplicateValue;
struct DuplicateVisitor;
impl<'de> serde::de::DeserializeSeed<'de> for DuplicateSeed {
    type Value = DuplicateValue;
    fn deserialize<D: serde::de::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        d.deserialize_any(DuplicateVisitor)
    }
}
impl<'de> serde::de::Visitor<'de> for DuplicateVisitor {
    type Value = DuplicateValue;
    fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("JSON")
    }
    fn visit_bool<E: serde::de::Error>(self, _: bool) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_i64<E: serde::de::Error>(self, _: i64) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_u64<E: serde::de::Error>(self, _: u64) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_f64<E: serde::de::Error>(self, _: f64) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_str<E: serde::de::Error>(self, _: &str) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_string<E: serde::de::Error>(self, _: String) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_none<E: serde::de::Error>(self) -> Result<Self::Value, E> {
        Ok(DuplicateValue)
    }
    fn visit_some<D: serde::de::Deserializer<'de>>(self, d: D) -> Result<Self::Value, D::Error> {
        DuplicateSeed.deserialize(d)
    }
    fn visit_seq<A: serde::de::SeqAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
        while a.next_element_seed(DuplicateSeed)?.is_some() {}
        Ok(DuplicateValue)
    }
    fn visit_map<A: serde::de::MapAccess<'de>>(self, mut a: A) -> Result<Self::Value, A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(k) = a.next_key::<String>()? {
            if !keys.insert(k.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate JSON key {k:?}"
                )));
            }
            a.next_value_seed(DuplicateSeed)?;
        }
        Ok(DuplicateValue)
    }
}
fn table<'a>(value: &'a Value, name: &str) -> Result<&'a [Value], PortableProofError> {
    list(value, name, "$")
}
fn list<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a [Value], PortableProofError> {
    match field(value, name, path)? {
        Value::List(v) => Ok(v),
        _ => Err(d3("InvalidTypedShape", path, "expected list")),
    }
}
fn field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a Value, PortableProofError> {
    let Value::Record(fields) = value else {
        return Err(d3("InvalidTypedShape", path, "expected record"));
    };
    fields
        .iter()
        .find(|(k, _)| k == name)
        .map(|(_, v)| v)
        .ok_or_else(|| d3("MissingField", path, "required field missing"))
}
fn text_field<'a>(value: &'a Value, name: &str, path: &str) -> Result<&'a str, PortableProofError> {
    match field(value, name, path)? {
        Value::String(s) | Value::Enum(s) => Ok(s),
        _ => Err(d3("InvalidTypedShape", path, "expected text")),
    }
}
fn bytes_field<'a>(
    value: &'a Value,
    name: &str,
    path: &str,
) -> Result<&'a [u8], PortableProofError> {
    match field(value, name, path)? {
        Value::Bytes(b) => Ok(b),
        _ => Err(d3("InvalidTypedShape", path, "expected bytes")),
    }
}
fn u32_field(value: &Value, name: &str, path: &str) -> Result<u32, PortableProofError> {
    match field(value, name, path)? {
        Value::UInt32(n) => Ok(*n),
        _ => Err(d3("InvalidTypedShape", path, "expected uint32")),
    }
}
fn variant<'a>(value: &'a Value, path: &str) -> Result<(&'a str, &'a Value), PortableProofError> {
    match value {
        Value::Variant { case, value } => Ok((case, value)),
        _ => Err(d3("InvalidTypedShape", path, "expected closed variant")),
    }
}
fn lowerhex(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn valid_spec(value: &str) -> bool {
    let Some((namespace, major)) = value.split_once('@') else {
        return false;
    };
    !namespace.is_empty()
        && major.parse::<u32>().is_ok_and(|n| n > 0)
        && namespace.split('.').all(|s| {
            !s.is_empty()
                && s.bytes()
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        })
}
fn valid_type(value: &str) -> bool {
    let mut c = value.chars();
    c.next().is_some_and(|x| x.is_ascii_uppercase()) && c.all(|x| x.is_ascii_alphanumeric())
}
fn valid_lower(value: &str) -> bool {
    let mut c = value.chars();
    c.next().is_some_and(|x| x.is_ascii_lowercase())
        && c.all(|x| x.is_ascii_lowercase() || x.is_ascii_digit() || x == '_')
}
fn d3(family: &'static str, path: &str, detail: &str) -> PortableProofError {
    PortableProofError::new(ValidationStage::D3, family, path, detail)
}
