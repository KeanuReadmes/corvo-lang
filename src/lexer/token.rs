use crate::span::{Position, Span};
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Keywords
    Prep,
    Static,
    Var,
    Try,
    Fallback,
    Loop,
    Browse,
    Terminate,
    DontPanic,
    Match,
    AssertEq,
    AssertNeq,
    AssertGt,
    AssertLt,
    AssertMatch,

    // Literals
    String(String),
    Number(f64),
    Boolean(bool),
    Regex(String, String), // pattern, flags
    Identifier(String),

    // Operators and delimiters
    LeftBrace,
    RightBrace,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    Colon,
    At,
    Equals,
    FatArrow,

    // Special
    StringInterpolation(Vec<Token>),
    Comment(String),
    Eof,
    Illegal(String),
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Prep => write!(f, "prep"),
            Self::Static => write!(f, "static"),
            Self::Var => write!(f, "var"),
            Self::Try => write!(f, "try"),
            Self::Fallback => write!(f, "fallback"),
            Self::Loop => write!(f, "loop"),
            Self::Browse => write!(f, "browse"),
            Self::Terminate => write!(f, "terminate"),
            Self::DontPanic => write!(f, "dont_panic"),
            Self::Match => write!(f, "match"),
            Self::AssertEq => write!(f, "assert_eq"),
            Self::AssertNeq => write!(f, "assert_neq"),
            Self::AssertGt => write!(f, "assert_gt"),
            Self::AssertLt => write!(f, "assert_lt"),
            Self::AssertMatch => write!(f, "assert_match"),
            Self::String(s) => write!(f, "\"{}\"", s),
            Self::Regex(pattern, flags) => write!(f, "/{}/{}", pattern, flags),
            Self::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            Self::Boolean(b) => write!(f, "{}", b),
            Self::Identifier(name) => write!(f, "{}", name),
            Self::LeftBrace => write!(f, "{{"),
            Self::RightBrace => write!(f, "}}"),
            Self::LeftParen => write!(f, "("),
            Self::RightParen => write!(f, ")"),
            Self::LeftBracket => write!(f, "["),
            Self::RightBracket => write!(f, "]"),
            Self::Comma => write!(f, ","),
            Self::Dot => write!(f, "."),
            Self::Colon => write!(f, ":"),
            Self::At => write!(f, "@"),
            Self::Equals => write!(f, "="),
            Self::FatArrow => write!(f, "=>"),
            Self::StringInterpolation(_) => write!(f, "string(...)"),
            Self::Comment(text) => write!(f, "#{}", text),
            Self::Eof => write!(f, "EOF"),
            Self::Illegal(ch) => write!(f, "ILLEGAL({})", ch),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub token_type: TokenType,
    pub span: Span,
}

impl Token {
    pub fn new(token_type: TokenType, span: Span) -> Self {
        Self { token_type, span }
    }

    pub fn string(value: String, start: Position, end: Position) -> Self {
        Self::new(TokenType::String(value), Span::new(start, end))
    }

    pub fn number(value: f64, start: Position, end: Position) -> Self {
        Self::new(TokenType::Number(value), Span::new(start, end))
    }

    pub fn boolean(value: bool, start: Position, end: Position) -> Self {
        Self::new(TokenType::Boolean(value), Span::new(start, end))
    }

    pub fn identifier(name: String, start: Position, end: Position) -> Self {
        Self::new(TokenType::Identifier(name), Span::new(start, end))
    }

    pub fn keyword(keyword: TokenType, start: Position, end: Position) -> Self {
        Self::new(keyword, Span::new(start, end))
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} @ {}", self.token_type, self.span)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_display_keywords() {
        let span = Span::point(Position::start());
        assert_eq!(
            Token::new(TokenType::Static, span).to_string(),
            "static @ 1:1..1:1"
        );
        assert_eq!(
            Token::new(TokenType::Var, span).to_string(),
            "var @ 1:1..1:1"
        );
    }

    #[test]
    fn test_token_display_literals() {
        let span = Span::point(Position::start());
        assert_eq!(
            Token::new(TokenType::String("hello".to_string()), span).to_string(),
            "\"hello\" @ 1:1..1:1"
        );
        assert_eq!(
            Token::new(TokenType::Number(42.0), span).to_string(),
            "42 @ 1:1..1:1"
        );
        assert_eq!(
            Token::new(TokenType::Boolean(true), span).to_string(),
            "true @ 1:1..1:1"
        );
    }

    #[test]
    fn test_token_display_operators() {
        let span = Span::point(Position::start());
        assert_eq!(
            Token::new(TokenType::LeftBrace, span).to_string(),
            "{ @ 1:1..1:1"
        );
        assert_eq!(
            Token::new(TokenType::RightParen, span).to_string(),
            ") @ 1:1..1:1"
        );
        assert_eq!(Token::new(TokenType::Dot, span).to_string(), ". @ 1:1..1:1");
    }

    #[test]
    fn test_token_display_special() {
        let span = Span::point(Position::start());
        assert_eq!(
            Token::new(TokenType::Eof, span).to_string(),
            "EOF @ 1:1..1:1"
        );
        assert_eq!(
            Token::new(TokenType::Illegal("@".to_string()), span).to_string(),
            "ILLEGAL(@) @ 1:1..1:1"
        );
    }
}
