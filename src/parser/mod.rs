//! Parser for Lemon source code
//!
//! Transforms tokens into an Abstract Syntax Tree.

mod expr;
mod item;

use crate::ast::*;
use crate::lexer::{Lexer, Span, SpannedToken, Token};
use thiserror::Error;

#[derive(Error, Debug, Clone)]
pub enum ParseError {
    #[error("unexpected token: expected {expected}, found {found:?}")]
    UnexpectedToken {
        expected: String,
        found: Token,
        span: Span,
    },

    #[error("unexpected end of file")]
    UnexpectedEof { span: Span },

    #[error("invalid number literal")]
    InvalidNumber { span: Span },

    #[error("{message}")]
    Custom { message: String, span: Span },
}

impl ParseError {
    pub fn span(&self) -> Span {
        match self {
            ParseError::UnexpectedToken { span, .. } => *span,
            ParseError::UnexpectedEof { span } => *span,
            ParseError::InvalidNumber { span } => *span,
            ParseError::Custom { span, .. } => *span,
        }
    }
}

pub type ParseResult<T> = Result<T, ParseError>;

/// Parser for Lemon source code
pub struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
}

impl Parser {
    pub fn new(source: &str) -> Self {
        let tokens = Lexer::tokenize(source);
        Self { tokens, pos: 0 }
    }

    /// Parse a complete source file
    pub fn parse_file(&mut self) -> ParseResult<SourceFile> {
        let start = self.current_span();
        let mut items = Vec::new();

        while !self.is_at_end() {
            items.push(self.parse_item()?);
        }

        let end = if items.is_empty() {
            start
        } else {
            self.prev_span()
        };

        Ok(SourceFile {
            items,
            span: start.merge(end),
        })
    }

    // === Token Navigation ===

    fn current(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn current_token(&self) -> Option<&Token> {
        self.current().map(|t| &t.token)
    }

    fn current_span(&self) -> Span {
        self.current().map(|t| t.span).unwrap_or(Span::new(0, 0))
    }

    fn prev_span(&self) -> Span {
        if self.pos > 0 {
            self.tokens[self.pos - 1].span
        } else {
            Span::new(0, 0)
        }
    }

    fn is_at_end(&self) -> bool {
        self.pos >= self.tokens.len()
    }

    fn advance(&mut self) -> Option<&SpannedToken> {
        if !self.is_at_end() {
            self.pos += 1;
        }
        self.tokens.get(self.pos - 1)
    }

    fn peek(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.pos + offset).map(|t| &t.token)
    }

    fn check(&self, token: &Token) -> bool {
        self.current_token() == Some(token)
    }

    fn check_any(&self, tokens: &[Token]) -> bool {
        self.current_token()
            .map(|t| tokens.contains(t))
            .unwrap_or(false)
    }

    fn consume(&mut self, expected: &Token) -> ParseResult<Span> {
        if self.check(expected) {
            let span = self.current_span();
            self.advance();
            Ok(span)
        } else {
            Err(ParseError::UnexpectedToken {
                expected: format!("{:?}", expected),
                found: self.current_token().cloned().unwrap_or(Token::Error),
                span: self.current_span(),
            })
        }
    }

    fn consume_ident(&mut self) -> ParseResult<Ident> {
        match self.current() {
            Some(SpannedToken {
                token: Token::Ident(name),
                span,
            }) => {
                let ident = Ident::new(name.clone(), *span);
                self.advance();
                Ok(ident)
            }
            // Allow 'new' as an identifier for constructor method names
            Some(SpannedToken {
                token: Token::New,
                span,
            }) => {
                let ident = Ident::new("new", *span);
                self.advance();
                Ok(ident)
            }
            // Allow 'Self' as an identifier for type references
            Some(SpannedToken {
                token: Token::SelfUpper,
                span,
            }) => {
                let ident = Ident::new("Self", *span);
                self.advance();
                Ok(ident)
            }
            Some(t) => Err(ParseError::UnexpectedToken {
                expected: "identifier".to_string(),
                found: t.token.clone(),
                span: t.span,
            }),
            None => Err(ParseError::UnexpectedEof {
                span: self.prev_span(),
            }),
        }
    }

