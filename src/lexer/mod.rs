//! Lemon lexer - tokenizes source code into tokens
//!
//! Uses the `logos` crate for fast lexical analysis.

mod token;

pub use token::Token;

use logos::Logos;

/// Source location for error reporting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(range: std::ops::Range<usize>) -> Self {
        Span::new(range.start, range.end)
    }
}

/// A token with its span in the source
#[derive(Debug, Clone, PartialEq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

/// Lexer for Lemon source code
pub struct Lexer<'src> {
    inner: logos::Lexer<'src, Token>,
}

impl<'src> Lexer<'src> {
    pub fn new(source: &'src str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }

    /// Tokenize all source code into a vector of spanned tokens
    pub fn tokenize(source: &'src str) -> Vec<SpannedToken> {
        let lexer = Self::new(source);
        lexer.collect()
    }
}

impl Iterator for Lexer<'_> {
    type Item = SpannedToken;

    fn next(&mut self) -> Option<Self::Item> {
        let token = self.inner.next()?;
        let span = self.inner.span().into();

        Some(SpannedToken {
            token: token.unwrap_or(Token::Error),
            span,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens: Vec<_> = Lexer::tokenize("let x = 42;")
            .into_iter()
            .map(|t| t.token)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("x".into()),
                Token::Eq,
                Token::Int(42),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn test_function() {
        let tokens: Vec<_> = Lexer::tokenize("fn add(a: int, b: int) -> int { a + b }")
            .into_iter()
            .map(|t| t.token)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Ident("add".into()),
                Token::LParen,
                Token::Ident("a".into()),
                Token::Colon,
                Token::Ident("int".into()),
                Token::Comma,
                Token::Ident("b".into()),
                Token::Colon,
                Token::Ident("int".into()),
                Token::RParen,
                Token::Arrow,
                Token::Ident("int".into()),
                Token::LBrace,
                Token::Ident("a".into()),
                Token::Plus,
                Token::Ident("b".into()),
                Token::RBrace,
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let tokens: Vec<_> = Lexer::tokenize(r#"let msg = "hello, world";"#)
            .into_iter()
            .map(|t| t.token)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Ident("msg".into()),
                Token::Eq,
                Token::String("hello, world".into()),
                Token::Semi,
            ]
        );
    }

    #[test]
    fn test_effects_and_capabilities() {
        let tokens: Vec<_> = Lexer::tokenize("fn read() effects[io] requires[FileRead]")
            .into_iter()
            .map(|t| t.token)
            .collect();

        assert_eq!(
            tokens,
            vec![
                Token::Fn,
                Token::Ident("read".into()),
                Token::LParen,
                Token::RParen,
                Token::Effects,
                Token::LBracket,
                Token::Ident("io".into()),
                Token::RBracket,
                Token::Requires,
                Token::LBracket,
                Token::Ident("FileRead".into()),
                Token::RBracket,
            ]
        );
    }
}
