//! Unpadded base64url (RFC 4648 §5, no `=` padding).
//!
//! ACE1 encodes machine `bytes` with this alphabet only: `A-Z a-z 0-9 - _`.
//! `+` and `/` never appear and padding is forbidden. Decoding is strict and
//! canonical: wrong alphabet, padding, impossible lengths, non-zero unused
//! trailing bits, and non-canonical re-encodings are all rejected.

use base64::Engine as _;
use base64::engine::DecodePaddingMode;
use base64::engine::GeneralPurpose;
use base64::engine::GeneralPurposeConfig;

use crate::error::AceError;

const fn engine(alphabet: &'static base64::alphabet::Alphabet) -> GeneralPurpose {
    GeneralPurpose::new(
        alphabet,
        GeneralPurposeConfig::new()
            .with_encode_padding(false)
            .with_decode_padding_mode(DecodePaddingMode::RequireNone)
            .with_decode_allow_trailing_bits(false),
    )
}

/// Strict unpadded base64url engine (`GeneralPurpose::new` is const).
const B64URL: GeneralPurpose = engine(&base64::alphabet::URL_SAFE);

/// Encodes bytes as unpadded base64url. Empty input encodes to the empty
/// string — there is exactly one representation for empty bytes.
pub fn encode(bytes: &[u8]) -> String {
    B64URL.encode(bytes)
}

/// Decodes strict, canonical, unpadded base64url.
pub fn decode(input: &str) -> Result<Vec<u8>, AceError> {
    if input.contains('+') || input.contains('/') {
        return Err(AceError::InvalidBytesEncoding {
            detail: "standard-base64 alphabet characters '+' or '/' are forbidden".into(),
        });
    }
    if input.contains('=') {
        return Err(AceError::InvalidBytesEncoding {
            detail: "padding characters are forbidden in ACE1 base64url".into(),
        });
    }
    let decoded = B64URL
        .decode(input)
        .map_err(|e| AceError::InvalidBytesEncoding {
            detail: format!("base64url decode failed: {e}"),
        })?;
    // Canonicality: re-encoding must reproduce the input byte-for-byte.
    // (Catches non-zero unused trailing bits and any other asymmetry.)
    let re_encoded = B64URL.encode(&decoded);
    if re_encoded != input {
        return Err(AceError::InvalidBytesEncoding {
            detail: "input is not the canonical base64url form of any byte string".into(),
        });
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encodes_rfc4648_vectors() {
        assert_eq!(encode(b""), "");
        assert_eq!(encode(b"f"), "Zg");
        assert_eq!(encode(b"fo"), "Zm8");
        assert_eq!(encode(b"foo"), "Zm9v");
        assert_eq!(encode(b"foob"), "Zm9vYg");
        assert_eq!(encode(b"fooba"), "Zm9vYmE");
        assert_eq!(encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn encodes_url_safe_alphabet() {
        let out = encode(&[0xfb, 0xff, 0xbf]);
        assert_eq!(
            out, "-_-_",
            "0xFBFFBF must encode with '-' and '_' characters"
        );
        assert!(!out.contains('+'));
        assert!(!out.contains('/'));
        assert!(!out.contains('='));
    }

    #[test]
    fn round_trips() {
        for len in 0..64usize {
            let bytes: Vec<u8> = (0..len)
                .map(|i| {
                    u8::try_from(i)
                        .expect("index < 64")
                        .wrapping_mul(37)
                        .wrapping_add(11)
                })
                .collect();
            let text = encode(&bytes);
            assert_eq!(decode(&text).unwrap(), bytes);
        }
    }

    #[test]
    fn rejects_padding_and_foreign_alphabet() {
        assert!(decode("Zg==").is_err());
        assert!(decode("Zm9v=").is_err());
        assert!(decode("Zm8+").is_err());
        assert!(decode("Zm8/").is_err());
    }

    #[test]
    fn rejects_bad_lengths_and_bits() {
        assert!(decode("Z").is_err(), "length 1 mod 4 is impossible");
        assert!(decode("Zh").is_err(), "non-zero unused trailing bits");
        assert!(decode("Zh ").is_err());
        assert!(decode("Zh\t").is_err());
        assert!(decode("!Zm8").is_err());
    }
}