    fn match_token(&mut self, token: &Token) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    // === Visibility Parsing ===

    fn parse_visibility(&mut self) -> Visibility {
        if self.match_token(&Token::Pub) {
            // TODO: Support pub(crate), pub(super), etc.
            Visibility::Public
        } else {
            Visibility::Private
        }
    }

    // === Attribute Parsing ===

    fn parse_attributes(&mut self) -> ParseResult<Vec<Attribute>> {
        let mut attrs = Vec::new();

        while self.check(&Token::Hash) {
            attrs.push(self.parse_attribute()?);
        }

        Ok(attrs)
    }

    fn parse_attribute(&mut self) -> ParseResult<Attribute> {
        let start = self.consume(&Token::Hash)?;
        self.consume(&Token::LBracket)?;

        let mut path = vec![self.consume_ident()?];
        while self.match_token(&Token::ColonColon) {
            path.push(self.consume_ident()?);
        }

        let args = if self.match_token(&Token::LParen) {
            let tokens = self.parse_attr_tokens_until(&Token::RParen)?;
            self.consume(&Token::RParen)?;
            Some(AttrArgs::Delimited(tokens))
        } else if self.match_token(&Token::Eq) {
            let token = self.parse_attr_token()?;
            Some(AttrArgs::Eq(token))
        } else {
            None
        };

        let end = self.consume(&Token::RBracket)?;

        Ok(Attribute {
            path,
            args,
            span: start.merge(end),
        })
    }

    fn parse_attr_tokens_until(&mut self, end: &Token) -> ParseResult<Vec<AttrToken>> {
        let mut tokens = Vec::new();

        while !self.check(end) && !self.is_at_end() {
            tokens.push(self.parse_attr_token()?);

            // Allow optional comma separation
            self.match_token(&Token::Comma);
        }

        Ok(tokens)
    }

