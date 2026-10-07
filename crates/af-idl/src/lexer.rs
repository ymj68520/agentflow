//! The AFIDL1 hand-written lexer.
//!
//! Tokens are generic (identifier class, integer, punctuation); keyword
//! interpretation is contextual and happens in the parser. Comments
//! (`// ...` and `/* ... */`, non-nesting) and whitespace are skipped and
//! never reach the AST. AFIDL syntax outside comments is ASCII; identifiers
//! are ASCII by design (no case folding, no Unicode normalization — value
//! strings are a separate ACE1 concern).

use crate::error::AfidlError;
use crate::span::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    UpperIdent,
    LowerIdent,
    Int,
    At,
    Dot,
    Semicolon,
    Colon,
    Comma,
    Equals,
    LeftBrace,
    RightBrace,
    LessThan,
    GreaterThan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
    pub span: SourceSpan,
}

/// Lexes an AFIDL1 source into tokens.
pub fn lex(source: &str) -> Result<Vec<Token>, AfidlError> {
    let b = source.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0usize;
    let mut line = 1u32;
    let mut col = 1u32;

    while i < b.len() {
        let c = b[i];
        match c {
            b' ' | b'\t' | b'\r' => {
                i += 1;
                col += 1;
            }
            b'\n' => {
                i += 1;
                line += 1;
                col = 1;
            }
            b'/' if b.get(i + 1) == Some(&b'/') => {
                while i < b.len() && b[i] != b'\n' {
                    i += 1;
                }
                // column is re-synced by the newline branch on the next pass
            }
            b'/' if b.get(i + 1) == Some(&b'*') => {
                let span_start = SourceSpan::new(i, i + 2, line, col);
                i += 2;
                col += 2;
                loop {
                    match b.get(i) {
                        None => {
                            return Err(AfidlError::Lex {
                                detail: "unterminated block comment".into(),
                                span: span_start,
                            });
                        }
                        Some(b'*') if b.get(i + 1) == Some(&b'/') => {
                            i += 2;
                            col += 2;
                            break;
                        }
                        Some(b'\n') => {
                            i += 1;
                            line += 1;
                            col = 1;
                        }
                        Some(_) => {
                            i += 1;
                            col += 1;
                        }
                    }
                }
            }
            b'{' => punct(
                &mut tokens,
                TokenKind::LeftBrace,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'}' => punct(
                &mut tokens,
                TokenKind::RightBrace,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'<' => punct(
                &mut tokens,
                TokenKind::LessThan,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'>' => punct(
                &mut tokens,
                TokenKind::GreaterThan,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b';' => punct(
                &mut tokens,
                TokenKind::Semicolon,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b':' => punct(
                &mut tokens,
                TokenKind::Colon,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b',' => punct(
                &mut tokens,
                TokenKind::Comma,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'=' => punct(
                &mut tokens,
                TokenKind::Equals,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'@' => punct(
                &mut tokens,
                TokenKind::At,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'.' => punct(
                &mut tokens,
                TokenKind::Dot,
                b,
                &mut i,
                &mut line,
                &mut col,
                1,
            ),
            b'A'..=b'Z' => {
                let start = i;
                i += 1;
                while i < b.len() && b[i].is_ascii_alphanumeric() {
                    i += 1;
                }
                push_ident(&mut tokens, TokenKind::UpperIdent, b, start, i, line, col);
                col += u32::try_from(i - start).expect("column within u32");
            }
            b'a'..=b'z' => {
                let start = i;
                i += 1;
                // '-' continues the token so kebab namespace segments lex as
                // one identifier; name positions re-validate the strict
                // `[a-z][a-z0-9_]*` pattern and reject hyphens there.
                while i < b.len()
                    && (b[i].is_ascii_lowercase()
                        || b[i].is_ascii_digit()
                        || b[i] == b'_'
                        || b[i] == b'-')
                {
                    i += 1;
                }
                push_ident(&mut tokens, TokenKind::LowerIdent, b, start, i, line, col);
                col += u32::try_from(i - start).expect("column within u32");
            }
            b'0'..=b'9' => {
                let start = i;
                i += 1;
                while i < b.len() && b[i].is_ascii_digit() {
                    i += 1;
                }
                let text = std::str::from_utf8(&b[start..i]).expect("ASCII digits");
                if text.len() > 1 && text.starts_with('0') {
                    return Err(AfidlError::Lex {
                        detail: format!("integer literal {text:?} has leading zeros"),
                        span: SourceSpan::new(start, i, line, col),
                    });
                }
                if text.parse::<u32>().is_err() {
                    return Err(AfidlError::Lex {
                        detail: format!("integer literal {text:?} does not fit in u32"),
                        span: SourceSpan::new(start, i, line, col),
                    });
                }
                tokens.push(Token {
                    kind: TokenKind::Int,
                    text: text.to_string(),
                    span: SourceSpan::new(start, i, line, col),
                });
                col += u32::try_from(i - start).expect("column within u32");
            }
            _ if c >= 0x80 => {
                return Err(AfidlError::Lex {
                    detail: format!(
                        "non-ASCII byte 0x{c:02X} outside a comment; AFIDL syntax and identifiers are ASCII"
                    ),
                    span: SourceSpan::new(i, i + 1, line, col),
                });
            }
            _ => {
                return Err(AfidlError::Lex {
                    detail: format!("unexpected character {:?}", c as char),
                    span: SourceSpan::new(i, i + 1, line, col),
                });
            }
        }
    }
    Ok(tokens)
}

#[allow(
    clippy::too_many_arguments,
    reason = "single-threaded scan state over (tokens, bytes, cursor, line, col)"
)]
fn punct(
    tokens: &mut Vec<Token>,
    kind: TokenKind,
    b: &[u8],
    i: &mut usize,
    line: &mut u32,
    col: &mut u32,
    width: usize,
) {
    tokens.push(Token {
        kind,
        text: std::str::from_utf8(&b[*i..*i + width])
            .expect("ASCII punctuation")
            .to_string(),
        span: SourceSpan::new(*i, *i + width, *line, *col),
    });
    *i += width;
    *col += u32::try_from(width).expect("column within u32");
}

fn push_ident(
    tokens: &mut Vec<Token>,
    kind: TokenKind,
    b: &[u8],
    start: usize,
    end: usize,
    line: u32,
    col: u32,
) {
    tokens.push(Token {
        kind,
        text: std::str::from_utf8(&b[start..end])
            .expect("ASCII identifier")
            .to_string(),
        span: SourceSpan::new(start, end, line, col),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexes_a_small_module() {
        let tokens =
            lex("afidl 1;\n\nschema agentflow.example @1;\nrecord User {\n  1 id: string;\n}\n")
                .unwrap();
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LowerIdent, // afidl
                TokenKind::Int,        // 1
                TokenKind::Semicolon,
                TokenKind::LowerIdent, // schema
                TokenKind::LowerIdent, // agentflow
                TokenKind::Dot,
                TokenKind::LowerIdent, // example
                TokenKind::At,
                TokenKind::Int, // 1
                TokenKind::Semicolon,
                TokenKind::LowerIdent, // record
                TokenKind::UpperIdent, // User
                TokenKind::LeftBrace,
                TokenKind::Int,        // 1
                TokenKind::LowerIdent, // id
                TokenKind::Colon,
                TokenKind::LowerIdent, // string
                TokenKind::Semicolon,
                TokenKind::RightBrace,
            ]
        );
    }

    #[test]
    fn skips_comments() {
        let tokens = lex("// line\n/* block\nspanning */ record /* mid */ A {}").unwrap();
        let kinds: Vec<TokenKind> = tokens.iter().map(|t| t.kind).collect();
        assert_eq!(
            kinds,
            vec![
                TokenKind::LowerIdent,
                TokenKind::UpperIdent,
                TokenKind::LeftBrace,
                TokenKind::RightBrace
            ]
        );
    }

    #[test]
    fn tracks_line_column() {
        let tokens = lex("record A {\n  1 x: string;\n}").unwrap();
        // 'record' at line 1 col 1; '}' at line 3 col 1.
        assert_eq!(tokens[0].span.line, 1);
        assert_eq!(tokens[0].span.column, 1);
        let last = tokens.last().unwrap();
        assert_eq!(last.span.line, 3);
        assert_eq!(last.span.column, 1);
    }

    #[test]
    fn rejects_leading_zeros_and_overflow() {
        let e = lex("1 x: 007;").unwrap_err();
        assert_eq!(e.kind(), crate::error::kind::LEX);
        assert!(lex("1 x: 4294967296;").is_err());
        assert!(lex("1 x: 0;").is_ok());
        assert!(lex("1 x: 4294967295;").is_ok());
    }

    #[test]
    fn rejects_non_ascii_and_unknown_punctuation() {
        assert_eq!(
            lex("record Ünïcode {}").unwrap_err().kind(),
            crate::error::kind::LEX
        );
        assert_eq!(
            lex("record A { 1 x: string; } &").unwrap_err().kind(),
            crate::error::kind::LEX
        );
    }

    #[test]
    fn rejects_unterminated_block_comment() {
        let e = lex("/* never closed").unwrap_err();
        assert_eq!(e.kind(), crate::error::kind::LEX);
    }

    #[test]
    fn underscore_splits_type_names() {
        // Foo_bar: UpperIdent(Foo) then '_' is not a legal continuation.
        assert!(lex("Foo_bar").is_err());
    }
}
