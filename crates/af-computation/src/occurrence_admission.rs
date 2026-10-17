//! Unit-returning INLINE metadata admission. No Source identity/Scope/Seed issued.
use crate::{
    CompilerSemanticsIdentity, EditionAdmissionFailure, EditionRegistration, RawAdmissionFailure,
    RawDocumentMember, RegisteredModelFailure, check_registered_compiler_identity,
    current_registered_analysis_edition, validate_registered_model_shape,
};
use af_contract::StaticContractProofContext;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentOccurrenceAdmissionFailure {
    MissingDocumentOccurrenceBinding,
    InvalidOccurrenceAssociation,
    BindingConflict,
    OwnerBindingConflict,
    MalformedSourcePackage,
    UnsupportedOwnerVariant,
    UnsupportedSourceDomain,
    UnsupportedPhysicalAssociation,
    InvalidPhysicalInput,
    InvalidContextBinding,
    InvalidIdentity,
    IncompleteFacts,
    UnsupportedModel,
    ResourceLimit,
}
use DocumentOccurrenceAdmissionFailure as Failure;

pub enum OccurrencePhysicalMetadata<'a> {
    NotSupplied,
    Supplied {
        source_owner: &'a Value,
        entry_ordinal: u64,
        actual_document: &'a Value,
        value_path: &'a [RawDocumentMember],
    },
}

/// Full actual supplied input, never a caller's trusted/supported assertion.
pub struct DocumentOccurrenceMetadataRequest<'a> {
    pub source_package: Option<&'a Value>,
    pub source_owner: Option<&'a Value>,
    pub entry_ordinal: Option<u64>,
    pub actual_document: &'a Value,
    pub candidate: &'a Value,
    pub internal_path: &'a [RawDocumentMember],
    pub physical: OccurrencePhysicalMetadata<'a>,
}

fn shape(name: &str, value: &Value) -> Result<(), Failure> {
    validate_registered_model_shape(name, value).map_err(|e| match e {
        RegisteredModelFailure::ResourceLimit => Failure::ResourceLimit,
        RegisteredModelFailure::IncompleteFacts | RegisteredModelFailure::IncompleteContent => {
            Failure::MissingDocumentOccurrenceBinding
        }
        _ => Failure::UnsupportedModel,
    })
}

fn budget(value: &Value) -> Result<(), Failure> {
    let mut work = 0usize;
    let mut pending = vec![(value, 0usize)];
    while let Some((v, depth)) = pending.pop() {
        work += 1;
        if work > 262_144 || depth > 96 {
            return Err(Failure::ResourceLimit);
        }
        match v {
            Value::Array(a) => {
                if a.len().saturating_add(pending.len()).saturating_add(work) > 262_144 {
                    return Err(Failure::ResourceLimit);
                }
                pending.extend(a.iter().map(|x| (x, depth + 1)));
            }
            Value::Object(o) => {
                if o.len().saturating_add(pending.len()).saturating_add(work) > 262_144 {
                    return Err(Failure::ResourceLimit);
                }
                pending.extend(o.values().map(|x| (x, depth + 1)));
            }
            _ => {}
        }
    }
    Ok(())
}

fn bytes(v: &Value) -> Result<Vec<u8>, Failure> {
    v.as_array()
        .ok_or(Failure::IncompleteFacts)?
        .iter()
        .map(|x| {
            x.as_u64()
                .and_then(|n| u8::try_from(n).ok())
                .ok_or(Failure::UnsupportedModel)
        })
        .collect()
}
type AnchorKey = (Vec<u8>, Vec<u8>, Vec<u8>, Vec<u8>);
fn anchor_key(a: &Value) -> Result<AnchorKey, Failure> {
    Ok((
        bytes(&a["domain"]["namespace"])?,
        bytes(&a["domain"]["domain_key"])?,
        bytes(&a["owner_key"])?,
        bytes(&a["revision_key"])?,
    ))
}

fn entries(owner: &Value) -> Result<&[Value], Failure> {
    let container = owner["container"]
        .as_object()
        .filter(|c| c.len() == 1)
        .ok_or(Failure::UnsupportedOwnerVariant)?;
    let (tag, payload) = container
        .iter()
        .next()
        .ok_or(Failure::UnsupportedOwnerVariant)?;
    let field = match tag.as_str() {
        "declaration_bundle" => "declarations",
        "import_manifest" => "imports",
        "workflow_source_collection" => "sources",
        _ => return Err(Failure::UnsupportedOwnerVariant),
    };
    payload[field]
        .as_array()
        .map(Vec::as_slice)
        .ok_or(Failure::IncompleteFacts)
}

/// Checks every owner in this complete finite package. Unit is not authority.
pub fn admit_source_package_metadata(package: &Value) -> Result<(), Failure> {
    budget(package)?;
    let owners = package["owners"]
        .as_array()
        .ok_or(Failure::MissingDocumentOccurrenceBinding)?;
    if owners.len() > 4096 {
        return Err(Failure::ResourceLimit);
    }
    for owner in owners {
        entries(owner)?;
    }
    if package["domain"]
        .as_object()
        .is_some_and(|d| d.keys().any(|k| k != "namespace" && k != "domain_key"))
    {
        return Err(Failure::UnsupportedSourceDomain);
    }
    shape("SourcePackageMetadata", package)?;
    let mut anchors = BTreeSet::new();
    let mut count = 0usize;
    for owner in owners {
        if owner["anchor"]["domain"] != package["domain"] {
            return Err(Failure::InvalidOccurrenceAssociation);
        }
        if !anchors.insert(anchor_key(&owner["anchor"])?) {
            return Err(Failure::MalformedSourcePackage);
        }
        count = count
            .checked_add(entries(owner)?.len())
            .ok_or(Failure::ResourceLimit)?;
        if count > 262_144 {
            return Err(Failure::ResourceLimit);
        }
    }
    Ok(())
}