    fn parse_attr_token(&mut self) -> ParseResult<AttrToken> {
        match self.current_token() {
            Some(Token::Ident(s)) => {
                let s = s.clone();
                self.advance();
                Ok(AttrToken::Ident(s))
            }
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                Ok(AttrToken::Literal(Literal::Int(n)))
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                Ok(AttrToken::Literal(Literal::Float(n)))
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                Ok(AttrToken::Literal(Literal::String(s)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(AttrToken::Literal(Literal::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(AttrToken::Literal(Literal::Bool(false)))
            }
            Some(Token::LParen) => {
                self.advance();
                let inner = self.parse_attr_tokens_until(&Token::RParen)?;
                self.consume(&Token::RParen)?;
                Ok(AttrToken::Group(inner))
            }
            Some(t) => {
                // Handle punctuation
                let c = match t {
                    Token::Eq => '=',
                    Token::Comma => ',',
                    Token::Colon => ':',
                    _ => {
                        return Err(ParseError::UnexpectedToken {
                            expected: "attribute token".to_string(),
                            found: t.clone(),
                            span: self.current_span(),
                        })
                    }
                };
                self.advance();
                Ok(AttrToken::Punct(c))
            }
            None => Err(ParseError::UnexpectedEof {
                span: self.prev_span(),
            }),
        }
    }

    // === Type Parsing ===

    fn parse_type(&mut self) -> ParseResult<Type> {
        // Handle reference types
        if self.match_token(&Token::Amp) {
            let mutable = self.match_token(&Token::Mut);
            let inner = self.parse_type()?;
            let span = self.prev_span();
            return Ok(Type::Reference {
                mutable,
                ty: Box::new(inner),
                span,
            });
        }

        // Handle tuple/unit types
        if self.check(&Token::LParen) {
            return self.parse_tuple_or_unit_type();
        }

        // Handle array/slice types
        if self.check(&Token::LBracket) {
            return self.parse_array_or_slice_type();
        }

        // Handle function types
        if self.check(&Token::Fn) {
            return self.parse_fn_type();
        }

        // Handle infer type
        if self.match_token(&Token::Ident("_".into())) {
            return Ok(Type::Infer(self.prev_span()));
        }

        // Path type (most common)
        self.parse_type_path().map(Type::Path)
    }

    fn parse_tuple_or_unit_type(&mut self) -> ParseResult<Type> {
        let start = self.consume(&Token::LParen)?;

        if self.match_token(&Token::RParen) {
            return Ok(Type::Unit(start.merge(self.prev_span())));
        }

        let first = self.parse_type()?;

        if self.match_token(&Token::RParen) {
            // Single type in parens - just return it
            return Ok(first);
        }

        // Tuple type
        let mut types = vec![first];
        while self.match_token(&Token::Comma) {
            if self.check(&Token::RParen) {
                break;
            }
            types.push(self.parse_type()?);
        }

        let end = self.consume(&Token::RParen)?;
        Ok(Type::Tuple(types, start.merge(end)))
    }

    fn parse_array_or_slice_type(&mut self) -> ParseResult<Type> {
        let start = self.consume(&Token::LBracket)?;
        let elem_type = self.parse_type()?;

        if self.match_token(&Token::Semi) {
            // Array type with size
            let size = self.parse_expr()?;
            let end = self.consume(&Token::RBracket)?;
            Ok(Type::Array(
                Box::new(elem_type),
                Box::new(size),
                start.merge(end),
            ))
        } else {
            // Slice type
            let end = self.consume(&Token::RBracket)?;
            Ok(Type::Slice(Box::new(elem_type), start.merge(end)))
        }
    }

    fn parse_fn_type(&mut self) -> ParseResult<Type> {
        let start = self.consume(&Token::Fn)?;
        self.consume(&Token::LParen)?;

        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            params.push(self.parse_type()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break;
                }
                params.push(self.parse_type()?);
            }
        }

        self.consume(&Token::RParen)?;

        let return_type = if self.match_token(&Token::Arrow) {
            Box::new(self.parse_type()?)
        } else {
            Box::new(Type::Unit(self.prev_span()))
        };

        let end = self.prev_span();
        Ok(Type::Function {
            params,
            return_type,
            span: start.merge(end),
        })
    }

    fn parse_type_path(&mut self) -> ParseResult<TypePath> {
        let start = self.current_span();
        let mut segments = Vec::new();

        segments.push(self.parse_path_segment()?);

        while self.match_token(&Token::ColonColon) {
            segments.push(self.parse_path_segment()?);
        }

        let end = self.prev_span();
        Ok(TypePath {
            segments,
            span: start.merge(end),
        })
    }

    fn parse_path_segment(&mut self) -> ParseResult<PathSegment> {
        let ident = self.consume_ident()?;
        let start = ident.span;

        let generics = if self.check(&Token::Lt) {
            Some(self.parse_generic_args()?)
        } else {
            None
        };

        let end = self.prev_span();
        Ok(PathSegment {
            ident,
            generics,
            span: start.merge(end),
        })
    }

    fn parse_generic_args(&mut self) -> ParseResult<Vec<GenericArg>> {
        self.consume(&Token::Lt)?;

        let mut args = Vec::new();

        if !self.check(&Token::Gt) {
            args.push(self.parse_generic_arg()?);

            while self.match_token(&Token::Comma) {
                if self.check(&Token::Gt) {
                    break;
                }
                args.push(self.parse_generic_arg()?);
            }
        }

        self.consume(&Token::Gt)?;
        Ok(args)
    }

    fn parse_generic_arg(&mut self) -> ParseResult<GenericArg> {
        // Try to parse as a type first
        // For now, we'll just parse types (const generics are more complex)
        let ty = self.parse_type()?;
        Ok(GenericArg::Type(ty))
    }

    // === Generics Parsing ===

    fn parse_generics(&mut self) -> ParseResult<Option<Generics>> {
        if !self.check(&Token::Lt) {
            return Ok(None);
        }

        let start = self.consume(&Token::Lt)?;
        let mut params = Vec::new();

        if !self.check(&Token::Gt) {
            params.push(self.parse_generic_param()?);

            while self.match_token(&Token::Comma) {
                if self.check(&Token::Gt) {
                    break;
                }
                params.push(self.parse_generic_param()?);
            }
        }

        let end = self.consume(&Token::Gt)?;
        Ok(Some(Generics {
            params,
            span: start.merge(end),
        }))
    }

    fn parse_generic_param(&mut self) -> ParseResult<GenericParam> {
        // TODO: Handle const generics
        let name = self.consume_ident()?;
        let start = name.span;

        let mut bounds = Vec::new();
        if self.match_token(&Token::Colon) {
            bounds.push(self.parse_type_path()?);
            while self.match_token(&Token::Plus) {
                bounds.push(self.parse_type_path()?);
            }
        }

        let default = if self.match_token(&Token::Eq) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let end = self.prev_span();
        Ok(GenericParam::Type(TypeParam {
            name,
            bounds,
            default,
            span: start.merge(end),
        }))
    }

    // === Where Clause ===

    fn parse_where_clause(&mut self) -> ParseResult<Option<WhereClause>> {
        if !self.match_token(&Token::Where) {
            return Ok(None);
        }

        let start = self.prev_span();
        let mut predicates = Vec::new();

        predicates.push(self.parse_where_predicate()?);

        while self.match_token(&Token::Comma) {
            // Allow trailing comma
            if self.check(&Token::LBrace) {
                break;
            }
            predicates.push(self.parse_where_predicate()?);
        }

        let end = self.prev_span();
        Ok(Some(WhereClause {
            predicates,
            span: start.merge(end),
        }))
    }

    fn parse_where_predicate(&mut self) -> ParseResult<WherePredicate> {
        let ty = self.parse_type()?;
        let start = ty.span();

        self.consume(&Token::Colon)?;

        let mut bounds = vec![self.parse_type_path()?];
        while self.match_token(&Token::Plus) {
            bounds.push(self.parse_type_path()?);
        }

        let end = self.prev_span();
        Ok(WherePredicate {
            ty,
            bounds,
            span: start.merge(end),
        })
    }

    // === Pattern Parsing ===

    fn parse_pattern(&mut self) -> ParseResult<Pattern> {
        self.parse_pattern_or()
    }

    fn parse_pattern_or(&mut self) -> ParseResult<Pattern> {
        let mut left = self.parse_pattern_primary()?;

        if self.check(&Token::Pipe) {
            let start = left.span();
            let mut patterns = vec![left];

            while self.match_token(&Token::Pipe) {
                patterns.push(self.parse_pattern_primary()?);
            }

            let end = self.prev_span();
            left = Pattern::Or(patterns, start.merge(end));
        }

        Ok(left)
    }

    fn parse_pattern_primary(&mut self) -> ParseResult<Pattern> {
        let span = self.current_span();

        // Wildcard
        if self.check(&Token::Ident(String::from("_"))) {
            self.advance();
            return Ok(Pattern::Wildcard(span));
        }

        // Reference pattern
        if self.match_token(&Token::Amp) {
            let mutable = self.match_token(&Token::Mut);
            let inner = self.parse_pattern_primary()?;
            let end = self.prev_span();
            return Ok(Pattern::Ref {
                mutable,
                pattern: Box::new(inner),
                span: span.merge(end),
            });
        }

        // Literal patterns
        match self.current_token() {
            Some(Token::Int(n)) => {
                let n = *n;
                self.advance();
                return Ok(Pattern::Literal(Literal::Int(n), span));
            }
            Some(Token::Float(n)) => {
                let n = *n;
                self.advance();
                return Ok(Pattern::Literal(Literal::Float(n), span));
            }
            Some(Token::String(s)) => {
                let s = s.clone();
                self.advance();
                return Ok(Pattern::Literal(Literal::String(s), span));
            }
            Some(Token::Char(c)) => {
                let c = *c;
                self.advance();
                return Ok(Pattern::Literal(Literal::Char(c), span));
            }
            Some(Token::True) => {
                self.advance();
                return Ok(Pattern::Literal(Literal::Bool(true), span));
            }
            Some(Token::False) => {
                self.advance();
                return Ok(Pattern::Literal(Literal::Bool(false), span));
            }
            _ => {}
        }

        // Tuple pattern
        if self.check(&Token::LParen) {
            return self.parse_tuple_pattern();
        }

        // Slice pattern
        if self.check(&Token::LBracket) {
            return self.parse_slice_pattern();
        }

        // Identifier or path pattern
        let mutable = self.match_token(&Token::Mut);

        if let Some(Token::Ident(_)) = self.current_token() {
            let ident = self.consume_ident()?;

            // Check if it's a path
            if self.check(&Token::ColonColon) || self.check(&Token::LBrace) || self.check(&Token::LParen) {
                // It's a path pattern
                let mut segments = vec![PathSegment {
                    ident: ident.clone(),
                    generics: None,
                    span: ident.span,
                }];

                while self.match_token(&Token::ColonColon) {
                    let seg_ident = self.consume_ident()?;
                    segments.push(PathSegment {
                        ident: seg_ident.clone(),
                        generics: None,
                        span: seg_ident.span,
                    });
                }

                let path = TypePath {
                    segments,
                    span: span.merge(self.prev_span()),
                };

                // Struct pattern
                if self.check(&Token::LBrace) {
                    return self.parse_struct_pattern(path);
                }

                // Tuple struct pattern
                if self.check(&Token::LParen) {
                    return self.parse_tuple_struct_pattern(path);
                }

                // Unit variant pattern
                return Ok(Pattern::Path(path, span.merge(self.prev_span())));
            }

            // Simple identifier binding
            return Ok(Pattern::Ident {
                mutable,
                name: ident,
                span,
            });
        }

        Err(ParseError::UnexpectedToken {
            expected: "pattern".to_string(),
            found: self.current_token().cloned().unwrap_or(Token::Error),
            span,
        })
    }

    fn parse_tuple_pattern(&mut self) -> ParseResult<Pattern> {
        let start = self.consume(&Token::LParen)?;
        let mut patterns = Vec::new();

        if !self.check(&Token::RParen) {
            patterns.push(self.parse_pattern()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break;
                }
                patterns.push(self.parse_pattern()?);
            }
        }

        let end = self.consume(&Token::RParen)?;
        Ok(Pattern::Tuple(patterns, start.merge(end)))
    }

