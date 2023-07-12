use crate::expr::*;
use crate::parser::*;
use crate::stmt::*;

pub struct Compiler {
    call_stack: Vec<CallStack>,
}

impl StmtVisitor for Compiler {}

impl ExprVisitor for Compiler {}

impl Compiler {}
