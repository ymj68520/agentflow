//! Closed registry tables: the rule-area table plus implementation-drift
//! checks for the pre-existing closed tables.

use crate::error::SpecError;
use crate::{fs, model};
use std::collections::BTreeSet;
use std::path::Path;

/// The loaded rule-area closed table.
#[derive(Debug, Clone)]
pub struct RuleAreas {
    pub areas: BTreeSet<String>,
}

/// Loads and validates `spec/tables/rule-areas.json` (or a corpus stand-in):
/// duplicates, canonical order, area identifier format.
///
/// # Errors
/// Returns a [`SpecError`] for IO/JSON problems, duplicate areas, unsorted
/// arrays or areas violating the `[A-Z][A-Z0-9]*` grammar.
pub fn load_rule_areas(root: &Path, relative: &str) -> Result<RuleAreas, SpecError> {
    let value = fs::load_plain_json(root, relative)?;
    let file: model::RuleAreasFile = fs::from_value(value, relative)?;
    let mut areas = BTreeSet::new();
    let mut previous: Option<&str> = None;
    for entry in &file.areas {
        if !entry
            .area
            .bytes()
            .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
            || !entry
                .area
                .bytes()
                .next()
                .is_some_and(|b| b.is_ascii_uppercase())
            || entry.area.len() < 2
        {
            return Err(SpecError::InvalidRuleId {
                rule_id: format!("AF-{}-0000", entry.area),
                detail: "area segment must match [A-Z][A-Z0-9]* with at least two characters"
                    .into(),
            });
        }
        if !areas.insert(entry.area.clone()) {
            return Err(SpecError::TableDrift {
                table: relative.to_string(),
                detail: format!("duplicate rule area {}", entry.area),
            });
        }
        if previous.is_some_and(|prev| prev.as_bytes() >= entry.area.as_bytes()) {
            return Err(SpecError::TableDrift {
                table: relative.to_string(),
                detail: "areas array is not sorted lexicographically".into(),
            });
        }
        previous = Some(entry.area.as_str());
    }
    Ok(RuleAreas { areas })
}

/// Drift check: `spec/tables/hash-domains.json` vs the frozen `DigestDomain`
/// registry in `af-core` (same names, same order, no extras).
///
/// # Errors
/// Returns a [`SpecError::TableDrift`] when the table and the enum diverge.
pub fn check_hash_domains(root: &Path, relative: &str) -> Result<(), SpecError> {
    let value = fs::load_plain_json(root, relative)?;
    let Some(domains) = value.get("domains").and_then(|d| d.as_array()) else {
        return Err(SpecError::TableDrift {
            table: relative.to_string(),
            detail: "missing domains array".into(),
        });
    };
    let mut table_names = Vec::new();
    for entry in domains {
        let Some(name) = entry.get("domain").and_then(|d| d.as_str()) else {
            return Err(SpecError::TableDrift {
                table: relative.to_string(),
                detail: "domain entry without a domain string".into(),
            });
        };
        table_names.push(name.to_string());
    }
    let enum_names: Vec<&str> = af_core::DigestDomain::ALL
        .iter()
        .map(af_core::DigestDomain::as_str)
        .collect();
    if table_names != enum_names {
        return Err(SpecError::TableDrift {
            table: relative.to_string(),
            detail: format!(
                "hash-domains.json {table_names:?} != DigestDomain registry {enum_names:?}"
            ),
        });
    }
    Ok(())
}

/// Drift check: `spec/tables/afidl-primitives.json` vs the frozen `Primitive`
/// enum in `af-idl` (same keywords, same order, no extras).
///
/// # Errors
/// Returns a [`SpecError::TableDrift`] when the table and the enum diverge.
pub fn check_afidl_primitives(root: &Path, relative: &str) -> Result<(), SpecError> {
    let value = fs::load_plain_json(root, relative)?;
    let Some(primitives) = value.get("primitives").and_then(|p| p.as_array()) else {
        return Err(SpecError::TableDrift {
            table: relative.to_string(),
            detail: "missing primitives array".into(),
        });
    };
    let mut table_names = Vec::new();
    for entry in primitives {
        let Some(name) = entry.get("name").and_then(|n| n.as_str()) else {
            return Err(SpecError::TableDrift {
                table: relative.to_string(),
                detail: "primitive entry without a name".into(),
            });
        };
        table_names.push(name.to_string());
    }
    let enum_names: Vec<&str> = af_idl::Primitive::ALL.iter().map(|p| p.keyword()).collect();
    if table_names != enum_names {
        return Err(SpecError::TableDrift {
            table: relative.to_string(),
            detail: format!(
                "afidl-primitives.json {table_names:?} != Primitive registry {enum_names:?}"
            ),
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rule_area_format_is_enforced() {
        let dir = std::env::temp_dir().join("af-spec-area-test");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("areas.json");
        std::fs::write(
            &path,
            r#"{"areas": [{"area": "ACE1", "description": "encoding"}, {"area": "DIGEST", "description": "digest"}]}"#,
        )
        .unwrap();
        let areas = load_rule_areas(&dir, "areas.json").unwrap();
        assert!(areas.areas.contains("ACE1"));
        assert_eq!(areas.areas.len(), 2);

        std::fs::write(
            &path,
            r#"{"areas": [{"area": "ACE1", "description": "a"}, {"area": "ACE1", "description": "b"}]}"#,
        )
        .unwrap();
        let err = load_rule_areas(&dir, "areas.json").unwrap_err();
        assert_eq!(err.kind(), "table_drift");
        std::fs::remove_file(&path).ok();
    }
}