    fn parse_slice_pattern(&mut self) -> ParseResult<Pattern> {
        let start = self.consume(&Token::LBracket)?;
        let mut patterns = Vec::new();

        if !self.check(&Token::RBracket) {
            patterns.push(self.parse_pattern()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RBracket) {
                    break;
                }
                patterns.push(self.parse_pattern()?);
            }
        }

        let end = self.consume(&Token::RBracket)?;
        Ok(Pattern::Slice {
            patterns,
            span: start.merge(end),
        })
    }

    fn parse_struct_pattern(&mut self, path: TypePath) -> ParseResult<Pattern> {
        let start = path.span;
        self.consume(&Token::LBrace)?;

        let mut fields = Vec::new();
        let mut rest = false;

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            // Check for rest pattern (..)
            if self.match_token(&Token::DotDot) {
                rest = true;
                self.match_token(&Token::Comma);
                break;
            }

            let field_name = self.consume_ident()?;
            let field_span = field_name.span;

            let pattern = if self.match_token(&Token::Colon) {
                Some(self.parse_pattern()?)
            } else {
                None
            };

            fields.push(FieldPattern {
                name: field_name,
                pattern,
                span: field_span,
            });

            if !self.match_token(&Token::Comma) {
                break;
            }
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(Pattern::Struct {
            path,
            fields,
            rest,
            span: start.merge(end),
        })
    }

    fn parse_tuple_struct_pattern(&mut self, path: TypePath) -> ParseResult<Pattern> {
        let start = path.span;
        self.consume(&Token::LParen)?;

        let mut fields = Vec::new();

        if !self.check(&Token::RParen) {
            fields.push(self.parse_pattern()?);
            while self.match_token(&Token::Comma) {
                if self.check(&Token::RParen) {
                    break;
                }
                fields.push(self.parse_pattern()?);
            }
        }

        let end = self.consume(&Token::RParen)?;
        Ok(Pattern::TupleStruct {
            path,
            fields,
            span: start.merge(end),
        })
    }

    // === Block Parsing ===

    fn parse_block(&mut self) -> ParseResult<Block> {
        let start = self.consume(&Token::LBrace)?;
        let mut stmts = Vec::new();

        while !self.check(&Token::RBrace) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }

        let end = self.consume(&Token::RBrace)?;
        Ok(Block {
            stmts,
            span: start.merge(end),
        })
    }

    fn parse_stmt(&mut self) -> ParseResult<Stmt> {
        // Let statement
        if self.check(&Token::Let) {
            return self.parse_let_stmt();
        }

        // Item statements
        if self.check(&Token::Fn)
            || self.check(&Token::Struct)
            || self.check(&Token::Enum)
            || self.check(&Token::Trait)
            || self.check(&Token::Impl)
            || self.check(&Token::Pub)
        {
            let item = self.parse_item()?;
            return Ok(Stmt::Item(Box::new(item)));
        }

        // Expression statement
        let expr = self.parse_expr()?;

        // Check for semicolon
        if self.match_token(&Token::Semi) {
            Ok(Stmt::Expr(expr))
        } else if matches!(
            &expr,
            Expr::Block(_)
                | Expr::If { .. }
                | Expr::Match { .. }
                | Expr::Loop { .. }
                | Expr::While { .. }
                | Expr::For { .. }
        ) {
            // Block-like expressions don't need semicolons
            Ok(Stmt::Expr(expr))
        } else if self.check(&Token::RBrace) {
            // Last expression in block (implicit return)
            Ok(Stmt::Expr(expr))
        } else {
            Err(ParseError::UnexpectedToken {
                expected: "';' or '}'".to_string(),
                found: self.current_token().cloned().unwrap_or(Token::Error),
                span: self.current_span(),
            })
        }
    }

    fn parse_let_stmt(&mut self) -> ParseResult<Stmt> {
        let start = self.consume(&Token::Let)?;
        let pattern = self.parse_pattern()?;

        let ty = if self.match_token(&Token::Colon) {
            Some(self.parse_type()?)
        } else {
            None
        };

        let value = if self.match_token(&Token::Eq) {
            Some(self.parse_expr()?)
        } else {
            None
        };

        self.consume(&Token::Semi)?;

        Ok(Stmt::Let {
            pattern,
            ty,
            value,
            span: start.merge(self.prev_span()),
        })
    }
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(span) => *span,
            Pattern::Ident { span, .. } => *span,
            Pattern::Literal(_, span) => *span,
            Pattern::Tuple(_, span) => *span,
            Pattern::Struct { span, .. } => *span,
            Pattern::TupleStruct { span, .. } => *span,
            Pattern::Path(_, span) => *span,
            Pattern::Or(_, span) => *span,
            Pattern::Ref { span, .. } => *span,
            Pattern::Range { span, .. } => *span,
            Pattern::Slice { span, .. } => *span,
        }
    }
}

/// Parse source code into AST
pub fn parse(source: &str) -> ParseResult<SourceFile> {
    let mut parser = Parser::new(source);
    parser.parse_file()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_file() {
        let result = parse("");
        assert!(result.is_ok());
        assert_eq!(result.unwrap().items.len(), 0);
    }

    #[test]
    fn test_parse_simple_function() {
        let source = r#"
            fn main() {
                let x = 42;
            }
        "#;
        let result = parse(source);
        assert!(result.is_ok());
        let file = result.unwrap();
        assert_eq!(file.items.len(), 1);
    }
}
