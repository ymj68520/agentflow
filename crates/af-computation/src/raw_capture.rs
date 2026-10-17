//! Occurrence-bound, branch-created LOCAL raw seeds. No normalized FactKey,
//! Scope rank, CompleteView or Checked constructor is exposed by this module.
use crate::{
    DocumentOccurrenceAdmissionFailure as Failure, DocumentOccurrenceMetadataRequest,
    RegisteredAnalysisEdition, RuleContentIdentity, admit_document_occurrence_metadata,
    current_registered_analysis_edition, registered_model_catalogue,
};
use af_contract::port_observation::{
    PortCondition, PortDirection, PortEvent, PortOccurrence, PortPrefixStop, observe_raw_ports,
};
use af_contract::{ComponentContract, ProofVerdict, StaticContractProofContext};
use serde_json::{Map, Value, json};

// Explicit non-wire Raw metadata adapter. It does not issue provenance. The
// entire closed metadata shape is admitted before conversion; String octets
// are decoded losslessly, never NFC-repaired. Only this closed Rust input root
// is supported, not a general JSON/AFIDL/Scope decoder.
fn snake(name: &str) -> String {
    let mut out = String::new();
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() && i != 0 {
            out.push('_');
        }
        out.push(c.to_ascii_lowercase());
    }
    out
}
fn lowered(types: &Value, d: &Value, v: &Value, depth: usize) -> Result<Value, Failure> {
    if depth > 96 {
        return Err(Failure::ResourceLimit);
    }
    if let Some(name) = d["ref"].as_str() {
        if name == "Raw($String)" {
            let bytes = v
                .as_array()
                .ok_or(Failure::UnsupportedModel)?
                .iter()
                .map(|x| {
                    x.as_u64()
                        .and_then(|n| u8::try_from(n).ok())
                        .ok_or(Failure::UnsupportedModel)
                })
                .collect::<Result<Vec<_>, _>>()?;
            return String::from_utf8(bytes)
                .map(Value::String)
                .map_err(|_| Failure::InvalidPhysicalInput);
        }
        let body = types.get(name).ok_or(Failure::UnsupportedModel)?;
        // Rust's two port fields have a serde authoring rename, not a new
        // registered field. All other field names retain their nominal name.
        let mut result = lowered(types, body, v, depth + 1)?;
        if matches!(
            name,
            "Raw(af-contract::ComponentInputPort)" | "Raw(af-contract::ComponentOutputPort)"
        ) {
            let o = result.as_object_mut().ok_or(Failure::UnsupportedModel)?;
            let ty = o.remove("type_ref").ok_or(Failure::IncompleteFacts)?;
            o.insert("type".into(), ty);
        }
        if name == "Raw(af-semantic::PrimitiveConstant)"
            && let Some(unit) = result.as_str()
        {
            result = json!({unit:Value::Null});
        }
        return Ok(result);
    }
    match d["kind"].as_str().ok_or(Failure::UnsupportedModel)? {
        "record" => {
            let mut o = Map::new();
            for row in d["fields"].as_array().ok_or(Failure::UnsupportedModel)? {
                let name = row[0].as_str().ok_or(Failure::UnsupportedModel)?;
                o.insert(
                    name.into(),
                    lowered(
                        types,
                        &row[1],
                        v.get(name).ok_or(Failure::IncompleteFacts)?,
                        depth + 1,
                    )?,
                );
            }
            Ok(Value::Object(o))
        }
        "enum" => {
            let (tag, payload) = if let Some(tag) = v.as_str() {
                (tag, &Value::Null)
            } else {
                let o = v
                    .as_object()
                    .filter(|o| o.len() == 1)
                    .ok_or(Failure::UnsupportedModel)?;
                let (k, v) = o.iter().next().ok_or(Failure::UnsupportedModel)?;
                (k.as_str(), v)
            };
            let row = d["variants"]
                .as_array()
                .and_then(|a| a.iter().find(|r| r[0] == tag))
                .ok_or(Failure::UnsupportedModel)?;
            let tag = snake(tag);
            if row[1]["kind"] == "unit" {
                return Ok(Value::String(tag));
            }
            let value = lowered(types, &row[1], payload, depth + 1)?;
            let value = if value
                .as_object()
                .is_some_and(|o| o.len() == 1 && o.contains_key("0"))
            {
                value["0"].clone()
            } else {
                value
            };
            Ok(json!({tag:value}))
        }
        "sequence" | "set" => v
            .as_array()
            .ok_or(Failure::UnsupportedModel)?
            .iter()
            .map(|v| lowered(types, &d["item"], v, depth + 1))
            .collect(),
        "option" => {
            if v == "none" || v.get("none").is_some() {
                Ok(Value::Null)
            } else {
                lowered(
                    types,
                    &d["item"],
                    v.get("some").ok_or(Failure::UnsupportedModel)?,
                    depth + 1,
                )
            }
        }
        "unit" | "null" | "bool" | "u8" | "u32" | "u64" | "i32" | "i64" | "decimal"
        | "finite_f64" | "timestamp_i64" | "duration_i64" => Ok(v.clone()),
        _ => Err(Failure::UnsupportedModel),
    }
}
fn decode_candidate(v: &Value) -> Result<ComponentContract, Failure> {
    let m = registered_model_catalogue().map_err(|_| Failure::UnsupportedModel)?;
    let wire = lowered(&m["types"], &m["types"]["RawCandidateContract"], v, 0)?;
    serde_json::from_value(wire).map_err(|_| Failure::InvalidPhysicalInput)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RawRejectionWitness {
    Zero {
        occurrence: PortOccurrence,
        raw_member_value: u32,
        minimum: u32,
    },
    Duplicate {
        occurrence: PortOccurrence,
        previous_original_ordinal: usize,
        previous_port_no: u32,
    },
}
/// Payload fields are private. Only real canonical_ports events issue seeds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalRawSeed {
    event: PortEvent,
    status: ProofVerdict,
    witness: Option<RawRejectionWitness>,
}
impl LocalRawSeed {
    pub fn event(&self) -> PortEvent {
        self.event
    }
    pub fn status(&self) -> ProofVerdict {
        self.status
    }
    pub fn witness(&self) -> Option<RawRejectionWitness> {
        self.witness
    }
    pub fn predicate(&self) -> (&'static str, &'static str) {
        ("raw_contract_validation", "positive_unique_io_ports")
    }
    pub fn role(&self) -> &'static str {
        "supporting_fact"
    }
    pub fn kind(&self) -> &'static str {
        "prerequisite"
    }
    pub fn stage(&self) -> &'static str {
        "contract_validation"
    }
    pub fn substage(&self) -> &'static str {
        "canonical_validation/io_port_check"
    }
    pub fn evidence_class(&self) -> &'static str {
        "static_declaration"
    }
    pub fn evaluated(&self) -> bool {
        true
    }
}
/// Complete *local borrowed* basis. `subject` is not an encoded DAG rank.
pub struct RawBasisRef<'a, 'c, 'u> {
    pub subject: RawSubjectRef<'a, 'c, 'u>,
    pub occurrence: PortOccurrence,
    pub predecessor: Option<(usize, u32)>,
    pub condition: PortCondition,
    pub selected_error: Option<&'static str>,
    pub rule: &'a RuleContentIdentity,
}
pub struct RawTargetRef<'a, 'c, 'u> {
    pub subject: RawSubjectRef<'a, 'c, 'u>,
    pub schema: &'a af_idl::CanonicalTypeSchema,
    pub rule: &'a RuleContentIdentity,
    pub direction: PortDirection,
    pub constraint: &'static str,
}
pub struct RawOriginRef<'a, 'c, 'u> {
    pub subject: RawSubjectRef<'a, 'c, 'u>,
    pub direction: PortDirection,
    pub original_ordinal: usize,
    pub source_ordinal: usize,
    pub mapping_ordinal: Option<usize>,
    pub declared_path: Vec<crate::native_capture::NativeDeclaredStep>,
}
pub struct RawLocalCapture<'a, 'c, 'u> {
    request: &'a DocumentOccurrenceMetadataRequest<'a>,
    raw: ComponentContract,
    context: &'c StaticContractProofContext<'u>,
    compiler: RegisteredAnalysisEdition,
    schema: &'c af_idl::CanonicalTypeSchema,
    rule: RuleContentIdentity,
    seeds: Vec<LocalRawSeed>,
    stop: Option<PortPrefixStop>,
    canonical_validated: bool,
    events: Vec<PortEvent>,
    native_documents: Vec<(usize, usize, Vec<ComponentContract>)>,
}
pub struct RawSubjectRef<'a, 'c, 'u> {
    pub source_package: &'a Value,
    pub source_owner: &'a Value,
    pub entry_ordinal: u64,
    pub document: &'a Value,
    pub candidate_metadata: &'a Value,
    pub candidate: &'a ComponentContract,
    pub internal_path: &'a [crate::RawDocumentMember],
    pub physical: &'a crate::OccurrencePhysicalMetadata<'a>,
    pub compiler: &'a RegisteredAnalysisEdition,
    pub context: &'c StaticContractProofContext<'u>,
}
impl<'c, 'u> RawLocalCapture<'_, 'c, 'u> {
    pub fn seeds(&self) -> &[LocalRawSeed] {
        &self.seeds
    }
    pub fn stop(&self) -> Option<PortPrefixStop> {
        self.stop
    }
    pub fn canonical_validated(&self) -> bool {
        self.canonical_validated
    }
    pub fn complete_scope(&self) -> bool {
        false
    }
    pub fn can_issue_root_export(&self) -> bool {
        false
    }
    pub fn subject(&self) -> RawSubjectRef<'_, 'c, 'u> {
        RawSubjectRef {
            source_package: self.request.source_package.expect("admitted package"),
            source_owner: self.request.source_owner.expect("admitted owner"),
            entry_ordinal: self.request.entry_ordinal.expect("admitted slot"),
            document: self.request.actual_document,
            candidate_metadata: self.request.candidate,
            candidate: &self.raw,
            internal_path: self.request.internal_path,
            physical: &self.request.physical,
            compiler: &self.compiler,
            context: self.context,
        }
    }
    pub fn schema(&self) -> &af_idl::CanonicalTypeSchema {
        self.schema
    }
    pub fn native_documents(&self) -> &[(usize, usize, Vec<ComponentContract>)] {
        &self.native_documents
    }
    pub fn rule(&self) -> &RuleContentIdentity {
        &self.rule
    }
    pub fn basis(&self, seed: &LocalRawSeed) -> Option<RawBasisRef<'_, 'c, 'u>> {
        self.owns(seed).then(|| RawBasisRef {
            subject: self.subject(),
            occurrence: seed.event.occurrence,
            predecessor: seed.event.predecessor,
            condition: seed.event.condition,
            selected_error: if seed.status == ProofVerdict::Violation {
                Some("invalid_or_duplicate_port")
            } else {
                None
            },
            rule: &self.rule,
        })
    }
    pub fn target(&self, seed: &LocalRawSeed) -> Option<RawTargetRef<'_, 'c, 'u>> {
        self.owns(seed).then(|| RawTargetRef {
            subject: self.subject(),
            schema: self.schema,
            rule: &self.rule,
            direction: seed.event.occurrence.direction,
            constraint: "positive_unique_io_ports",
        })
    }
    pub fn origin(&self, seed: &LocalRawSeed) -> Option<RawOriginRef<'_, 'c, 'u>> {
        Some(RawOriginRef {
            subject: self.subject(),
            direction: seed.event.occurrence.direction,
            original_ordinal: seed.event.occurrence.original_ordinal,
            source_ordinal: seed.event.occurrence.original_ordinal,
            mapping_ordinal: None,
            declared_path: self.declared_path(seed)?,
        })
    }
    pub fn owns(&self, seed: &LocalRawSeed) -> bool {
        self.seeds.iter().any(|s| std::ptr::eq(s, seed))
    }
    pub fn context_roles(&self) -> [[&'static str; 2]; 1] {
        [["analysis", "canonical_validation"]]
    }
    pub fn same_subject(&self, other: &RawLocalCapture<'_, '_, '_>) -> bool {
        let a = self.subject();
        let b = other.subject();
        let c = a.context;
        let d = b.context;
        a.source_package == b.source_package
            && a.source_owner == b.source_owner
            && a.entry_ordinal == b.entry_ordinal
            && a.document == b.document
            && a.candidate == b.candidate
            && a.candidate_metadata == b.candidate_metadata
            && a.internal_path == b.internal_path
            && self.compiler == other.compiler
            && c.modules == d.modules
            && c.schema_universe == d.schema_universe
            && c.effects == d.effects
            && c.resources == d.resources
            && c.flow == d.flow
            && c.usage_meters == d.usage_meters
            && c.recovery == d.recovery
            && c.execution == d.execution
            && c.relations == d.relations
    }
    pub fn declared_path(
        &self,
        seed: &LocalRawSeed,
    ) -> Option<Vec<crate::native_capture::NativeDeclaredStep>> {
        use crate::native_capture::NativeDeclaredStep as Step;
        if !self.owns(seed) {
            return None;
        }
        let field = |owner, field| Step::Field { owner, field };
        let (name, port) = match seed.event.occurrence.direction {
            PortDirection::Input => ("inputs", "Raw(af-contract::ComponentInputPort)"),
            PortDirection::Output => ("outputs", "Raw(af-contract::ComponentOutputPort)"),
        };
        Some(vec![
            field("RawValidationSubject", "input"),
            field("RawParsedInput", "candidate"),
            field("Raw(af-contract::ComponentContract)", name),
            Step::Entry {
                index: seed.event.occurrence.original_ordinal,
            },
            field(port, "port_no"),
        ])
    }
    pub fn validate(&self) -> Result<(), crate::RegisteredModelFailure> {
        use crate::RegisteredModelFailure::{IncompleteFacts, InvalidFacts};
        if self.seeds.len() != self.events.len() {
            return Err(IncompleteFacts);
        }
        for (s, e) in self.seeds.iter().zip(&self.events) {
            if s.event != *e {
                return Err(InvalidFacts);
            }
            let o = e.occurrence;
            let numbers = match o.direction {
                PortDirection::Input => self
                    .raw
                    .inputs
                    .iter()
                    .map(|p| p.port_no)
                    .collect::<Vec<_>>(),
                PortDirection::Output => self.raw.outputs.iter().map(|p| p.port_no).collect(),
            };
            if numbers.get(o.original_ordinal) != Some(&o.port_no) {
                return Err(InvalidFacts);
            }
            let mut sorted: Vec<_> = numbers.iter().enumerate().collect();
            sorted.sort_by_key(|(_, n)| **n);
            if sorted.get(o.sorted_ordinal).map(|(i, n)| (*i, **n))
                != Some((o.original_ordinal, o.port_no))
            {
                return Err(InvalidFacts);
            }
            let predecessor = o
                .sorted_ordinal
                .checked_sub(1)
                .and_then(|i| sorted.get(i))
                .map(|(i, n)| (*i, **n));
            if predecessor != e.predecessor {
                return Err(InvalidFacts);
            }
            let expected = match e.condition {
                PortCondition::PositiveFresh
                    if o.port_no > 0 && predecessor.is_none_or(|(_, n)| n != o.port_no) =>
                {
                    (ProofVerdict::Proven, None)
                }
                PortCondition::Zero if o.port_no == 0 && predecessor.is_none() => (
                    ProofVerdict::Violation,
                    Some(RawRejectionWitness::Zero {
                        occurrence: o,
                        raw_member_value: 0,
                        minimum: 1,
                    }),
                ),
                PortCondition::Duplicate
                    if o.port_no > 0 && predecessor.is_some_and(|(_, n)| n == o.port_no) =>
                {
                    let (i, n) = predecessor.ok_or(IncompleteFacts)?;
                    (
                        ProofVerdict::Violation,
                        Some(RawRejectionWitness::Duplicate {
                            occurrence: o,
                            previous_original_ordinal: i,
                            previous_port_no: n,
                        }),
                    )
                }
                _ => return Err(InvalidFacts),
            };
            if (s.status, s.witness) != expected {
                return Err(InvalidFacts);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RawLocalObservation {
    CaptureFacts,
    ContractDiagnostic,
}
pub struct CollectedRawSeed<'s, 'a, 'c, 'u> {
    capture: &'s RawLocalCapture<'a, 'c, 'u>,
    seed_ordinal: usize,
    observations: Vec<RawLocalObservation>,
}
impl<'s, 'a, 'c, 'u> CollectedRawSeed<'s, 'a, 'c, 'u> {
    pub fn capture(&self) -> &'s RawLocalCapture<'a, 'c, 'u> {
        self.capture
    }
    pub fn seed(&self) -> &'s LocalRawSeed {
        &self.capture.seeds[self.seed_ordinal]
    }
    pub fn observations(&self) -> &[RawLocalObservation] {
        &self.observations
    }
}
#[derive(Default)]
pub struct RawLocalCollector<'s, 'a, 'c, 'u> {
    facts: Vec<CollectedRawSeed<'s, 'a, 'c, 'u>>,
}
impl<'s, 'a, 'c, 'u> RawLocalCollector<'s, 'a, 'c, 'u> {
    pub fn facts(&self) -> &[CollectedRawSeed<'s, 'a, 'c, 'u>] {
        &self.facts
    }
    pub fn insert(
        &mut self,
        capture: &'s RawLocalCapture<'a, 'c, 'u>,
        observation: RawLocalObservation,
    ) -> Result<(), crate::RegisteredModelFailure> {
        use crate::RegisteredModelFailure::{InvalidFacts, ResourceLimit};
        capture.validate()?;
        let selected = if observation == RawLocalObservation::ContractDiagnostic {
            let Some(PortPrefixStop::PortRejection(o)) = capture.stop else {
                return Err(InvalidFacts);
            };
            let report = af_contract::analyze_contract(&capture.raw, capture.context);
            if !report.dimensions.iter().any(|d| {
                d.dimension == "schema-validity"
                    && d.verdict == ProofVerdict::Violation
                    && d.rule_id == "AF-CONTRACT-0004"
            }) {
                return Err(InvalidFacts);
            }
            Some(o)
        } else {
            None
        };
        // Preflight every collision, then mutate atomically. Status is payload,
        // not part of this originating key; no last-writer/verdict-max merge.
        for s in capture
            .seeds
            .iter()
            .filter(|s| selected.is_none_or(|o| o == s.event.occurrence))
        {
            for prior in &self.facts {
                if prior.capture.same_subject(capture)
                    && prior.seed().event.occurrence == s.event.occurrence
                    && prior.seed() != s
                {
                    return Err(InvalidFacts);
                }
            }
        }
        let additions = capture
            .seeds
            .iter()
            .filter(|s| selected.is_none_or(|o| o == s.event.occurrence))
            .filter(|s| {
                !self.facts.iter().any(|p| {
                    p.capture.same_subject(capture)
                        && p.seed().event.occurrence == s.event.occurrence
                })
            })
            .count();
        if self.facts.len().saturating_add(additions) > 65_536 {
            return Err(ResourceLimit);
        }
        for (i, s) in capture
            .seeds
            .iter()
            .enumerate()
            .filter(|(_, s)| selected.is_none_or(|o| o == s.event.occurrence))
        {
            if let Some(prior) = self.facts.iter_mut().find(|p| {
                p.capture.same_subject(capture) && p.seed().event.occurrence == s.event.occurrence
            }) {
                if !prior.observations.contains(&observation) {
                    prior.observations.push(observation);
                    prior.observations.sort();
                }
            } else {
                self.facts.push(CollectedRawSeed {
                    capture,
                    seed_ordinal: i,
                    observations: vec![observation],
                });
            }
        }
        Ok(())
    }
}

pub fn capture_raw_ports<'a, 'c, 'u>(
    request: &'a DocumentOccurrenceMetadataRequest<'a>,
    analysis: &'c StaticContractProofContext<'u>,
    validation: &StaticContractProofContext<'_>,
    compiler: &crate::CompilerSemanticsIdentity,
) -> Result<RawLocalCapture<'a, 'c, 'u>, Failure> {
    admit_document_occurrence_metadata(request, analysis, validation, compiler)?;
    let mut native_documents = Vec::new();
    let package = request
        .source_package
        .ok_or(Failure::MissingDocumentOccurrenceBinding)?;
    for (owner, record) in package["owners"]
        .as_array()
        .ok_or(Failure::IncompleteFacts)?
        .iter()
        .enumerate()
    {
        let (tag, body) = record["container"]
            .as_object()
            .and_then(|o| o.iter().next())
            .ok_or(Failure::IncompleteFacts)?;
        let field = match tag.as_str() {
            "declaration_bundle" => "declarations",
            "import_manifest" => "imports",
            "workflow_source_collection" => "sources",
            _ => return Err(Failure::UnsupportedOwnerVariant),
        };
        for (slot, entry) in body[field]
            .as_array()
            .ok_or(Failure::IncompleteFacts)?
            .iter()
            .enumerate()
        {
            if let Some(d) = entry["document"].get("declared_native_document") {
                let declarations = d["declarations"]
                    .as_array()
                    .ok_or(Failure::IncompleteFacts)?
                    .iter()
                    .map(decode_candidate)
                    .collect::<Result<Vec<_>, _>>()?;
                native_documents.push((owner, slot, declarations));
            }
        }
    }
    let raw = decode_candidate(request.candidate)?;
    let schema = analysis
        .modules
        .type_by_identity("agentflow.component.contract@1/ComponentContract")
        .ok_or(Failure::IncompleteFacts)?;
    let registered = current_registered_analysis_edition();
    let rule = registered
        .compiler
        .rules
        .iter()
        .find(|r| r.rule_id == "AF-CONTRACT-0004")
        .cloned()
        .ok_or(Failure::IncompleteFacts)?;
    let observed = observe_raw_ports(&raw, analysis);
    let mut seeds = Vec::new();
    for event in observed.events() {
        let (status, witness) = match event.condition {
            PortCondition::PositiveFresh => (ProofVerdict::Proven, None),
            PortCondition::Zero => (
                ProofVerdict::Violation,
                Some(RawRejectionWitness::Zero {
                    occurrence: event.occurrence,
                    raw_member_value: 0,
                    minimum: 1,
                }),
            ),
            PortCondition::Duplicate => {
                let (previous_original_ordinal, previous_port_no) =
                    event.predecessor.ok_or(Failure::IncompleteFacts)?;
                (
                    ProofVerdict::Violation,
                    Some(RawRejectionWitness::Duplicate {
                        occurrence: event.occurrence,
                        previous_original_ordinal,
                        previous_port_no,
                    }),
                )
            }
        };
        seeds.push(LocalRawSeed {
            event: *event,
            status,
            witness,
        });
    }
    let stop = observed.stop();
    let canonical_validated = observed.canonical_validation_succeeded();
    let events = observed.events().to_vec();
    Ok(RawLocalCapture {
        request,
        raw,
        context: analysis,
        compiler: registered,
        schema,
        rule,
        seeds,
        stop,
        canonical_validated,
        events,
        native_documents,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn sealed_collector_rejects_conflicting_same_key_payload_without_mutation() {
        use crate::{OccurrencePhysicalMetadata, RawDocumentMember, RegisteredModelFailure};
        fn sources(dir: &std::path::Path, out: &mut Vec<(String, String)>) {
            for entry in std::fs::read_dir(dir).unwrap() {
                let path = entry.unwrap().path();
                if path.is_dir() {
                    sources(&path, out);
                } else if path.extension().is_some_and(|x| x == "afidl") {
                    out.push((
                        path.to_string_lossy().into_owned(),
                        std::fs::read_to_string(path).unwrap(),
                    ));
                }
            }
        }
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let mut inputs = Vec::new();
        sources(&root.join("spec/idl"), &mut inputs);
        let modules = af_idl::compile_module_set(&inputs).unwrap();
        let schemas = af_semantic::TypeSchemaUniverse::new();
        let effects = af_effect::EffectCatalogUniverse::new();
        let resources = af_resource::ResourceTypeUniverse::new();
        let flow = af_flow::FlowCatalogs::new();
        let usage = af_usage::UsageMeterUniverse::new();
        let recovery = af_recovery::RecoveryOperationUniverse::new();
        let execution = af_execution::ExecutionUniverses::default();
        let relations = af_contract::RelationSnapshots::default();
        let c = StaticContractProofContext {
            modules: &modules,
            schema_universe: &schemas,
            effects: &effects,
            resources: &resources,
            flow: &flow,
            usage_meters: &usage,
            recovery: &recovery,
            execution: &execution,
            relations: &relations,
        };
        let vectors: Value = serde_json::from_str(include_str!(
            "../../../spec/conformance/g01-document-occurrence-staging/admission-cases.json"
        ))
        .unwrap();
        let package = &vectors["cases"][0]["package"];
        let owner = &package["owners"][0]["anchor"];
        let doc =
            &package["owners"][0]["container"]["declaration_bundle"]["declarations"][0]["document"];
        let candidate = &doc["declared_native_document"]["declarations"][0];
        let path = [RawDocumentMember::Entry(0)];
        let request = DocumentOccurrenceMetadataRequest {
            source_package: Some(package),
            source_owner: Some(owner),
            entry_ordinal: Some(0),
            actual_document: doc,
            candidate,
            internal_path: &path,
            physical: OccurrencePhysicalMetadata::NotSupplied,
        };
        let compiler = crate::current_compiler_semantics_identity();
        let original = capture_raw_ports(&request, &c, &c, &compiler).unwrap();
        let mut changed = capture_raw_ports(&request, &c, &c, &compiler).unwrap();
        let saved = changed.seeds[0].clone();
        let mut collector = RawLocalCollector::default();
        collector
            .insert(&original, RawLocalObservation::CaptureFacts)
            .unwrap();
        let mut mutated = Vec::new();
        let mut s = saved.clone();
        s.status = ProofVerdict::Proven;
        mutated.push(s);
        let mut s = saved.clone();
        s.witness = None;
        mutated.push(s);
        let mut s = saved.clone();
        s.event.occurrence.direction = PortDirection::Output;
        mutated.push(s);
        let mut s = saved.clone();
        s.event.occurrence.original_ordinal = 9;
        mutated.push(s);
        let mut s = saved.clone();
        s.event.occurrence.sorted_ordinal = 9;
        mutated.push(s);
        let mut s = saved.clone();
        s.event.predecessor = Some((0, 0));
        mutated.push(s);
        let mut s = saved.clone();
        s.event.condition = PortCondition::PositiveFresh;
        mutated.push(s);
        let mut s = saved.clone();
        s.witness = Some(RawRejectionWitness::Zero {
            occurrence: s.event.occurrence,
            raw_member_value: 0,
            minimum: 0,
        });
        mutated.push(s);
        for s in mutated {
            changed.seeds[0] = s;
            let mut test_collector = RawLocalCollector::default();
            test_collector
                .insert(&original, RawLocalObservation::CaptureFacts)
                .unwrap();
            assert_eq!(
                test_collector.insert(&changed, RawLocalObservation::CaptureFacts),
                Err(RegisteredModelFailure::InvalidFacts)
            );
            assert_eq!(test_collector.facts().len(), 1);
            assert_eq!(
                test_collector.facts()[0].seed().status(),
                ProofVerdict::Violation
            );
        }
        changed.seeds[0] = saved;
        let mut test_collector = RawLocalCollector::default();
        test_collector
            .insert(&original, RawLocalObservation::CaptureFacts)
            .unwrap();
        test_collector
            .insert(&changed, RawLocalObservation::CaptureFacts)
            .unwrap();
        assert_eq!(test_collector.facts().len(), 1);
        test_collector
            .insert(&original, RawLocalObservation::ContractDiagnostic)
            .unwrap();
        assert_eq!(test_collector.facts()[0].observations().len(), 2);
    }
}
