//! Source-bound, non-authoritative pre-capture entry. No C/port/Fact execution.
use crate::{CompilerSemanticsIdentity, DocumentOccurrenceMetadataRequest};
use af_contract::StaticContractProofContext;
use af_core::Value as V;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputBindingFailure {
    domain: &'static str,
    reason: &'static str,
    path: Vec<String>,
    stage: &'static str,
}
impl InputBindingFailure {
    #[must_use]
    pub fn domain(&self) -> &str {
        self.domain
    }
    #[must_use]
    pub fn reason(&self) -> &str {
        self.reason
    }
    #[must_use]
    pub fn path(&self) -> &[String] {
        &self.path
    }
    #[must_use]
    pub fn stage(&self) -> &str {
        self.stage
    }
    #[must_use]
    pub fn metadata(&self) -> serde_json::Value {
        serde_json::json!({"domain":self.domain,"reason":self.reason,"stage":self.stage,"site":{"none":null},"expected_type":{"none":null},"observed_type":{"none":null},"first_input_path":self.path.iter().map(|p|serde_json::json!({"field":p})).collect::<Vec<_>>(),"second_input_path":{"none":null}})
    }
}
impl From<crate::input_graph::Failure> for InputBindingFailure {
    fn from(f: crate::input_graph::Failure) -> Self {
        Self {
            domain: f.domain,
            reason: f.reason,
            path: f.path,
            stage: "pre_capture_binding",
        }
    }
}
/// Cannot be constructed/deserialized by a caller. Holding the actual source
/// and context borrows prevents mutation across this native association.
/// The returned value is NOT a checked contract, Subject admission or capability.
/// ```compile_fail
/// let forged = af_computation::VerifiedInputGraph { };
/// ```
/// ```compile_fail
/// let forged = serde_json::from_str::<af_computation::VerifiedInputGraph>("{}");
/// ```
pub struct VerifiedInputGraph<'s, 'c, 'u> {
    graph: crate::input_graph::Graph,
    canonical_bytes: Vec<u8>,
    source: &'s DocumentOccurrenceMetadataRequest<'s>,
    context: &'c StaticContractProofContext<'u>,
}
/// A real assigned Node<RawValidationSubject> in this exact immutable G, not
/// a borrowed Raw value or a caller-selected rank. The wrapper has no constructor.
pub struct RawSubjectInputNode<'g, 's, 'c, 'u> {
    owner: &'g VerifiedInputGraph<'s, 'c, 'u>,
    rank: u64,
}
impl<'g, 's, 'c, 'u> RawSubjectInputNode<'g, 's, 'c, 'u> {
    #[must_use]
    pub fn dag_rank(&self) -> u64 {
        self.rank
    }
    #[must_use]
    pub fn carrier(&self) -> &'g VerifiedInputGraph<'s, 'c, 'u> {
        self.owner
    }
}
impl<'s, 'c, 'u> VerifiedInputGraph<'s, 'c, 'u> {
    #[must_use]
    pub fn value(&self) -> &V {
        &self.graph.value
    }
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
    #[must_use]
    pub fn node_count(&self) -> usize {
        self.graph.nodes
    }
    #[must_use]
    pub fn structural_sites(&self) -> &std::collections::BTreeSet<String> {
        &self.graph.sites
    }
    #[must_use]
    pub fn source(&self) -> &'s DocumentOccurrenceMetadataRequest<'s> {
        self.source
    }
    #[must_use]
    pub fn context(&self) -> &'c StaticContractProofContext<'u> {
        self.context
    }
    #[must_use]
    pub fn can_issue_root_export(&self) -> bool {
        false
    }
    #[must_use]
    pub fn subject_node(&self) -> RawSubjectInputNode<'_, 's, 'c, 'u> {
        let V::Record(fields) = &self.graph.value else {
            unreachable!("private registered carrier")
        };
        let V::List(roots) = &fields.iter().find(|(n, _)| n == "roots").unwrap().1 else {
            unreachable!("private registered roots")
        };
        let V::UInt64(rank) = roots[0] else {
            unreachable!("private registered root rank")
        };
        RawSubjectInputNode { owner: self, rank }
    }
    /// The exact registered output data claim. Native association remains in
    /// this private borrow-bound issuer, never in the assurance enum spelling.
    #[must_use]
    pub fn registered_output_claim(&self) -> V {
        let mt = rec(vec![
            ("owner", V::String(crate::STRUCTURAL_MODEL_ID.into())),
            ("name", V::String("RawValidationSubject".into())),
        ]);
        let edition = rec(vec![
            (
                "analysis",
                V::String(crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION.into()),
            ),
            ("model", V::String(crate::STRUCTURAL_MODEL_ID.into())),
            ("report", V::String(crate::REPORT_SHAPE_EDITION.into())),
            (
                "obligation",
                V::String(crate::OBLIGATION_IDENTITY_EDITION.into()),
            ),
        ]);
        let environment = rec(vec![
            ("edition", edition),
            ("profile", V::Enum("raw_contract_validation".into())),
            ("input_graph", self.graph.value.clone()),
            ("witness_graph", V::Option(None)),
            ("root_types", V::List(vec![mt])),
        ]);
        rec(vec![
            ("environment", environment),
            ("source_assurance", V::Enum("native_associated".into())),
        ])
    }
}
fn rec(fields: Vec<(&str, V)>) -> V {
    V::Record(fields.into_iter().map(|(n, v)| (n.to_owned(), v)).collect())
}
fn source_failure(f: crate::input_graph::Failure, field: &str) -> InputBindingFailure {
    let mut path = vec![field.to_owned()];
    path.extend(f.path);
    InputBindingFailure {
        domain: f.domain,
        reason: f.reason,
        path,
        stage: "source_admission",
    }
}
fn path(path: &[crate::RawDocumentMember]) -> V {
    V::List(
        path.iter()
            .map(|p| match p {
                crate::RawDocumentMember::Field(b) => V::Variant {
                    case: "field".into(),
                    value: Box::new(rec(vec![("name", V::Bytes(b.clone()))])),
                },
                crate::RawDocumentMember::Entry(i) => V::Variant {
                    case: "entry".into(),
                    value: Box::new(rec(vec![("ordinal", V::UInt64(*i))])),
                },
            })
            .collect(),
    )
}
/// Actual complete declared source + actual native C/K context association.
/// C validation is deliberately NOT invoked; C-Missing cannot invalidate an
/// otherwise admitted input G, and G success cannot manufacture C Valid.
pub fn bind_raw_input_graph<'s, 'c, 'u>(
    source: &'s DocumentOccurrenceMetadataRequest<'s>,
    analysis: &'c StaticContractProofContext<'u>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
) -> Result<VerifiedInputGraph<'s, 'c, 'u>, InputBindingFailure> {
    crate::admit_document_occurrence_metadata(source, analysis, canonical_validation, compiler)
        .map_err(|e| {
            use crate::DocumentOccurrenceAdmissionFailure as F;
            let (domain, reason, field) = match e {
                F::ResourceLimit => ("ResourceLimit", "resource_bound", "package"),
                F::MissingDocumentOccurrenceBinding | F::IncompleteFacts => (
                    "IncompleteFacts",
                    "missing_node",
                    if source.source_package.is_none() {
                        "package"
                    } else if source.source_owner.is_none() {
                        "owner"
                    } else if source.entry_ordinal.is_none() {
                        "occurrence"
                    } else {
                        "package"
                    },
                ),
                F::InvalidIdentity
                    if compiler.analysis_semantics_version
                        != crate::ROOT_EXPORT_ANALYSIS_SEMANTICS_VERSION =>
                {
                    ("UnsupportedModel", "cross_edition", "compiler")
                }
                F::InvalidIdentity => ("InvalidFacts", "source_association", "compiler"),
                F::UnsupportedOwnerVariant
                | F::UnsupportedSourceDomain
                | F::UnsupportedPhysicalAssociation
                | F::UnsupportedModel => ("UnsupportedModel", "invalid_shape", "package"),
                F::InvalidContextBinding => ("InvalidFacts", "source_association", "context"),
                _ => ("InvalidFacts", "source_association", "package"),
            };
            InputBindingFailure {
                domain,
                reason,
                path: vec![field.into()],
                stage: "source_admission",
            }
        })?;
    let _native_candidate =
        crate::input_source::native_source(source.source_package.unwrap(), source.candidate)
            .map_err(|f| source_failure(f, "candidate"))?;
    let edition = crate::current_registered_analysis_edition();
    let owner =
        crate::input_source::from_metadata("DocumentOwnerAnchor", source.source_owner.unwrap())
            .map_err(|f| source_failure(f, "owner"))?;
    let occurrence = rec(vec![
        (
            "source_package",
            crate::input_source::from_metadata("SourcePackage", source.source_package.unwrap())
                .map_err(|f| source_failure(f, "package"))?,
        ),
        ("source_owner", owner),
        (
            "occurrence_key",
            rec(vec![(
                "entry_ordinal",
                V::UInt64(source.entry_ordinal.unwrap()),
            )]),
        ),
        (
            "actual_document",
            crate::input_source::from_metadata("RawDocument", source.actual_document)
                .map_err(|f| source_failure(f, "package"))?,
        ),
    ]);
    let physical = match &source.physical {
        crate::OccurrencePhysicalMetadata::NotSupplied => V::Variant {
            case: "not_supplied".into(),
            value: Box::new(V::Unit),
        },
        crate::OccurrencePhysicalMetadata::Supplied { value_path, .. } => V::Variant {
            case: "supplied".into(),
            value: Box::new(rec(vec![
                ("document_binding", occurrence.clone()),
                ("value_path", path(value_path)),
            ])),
        },
    };
    let input = rec(vec![
        (
            "candidate",
            crate::input_source::from_metadata("RawCandidateContract", source.candidate)
                .map_err(|f| source_failure(f, "candidate"))?,
        ),
        (
            "origin",
            rec(vec![
                ("document_binding", occurrence),
                ("path", path(source.internal_path)),
            ]),
        ),
        ("physical", physical),
    ]);
    let value = rec(vec![
        (
            "owner_compiler",
            crate::input_source::compiler(&edition).map_err(|f| source_failure(f, "compiler"))?,
        ),
        ("input", input),
        (
            "context",
            crate::input_source::context(analysis).map_err(|f| source_failure(f, "context"))?,
        ),
    ]);
    let graph = crate::input_graph::construct_admitted_subject(&value).map_err(|(f, stage)| {
        InputBindingFailure {
            domain: f.domain,
            reason: f.reason,
            path: f.path,
            stage,
        }
    })?;
    let m = crate::structural_model_catalogue().map_err(|_| InputBindingFailure {
        domain: "IncompleteContent",
        reason: "missing_site",
        path: vec![],
        stage: "pre_capture_binding",
    })?;
    let shape = carrier_shape(&m["types"], &m["types"]["CanonicalValueDag"], 0)?;
    let canonical_bytes =
        af_core::encode(&shape, &graph.value).map_err(|_| InputBindingFailure {
            domain: "InvalidFacts",
            reason: "invalid_shape",
            path: vec![],
            stage: "pre_capture_binding",
        })?;
    if canonical_bytes.len() > 134_217_728 {
        return Err(InputBindingFailure {
            domain: "ResourceLimit",
            reason: "resource_bound",
            path: vec![],
            stage: "pre_capture_binding",
        });
    }
    Ok(VerifiedInputGraph {
        graph,
        canonical_bytes,
        source,
        context: analysis,
    })
}
fn carrier_shape(
    types: &serde_json::Value,
    d: &serde_json::Value,
    depth: u64,
) -> Result<af_core::Shape, InputBindingFailure> {
    use af_core::{RecordShape, Shape as S, VariantShape};
    let bad = || InputBindingFailure {
        domain: "UnsupportedModel",
        reason: "invalid_shape",
        path: vec![],
        stage: "pre_capture_binding",
    };
    if depth > 96 {
        return Err(bad());
    }
    if let Some(n) = d["ref"].as_str() {
        return carrier_shape(types, &types[n], depth + 1);
    }
    Ok(match d["kind"].as_str().ok_or_else(bad)? {
        "unit" => S::Unit,
        "null" => S::Null,
        "bool" => S::Bool,
        "i32" => S::Int32,
        "u8" | "u32" => S::UInt32,
        "i64" => S::Int64,
        "u64" => S::UInt64,
        "decimal" => S::Decimal,
        "finite_f64" => S::Float64,
        "nfc_text" => S::String,
        "bytes" => S::Bytes,
        "timestamp_i64" => S::Timestamp,
        "duration_i64" => S::Duration,
        "sequence" | "set" => S::List(Box::new(carrier_shape(types, &d["item"], depth + 1)?)),
        "record" => S::Record(RecordShape::new(
            "registered-input-g-record",
            d["fields"]
                .as_array()
                .ok_or_else(bad)?
                .iter()
                .map(|r| {
                    Ok((
                        r[0].as_str().ok_or_else(bad)?.to_owned(),
                        carrier_shape(types, &r[1], depth + 1)?,
                    ))
                })
                .collect::<Result<Vec<_>, InputBindingFailure>>()?,
        )),
        "enum" => S::Variant(VariantShape::new(
            "registered-input-g-variant",
            d["variants"]
                .as_array()
                .ok_or_else(bad)?
                .iter()
                .map(|r| {
                    Ok((
                        r[0].as_str().ok_or_else(bad)?.to_owned(),
                        carrier_shape(types, &r[1], depth + 1)?,
                    ))
                })
                .collect::<Result<Vec<_>, InputBindingFailure>>()?,
        )),
        _ => return Err(bad()),
    })
}
