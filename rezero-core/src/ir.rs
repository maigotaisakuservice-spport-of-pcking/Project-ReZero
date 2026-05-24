use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Type {
    Int,
    Bool,
    Array(Box<Type>, usize),
    Pointer(Box<Type>),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Op { Add, Sub, Mul, Div }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum CompOp { Gt, Ge, Lt, Le, Eq, Ne }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Expr {
    Literal(i32),
    Variable(String),
    Binary { op: Op, left: Box<Expr>, right: Box<Expr> },
    Compare { op: CompOp, left: Box<Expr>, right: Box<Expr> },
    ArrayAccess { name: String, index: Box<Expr> },
    PointerDereference(String),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum IR {
    Declare { name: String, ty: Type, init: Option<Expr> },
    Assign { name: String, val: Expr },
    If { cond: Expr, then_b: Vec<IR>, else_b: Vec<IR> },
    Return(Expr),
    AssertPre(Expr),
    AssertPost(Expr),
    PushLRS { name: String, ty: Type },
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<(String, Type)>,
    pub ret_ty: Type,
    pub body: Vec<IR>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Program {
    pub functions: Vec<Function>,
}
