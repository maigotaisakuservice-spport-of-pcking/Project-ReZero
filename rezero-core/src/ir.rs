use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Type {
    Int32,
    Bool,
    Array(Box<Type>, usize),
    Pointer(Box<Type>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op {
    Add, Sub, Mul, Div,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompOp {
    Gt, Ge, Lt, Le, Eq, Ne,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Expr {
    Literal(i32),
    BoolLiteral(bool),
    Variable(String),
    Binary { op: Op, left: Box<Expr>, right: Box<Expr> },
    Compare { op: CompOp, left: Box<Expr>, right: Box<Expr> },
    ArrayAccess { array_name: String, index: Box<Expr> },
    PointerDereference { pointer_name: String },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IR {
    Declare { name: String, ty: Type, init_val: Option<Expr> },
    Assign { name: String, val: Expr },
    PushLRS { name: String, ty: Type },
    Assert { cond: Expr, message: String },
    If { cond: Expr, then_branch: Vec<IR>, else_branch: Vec<IR> },
    Return(Expr),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub return_type: Type,
    pub requires: Vec<Expr>,
    pub ensures: Vec<Expr>,
    pub body: Vec<IR>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}
