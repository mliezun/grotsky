use std::collections::HashMap;

use crate::expr::*;
use crate::instruction::*;
use crate::stmt::*;
use crate::token::*;
use crate::value::*;

#[derive(Debug)]
pub struct Compiler {
    // call_stack: Vec<CallStack>,
    pub constants: Vec<Value>,
    pub contexts: Vec<FnContext>,
    pub prototypes: Vec<FnPrototype>,
}

impl Compiler {
    fn add_chunk(&mut self, chunk: Chunk) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.chunks.push(chunk);
    }

    fn allocate_register(&mut self, var_name: String, reg: u8) {
        let current_context = self.contexts.last_mut().unwrap();
        let current_block = current_context.blocks.last_mut().unwrap();
        current_block.locals.push(Local {
            var_name: var_name,
            reg: reg,
        });
    }

    fn next_register(&mut self) -> u8 {
        let current_context = self.contexts.last_mut().unwrap();
        let reg = current_context.register_count;
        current_context.register_count += 1;
        return reg;
    }

    fn enter_block(&mut self) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.blocks.push(Block { locals: vec![] });
    }

    fn leave_block(&mut self) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.blocks.pop();
    }

    fn enter_function(&mut self, name: String) {
        self.contexts.push(FnContext {
            name: name,
            loop_count: 0,
            register_count: 0,
            chunks: vec![],
            blocks: vec![Block { locals: vec![] }],
        });
    }

    fn leave_function(&mut self) -> u16 {
        let current_context = self.contexts.pop().unwrap();
        let prototype_ix = self.prototypes.len();
        self.prototypes.push(FnPrototype {
            instructions: current_context
                .chunks
                .iter()
                .map(|c| c.instructions.clone())
                .flatten()
                .collect(),
            register_count: current_context.register_count,
        });
        return prototype_ix as u16;
    }

    fn result_register(&self) -> u8 {
        let current_context = self.contexts.last().unwrap();
        return current_context.chunks.last().unwrap().result_register;
    }

    fn get_var_register(&self, var_name: String) -> Option<u8> {
        let current_context = self.contexts.last().unwrap();
        current_context
            .blocks
            .iter()
            .map(|b| b.locals.clone())
            .flatten()
            .rev()
            .find_map(|l| {
                if l.var_name.eq(&var_name) {
                    Some(l.reg)
                } else {
                    None
                }
            })
    }

    pub fn compile(&mut self, stmts: Vec<Stmt>) {
        for stmt in stmts {
            let chunk = stmt.accept(self);
            self.add_chunk(chunk);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FnPrototype {
    pub instructions: Vec<Instruction>,
    pub register_count: u8,
}

#[derive(Debug)]
pub struct FnContext {
    pub name: String,
    pub loop_count: u8,
    pub register_count: u8,
    pub chunks: Vec<Chunk>,
    pub blocks: Vec<Block>,
}

#[derive(Clone, Debug)]
pub struct Block {
    pub locals: Vec<Local>,
}

#[derive(Clone, Debug)]
pub struct Local {
    var_name: String,
    reg: u8,
}

#[derive(Debug)]
pub struct Chunk {
    pub instructions: Vec<Instruction>,
    pub result_register: u8,
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
        if let Some(init) = &stmt.initializer {
            let chunk = init.accept(self);
            self.allocate_register(stmt.name.lexeme.to_string(), chunk.result_register);
            return chunk;
        } else {
            let reg = self.next_register();
            self.allocate_register(stmt.name.lexeme.to_string(), reg);
            return Chunk {
                instructions: vec![],
                result_register: reg,
            };
        }
    }

    fn visit_block_stmt(&mut self, stmt: &BlockStmt) -> Chunk {
        self.enter_block();
        let chunk = Chunk {
            instructions: stmt
                .stmts
                .iter()
                .map(|s| s.accept(self).instructions)
                .flatten()
                .collect(),
            result_register: self.result_register(),
        };
        self.leave_block();
        return chunk;
    }

    fn visit_while_stmt(&mut self, stmt: &WhileStmt) -> Chunk {
        let cond_chunk = stmt.condition.accept(self);
        let body_chunk = stmt.body.accept(self);
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: cond_chunk.result_register,
        };
        chunk
            .instructions
            .append(&mut cond_chunk.instructions.clone());
        chunk.instructions.push(Instruction {
            opcode: OpCode::Test,
            a: cond_chunk.result_register,
            b: cond_chunk.result_register,
            c: 0,
        });
        let jump_size = (body_chunk.instructions.len() + 2) as i16;
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (jump_size >> 8) as u8,
            c: jump_size as u8,
        });
        chunk
            .instructions
            .append(&mut body_chunk.instructions.clone());
        // CAUTION
        // Here im adding +1, but I think it should be +2. It still works for the examples
        // that I've tried. But I remain suspicious.
        // Having a +1 here makes the loop faster, because it has to execute 1 instruction less.
        // If there are any issues regarding while loops then change to +2.
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 1) as i16);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (loop_size >> 8) as u8,
            c: loop_size as u8,
        });
        return chunk;
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
        let reg = self.next_register();
        self.allocate_register(stmt.name.lexeme.to_string(), reg);
        self.enter_function(stmt.name.lexeme.to_string());
        for p in stmt.params.iter() {
            let reg = self.next_register();
            self.allocate_register(p.lexeme.to_string(), reg);
        }
        self.enter_block();
        for s in &stmt.body {
            let chunk = s.accept(self);
            self.add_chunk(chunk);
        }
        self.leave_block();
        let prototype_ix = self.leave_function();
        let result_register = self.next_register();
        return Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::Closure,
                a: result_register,
                b: (prototype_ix >> 8) as u8,
                c: prototype_ix as u8,
            }],
            result_register: result_register,
        };
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
        return Chunk {
            instructions: vec![],
            result_register: self
                .get_var_register(expr.name.clone().unwrap().lexeme.to_string())
                .unwrap(),
        };
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Chunk {
        todo!()
    }

    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Chunk {
        todo!()
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Chunk {
        if let Some(reg) = self.get_var_register(expr.name.lexeme.to_string()) {
            let mut chunk = expr.value.accept(self);
            if !chunk.instructions.is_empty() {
                chunk.instructions.last_mut().unwrap().a = reg;
            }
            chunk.result_register = reg;
            return chunk;
        }
        panic!("Var doesn't exist!");
    }

    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Chunk {
        todo!()
    }

    fn visit_binary_expr(&mut self, expr: &BinaryExpr) -> Chunk {
        let left_chunk = expr.left.accept(self);
        let right_chunk = expr.right.accept(self);
        let opcode = match expr.operator.token {
            Token::Plus => OpCode::Add,
            Token::Less => OpCode::Lt,
            Token::EqualEqual => OpCode::Eq,
            Token::BangEqual => OpCode::Neq,
            Token::Greater => OpCode::Gt,
            Token::GreaterEqual => OpCode::Gte,
            Token::LessEqual => OpCode::Lte,
            Token::Minus => OpCode::Sub,
            Token::Slash => OpCode::Div,
            Token::Mod => OpCode::Mod,
            Token::Star => OpCode::Mul,
            Token::Power => OpCode::Pow,
            _ => unreachable!(),
        };
        let result_register = self.next_register();
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
        expr.expression.accept(self)
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Chunk {
        let val = match expr.value.clone() {
            Literal::Number(n) => Value::Number(NumberValue { n: n }),
            Literal::String(s) => Value::String(StringValue { s: s }),
            Literal::Boolean(b) => Value::Bool(BoolValue { b: b }),
            Literal::Nil => Value::Nil,
        };
        let result_register = self.next_register();
        let constant_ix = self.constants.len() as u16;
        self.constants.push(val);
        let chunk = Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::LoadK,
                a: result_register,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            }],
            result_register: result_register,
        };
        return chunk;
    }

    fn visit_logical_expr(&mut self, expr: &LogicalExpr) -> Chunk {
        let left_chunk = expr.left.accept(self);
        let right_chunk = expr.right.accept(self);
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: left_chunk.result_register,
        };
        chunk
            .instructions
            .append(&mut left_chunk.instructions.clone());
        if expr.operator.token == Token::Or {
            chunk.instructions.push(Instruction {
                opcode: OpCode::Test,
                a: left_chunk.result_register,
                b: left_chunk.result_register,
                c: 1,
            });
            let jump_size = (right_chunk.instructions.len() + 1) as u16;
            chunk.instructions.push(Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jump_size >> 8) as u8,
                c: jump_size as u8,
            });
            chunk
                .instructions
                .append(&mut right_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::Move,
                a: chunk.result_register,
                b: right_chunk.result_register,
                c: 0,
            });
        } else {
            chunk.instructions.push(Instruction {
                opcode: OpCode::Test,
                a: left_chunk.result_register,
                b: left_chunk.result_register,
                c: 0,
            });
            let jump_size = (right_chunk.instructions.len() + 1) as u16;
            chunk.instructions.push(Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jump_size >> 8) as u8,
                c: jump_size as u8,
            });
            chunk
                .instructions
                .append(&mut right_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::Move,
                a: chunk.result_register,
                b: right_chunk.result_register,
                c: 0,
            });
        }
        return chunk;
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Chunk {
        todo!()
    }

    fn visit_unary_expr(&mut self, expr: &UnaryExpr) -> Chunk {
        let mut chunk = expr.right.accept(self);
        let inst = match expr.operator.token {
            Token::Not => Instruction {
                opcode: OpCode::Not,
                a: chunk.result_register,
                b: chunk.result_register,
                c: 0,
            },
            Token::Minus => Instruction {
                opcode: OpCode::Neg,
                a: chunk.result_register,
                b: chunk.result_register,
                c: 0,
            },
            _ => unreachable!(),
        };
        chunk.instructions.push(inst);
        return chunk;
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
