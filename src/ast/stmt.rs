use super::expr::Expr;

#[derive(Debug, Clone, PartialEq)]
pub struct FallbackBlock {
    pub body: Vec<Stmt>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    PrepBlock {
        body: Vec<Stmt>,
    },
    StaticSet {
        name: String,
        value: Expr,
    },
    VarSet {
        name: String,
        value: Expr,
    },
    ExprStmt {
        expr: Expr,
    },
    TryBlock {
        body: Vec<Stmt>,
        fallbacks: Vec<FallbackBlock>,
    },
    Loop {
        body: Vec<Stmt>,
    },
    Browse {
        iterable: Expr,
        key: String,
        value: String,
        body: Vec<Stmt>,
    },
    Terminate,
    Assert {
        kind: AssertKind,
        args: Vec<Expr>,
    },
    DontPanic {
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum AssertKind {
    Eq,
    Neq,
    Gt,
    Lt,
    Match,
}
