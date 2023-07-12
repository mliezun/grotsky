use crate::expr::*;
use crate::tokens::*;

#[derive(Debug, PartialEq, Clone)]
pub struct LetStmt {
    pub name: TokenData,
    pub initializer: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassicForStmt {
    pub keyword: TokenData,
    pub initializer: Option<Box<Stmt>>,
    pub condition: Expr,
    pub increment: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnhancedForStmt {
    pub keyword: TokenData,
    pub identifiers: Vec<TokenData>,
    pub collection: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TryCatchStmt {
    pub try_body: Box<Stmt>,
    pub name: TokenData,
    pub catch_body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileStmt {
    pub keyword: TokenData,
    pub condition: Expr,
    pub body: Box<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStmt {
    pub keyword: TokenData,
    pub value: Option<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BreakStmt {
    pub keyword: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ContinueStmt {
    pub keyword: TokenData,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ElifBranch {
    pub condition: Expr,
    pub then_branch: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfStmt {
    pub keyword: TokenData,
    pub condition: Expr,
    pub then_branch: Vec<Stmt>,
    pub elifs: Vec<ElifBranch>,
    pub else_branch: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FnStmt {
    pub name: TokenData,
    pub params: Vec<TokenData>,
    pub body: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassStmt {
    pub name: Option<TokenData>,
    pub methods: Vec<FnStmt>,
    pub static_methods: Vec<FnStmt>,
    pub superclass: Option<VarExpr>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BlockStmt {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ExprStmt {
    pub last: Option<TokenData>,
    pub expression: Expr,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Stmt {
    Fn(FnStmt),
    Let(LetStmt),
    Block(BlockStmt),
    Class(ClassStmt),
    ClassicFor(ClassicForStmt),
    EnhancedFor(EnhancedForStmt),
    While(WhileStmt),
    If(IfStmt),
    Continue(ContinueStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    TryCatch(TryCatchStmt),
    Expr(ExprStmt),
}
