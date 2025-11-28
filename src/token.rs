use serde::{Deserialize, Serialize};

// #[derive(Debug)]
// pub enum Operator {
//     Add,
//     Sub,
//     Div,
//     Mod,
//     Mul,
//     Pow,
//     Neg,
//     Eq,
//     Neq,
//     Gt,
//     Gte,
//     Lt,
//     Lte,
// }

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Token {
    EOF,
    Newline,
    // Single-character tokens.
    // (, ), [, ], {, } ',', ., -, +, ;, /, %, *, ^, :, ;
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    RightCurlyBrace,
    LeftCurlyBrace,
    Comma,
    Dot,
    Minus,
    Plus,
    Slash,
    Mod,
    Star,
    Power,
    Colon,
    Semicolon,
    // One or two character tokens.
    // !=, =, ==, >, >=, <, <=
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,
    // Literals.
    // *variable*, string, int,
    Identifier,
    String,
    Number,
    // Keywords.
    // and, class, else, false, fn, for, if, elif, nil, or,
    // return, break, continue, super, this, true, let, while, not, in, begin, end
    And,
    Class,
    Else,
    False,
    Fn,
    For,
    If,
    Elif,
    Nil,
    Or,
    Return,
    Break,
    Continue,
    Super,
    This,
    True,
    Let,
    While,
    Not,
    In,
    Try,
    Catch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Literal {
    String(String),
    Number(f64),
    Boolean(bool),
    Nil,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenData {
    pub token: Token,
    pub lexeme: String,
    pub literal: Option<Literal>,
    pub line: i32,
}