/// Only an explicit finite joint scope, never a global ownership ledger.
pub fn check_joint_source_packages_metadata(packages: &[&Value]) -> Result<(), Failure> {
    let mut owners = BTreeMap::<AnchorKey, &Value>::new();
    if packages.len() > 4096 {
        return Err(Failure::ResourceLimit);
    }
    let mut total = 0usize;
    for package in packages {
        admit_source_package_metadata(package)?;
        for owner in package["owners"]
            .as_array()
            .ok_or(Failure::IncompleteFacts)?
        {
            total += 1;
            if total > 4096 {
                return Err(Failure::ResourceLimit);
            }
            let key = anchor_key(&owner["anchor"])?;
            if owners.get(&key).is_some_and(|prior| *prior != owner) {
                return Err(Failure::OwnerBindingConflict);
            }
            owners.insert(key, owner);
        }
    }
    Ok(())
}

fn selected<'a>(package: &'a Value, anchor: &Value, ordinal: u64) -> Result<&'a Value, Failure> {
    let owner = package["owners"]
        .as_array()
        .ok_or(Failure::IncompleteFacts)?
        .iter()
        .find(|o| &o["anchor"] == anchor)
        .ok_or(Failure::InvalidOccurrenceAssociation)?;
    let index = usize::try_from(ordinal).map_err(|_| Failure::InvalidOccurrenceAssociation)?;
    entries(owner)?
        .get(index)
        .map(|e| &e["document"])
        .ok_or(Failure::InvalidOccurrenceAssociation)
}

fn native_candidate<'a>(
    document: &'a Value,
    path: &[RawDocumentMember],
) -> Result<&'a Value, Failure> {
    let mut value = &document["declared_native_document"]["declarations"];
    for member in path {
        value = match member {
            RawDocumentMember::Field(name) => value
                .get(std::str::from_utf8(name).map_err(|_| Failure::InvalidOccurrenceAssociation)?),
            RawDocumentMember::Entry(n) => usize::try_from(*n).ok().and_then(|i| value.get(i)),
        }
        .ok_or(Failure::InvalidOccurrenceAssociation)?;
    }
    Ok(value)
}

/// Checks actual source borrow and C/K associations. No predicate is evaluated.
pub fn admit_document_occurrence_metadata(
    request: &DocumentOccurrenceMetadataRequest<'_>,
    analysis: &StaticContractProofContext<'_>,
    canonical_validation: &StaticContractProofContext<'_>,
    compiler: &CompilerSemanticsIdentity,
) -> Result<(), Failure> {
    check_registered_compiler_identity(compiler).map_err(|e| match e {
        EditionAdmissionFailure::ResourceLimit => Failure::ResourceLimit,
        _ => Failure::InvalidIdentity,
    })?;
    if current_registered_analysis_edition().registration != EditionRegistration::Registered {
        return Err(Failure::InvalidIdentity);
    }
    if !std::ptr::eq(analysis, canonical_validation) {
        return Err(Failure::InvalidContextBinding);
    }
    let package = request
        .source_package
        .ok_or(Failure::MissingDocumentOccurrenceBinding)?;
    let anchor = request
        .source_owner
        .ok_or(Failure::MissingDocumentOccurrenceBinding)?;
    let ordinal = request
        .entry_ordinal
        .ok_or(Failure::MissingDocumentOccurrenceBinding)?;
    admit_source_package_metadata(package)?;
    shape("DocumentOwnerAnchor", anchor)?;
    budget(request.actual_document)?;
    budget(request.candidate)?;
    if request.internal_path.len() > 96 {
        return Err(Failure::ResourceLimit);
    }
    let document = selected(package, anchor, ordinal)?;
    if document != request.actual_document {
        return Err(Failure::BindingConflict);
    }
    if !std::ptr::eq(document, request.actual_document) {
        return Err(Failure::InvalidOccurrenceAssociation);
    }
    shape("RawCandidateContract", request.candidate)?;
    match (&request.physical, document.get("physical_json_utf8")) {
        (OccurrencePhysicalMetadata::NotSupplied, None) => {
            if native_candidate(document, request.internal_path)? != request.candidate {
                return Err(Failure::InvalidOccurrenceAssociation);
            }
        }
        (
            OccurrencePhysicalMetadata::Supplied {
                source_owner,
                entry_ordinal,
                actual_document,
                value_path,
            },
            Some(physical),
        ) => {
            if *source_owner != anchor
                || *entry_ordinal != ordinal
                || !std::ptr::eq(*actual_document, document)
                || *value_path != request.internal_path
            {
                return Err(Failure::InvalidOccurrenceAssociation);
            }
            crate::raw_admission::validate_physical_metadata(
                request.candidate,
                &bytes(&physical["bytes"])?,
                value_path,
            )
            .map_err(|e| match e {
                RawAdmissionFailure::ResourceLimit => Failure::ResourceLimit,
                RawAdmissionFailure::InvalidPhysicalInput => Failure::InvalidPhysicalInput,
                _ => Failure::InvalidOccurrenceAssociation,
            })?;
        }
        _ => return Err(Failure::UnsupportedPhysicalAssociation),
    }
    Ok(())
}
