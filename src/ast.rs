#[derive(Clone, Debug, PartialEq)]
#[allow(dead_code)]
pub enum Stmt {
    TailReturn(Box<Expr>),
    Return(Box<Expr>),
    Expr(Box<Expr>),
    NotPopValueExpr(Box<Expr>),
    Let(Box<Expr>, Box<Node>),
}

#[derive(Clone, Debug, PartialEq)]
pub enum Expr {
    Number(i64),
    Float(String),
    String(String),
    Ident(String),
    BinaryOp(Box<Expr>, String, Box<Expr>),
    UnaryOp(String, Box<Expr>),
    Assign(String, Box<Expr>),
    Block(Vec<Node>),
    Call(String, Vec<Expr>),
    DynCall(Box<Expr>, Vec<Expr>),
    Lambda(Vec<String>, Box<Expr>),
    Null,                                        // not null value, this is null expr
    IdentList(Vec<String>),                      // for function parameters
    If(Box<Expr>, Box<Expr>, Option<Box<Expr>>), // condition, then, else
}
#[derive(Clone, Debug, PartialEq)]
pub struct Node {
    pub stmt: Stmt,
    pub l: usize,
    pub c: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Program {
    pub body: Vec<Node>,
}
impl Program {
    pub fn new(body: Vec<Node>) -> Self {
        Self { body }
    }
}
