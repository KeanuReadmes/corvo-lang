use crate::span::Span;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum MatchPattern {
    Literal(crate::type_system::Value),
    Regex(String, String), // pattern, flags
    Wildcard,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: MatchPattern,
    pub body: Box<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        value: crate::type_system::Value,
    },
    VarGet {
        name: String,
    },
    StaticGet {
        name: String,
    },
    StringInterpolation {
        parts: Vec<Expr>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expr>,
        named_args: HashMap<String, Expr>,
    },
    IndexAccess {
        target: Box<Expr>,
        index: Box<Expr>,
    },
    Match {
        value: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    MethodCall {
        target: Box<Expr>,
        method: String,
        args: Vec<Expr>,
        named_args: HashMap<String, Expr>,
    },
}

impl Expr {
    pub fn span(&self) -> Option<Span> {
        None
    }
}
