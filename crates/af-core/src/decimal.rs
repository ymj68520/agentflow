//! Strict canonical decimal values.
//!
//! ACE1 machine decimals are exact base-10 values encoded as canonical decimal
//! text inside a JSON string. They never pass through IEEE-754 or any binary
//! floating-point type.
//!
//! Accepted input grammar (ASCII only, no exponent notation):
//!
//! ```text
//! decimal := "-"? int-part ( "." frac-part )?
//! int-part  := "0" | [1-9] [0-9]*
//! frac-part := [0-9]+
//! ```
//!
//! Canonical form (what is stored and encoded):
//!
//! - trailing zeros in the fractional part are stripped (`1.50` → `1.5`);
//! - if the fractional part becomes empty the radix point is dropped
//!   (`1.0` → `1`);
//! - negative zero normalizes to `0` (`-0`, `-0.000` → `0`);
//! - no leading zeros (`01.5` is rejected, not normalized);
//! - no `+` sign, no exponent notation, no whitespace.
//!
//! Precision is unbounded: values are carried as canonical text; Phase 1 needs
//! identity and canonicalization, not arithmetic.

use crate::error::AceError;
use std::fmt;

/// A validated, canonically rendered decimal value.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Decimal(String);

impl Decimal {
    /// Parses and canonicalizes a decimal literal.
    pub fn parse(input: &str) -> Result<Self, AceError> {
        let canonical = canonicalize_decimal(input).ok_or_else(|| AceError::InvalidDecimal {
            input: input.to_string(),
        })?;
        Ok(Self(canonical))
    }

    /// The canonical textual form (guaranteed by construction).
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Canonicalizes a decimal literal; returns `None` when the input is not a legal
/// decimal in the accepted grammar above.
pub(crate) fn canonicalize_decimal(input: &str) -> Option<String> {
    let bytes = input.as_bytes();
    let mut i = 0usize;

    let negative = match bytes.first() {
        Some(b'-') => {
            i += 1;
            true
        }
        Some(_) => false,
        None => return None,
    };

    // Integer part: "0" or [1-9][0-9]*. Leading zeros are rejected outright.
    let int_start = i;
    if i >= bytes.len() || !bytes[i].is_ascii_digit() {
        return None;
    }
    if bytes[i] == b'0' {
        i += 1;
    } else {
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
    }
    let int_part = &input[int_start..i];

    // Optional fractional part.
    let mut frac_part: Option<&str> = None;
    if i < bytes.len() {
        if bytes[i] != b'.' {
            return None;
        }
        i += 1;
        let frac_start = i;
        while i < bytes.len() && bytes[i].is_ascii_digit() {
            i += 1;
        }
        if i == frac_start {
            return None; // "." or "1." — at least one fractional digit required
        }
        frac_part = Some(&input[frac_start..i]);
    }
    if i != bytes.len() {
        return None; // trailing junk
    }

    let frac = frac_part.unwrap_or("");
    let is_zero = is_all_zero(int_part) && is_all_zero(frac);

    if is_zero {
        return Some("0".to_string());
    }

    let trimmed_frac = frac.trim_end_matches('0');
    let sign = if negative { "-" } else { "" };
    if trimmed_frac.is_empty() {
        Some(format!("{sign}{int_part}"))
    } else {
        Some(format!("{sign}{int_part}.{trimmed_frac}"))
    }
}

fn is_all_zero(s: &str) -> bool {
    s.bytes().all(|b| b == b'0')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn canonical(input: &str) -> String {
        Decimal::parse(input).unwrap().as_str().to_string()
    }

    fn invalid(input: &str) {
        assert!(
            Decimal::parse(input).is_err(),
            "expected {input:?} to be rejected"
        );
    }

    #[test]
    fn canonicalizes_equivalent_forms() {
        assert_eq!(canonical("0"), "0");
        assert_eq!(canonical("0.0"), "0");
        assert_eq!(canonical("-0"), "0");
        assert_eq!(canonical("-0.000"), "0");
        assert_eq!(canonical("1"), "1");
        assert_eq!(canonical("1.0"), "1");
        assert_eq!(canonical("1.000"), "1");
        assert_eq!(canonical("1.50"), "1.5");
        assert_eq!(canonical("1.500"), "1.5");
        assert_eq!(canonical("-1.500"), "-1.5");
        assert_eq!(canonical("0.10"), "0.1");
        assert_eq!(canonical("0.001"), "0.001");
        assert_eq!(canonical("-0.500"), "-0.5");
        assert_eq!(canonical("100"), "100");
        assert_eq!(canonical("100.001"), "100.001");
        assert_eq!(canonical("-42"), "-42");
    }

    #[test]
    fn rejects_non_canonical_and_foreign_forms() {
        invalid("");
        invalid("-");
        invalid("+1");
        invalid("01");
        invalid("01.5");
        invalid("-01.5");
        invalid("00");
        invalid("1.");
        invalid(".5");
        invalid("-.5");
        invalid("1.2.3");
        invalid("--1");
        invalid("1e5");
        invalid("1E5");
        invalid("1.5e2");
        invalid("NaN");
        invalid("Infinity");
        invalid("abc");
        invalid("1 2");
        invalid(" 1");
        invalid("1 ");
        invalid("1_000");
        invalid("١٢٣"); // Arabic-Indic digits are not ASCII digits
        invalid("０"); // full-width zero
    }

    #[test]
    fn preserves_exact_precision_beyond_f64() {
        // 2^63 and beyond: exact decimal text, no float round-trip.
        assert_eq!(canonical("9223372036854775809"), "9223372036854775809");
        assert_eq!(
            canonical("0.123456789012345678901234567890"),
            "0.12345678901234567890123456789"
        );
    }
}
