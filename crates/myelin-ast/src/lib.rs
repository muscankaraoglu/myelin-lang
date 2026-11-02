//! Abstract Syntax Tree (AST) definitions for Myelin (`.myl`) v0.1.

use myelin_diag::Span;

/// A complete parsed `.myl` module.
#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub items: Vec<Item>,
    pub span: Span,
}

/// Dotted identifier path such as `bank.ledger` or `db.rw`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Path {
    pub segments: Vec<String>,
    pub span: Span,
}

impl Path {
    pub fn as_dotted(&self) -> String {
        self.segments.join(".")
    }
}

/// Explicit OS/effect capability tag such as `!db.rw`, `!fs.r`, or `!unsafe.mem`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Capability {
    pub path: Path,
    pub span: Span,
}

/// Top-level `#` declarations starting at column 0.
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Mod(ModDecl),
    Use(UseDecl),
    Rec(RecDecl),
    Sum(SumDecl),
    Ext(ExtDecl),
    Fn(FnDecl),
    Test(TestDecl),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModDecl {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecl {
    pub path: Path,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecDecl {
    pub name: String,
    pub fields: Vec<Param>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SumDecl {
    pub name: String,
    pub variants: Vec<Variant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variant {
    pub name: String,
    pub payload: Vec<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub ty: TypeExpr,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Signature {
    pub params: Vec<Param>,
    pub ret: Option<TypeExpr>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExtDecl {
    pub abi: String,
    pub name: String,
    pub sig: Signature,
    pub caps: Vec<Capability>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FnDecl {
    pub name: String,
    pub sig: Signature,
    pub caps: Vec<Capability>,
    pub contracts: Vec<Contract>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TestDecl {
    pub name: String,
    pub caps: Vec<Capability>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractKind {
    /// `req <expr>` — caller precondition
    Req,
    /// `ens <expr>` — callee postcondition (`ret` binds return value)
    Ens,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Contract {
    pub kind: ContractKind,
    pub expr: Expr,
    pub span: Span,
}

/// First-class type expressions (`i64`, `[]Account`, `[str:i64]`, `?User`, `i64!TxErr`, `*u8`).
#[derive(Debug, Clone, PartialEq)]
pub enum TypeExpr {
    Named {
        path: Path,
        span: Span,
    },
    Slice {
        elem: Box<TypeExpr>,
        span: Span,
    },
    Map {
        key: Box<TypeExpr>,
        val: Box<TypeExpr>,
        span: Span,
    },
    Optional {
        inner: Box<TypeExpr>,
        span: Span,
    },
    Result {
        ok: Box<TypeExpr>,
        err: Box<TypeExpr>,
        span: Span,
    },
    Pointer {
        pointee: Box<TypeExpr>,
        span: Span,
    },
}

impl TypeExpr {
    pub fn span(&self) -> Span {
        match self {
            TypeExpr::Named { span, .. }
            | TypeExpr::Slice { span, .. }
            | TypeExpr::Map { span, .. }
            | TypeExpr::Optional { span, .. }
            | TypeExpr::Result { span, .. }
            | TypeExpr::Pointer { span, .. } => *span,
        }
    }
}

/// Atomic flat-flow statements inside `#fn`, `#test`, or control blocks.
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    /// `target = value` (lowered to SSA in `myelin-mir`)
    Bind {
        target: String,
        value: Expr,
        span: Span,
    },
    /// `= expr` or bare `=` (unit return)
    Return { value: Option<Expr>, span: Span },
    /// `^ expr` (early error return)
    Fail { error: Expr, span: Span },
    /// `? cond` (assert) or `? cond | action` (flat guard)
    Guard {
        cond: Expr,
        otherwise: Option<GuardAction>,
        span: Span,
    },
    /// `@if cond ... (@elif ...)* (@el ...)?`
    If {
        branches: Vec<(Expr, Vec<Stmt>)>,
        else_body: Option<Vec<Stmt>>,
        span: Span,
    },
    /// `@for item : iter ...`
    For {
        binding: String,
        iter: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    /// `@loop ...`
    Loop { body: Vec<Stmt>, span: Span },
    /// `@match scrutinee ...`
    Match {
        scrutinee: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    /// `@next` (loop continue)
    Next { span: Span },
    /// `@brk` (loop break)
    Break { span: Span },
    /// Standalone call/expression statement (`db.seed_acc 1 true 500`)
    Expr { expr: Expr, span: Span },
}

#[derive(Debug, Clone, PartialEq)]
pub enum GuardAction {
    Fail(Expr),
    Return(Option<Expr>),
    Next,
    Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Wildcard(Span),
    Literal(Literal, Span),
    Variant {
        path: Path,
        args: Vec<Pattern>,
        span: Span,
    },
}

impl Pattern {
    pub fn span(&self) -> Span {
        match self {
            Pattern::Wildcard(s) | Pattern::Literal(_, s) | Pattern::Variant { span: s, .. } => *s,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    None,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Rem,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Coalesce,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Literal {
        lit: Literal,
        span: Span,
    },
    Ident {
        name: String,
        span: Span,
    },
    List {
        items: Vec<Expr>,
        span: Span,
    },
    Field {
        base: Box<Expr>,
        field: String,
        span: Span,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
        span: Span,
    },
    /// `expr ?` or `expr ? ^ErrExpr`
    Try {
        expr: Box<Expr>,
        map_err: Option<Box<Expr>>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Literal { span, .. }
            | Expr::Ident { span, .. }
            | Expr::List { span, .. }
            | Expr::Field { span, .. }
            | Expr::Index { span, .. }
            | Expr::Call { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::Try { span, .. } => *span,
        }
    }
}
