use crate::expr::*;
use crate::instruction::*;
use crate::parser::*;
use crate::stmt::*;
use crate::token::Literal;
use crate::token::Token;
use crate::value::*;

#[derive(Debug)]
pub struct Compiler {
    // call_stack: Vec<CallStack>,
    pub constants: Vec<Value>,
    pub register_count: u8,
    pub chunks: Vec<Chunk>,
}

impl Compiler {
    pub fn compile(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            let chunk = stmt.accept(self);
            self.chunks.push(chunk);
        }
    }
}

struct FnContext {}

#[derive(Debug)]
pub struct Chunk {
    instructions: Vec<Instruction>,
    result_register: u8,
}

impl StmtVisitor<Chunk> for Compiler {
    fn visit_expr_stmt(&mut self, stmt: &ExprStmt) -> Chunk {
        return stmt.expression.accept(self);
    }

    fn visit_try_catch_stmt(&mut self, stmt: &TryCatchStmt) -> Chunk {
        todo!()
    }

    fn visit_classic_for_stmt(&mut self, stmt: &ClassicForStmt) -> Chunk {
        todo!()
    }

    fn visit_enhanced_for_stmt(&mut self, stmt: &EnhancedForStmt) -> Chunk {
        todo!()
    }

    fn visit_let_stmt(&mut self, stmt: &LetStmt) -> Chunk {
        todo!()
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Chunk {
        todo!()
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Chunk {
        todo!()
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Chunk {
        todo!()
    }

    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> Chunk {
        todo!()
    }

    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> Chunk {
        todo!()
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Chunk {
        todo!()
    }

    fn visit_fn_stmt(&mut self, stmt: &FnStmt) -> Chunk {
        todo!()
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Chunk {
        todo!()
    }
}

impl ExprVisitor<Chunk> for Compiler {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> Chunk {
        todo!()
    }

    fn visit_variable_expr(&mut self, expr: &VarExpr) -> Chunk {
        todo!()
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Chunk {
        todo!()
    }

    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Chunk {
        todo!()
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Chunk {
        todo!()
    }

    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Chunk {
        todo!()
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Chunk {
        let left_chunk = expr.left.accept(self);
        let right_chunk = expr.right.accept(self);
        let opcode = match expr.operator.token {
            Token::Plus => OpCode::Add,
            _ => todo!(),
        };
        let result_register = self.register_count;
        self.register_count += 1;
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: result_register,
        };
        chunk
            .instructions
            .append(&mut left_chunk.instructions.clone());
        chunk
            .instructions
            .append(&mut right_chunk.instructions.clone());
        chunk.instructions.push(Instruction {
            opcode: opcode,
            a: result_register,
            b: left_chunk.result_register,
            c: right_chunk.result_register,
        });
        return chunk;
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Chunk {
        todo!()
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> Chunk {
        todo!()
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> Chunk {
        todo!()
    }

    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Chunk {
        todo!()
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Chunk {
        todo!()
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Chunk {
        let val = match expr.value {
            Literal::Number(n) => Value::Number(NumberValue { n: n }),
            _ => todo!(),
        };
        let result_register = self.register_count;
        self.register_count += 1;
        self.constants.push(val);
        let len_constants = self.constants.len() as u16 - 1;
        let chunk = Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::LoadK,
                a: result_register,
                b: (len_constants >> 8) as u8,
                c: len_constants as u8,
            }],
            result_register: result_register,
        };
        return chunk;
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Chunk {
        todo!()
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Chunk {
        todo!()
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Chunk {
        todo!()
    }
}

impl StmtAcceptor<Chunk> for Stmt {
    fn accept(&self, visitor: &mut dyn StmtVisitor<Chunk>) -> Chunk {
        match self {
            Stmt::Fn(stmt) => visitor.visit_fn_stmt(&stmt),
            Stmt::Let(stmt) => visitor.visit_let_stmt(&stmt),
            Stmt::Block(stmt) => visitor.visit_block_stmt(&stmt),
            Stmt::Class(stmt) => visitor.visit_class_stmt(&stmt),
            Stmt::ClassicFor(stmt) => visitor.visit_classic_for_stmt(&stmt),
            Stmt::EnhancedFor(stmt) => visitor.visit_enhanced_for_stmt(&stmt),
            Stmt::While(stmt) => visitor.visit_while_stmt(&stmt),
            Stmt::If(stmt) => visitor.visit_if_stmt(&stmt),
            Stmt::Continue(stmt) => visitor.visit_continue_stmt(&stmt),
            Stmt::Return(stmt) => visitor.visit_return_stmt(&stmt),
            Stmt::Break(stmt) => visitor.visit_break_stmt(&stmt),
            Stmt::TryCatch(stmt) => visitor.visit_try_catch_stmt(&stmt),
            Stmt::Expr(stmt) => visitor.visit_expr_stmt(&stmt),
        }
    }
}

impl ExprAcceptor<Chunk> for Expr {
    fn accept(&self, visitor: &mut dyn ExprVisitor<Chunk>) -> Chunk {
        match self {
            Expr::Fn(expr) => visitor.visit_function_expr(&expr),
            Expr::Var(expr) => visitor.visit_variable_expr(&expr),
            Expr::List(expr) => visitor.visit_list_expr(&expr),
            Expr::Dictionary(expr) => visitor.visit_dictionary_expr(&expr),
            Expr::Assign(expr) => visitor.visit_assign_expr(&expr),
            Expr::Access(expr) => visitor.visit_access_expr(&expr),
            Expr::Binary(expr) => visitor.visit_binary_expr(&expr),
            Expr::Call(expr) => visitor.visit_call_expr(&expr),
            Expr::Get(expr) => visitor.visit_get_expr(&expr),
            Expr::Set(expr) => visitor.visit_set_expr(&expr),
            Expr::Super(expr) => visitor.visit_super_expr(&expr),
            Expr::Grouping(expr) => visitor.visit_grouping_expr(&expr),
            Expr::Literal(expr) => visitor.visit_literal_expr(&expr),
            Expr::Logical(expr) => visitor.visit_logical_expr(&expr),
            Expr::This(expr) => visitor.visit_this_expr(&expr),
            Expr::Unary(expr) => visitor.visit_unary_expr(&expr),
            Expr::Empty => unreachable!(),
        }
    }
}
