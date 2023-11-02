use crate::stmt::*;
use crate::token::*;

#[derive(Debug, Clone)]
pub struct InterpreterError {
    pub message: String,
    pub line: i32,
    pub pos: usize,
}

#[derive(Debug, Clone)]
pub struct InterpreterState {
    pub source: String,
    pub tokens: Vec<TokenData>,
    pub errors: Vec<InterpreterError>,
    pub stmts: Vec<Stmt>,
}

impl InterpreterState {
    pub fn new(source: String) -> Self {
        InterpreterState {
            source: source,
            tokens: vec![],
            errors: vec![],
            stmts: vec![],
        }
    }

    pub fn set_error(&mut self, err: InterpreterError) {
        self.errors.push(err);
    }

    pub fn fatal_error(&mut self, err: InterpreterError) {
        self.errors.push(err);
        panic!();
    }
}
