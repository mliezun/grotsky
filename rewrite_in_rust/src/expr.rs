use crate::stmt::*;
use crate::token::*;

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

impl Expr {
    pub fn is_empty(&self) -> bool {
        match self {
            Expr::Empty => true,
            _ => false,
        }
    }
}

pub trait ExprVisitor<T> {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> T;
    fn visit_variable_expr(&mut self, expr: &VarExpr) -> T;
    fn visit_list_expr(&mut self, expr: &ListExpr) -> T;
    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> T;
    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> T;
    fn visit_access_expr(&mut self, expr: &AccessExpr) -> T;
    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> T;
    fn visit_call_expr(&mut self, expr: &CallExpr) -> T;
    fn visit_get_expr(&mut self, expr: &GetExpr) -> T;
    fn visit_set_expr(&mut self, expr: &SetExpr) -> T;
    fn visit_super_expr(&mut self, expr: &SuperExpr) -> T;
    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> T;
    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> T;
    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> T;
    fn visit_this_expr(&mut self, expr: &ThisExpr) -> T;
    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> T;
}

pub trait ExprAcceptor<T> {
    fn accept(&self, visitor: &mut dyn ExprVisitor<T>) -> T;
}
