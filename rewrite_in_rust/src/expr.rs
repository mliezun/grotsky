use crate::stmt::*;
use crate::tokens::*;

#[derive(Debug, PartialEq, Clone)]
pub struct FnExpr {
    pub params: Vec<TokenData>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VarExpr {
    pub name: Option<TokenData>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ListExpr {
    pub elements: Vec<Expr>,
    pub brace: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DictionaryExpr {
    pub elements: Vec<Expr>,
    pub curly_brace: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AssignExpr {
    pub name: TokenData,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AccessExpr {
    pub object: Box<Expr>,
    pub brace: TokenData,
    pub first: Box<Expr>,
    pub first_colon: TokenData,
    pub second: Box<Expr>,
    pub second_colon: TokenData,
    pub third: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: TokenData,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub paren: TokenData,
    pub arguments: Vec<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GetExpr {
    pub object: Box<Expr>,
    pub name: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SetExpr {
    pub object: Box<Expr>,
    pub name: TokenData,
    pub value: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SuperExpr {
    pub keyword: TokenData,
    pub method: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct GroupingExpr {
    pub expression: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralExpr {
    pub value: Literal,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LogicalExpr {
    pub left: Box<Expr>,
    pub operator: TokenData,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThisExpr {
    pub keyword: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryExpr {
    pub operator: TokenData,
    pub right: Box<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr {
    Fn(FnExpr),
    Var(VarExpr),
    List(ListExpr),
    Dictionary(DictionaryExpr),
    Assign(AssignExpr),
    Access(AccessExpr),
    Binary(BinaryExpr),
    Call(CallExpr),
    Get(GetExpr),
    Set(SetExpr),
    Super(SuperExpr),
    Grouping(GroupingExpr),
    Literal(LiteralExpr),
    Logical(LogicalExpr),
    This(ThisExpr),
    Unary(UnaryExpr),
    Empty,
}
