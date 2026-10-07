//! The AFIDL1 recursive-descent parser.
//!
//! Deterministic, allocation-light, keyword-contextual: `afidl`, `schema`,
//! `alias`, `enum`, `record`, `variant` and the composite markers `list`,
//! `map`, `option` are recognized in their syntactic positions only, so they
//! remain usable as field/case/value names. Unknown declaration keywords are
//! rejected fail-closed; there is no unknown-node storage.

use crate::ast::{
    AliasDecl, Decl, EnumDecl, EnumValueDecl, FieldDecl, Module, RecordDecl, SchemaDecl, TypeExpr,
    TypeExprKind, VariantCaseDecl, VariantDecl,
};
use crate::error::AfidlError;
use crate::lexer::{Token, TokenKind, lex};
use crate::primitives::{AFIDL1_VERSION, Primitive};
use crate::span::SourceSpan;
use crate::specification::SpecificationId;

/// Parses an AFIDL1 source into a [`Module`].
pub fn parse(source: &str) -> Result<Module, AfidlError> {
    let tokens = lex(source)?;
    let mut parser = Parser {
        tokens: &tokens,
        pos: 0,
    };
    parser.module()
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl Parser<'_> {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn bump(&mut self) -> Token {
        let token = self.tokens[self.pos].clone();
        self.pos += 1;
        token
    }

    fn err_parse(&self, detail: impl Into<String>) -> AfidlError {
        let span = self
            .peek()
            .map_or(self.tokens.last().map(|t| t.span), |t| Some(t.span))
            .unwrap_or(SourceSpan::new(0, 0, 1, 1));
        AfidlError::Parse {
            detail: detail.into(),
            span,
        }
    }

    fn expect_punct(&mut self, kind: TokenKind, what: &str) -> Result<SourceSpan, AfidlError> {
        match self.peek() {
            Some(t) if t.kind == kind => Ok(self.bump().span),
            Some(t) => Err(AfidlError::Parse {
                detail: format!("expected {what}, found {:?}", t.text),
                span: t.span,
            }),
            None => Err(self.err_parse(format!("expected {what}, found end of file"))),
        }
    }

    fn expect_keyword(&mut self, word: &str) -> Result<SourceSpan, AfidlError> {
        match self.peek() {
            Some(t) if t.kind == TokenKind::LowerIdent && t.text == word => Ok(self.bump().span),
            Some(t) => Err(AfidlError::Parse {
                detail: format!("expected keyword {word:?}, found {:?}", t.text),
                span: t.span,
            }),
            None => Err(self.err_parse(format!("expected keyword {word:?}, found end of file"))),
        }
    }

    fn type_name(&mut self) -> Result<(String, SourceSpan), AfidlError> {
        match self.peek() {
            Some(t) if t.kind == TokenKind::UpperIdent => {
                let t = self.bump();
                Ok((t.text, t.span))
            }
            Some(t) => Err(AfidlError::InvalidIdentifier {
                detail: format!(
                    "type names are UpperCamelCase ASCII identifiers, found {:?}",
                    t.text
                ),
                span: t.span,
            }),
            None => Err(self.err_parse("expected a type name, found end of file")),
        }
    }

    fn lower_name(&mut self) -> Result<(String, SourceSpan), AfidlError> {
        match self.peek() {
            Some(t) if t.kind == TokenKind::LowerIdent => {
                let t = self.bump();
                // The lexer token may contain kebab hyphens (for schema id
                // namespaces); field/case/value names may not.
                let valid = t
                    .text
                    .bytes()
                    .skip(1)
                    .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
                if !valid {
                    return Err(AfidlError::InvalidIdentifier {
                        detail: format!(
                            "field/case/value names may not contain '-', found {:?}",
                            t.text
                        ),
                        span: t.span,
                    });
                }
                Ok((t.text, t.span))
            }
            Some(t) => Err(AfidlError::InvalidIdentifier {
                detail: format!(
                    "field/case/value names are lowercase ASCII identifiers, found {:?}",
                    t.text
                ),
                span: t.span,
            }),
            None => Err(self.err_parse("expected a name, found end of file")),
        }
    }

    fn uint(&mut self) -> Result<(u32, SourceSpan), AfidlError> {
        match self.peek() {
            Some(t) if t.kind == TokenKind::Int => {
                let t = self.bump();
                let value: u32 = t.text.parse().expect("lexer validated u32 range");
                Ok((value, t.span))
            }
            Some(t) => Err(AfidlError::Parse {
                detail: format!("expected a number, found {:?}", t.text),
                span: t.span,
            }),
            None => Err(self.err_parse("expected a number, found end of file")),
        }
    }

    fn module(&mut self) -> Result<Module, AfidlError> {
        // afidl 1;
        self.expect_keyword("afidl")?;
        let (version, version_span) = self.uint()?;
        if version != AFIDL1_VERSION {
            return Err(AfidlError::InvalidLanguageVersion {
                found: version,
                span: version_span,
            });
        }
        self.expect_punct(
            TokenKind::Semicolon,
            "';' after the afidl language declaration",
        )?;

        // schema <specification-id>;
        let schema_kw = self.expect_keyword("schema")?;
        let id = self.specification_id()?;
        let id_end = self.expect_punct(TokenKind::Semicolon, "';' after the schema declaration")?;
        let schema = SchemaDecl {
            id,
            span: SourceSpan::merge(schema_kw, id_end),
        };

        let mut declarations = Vec::new();
        loop {
            match self.peek() {
                None => break,
                Some(t) if t.kind == TokenKind::LowerIdent => match t.text.as_str() {
                    "alias" => declarations.push(Decl::Alias(self.alias_decl()?)),
                    "enum" => declarations.push(Decl::Enum(self.enum_decl()?)),
                    "record" => declarations.push(Decl::Record(self.record_decl()?)),
                    "variant" => declarations.push(Decl::Variant(self.variant_decl()?)),
                    // A module declares its schema identity exactly once.
                    "schema" => {
                        return Err(AfidlError::DuplicateSchemaDeclaration { span: t.span });
                    }
                    other => {
                        return Err(AfidlError::Parse {
                            detail: format!("unknown declaration keyword {other:?}"),
                            span: t.span,
                        });
                    }
                },
                Some(t) => {
                    return Err(AfidlError::Parse {
                        detail: format!("expected a declaration, found {:?}", t.text),
                        span: t.span,
                    });
                }
            }
        }
        Ok(Module {
            language_version: AFIDL1_VERSION,
            schema,
            declarations,
        })
    }

    /// Assembles `ns-segment ("." ns-segment)* "@" major` from tokens and
    /// validates it strictly. Any structural deviation is
    /// `InvalidSpecificationId`.
    fn specification_id(&mut self) -> Result<SpecificationId, AfidlError> {
        let mut text = String::new();
        let start_span;
        match self.peek() {
            Some(t) if t.kind == TokenKind::LowerIdent => {
                let t = self.bump();
                start_span = t.span;
                text.push_str(&t.text);
            }
            Some(t) => {
                return Err(AfidlError::InvalidSpecificationId {
                    detail: format!(
                        "schema ids start with lowercase kebab segments, found {:?}",
                        t.text
                    ),
                    span: t.span,
                });
            }
            None => return Err(self.err_parse("expected a schema id, found end of file")),
        }
        loop {
            match self.peek() {
                Some(t) if t.kind == TokenKind::Dot => {
                    let dot = self.bump();
                    text.push('.');
                    match self.peek() {
                        Some(t) if t.kind == TokenKind::LowerIdent => {
                            let t = self.bump();
                            text.push_str(&t.text);
                        }
                        Some(t) => {
                            return Err(AfidlError::InvalidSpecificationId {
                                detail: format!("empty namespace segment before {:?}", t.text),
                                span: t.span,
                            });
                        }
                        None => {
                            return Err(AfidlError::InvalidSpecificationId {
                                detail: "namespace ends after '.'".into(),
                                span: dot.span,
                            });
                        }
                    }
                }
                Some(t) if t.kind == TokenKind::At => break,
                Some(t) => {
                    return Err(AfidlError::InvalidSpecificationId {
                        detail: format!("expected '.' or '@' in schema id, found {:?}", t.text),
                        span: t.span,
                    });
                }
                None => return Err(self.err_parse("schema id ended at end of file")),
            }
        }
        // '@' major
        let at = self.bump();
        let major = match self.peek() {
            Some(t) if t.kind == TokenKind::Int => self.bump().text.parse::<u32>().ok(),
            _ => None,
        }
        .ok_or_else(|| AfidlError::InvalidSpecificationId {
            detail: "expected the schema major after '@'".into(),
            span: at.span,
        })?;
        let end_span = SourceSpan::merge(start_span, at.span);
        SpecificationId::from_parts(text, major).map_err(|e| AfidlError::InvalidSpecificationId {
            detail: e.to_string(),
            span: end_span,
        })
    }

    fn alias_decl(&mut self) -> Result<AliasDecl, AfidlError> {
        let kw = self.expect_keyword("alias")?;
        let (name, _) = self.type_name()?;
        self.expect_punct(TokenKind::Equals, "'=' in the alias declaration")?;
        let target = self.type_expr()?;
        let end = self.expect_punct(TokenKind::Semicolon, "';' after the alias declaration")?;
        Ok(AliasDecl {
            name,
            target,
            span: SourceSpan::merge(kw, end),
        })
    }

    fn enum_decl(&mut self) -> Result<EnumDecl, AfidlError> {
        let kw = self.expect_keyword("enum")?;
        let (name, _) = self.type_name()?;
        self.expect_punct(TokenKind::LeftBrace, "'{' after the enum name")?;
        let mut values = Vec::new();
        loop {
            match self.peek() {
                Some(t) if t.kind == TokenKind::RightBrace => {
                    let end = self.bump().span;
                    return Ok(EnumDecl {
                        name,
                        values,
                        span: SourceSpan::merge(kw, end),
                    });
                }
                Some(t) if t.kind == TokenKind::Int => {
                    let (no, no_span) = self.uint()?;
                    let (value_name, _) = self.lower_name()?;
                    let semi =
                        self.expect_punct(TokenKind::Semicolon, "';' after the enum value")?;
                    values.push(EnumValueDecl {
                        no,
                        name: value_name,
                        span: SourceSpan::merge(no_span, semi),
                    });
                }
                Some(t) => {
                    return Err(AfidlError::Parse {
                        detail: format!("expected an enum value or '}}', found {:?}", t.text),
                        span: t.span,
                    });
                }
                None => return Err(self.err_parse("enum declaration ended at end of file")),
            }
        }
    }

    fn record_decl(&mut self) -> Result<RecordDecl, AfidlError> {
        let kw = self.expect_keyword("record")?;
        let (name, _) = self.type_name()?;
        self.expect_punct(TokenKind::LeftBrace, "'{' after the record name")?;
        let mut fields = Vec::new();
        loop {
            match self.peek() {
                Some(t) if t.kind == TokenKind::RightBrace => {
                    let end = self.bump().span;
                    return Ok(RecordDecl {
                        name,
                        fields,
                        span: SourceSpan::merge(kw, end),
                    });
                }
                Some(t) if t.kind == TokenKind::Int => {
                    fields.push(self.field_like()?);
                }
                Some(t) => {
                    return Err(AfidlError::Parse {
                        detail: format!("expected a field or '}}', found {:?}", t.text),
                        span: t.span,
                    });
                }
                None => return Err(self.err_parse("record declaration ended at end of file")),
            }
        }
    }

    fn variant_decl(&mut self) -> Result<VariantDecl, AfidlError> {
        let kw = self.expect_keyword("variant")?;
        let (name, _) = self.type_name()?;
        self.expect_punct(TokenKind::LeftBrace, "'{' after the variant name")?;
        let mut cases = Vec::new();
        loop {
            match self.peek() {
                Some(t) if t.kind == TokenKind::RightBrace => {
                    let end = self.bump().span;
                    return Ok(VariantDecl {
                        name,
                        cases,
                        span: SourceSpan::merge(kw, end),
                    });
                }
                Some(t) if t.kind == TokenKind::Int => {
                    let (no, no_span, case_name, name_span, payload, payload_end) =
                        self.numbered_member()?;
                    let _ = name_span;
                    cases.push(VariantCaseDecl {
                        no,
                        name: case_name,
                        payload,
                        span: SourceSpan::merge(no_span, payload_end),
                    });
                }
                Some(t) => {
                    return Err(AfidlError::Parse {
                        detail: format!("expected a case or '}}', found {:?}", t.text),
                        span: t.span,
                    });
                }
                None => return Err(self.err_parse("variant declaration ended at end of file")),
            }
        }
    }

    /// Parses `<no> <name> ":" <type> ";"` — the shared shape of record fields
    /// and variant cases.
    fn field_like(&mut self) -> Result<FieldDecl, AfidlError> {
        let (no, no_span, field_name, _, type_expr, end) = self.numbered_member()?;
        Ok(FieldDecl {
            no,
            name: field_name,
            type_expr,
            span: SourceSpan::merge(no_span, end),
        })
    }

    #[allow(
        clippy::type_complexity,
        reason = "one tuple for one grammatical production (no, name, type, span)"
    )]
    fn numbered_member(
        &mut self,
    ) -> Result<(u32, SourceSpan, String, SourceSpan, TypeExpr, SourceSpan), AfidlError> {
        let (no, no_span) = self.uint()?;
        let (member_name, name_span) = self.lower_name()?;
        self.expect_punct(TokenKind::Colon, "':' after the member name")?;
        let type_expr = self.type_expr()?;
        let end = self.expect_punct(TokenKind::Semicolon, "';' after the member")?;
        Ok((no, no_span, member_name, name_span, type_expr, end))
    }

    fn type_expr(&mut self) -> Result<TypeExpr, AfidlError> {
        let token = self
            .peek()
            .ok_or_else(|| self.err_parse("expected a type, found end of file"))?
            .clone();
        match token.kind {
            TokenKind::UpperIdent => {
                self.bump();
                Ok(TypeExpr {
                    kind: TypeExprKind::NamedRef(token.text),
                    span: token.span,
                })
            }
            TokenKind::LowerIdent => match token.text.as_str() {
                "list" => {
                    self.bump();
                    self.expect_punct(TokenKind::LessThan, "'<' after 'list'")?;
                    let item = self.type_expr()?;
                    let end =
                        self.expect_punct(TokenKind::GreaterThan, "'>' closing 'list<...>'")?;
                    Ok(TypeExpr {
                        kind: TypeExprKind::List(Box::new(item)),
                        span: SourceSpan::merge(token.span, end),
                    })
                }
                "map" => {
                    self.bump();
                    self.expect_punct(TokenKind::LessThan, "'<' after 'map'")?;
                    let key = self.lower_name()?;
                    if key.0 != "string" {
                        return Err(AfidlError::Parse {
                            detail: format!(
                                "map keys are exactly 'string' in AFIDL1, found {:?}",
                                key.0
                            ),
                            span: key.1,
                        });
                    }
                    self.expect_punct(TokenKind::Comma, "',' between the map key and value types")?;
                    let value = self.type_expr()?;
                    let end =
                        self.expect_punct(TokenKind::GreaterThan, "'>' closing 'map<...>'")?;
                    Ok(TypeExpr {
                        kind: TypeExprKind::MapString(Box::new(value)),
                        span: SourceSpan::merge(token.span, end),
                    })
                }
                "option" => {
                    self.bump();
                    self.expect_punct(TokenKind::LessThan, "'<' after 'option'")?;
                    let inner = self.type_expr()?;
                    let end =
                        self.expect_punct(TokenKind::GreaterThan, "'>' closing 'option<...>'")?;
                    Ok(TypeExpr {
                        kind: TypeExprKind::Option(Box::new(inner)),
                        span: SourceSpan::merge(token.span, end),
                    })
                }
                other => {
                    self.bump();
                    match Primitive::from_keyword(other) {
                        Some(primitive) => Ok(TypeExpr {
                            kind: TypeExprKind::Primitive(primitive),
                            span: token.span,
                        }),
                        // Unknown identifiers become named references; they are
                        // resolved (fail-closed) during validation.
                        None => Ok(TypeExpr {
                            kind: TypeExprKind::NamedRef(other.to_string()),
                            span: token.span,
                        }),
                    }
                }
            },
            _ => Err(AfidlError::Parse {
                detail: format!("expected a type, found {:?}", token.text),
                span: token.span,
            }),
        }
    }
}
