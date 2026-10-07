//! Anchor lookup, provisional-marker detection, RuleId syntax and the
//! identifier grammars shared with AFIDL1.

use crate::error::SpecError;
use std::collections::BTreeSet;

/// Markdown anchor existence: an explicit `<a id="anchor">` or a heading
/// whose slugified text equals the anchor.
pub fn markdown_anchor_exists(source: &str, anchor: &str) -> bool {
    if source.contains(&format!("<a id=\"{anchor}\">")) {
        return true;
    }
    source.lines().any(|line| {
        let trimmed = line.trim_start();
        let Some(heading) = trimmed.strip_prefix('#') else {
            return false;
        };
        let text = heading.trim_start_matches('#').trim();
        slugify(text) == anchor
    })
}

/// Heading slug: lowercase, whitespace runs to `-`, only `[a-z0-9-_]` kept.
fn slugify(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut last_dash = false;
    for ch in text.chars() {
        if ch.is_whitespace() {
            if !last_dash && !out.is_empty() {
                out.push('-');
                last_dash = true;
            }
        } else if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            for low in ch.to_lowercase() {
                out.push(low);
            }
            last_dash = false;
        }
    }
    while out.ends_with('-') {
        out.pop();
    }
    out
}

/// AFIDL anchor existence: the anchor must be a declared type name in the
/// compiled module.
pub fn afidl_anchor_exists(source: &str, anchor: &str) -> bool {
    af_idl::compile(source)
        .map(|compiled| compiled.by_name(anchor).is_some())
        .unwrap_or(false)
}

/// Detects a provisional status marker in a (would-be) frozen source.
///
/// Markers: a line whose trimmed form starts with `status:` and continues
/// with `provisional` (case-insensitive), or a line starting with the
/// uppercase token `PROVISIONAL` followed by `:`, whitespace or end of line.
/// Prose that merely mentions the word "provisional" is not a marker.
pub fn has_provisional_marker(source: &str) -> bool {
    source.lines().any(|line| {
        // Strip blockquote prefixes so `> Status: provisional` is caught.
        let trimmed = line.trim().trim_start_matches('>').trim();
        if let Some(rest) = trimmed.strip_prefix("PROVISIONAL") {
            return rest.is_empty() || rest.starts_with(':') || rest.starts_with([' ', '\t']);
        }
        let lower = trimmed.to_ascii_lowercase();
        if let Some(rest) = lower.strip_prefix("status:") {
            return rest.trim_start().starts_with("provisional");
        }
        false
    })
}

/// Parses `AF-<AREA>-<4DIGIT>` and checks the area against the closed table.
pub fn parse_rule_id(rule_id: &str, areas: &BTreeSet<String>) -> Result<(String, u32), SpecError> {
    let invalid = |detail: &str| SpecError::InvalidRuleId {
        rule_id: rule_id.to_string(),
        detail: detail.to_string(),
    };
    let rest = rule_id
        .strip_prefix("AF-")
        .ok_or_else(|| invalid("missing AF- prefix"))?;
    let (area, number) = rest
        .rsplit_once('-')
        .ok_or_else(|| invalid("missing number part"))?;
    if area.is_empty() {
        return Err(invalid("empty area segment"));
    }
    let area_ok = area
        .bytes()
        .all(|b| b.is_ascii_uppercase() || b.is_ascii_digit())
        && area.bytes().next().is_some_and(|b| b.is_ascii_uppercase());
    if !area_ok {
        return Err(invalid("area must match [A-Z][A-Z0-9]*"));
    }
    if number.len() != 4 || !number.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid("number must be exactly four digits"));
    }
    let parsed: u32 = number.parse().map_err(|_| invalid("unparsable number"))?;
    if parsed == 0 {
        return Err(invalid("number 0000 is reserved; numbering starts at 0001"));
    }
    if !areas.contains(area) {
        return Err(SpecError::UnknownRuleArea {
            area: area.to_string(),
        });
    }
    Ok((area.to_string(), parsed))
}

/// AFIDL1 `lower-name` grammar: `[a-z][a-z0-9_]*`.
pub fn is_lower_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

/// AFIDL1 `TYPE-NAME` grammar: `[A-Z][A-Za-z0-9]*`.
pub fn is_type_name(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(first) if first.is_ascii_uppercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn markdown_anchors_via_id_tag_and_slug() {
        let src = "# Title\n\n<a id=\"my-anchor\"></a>\n\n## My Section\n";
        assert!(markdown_anchor_exists(src, "my-anchor"));
        assert!(markdown_anchor_exists(src, "my-section"));
        assert!(!markdown_anchor_exists(src, "other"));
    }

    #[test]
    fn provisional_markers_are_precise() {
        assert!(has_provisional_marker(
            "> Status: provisional — do not rely\n"
        ));
        assert!(has_provisional_marker(
            "Status: provisional -- do not rely\n"
        ));
        assert!(has_provisional_marker("PROVISIONAL: subject to change\n"));
        assert!(has_provisional_marker("PROVISIONAL\n"));
        assert!(!has_provisional_marker(
            "> superseding the Phase 1A provisional declaration file\n"
        ));
        assert!(!has_provisional_marker("> Status: frozen as of Phase 2B\n"));
        assert!(!has_provisional_marker("status: frozen\n"));
        assert!(!has_provisional_marker(""));
    }

    #[test]
    fn rule_id_parsing() {
        let areas: BTreeSet<String> = ["ACE1", "AFIDL"].iter().map(ToString::to_string).collect();
        assert_eq!(parse_rule_id("AF-ACE1-0001", &areas).unwrap().1, 1);
        assert!(parse_rule_id("AF-ACE1-1", &areas).is_err());
        assert!(parse_rule_id("AF-ACE1-0000", &areas).is_err());
        assert!(parse_rule_id("AF-JOURNAL-0001", &areas).is_err());
        assert!(parse_rule_id("RULE-24.1.3", &areas).is_err());
        assert!(parse_rule_id("AF-ace1-0001", &areas).is_err());
    }

    #[test]
    fn identifier_grammars() {
        assert!(is_lower_name("schema_digest"));
        assert!(is_lower_name("a"));
        assert!(!is_lower_name("Type"));
        assert!(!is_lower_name("kebab-name"));
        assert!(!is_lower_name("_leading"));
        assert!(is_type_name("SpecificationKind"));
        assert!(!is_type_name("lower"));
    }
}
