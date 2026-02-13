//! Token definitions for the Lemon lexer

use logos::Logos;

/// All tokens in the Lemon language
#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*([^*]|\*[^/])*\*/")]
pub enum Token {
    // === Keywords ===
    #[token("let")]
    Let,

    #[token("mut")]
    Mut,

    #[token("fn")]
    Fn,

    #[token("async")]
    Async,

    #[token("await")]
    Await,

    #[token("return")]
    Return,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("match")]
    Match,

    #[token("for")]
    For,

    #[token("while")]
    While,

    #[token("loop")]
    Loop,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("in")]
    In,

    #[token("struct")]
    Struct,

    #[token("enum")]
    Enum,

    #[token("trait")]
    Trait,

    #[token("impl")]
    Impl,

    #[token("type")]
    Type,

    #[token("schema")]
    Schema,

    #[token("capability")]
    Capability,

    #[token("actor")]
    Actor,

    // === OOP Keywords ===
    #[token("class")]
    Class,

    #[token("interface")]
    Interface,

    #[token("extends")]
    Extends,

    #[token("implements")]
    Implements,

    #[token("delegate")]
    Delegate,

    #[token("new")]
    New,

    #[token("this")]
    This,

    #[token("super")]
    Super,

    #[token("static")]
    Static,

    #[token("abstract")]
    Abstract,

    #[token("override")]
    Override,

    #[token("final")]
    Final,

    #[token("protected")]
    Protected,

    #[token("private")]
    Private,

    #[token("pub")]
    Pub,

    #[token("use")]
    Use,

    #[token("mod")]
    Mod,

    #[token("self")]
    SelfLower,

    #[token("Self")]
    SelfUpper,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("as")]
    As,

    #[token("where")]
    Where,

    // === Lemon-specific keywords ===
    #[token("effects")]
    Effects,

    #[token("requires")]
    Requires,

    #[token("spawn")]
    Spawn,

    #[token("scope")]
    Scope,

    #[token("select")]
    Select,

    // === Operators ===
    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[token("&")]
    Amp,

    #[token("&&")]
    AmpAmp,

    #[token("|")]
    Pipe,

    #[token("||")]
    PipePipe,

    #[token("^")]
    Caret,

    #[token("!")]
    Bang,

    #[token("~")]
    Tilde,

    #[token("=")]
    Eq,

    #[token("==")]
    EqEq,

    #[token("!=")]
    BangEq,

    #[token("<")]
    Lt,

    #[token("<=")]
    LtEq,

    #[token(">")]
    Gt,

    #[token(">=")]
    GtEq,

    #[token("<<")]
    LtLt,

    #[token(">>")]
    GtGt,

    #[token("->")]
    Arrow,

    #[token("=>")]
    FatArrow,

    #[token("?")]
    Question,

    #[token(".")]
    Dot,

    #[token("..")]
    DotDot,

    #[token("..=")]
    DotDotEq,

    #[token("::")]
    ColonColon,

    // === Delimiters ===
    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token(";")]
    Semi,

    #[token("@")]
    At,

    #[token("#")]
    Hash,

    // === Literals ===
    #[regex(r"[0-9][0-9_]*", |lex| lex.slice().replace('_', "").parse::<i64>().ok())]
    Int(i64),

    #[regex(r"0x[0-9a-fA-F][0-9a-fA-F_]*", parse_hex)]
    HexInt(i64),

    #[regex(r"0b[01][01_]*", parse_binary)]
    BinInt(i64),

    #[regex(r"[0-9][0-9_]*\.[0-9][0-9_]*([eE][+-]?[0-9]+)?", |lex| lex.slice().replace('_', "").parse::<f64>().ok())]
    Float(f64),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| {
        let s = lex.slice();
        Some(unescape_string(&s[1..s.len()-1]))
    })]
    String(String),

    #[regex(r"'([^'\\]|\\.)'", |lex| {
        let s = lex.slice();
        parse_char(&s[1..s.len()-1])
    })]
    Char(char),

    // === Identifiers ===
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    /// Error token for invalid input
    Error,
}

fn parse_hex(lex: &logos::Lexer<Token>) -> Option<i64> {
    let s = lex.slice();
    i64::from_str_radix(&s[2..].replace('_', ""), 16).ok()
}

fn parse_binary(lex: &logos::Lexer<Token>) -> Option<i64> {
    let s = lex.slice();
    i64::from_str_radix(&s[2..].replace('_', ""), 2).ok()
}

fn parse_char(s: &str) -> Option<char> {
    let mut chars = s.chars();
    match chars.next()? {
        '\\' => match chars.next()? {
            'n' => Some('\n'),
            'r' => Some('\r'),
            't' => Some('\t'),
            '\\' => Some('\\'),
            '\'' => Some('\''),
            '0' => Some('\0'),
            _ => None,
        },
        c => Some(c),
    }
}

fn unescape_string(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('n') => result.push('\n'),
                Some('r') => result.push('\r'),
                Some('t') => result.push('\t'),
                Some('\\') => result.push('\\'),
                Some('"') => result.push('"'),
                Some('0') => result.push('\0'),
                Some(other) => {
                    result.push('\\');
                    result.push(other);
                }
                None => result.push('\\'),
            }
        } else {
            result.push(c);
        }
    }

    result
}

impl Token {
    /// Returns true if this token can start an expression
    pub fn can_start_expr(&self) -> bool {
        matches!(
            self,
            Token::Ident(_)
                | Token::Int(_)
                | Token::Float(_)
                | Token::String(_)
                | Token::Char(_)
                | Token::True
                | Token::False
                | Token::LParen
                | Token::LBracket
                | Token::LBrace
                | Token::If
                | Token::Match
                | Token::Loop
                | Token::While
                | Token::For
                | Token::Return
                | Token::Break
                | Token::Continue
                | Token::Bang
                | Token::Minus
                | Token::Star
                | Token::Amp
                | Token::SelfLower
                // OOP expression starters
                | Token::New
                | Token::This
                | Token::Super
                | Token::SelfUpper
        )
    }

    /// Returns true if this is a binary operator
    pub fn is_binary_op(&self) -> bool {
        matches!(
            self,
            Token::Plus
                | Token::Minus
                | Token::Star
                | Token::Slash
                | Token::Percent
                | Token::AmpAmp
                | Token::PipePipe
                | Token::Amp
                | Token::Pipe
                | Token::Caret
                | Token::EqEq
                | Token::BangEq
                | Token::Lt
                | Token::LtEq
                | Token::Gt
                | Token::GtEq
                | Token::LtLt
                | Token::GtGt
        )
    }
}
