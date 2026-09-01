//! The shapes the parser produces and the type checker and interpreter walk.

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Bool,
    Str,
    Fn(Vec<Type>, Box<Type>),
}

impl std::fmt::Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Int => write!(f, "Int"),
            Type::Bool => write!(f, "Bool"),
            Type::Str => write!(f, "Str"),
            Type::Fn(ps, r) => {
                let ps: Vec<String> = ps.iter().map(|t| t.to_string()).collect();
                write!(f, "({}) -> {}", ps.join(", "), r)
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Concat, // ++ on strings
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Int(i64),
    Bool(bool),
    Str(String),
    Var(String),
    Unary(UnOp, Box<Expr>),
    Binary(BinOp, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Lambda {
        params: Vec<(String, Type)>,
        ret: Option<Type>,
        body: Box<Expr>,
    },
    Call(Box<Expr>, Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Let {
        name: String,
        rec: bool,
        ty: Option<Type>,
        value: Expr,
    },
    Expr(Expr),
}

pub type Program = Vec<Stmt>;
