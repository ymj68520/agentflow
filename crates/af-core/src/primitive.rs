//! Public canonical-text validators/parsers for machine integer primitives.
//!
//! These are the strict textual entry points shared by the ACE1 decoder and
//! available to future AFIDL tooling. All arithmetic is exact integer
//! arithmetic (`i128`); no value ever passes through IEEE-754.
//!
//! Canonical integer text rules:
//!
//! - optional single `-` sign; `+` is forbidden;
//! - ASCII digits only; empty text is invalid;
//! - no non-canonical leading zeros (`0` itself is fine);
//! - `-0` is rejected as non-canonical;
//! - the value must fit the declared primitive's exact range;
//! - unsigned primitives reject negative values.

use crate::error::AceError;

/// Validates canonical integer text; returns `(negative, digits)` or `None`.
pub(crate) fn validate_canonical_int_text(s: &str) -> Option<(bool, &str)> {
    let (negative, digits) = match s.strip_prefix('-') {
        Some(d) => (true, d),
        None => (false, s),
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    if digits.len() > 1 && digits.starts_with('0') {
        return None;
    }
    if negative && digits == "0" {
        return None;
    }
    Some((negative, digits))
}

fn parse_in_range(s: &str, min: i128, max: i128, type_name: &str) -> Result<i128, AceError> {
    let Some((negative, digits)) = validate_canonical_int_text(s) else {
        return Err(AceError::InvalidInteger {
            input: s.to_string(),
        });
    };
    let magnitude: i128 = digits
        .parse::<i128>()
        .map_err(|_| AceError::IntegerOutOfRange {
            input: s.to_string(),
            type_name: type_name.to_string(),
        })?;
    let value = if negative { -magnitude } else { magnitude };
    if value < min || value > max {
        return Err(AceError::IntegerOutOfRange {
            input: s.to_string(),
            type_name: type_name.to_string(),
        });
    }
    Ok(value)
}

/// Parses canonical `int32` text.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the value was just range-checked against the exact i32 bounds"
)]
pub fn parse_int32_text(s: &str) -> Result<i32, AceError> {
    let v = parse_in_range(s, i128::from(i32::MIN), i128::from(i32::MAX), "int32")?;
    Ok(v as i32)
}

/// Parses canonical `uint32` text (negative values are out of range).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value was just range-checked against the exact u32 bounds"
)]
pub fn parse_uint32_text(s: &str) -> Result<u32, AceError> {
    let v = parse_in_range(s, 0, i128::from(u32::MAX), "uint32")?;
    Ok(v as u32)
}

/// Parses canonical `int64` text.
#[allow(
    clippy::cast_possible_truncation,
    reason = "the value was just range-checked against the exact i64 bounds"
)]
pub fn parse_int64_text(s: &str) -> Result<i64, AceError> {
    let v = parse_in_range(s, i128::from(i64::MIN), i128::from(i64::MAX), "int64")?;
    Ok(v as i64)
}

/// Parses canonical `uint64` text (negative values are out of range).
#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "the value was just range-checked against the exact u64 bounds"
)]
pub fn parse_uint64_text(s: &str) -> Result<u64, AceError> {
    let v = parse_in_range(s, 0, i128::from(u64::MAX), "uint64")?;
    Ok(v as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boundaries() {
        assert_eq!(parse_int32_text("-2147483648").unwrap(), i32::MIN);
        assert_eq!(parse_int32_text("2147483647").unwrap(), i32::MAX);
        assert_eq!(parse_uint32_text("4294967295").unwrap(), u32::MAX);
        assert_eq!(parse_int64_text("-9223372036854775808").unwrap(), i64::MIN);
        assert_eq!(parse_int64_text("9223372036854775807").unwrap(), i64::MAX);
        assert_eq!(parse_uint64_text("18446744073709551615").unwrap(), u64::MAX);
    }

    #[test]
    fn out_of_range() {
        assert!(matches!(
            parse_int32_text("2147483648"),
            Err(AceError::IntegerOutOfRange { .. })
        ));
        assert!(matches!(
            parse_int32_text("-2147483649"),
            Err(AceError::IntegerOutOfRange { .. })
        ));
        assert!(matches!(
            parse_uint32_text("-1"),
            Err(AceError::IntegerOutOfRange { .. })
        ));
        assert!(matches!(
            parse_uint64_text("-0"),
            Err(AceError::InvalidInteger { .. })
        ));
        assert!(matches!(
            parse_int64_text("-9223372036854775809"),
            Err(AceError::IntegerOutOfRange { .. })
        ));
        assert!(matches!(
            parse_uint64_text("18446744073709551616"),
            Err(AceError::IntegerOutOfRange { .. })
        ));
    }

    #[test]
    fn non_canonical_text_rejected() {
        for bad in ["+1", "007", "-0", "", "-", "1_0", "12a", " 1", "1 ", "٣"] {
            assert!(
                matches!(parse_int64_text(bad), Err(AceError::InvalidInteger { .. })),
                "expected {bad:?} to be invalid"
            );
        }
    }

    #[test]
    fn int64_never_passes_through_float() {
        // 2^53 + 1 is the classic value a float round-trip corrupts.
        let v = parse_int64_text("9007199254740993").unwrap();
        assert_eq!(v, 9_007_199_254_740_993);
        // u64::MAX survives exactly.
        assert_eq!(parse_uint64_text("18446744073709551615").unwrap(), u64::MAX);
    }
}
