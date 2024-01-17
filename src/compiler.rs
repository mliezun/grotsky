use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::errors::RuntimeErr;
use crate::errors::{ERR_GLOBAL_ALREADY_DEFINED, ERR_UNDEFINED_VAR};
use crate::expr::*;
use crate::instruction::*;
use crate::stmt::*;
use crate::token::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Compiler {
    // call_stack: Vec<CallStack>,
    pub constants: Vec<Literal>,
    pub contexts: Vec<FnContext>,
    pub prototypes: Vec<FnPrototype>,
    pub globals: HashSet<String>,
}

impl Compiler {
    fn add_chunk(&mut self, chunk: Chunk) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.chunks.push(chunk);
    }

    fn allocate_register(&mut self, var_name: String, reg: u8) {
        // TODO: handle what to do here when we're in the global context
        let current_context = self.contexts.last_mut().unwrap();
        let current_block = current_context.blocks.last_mut().unwrap();
        current_block.locals.push(Local {
            var_name: var_name,
            reg: reg,
        });
    }

    fn next_register(&mut self) -> u8 {
        let current_context = self.contexts.last_mut().unwrap();
        if current_context.register_count == 255 {
            panic!("Ran out of registers");
        }
        let reg = current_context.register_count;
        current_context.register_count += 1;
        return reg;
    }

    fn reg_count(&self) -> u8 {
        let current_context = self.contexts.last().unwrap();
        return current_context.register_count;
    }

    fn set_reg_count(&mut self, reg_count: u8) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.register_count = reg_count;
    }

    fn enter_block(&mut self) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.blocks.push(Block { locals: vec![] });
    }

    fn leave_block(&mut self) {
        let current_context = self.contexts.last_mut().unwrap();
        current_context.blocks.pop();
    }

    pub fn enter_function(&mut self, name: String) {
        self.contexts.push(FnContext {
            name: name,
            loop_count: 0,
            register_count: 0,
            chunks: vec![],
            blocks: vec![Block { locals: vec![] }],
            upvalues: vec![],
        });
    }

    pub fn leave_function(&mut self, param_count: usize) -> u16 {
        let current_context = self.contexts.pop().unwrap();
        let prototype_ix = self.prototypes.len();
        let mut instructions: Vec<InstSrc> = current_context
            .chunks
            .iter()
            .map(|c| c.instructions.clone())
            .flatten()
            .collect();
        instructions.push(InstSrc {
            inst: Instruction {
                opcode: OpCode::Return,
                a: 0,
                b: 0,
                c: 0,
            },
            src: None,
        });
        self.prototypes.push(FnPrototype {
            instructions: instructions.iter().map(|i| i.inst.clone()).collect(),
            register_count: current_context.register_count,
            upvalues: current_context.upvalues,
            instruction_data: instructions.iter().map(|i| i.src.clone()).collect(),
            param_count: param_count,
            name: current_context.name,
        });
        return prototype_ix as u16;
    }

    fn result_register(&self) -> u8 {
        let current_context = self.contexts.last().unwrap();
        if current_context.chunks.is_empty() {
            // TODO: why is this needed?
            return 0;
        }
        return current_context.chunks.last().unwrap().result_register;
    }

    fn get_var_register_by_context(&self, context: &FnContext, var_name: &String) -> Option<u8> {
        context
            .blocks
            .iter()
            .map(|b| b.locals.clone())
            .flatten()
            .rev()
            .find_map(|l| {
                // println!(
                //     "Comparing locals: l({:#?}) == {:#?} | reg({})",
                //     l.var_name, var_name, l.reg
                // );
                if l.var_name.eq(var_name) {
                    Some(l.reg)
                } else {
                    None
                }
            })
    }

    fn get_var_register(&self, var_name: &String) -> Option<u8> {
        self.get_var_register_by_context(self.contexts.last().unwrap(), var_name)
    }

    fn get_upvalue(&mut self, var_name: &String) -> Option<u8> {
        // Edge case - when variable cannot be found
        if self.contexts.is_empty() || self.is_global_context() {
            return None;
        }
        let mut saved_context = self.contexts.pop().unwrap();
        let upvalue = self._get_upvalue(&mut saved_context, var_name);
        self.contexts.push(saved_context);
        upvalue
    }

    fn _get_upvalue(&mut self, previous_context: &mut FnContext, var_name: &String) -> Option<u8> {
        // Edge case - when variable cannot be found
        if self.contexts.is_empty() || self.is_global_context() {
            return None;
        }
        let mut context = self.contexts.pop().unwrap();
        if let Some(reg) = self.get_var_register_by_context(&context, var_name) {
            // Add upvalue to previous context
            let upval_ix = previous_context.add_upvalue(reg, true);
            // Restore context
            self.contexts.push(context);
            return Some(upval_ix);
        }
        if let Some(up) = self._get_upvalue(&mut context, var_name) {
            // Add upvalue to previous context
            let upvalue_ix = previous_context.add_upvalue(up, false);
            // Restore context
            self.contexts.push(context);
            return Some(upvalue_ix);
        }
        // Restore context
        self.contexts.push(context);
        return None;
    }

    pub fn is_builtin_var(&self, var_name: String) -> bool {
        return var_name == "io".to_string()
            || var_name == "strings".to_string()
            || var_name == "type".to_string()
            || var_name == "env".to_string()
            || var_name == "import".to_string()
            || var_name == "net".to_string()
            || var_name == "re".to_string();
    }

    pub fn is_global_var(&self, var_name: String) -> bool {
        return self.globals.contains(&var_name);
    }

    pub fn is_global_context(&self) -> bool {
        return self.contexts.len() == 1 && self.contexts.last().unwrap().blocks.len() == 1;
    }

    pub fn compile(&mut self, stmts: Vec<Stmt>) {
        if self.contexts.is_empty() {
            self.contexts.push(FnContext {
                chunks: vec![],
                register_count: 0,
                name: "".to_string(),
                loop_count: 0,
                blocks: vec![Block { locals: vec![] }],
                upvalues: vec![],
            });
        }
        for stmt in stmts {
            let chunk = stmt.accept(self);
            self.add_chunk(chunk);
        }
    }

    pub fn compilation_error(&self, msg: RuntimeErr, token_data: Option<TokenData>) {
        if let Some(tk) = token_data {
            print!(
                "Compilation Error on line {}\n\t{}: {}\n",
                tk.line, msg.msg, tk.lexeme,
            );
        } else {
            print!("Compilation Error\n\t{}\n", msg.msg);
        }
        std::process::exit(0);
    }

    fn access_collection(&mut self, expr: &AccessExpr, access_op: OpCode) -> Chunk {
        // println!("{:#?}", expr);
        let mut chunk = Chunk {
            result_register: self.next_register(),
            instructions: vec![],
        };
        let obj_chunk = expr.object.accept(self);
        let token_data = if expr.second_colon.token != Token::Nil {
            Some(expr.second_colon.clone())
        } else {
            Some(expr.brace.clone())
        };
        chunk
            .instructions
            .append(&mut obj_chunk.instructions.clone());
        if !expr.first.is_empty()
            && expr.first_colon.token == Token::Nil
            && expr.second.is_empty()
            && expr.second_colon.token == Token::Nil
            && expr.third.is_empty()
        {
            let first_chunk = expr.first.accept(self);
            chunk
                .instructions
                .append(&mut first_chunk.instructions.clone());
            if access_op == OpCode::Access {
                chunk.push(
                    Instruction {
                        opcode: access_op,
                        a: chunk.result_register,
                        b: obj_chunk.result_register,
                        c: first_chunk.result_register,
                    },
                    token_data.clone(),
                );
            } else {
                chunk.push(
                    Instruction {
                        opcode: access_op,
                        a: obj_chunk.result_register,
                        b: first_chunk.result_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            }
        } else {
            let list_register = self.next_register();
            chunk.push(
                Instruction {
                    opcode: OpCode::List,
                    a: list_register,
                    b: 0,
                    c: 0,
                },
                token_data.clone(),
            );
            let nil_register = self.next_register();
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadNil,
                    a: nil_register,
                    b: 0,
                    c: 0,
                },
                token_data.clone(),
            );
            let one_register = self.next_register();
            let constant_ix = self.constants.len() as u16;
            self.constants.push(Literal::Number(1.0));
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadK,
                    a: one_register,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                token_data.clone(),
            );
            if !expr.first.is_empty() {
                let first_chunk = expr.first.accept(self);
                chunk.append(&mut first_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: first_chunk.result_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            } else {
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: nil_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            }
            if !expr.second.is_empty() {
                let second_chunk = expr.second.accept(self);
                chunk.append(&mut second_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: second_chunk.result_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            } else {
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: nil_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            }
            if !expr.third.is_empty() {
                let third_chunk = expr.third.accept(self);
                chunk.append(&mut third_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: third_chunk.result_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            } else if expr.second_colon.token == Token::Nil {
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: one_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            } else {
                chunk.push(
                    Instruction {
                        opcode: OpCode::PushList,
                        a: list_register,
                        b: nil_register,
                        c: 0,
                    },
                    token_data.clone(),
                );
            }
            chunk.push(
                Instruction {
                    opcode: OpCode::Slice,
                    a: list_register,
                    b: list_register,
                    c: 0,
                },
                token_data.clone(),
            );
            chunk.push(
                Instruction {
                    opcode: access_op,
                    a: chunk.result_register,
                    b: obj_chunk.result_register,
                    c: list_register,
                },
                token_data.clone(),
            );
        }
        return chunk;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpvalueRef {
    pub is_local: bool,
    pub index: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnPrototype {
    pub instructions: Vec<Instruction>,
    pub register_count: u8,
    pub upvalues: Vec<UpvalueRef>,
    pub instruction_data: Vec<Option<TokenData>>,
    pub param_count: usize,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FnContext {
    pub name: String,
    pub loop_count: u8,
    pub register_count: u8,
    pub chunks: Vec<Chunk>,
    pub blocks: Vec<Block>,
    pub upvalues: Vec<UpvalueRef>,
}

impl FnContext {
    pub fn add_upvalue(&mut self, index: u8, is_local: bool) -> u8 {
        for (i, up) in self.upvalues.iter().enumerate() {
            if up.is_local == is_local && up.index == index {
                return i as u8;
            }
        }

        if self.upvalues.len() == 255 {
            panic!("Too many upvalues");
        }

        self.upvalues.push(UpvalueRef {
            is_local: is_local,
            index: index,
        });
        return (self.upvalues.len() - 1) as u8;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub locals: Vec<Local>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Local {
    pub var_name: String,
    pub reg: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstSrc {
    pub inst: Instruction,
    pub src: Option<TokenData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub instructions: Vec<InstSrc>,
    pub result_register: u8,
}

impl Chunk {
    pub fn push(&mut self, inst: Instruction, src: Option<TokenData>) {
        self.instructions.push(InstSrc { inst, src });
    }

    pub fn append(&mut self, instructions: &mut Vec<InstSrc>) {
        self.instructions.append(instructions);
    }
}

impl StmtVisitor<Chunk> for Compiler {
    fn visit_expr_stmt(&mut self, stmt: &ExprStmt) -> Chunk {
        return stmt.expression.accept(self);
    }

    fn visit_try_catch_stmt(&mut self, stmt: &TryCatchStmt) -> Chunk {
        let mut chunk = Chunk {
            result_register: 0,
            instructions: vec![],
        };

        // Stores the jump to the catch section, needs to be patched
        chunk.push(
            Instruction {
                opcode: OpCode::RegisterTryCatch,
                a: 0,
                b: 0,
                c: 0,
            },
            None,
        );

        let try_body_chunk = stmt.try_body.accept(self);
        chunk.append(&mut try_body_chunk.instructions.clone());
        chunk.push(
            Instruction {
                opcode: OpCode::DeregisterTryCatch,
                a: 0,
                b: 0,
                c: 0,
            },
            None,
        );

        // Jmp to end, needs to be patched
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: 0,
                c: 0,
            },
            None,
        );

        let catch_offset = chunk.instructions.len();
        chunk.instructions[0].inst.b = (catch_offset >> 8) as u8;
        chunk.instructions[0].inst.c = catch_offset as u8;

        self.enter_block();
        let catch_var_name_reg = self.next_register();
        self.allocate_register(stmt.name.lexeme.to_string(), catch_var_name_reg);
        chunk.push(
            Instruction {
                opcode: OpCode::GetExcept,
                a: catch_var_name_reg,
                b: 0,
                c: 0,
            },
            Some(stmt.name.clone()),
        );
        chunk.push(
            Instruction {
                opcode: OpCode::DeregisterTryCatch,
                a: 0,
                b: 0,
                c: 0,
            },
            None,
        );
        let catch_body_chunk = stmt.catch_body.accept(self);
        self.leave_block();
        chunk.append(&mut catch_body_chunk.instructions.clone());

        let jmp_offset = chunk.instructions.len() - catch_offset + 1;
        for inst in &mut chunk.instructions {
            if inst.inst.opcode == OpCode::Jmp
                && inst.inst.a == 0
                && inst.inst.b == 0
                && inst.inst.c == 0
            {
                inst.inst.b = (jmp_offset >> 8) as u8;
                inst.inst.c = jmp_offset as u8;
                break;
            }
        }

        return chunk;
    }

    fn visit_classic_for_stmt(&mut self, stmt: &ClassicForStmt) -> Chunk {
        let mut chunk = Chunk {
            result_register: 0,
            instructions: vec![],
        };
        if let Some(init) = &stmt.initializer {
            let init_chunk = init.accept(self);
            chunk
                .instructions
                .append(&mut init_chunk.instructions.clone());
        }
        let cond_chunk = stmt.condition.accept(self);
        let mut body_chunk = stmt.body.accept(self);
        let inc_chunk = stmt.increment.accept(self);
        body_chunk
            .instructions
            .append(&mut inc_chunk.instructions.clone());
        chunk
            .instructions
            .append(&mut cond_chunk.instructions.clone());
        chunk.push(
            Instruction {
                opcode: OpCode::Test,
                a: cond_chunk.result_register,
                b: cond_chunk.result_register,
                c: 0,
            },
            Some(stmt.keyword.clone()),
        );
        let jump_size = (body_chunk.instructions.len() + 2) as i16;
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jump_size >> 8) as u8,
                c: jump_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        chunk
            .instructions
            .append(&mut body_chunk.instructions.clone());
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (loop_size >> 8) as u8,
                c: loop_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            } else if inst.inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            }
        }
        return chunk;
    }

    fn visit_enhanced_for_stmt(&mut self, stmt: &EnhancedForStmt) -> Chunk {
        let counter_reg = self.next_register();
        let length_reg = self.next_register();
        let cond_reg = self.next_register();
        let element_reg = self.next_register();
        let constant_ix = self.constants.len() as u16;
        self.constants.push(Literal::Number(0.0));
        let mut chunk = Chunk {
            result_register: 0,
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::LoadK,
                    a: counter_reg,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                src: Some(stmt.keyword.clone()),
            }],
        };
        let collection_chunk = stmt.collection.accept(self);
        let cond_chunk = Chunk {
            result_register: cond_reg,
            instructions: vec![
                InstSrc {
                    inst: Instruction {
                        opcode: OpCode::Length,
                        a: length_reg,
                        b: collection_chunk.result_register,
                        c: 0,
                    },
                    src: Some(stmt.keyword.clone()),
                },
                InstSrc {
                    inst: Instruction {
                        opcode: OpCode::Lt,
                        a: cond_reg,
                        b: counter_reg,
                        c: length_reg,
                    },
                    src: Some(stmt.keyword.clone()),
                },
            ],
        };
        let mut body_chunk = Chunk {
            result_register: 0,
            instructions: vec![],
        };
        if stmt.identifiers.len() > 1 {
            body_chunk.push(
                Instruction {
                    opcode: OpCode::GetIter,
                    a: element_reg,
                    b: collection_chunk.result_register,
                    c: counter_reg,
                },
                Some(stmt.keyword.clone()),
            );
            for (i, tk) in stmt.identifiers.iter().enumerate() {
                let var_reg = self.next_register();
                self.allocate_register(tk.lexeme.to_string(), var_reg);
                body_chunk.push(
                    Instruction {
                        opcode: OpCode::GetIteri,
                        a: var_reg,
                        b: element_reg,
                        c: i as u8,
                    },
                    Some(stmt.keyword.clone()),
                )
            }
        } else {
            let tk = stmt.identifiers.first().unwrap();
            self.allocate_register(tk.lexeme.to_string(), element_reg);
            body_chunk.push(
                Instruction {
                    opcode: OpCode::GetIterk,
                    a: element_reg,
                    b: collection_chunk.result_register,
                    c: counter_reg,
                },
                Some(stmt.keyword.clone()),
            );
        }
        let loop_chunk = stmt.body.accept(self);
        body_chunk.result_register = loop_chunk.result_register;
        body_chunk.append(&mut loop_chunk.instructions.clone());
        body_chunk.push(
            Instruction {
                opcode: OpCode::Addi,
                a: counter_reg,
                b: counter_reg,
                c: 1,
            },
            Some(stmt.keyword.clone()),
        );

        // Build final chunk
        // 1. Collection chunk: evaluate collection and have value available to use
        // 2. Condition chunk: check that we haven't reached the end of the collection
        // 3. Body: body of the loop, handle break and continue
        chunk.append(&mut collection_chunk.instructions.clone());
        chunk.append(&mut cond_chunk.instructions.clone());
        chunk.push(
            Instruction {
                opcode: OpCode::Test,
                a: cond_chunk.result_register,
                b: cond_chunk.result_register,
                c: 0,
            },
            Some(stmt.keyword.clone()),
        );
        let jump_size = (body_chunk.instructions.len() + 2) as i16;
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jump_size >> 8) as u8,
                c: jump_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        chunk.append(&mut body_chunk.instructions.clone());
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (loop_size >> 8) as u8,
                c: loop_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            } else if inst.inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            }
        }
        return chunk;
    }

    fn visit_let_stmt(&mut self, stmt: &LetStmt) -> Chunk {
        if self.is_global_context() {
            if self.globals.contains(&stmt.name.lexeme.to_string()) {
                self.compilation_error(ERR_GLOBAL_ALREADY_DEFINED, Some(stmt.name.clone()));
                unreachable!();
            }
            let mut chunk = Chunk {
                result_register: 0,
                instructions: vec![],
            };
            self.globals.insert(stmt.name.lexeme.to_string());
            let constant_ix = self.constants.len() as u16;
            self.constants
                .push(Literal::String(stmt.name.lexeme.to_string()));
            if let Some(init) = &stmt.initializer {
                let init_chunk = init.accept(self);
                chunk.result_register = init_chunk.result_register;
                chunk.append(&mut init_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::SetGlobal,
                        a: init_chunk.result_register,
                        b: (constant_ix >> 8) as u8,
                        c: constant_ix as u8,
                    },
                    Some(stmt.name.clone()),
                );
            } else {
                let reg = self.next_register();
                chunk.push(
                    Instruction {
                        opcode: OpCode::LoadNil,
                        a: reg,
                        b: 0,
                        c: 0,
                    },
                    Some(stmt.name.clone()),
                );
                chunk.push(
                    Instruction {
                        opcode: OpCode::SetGlobal,
                        a: reg,
                        b: (constant_ix >> 8) as u8,
                        c: constant_ix as u8,
                    },
                    Some(stmt.name.clone()),
                );
            }
            return chunk;
        } else {
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
        chunk.push(
            Instruction {
                opcode: OpCode::Test,
                a: cond_chunk.result_register,
                b: cond_chunk.result_register,
                c: 0,
            },
            Some(stmt.keyword.clone()),
        );
        let jump_size = (body_chunk.instructions.len() + 2) as i16;
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jump_size >> 8) as u8,
                c: jump_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        chunk
            .instructions
            .append(&mut body_chunk.instructions.clone());
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (loop_size >> 8) as u8,
                c: loop_size as u8,
            },
            Some(stmt.keyword.clone()),
        );
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            } else if inst.inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.inst.a = 0;
                inst.inst.b = (jump_offset >> 8) as u8;
                inst.inst.c = jump_offset as u8;
            }
        }
        return chunk;
    }

    fn visit_return_stmt(&mut self, stmt: &ReturnStmt) -> Chunk {
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: 0,
        };
        if let Some(val) = &stmt.value {
            let val_chunk: Chunk = val.accept(self);
            chunk
                .instructions
                .append(&mut val_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::Return,
                    a: val_chunk.result_register,
                    b: val_chunk.result_register + 2,
                    c: 0,
                },
                Some(stmt.keyword.clone()),
            );
        } else {
            chunk.push(
                Instruction {
                    opcode: OpCode::Return,
                    a: 0,
                    b: 0,
                    c: 0,
                },
                Some(stmt.keyword.clone()),
            );
        }
        return chunk;
    }

    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> Chunk {
        return Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::Jmp,
                    a: JMP_BREAK,
                    b: 0,
                    c: 0,
                },
                src: Some(stmt.keyword.clone()),
            }],
            result_register: 0,
        };
    }

    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> Chunk {
        return Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::Jmp,
                    a: JMP_CONTINUE,
                    b: 0,
                    c: 0,
                },
                src: Some(stmt.keyword.clone()),
            }],
            result_register: 0,
        };
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Chunk {
        let mut if_cond_chunk = stmt.condition.accept(self);
        let mut if_body: Vec<InstSrc> = stmt
            .then_branch
            .iter()
            .map(|c| c.accept(self).instructions)
            .flatten()
            .collect();
        let mut elifs_cond_chunks: Vec<Chunk> = vec![];
        let mut elifs_body: Vec<Vec<InstSrc>> = vec![];
        for e in &stmt.elifs {
            let elif_cond_chunk = e.condition.accept(self);
            let elif_body: Vec<InstSrc> = e
                .then_branch
                .iter()
                .map(|c| c.accept(self).instructions)
                .flatten()
                .collect();
            elifs_cond_chunks.push(elif_cond_chunk);
            elifs_body.push(elif_body);
        }
        let else_body: Vec<InstSrc> = stmt
            .else_branch
            .iter()
            .map(|c| c.accept(self).instructions)
            .flatten()
            .collect();

        // Add Test and Jump instructions
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: 0,
        };
        chunk.append(&mut if_cond_chunk.instructions);
        chunk.push(
            Instruction {
                opcode: OpCode::Test,
                a: if_cond_chunk.result_register,
                b: if_cond_chunk.result_register,
                c: 0,
            },
            Some(stmt.keyword.clone()),
        );
        // Body length + 2 jmps
        let mut jmp_offset = (if_body.len() + 2) as u16;
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jmp_offset >> 8) as u8,
                c: jmp_offset as u8,
            },
            Some(stmt.keyword.clone()),
        );
        chunk.append(&mut if_body);
        // Jump to end, needs to be patched after adding all insts to chunk
        chunk.push(
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: 0,
                c: 0,
            },
            Some(stmt.keyword.clone()),
        );
        for (i, elif_cond_chunk) in elifs_cond_chunks.iter().enumerate() {
            chunk
                .instructions
                .append(&mut elif_cond_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::Test,
                    a: elif_cond_chunk.result_register,
                    b: elif_cond_chunk.result_register,
                    c: 0,
                },
                None,
            );
            let elif_body = &elifs_body[i];
            // Body length + 2 jmps
            jmp_offset = (elif_body.len() + 2) as u16;
            chunk.push(
                Instruction {
                    opcode: OpCode::Jmp,
                    a: 0,
                    b: (jmp_offset >> 8) as u8,
                    c: jmp_offset as u8,
                },
                None,
            );
            chunk.append(&mut elif_body.clone());
            // Jump to end, needs to be patched after adding all insts to chunk
            chunk.push(
                Instruction {
                    opcode: OpCode::Jmp,
                    a: 0,
                    b: 0,
                    c: 0,
                },
                None,
            );
        }
        chunk.append(&mut else_body.clone());
        let chunk_size = chunk.instructions.len();
        for (offset, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.inst.opcode == OpCode::Jmp
                && inst.inst.a == 0
                && inst.inst.b == 0
                && inst.inst.c == 0
            {
                // Jump to the next instruction after this chunk
                let jmp_offset = chunk_size - offset;
                inst.inst.b = (jmp_offset >> 8) as u8;
                inst.inst.c = jmp_offset as u8;
            }
        }
        return chunk;
    }

    fn visit_fn_stmt(&mut self, stmt: &FnStmt) -> Chunk {
        let result_register: u8 = self.next_register();
        if self.is_global_context() {
            self.globals.insert(stmt.name.lexeme.to_string());
        }
        self.enter_function(stmt.name.lexeme.to_string());
        // Register name inside function
        let self_fn_name_reg = self.next_register();
        self.allocate_register(stmt.name.lexeme.to_string(), self_fn_name_reg);
        self.add_chunk(Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::GetCurrentFunc,
                    a: self_fn_name_reg,
                    b: 0,
                    c: 0,
                },
                src: Some(stmt.name.clone()),
            }],
            result_register: self_fn_name_reg,
        });
        // Register name for input arguments
        for p in stmt.params.iter() {
            let reg = self.next_register();
            self.allocate_register(p.lexeme.to_string(), reg);
        }
        self.enter_block();
        for s in &stmt.body {
            let mut chunk = s.accept(self);
            if stmt.body.len() == 1 {
                chunk.push(
                    Instruction {
                        opcode: OpCode::Return,
                        a: chunk.result_register,
                        b: chunk.result_register + 2,
                        c: 0,
                    },
                    chunk.instructions.last().unwrap().src.clone(),
                );
            }
            self.add_chunk(chunk);
        }
        self.leave_block();
        let prototype_ix = self.leave_function(stmt.params.len());

        let mut chunk = Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::Closure,
                    a: result_register,
                    b: (prototype_ix >> 8) as u8,
                    c: prototype_ix as u8,
                },
                src: Some(stmt.name.clone()),
            }],
            result_register: result_register,
        };
        if self.is_global_context() {
            let constant_ix = self.constants.len();
            self.constants
                .push(Literal::String(stmt.name.lexeme.to_string()));
            chunk.push(
                Instruction {
                    opcode: OpCode::SetGlobal,
                    a: chunk.result_register,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                Some(stmt.name.clone()),
            );
        } else {
            self.allocate_register(stmt.name.lexeme.to_string(), chunk.result_register);
        }
        return chunk;
    }

    fn visit_class_stmt(&mut self, stmt: &ClassStmt) -> Chunk {
        let result_register: u8 = self.next_register();
        if self.is_global_context() {
            self.globals
                .insert(stmt.name.clone().unwrap().lexeme.to_string());
        }
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: result_register,
        };
        let tmp_reg = self.next_register();
        let constant_ix = self.constants.len() as u16;
        self.constants.push(Literal::String(
            stmt.name.clone().unwrap().lexeme.to_string(),
        ));
        chunk.push(
            Instruction {
                opcode: OpCode::LoadK,
                a: tmp_reg,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            },
            stmt.name.clone(),
        );
        let superclass_reg = if let Some(superclass) = &stmt.superclass {
            let var_chunk = self.visit_variable_expr(superclass);
            chunk
                .instructions
                .append(&mut var_chunk.instructions.clone());
            var_chunk.result_register
        } else {
            let nil_reg = self.next_register();
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadNil,
                    a: nil_reg,
                    b: 0,
                    c: 0,
                },
                stmt.name.clone(),
            );
            nil_reg
        };
        chunk.push(
            Instruction {
                opcode: OpCode::Class,
                a: chunk.result_register,
                b: tmp_reg,
                c: superclass_reg,
            },
            stmt.name.clone(),
        );
        self.enter_block();
        for met in &stmt.methods {
            let met_chunk = self.visit_fn_stmt(met);
            chunk
                .instructions
                .append(&mut met_chunk.instructions.clone());
            let constant_ix = self.constants.len() as u16;
            self.constants
                .push(Literal::String(met.name.lexeme.to_string()));
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadK,
                    a: tmp_reg,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                stmt.name.clone(),
            );
            chunk.push(
                Instruction {
                    opcode: OpCode::ClassMeth,
                    a: chunk.result_register,
                    b: tmp_reg,
                    c: met_chunk.result_register,
                },
                stmt.name.clone(),
            );
        }
        for met in &stmt.static_methods {
            let met_chunk = self.visit_fn_stmt(met);
            chunk.append(&mut met_chunk.instructions.clone());
            let constant_ix = self.constants.len() as u16;
            self.constants
                .push(Literal::String(met.name.lexeme.to_string()));
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadK,
                    a: tmp_reg,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                stmt.name.clone(),
            );
            chunk.push(
                Instruction {
                    opcode: OpCode::ClassStMeth,
                    a: chunk.result_register,
                    b: tmp_reg,
                    c: met_chunk.result_register,
                },
                stmt.name.clone(),
            );
        }
        self.leave_block();
        if self.is_global_context() {
            let constant_ix = self.constants.len();
            self.constants.push(Literal::String(
                stmt.name.clone().unwrap().lexeme.to_string(),
            ));
            chunk.push(
                Instruction {
                    opcode: OpCode::SetGlobal,
                    a: chunk.result_register,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                stmt.name.clone(),
            );
        } else {
            self.allocate_register(
                stmt.name.clone().unwrap().lexeme.to_string(),
                result_register,
            );
        }
        return chunk;
    }
}

impl ExprVisitor<Chunk> for Compiler {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> Chunk {
        let result_register: u8 = self.next_register();
        self.enter_function("".to_string());
        // FIXME: We need to skip first register because in fn_stmt we use
        // it for storing the function
        self.next_register();
        for p in expr.params.iter() {
            let reg = self.next_register();
            self.allocate_register(p.lexeme.to_string(), reg);
        }
        self.enter_block();
        for s in &expr.body {
            let mut chunk = s.accept(self);
            if expr.body.len() == 1 {
                chunk.push(
                    Instruction {
                        opcode: OpCode::Return,
                        a: chunk.result_register,
                        b: chunk.result_register + 2,
                        c: 0,
                    },
                    chunk.instructions.last().unwrap().src.clone(),
                );
            }
            self.add_chunk(chunk);
        }
        self.leave_block();
        let prototype_ix = self.leave_function(expr.params.len());
        let token_data = expr.params.get(0).and_then(|p| Some(p.clone()));
        return Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::Closure,
                    a: result_register,
                    b: (prototype_ix >> 8) as u8,
                    c: prototype_ix as u8,
                },
                src: token_data.clone(),
            }],
            result_register: result_register,
        };
    }

    fn visit_variable_expr(&mut self, expr: &VarExpr) -> Chunk {
        let var_name = expr.name.clone().unwrap().lexeme.to_string();
        if let Some(reg) = self.get_var_register(&var_name) {
            return Chunk {
                instructions: vec![],
                result_register: reg,
            };
        } else if let Some(upvalue_ix) = self.get_upvalue(&var_name) {
            let reg = self.next_register();
            return Chunk {
                instructions: vec![InstSrc {
                    inst: Instruction {
                        opcode: OpCode::GetUpval,
                        a: reg,
                        b: upvalue_ix,
                        c: 0,
                    },
                    src: expr.name.clone(),
                }],
                result_register: reg,
            };
        } else if self.is_builtin_var(var_name.clone()) {
            let reg = self.next_register();
            let constant_ix = self.constants.len();
            self.constants.push(Literal::String(var_name.clone()));
            return Chunk {
                instructions: vec![InstSrc {
                    inst: Instruction {
                        opcode: OpCode::GetBuiltin,
                        a: reg,
                        b: (constant_ix >> 8) as u8,
                        c: constant_ix as u8,
                    },
                    src: expr.name.clone(),
                }],
                result_register: reg,
            };
        } else if self.is_global_var(var_name.clone()) {
            let reg = self.next_register();
            let constant_ix = self.constants.len();
            self.constants.push(Literal::String(var_name.clone()));
            return Chunk {
                instructions: vec![InstSrc {
                    inst: Instruction {
                        opcode: OpCode::GetGlobal,
                        a: reg,
                        b: (constant_ix >> 8) as u8,
                        c: constant_ix as u8,
                    },
                    src: expr.name.clone(),
                }],
                result_register: reg,
            };
        }
        self.compilation_error(ERR_UNDEFINED_VAR, expr.name.clone());
        unreachable!();
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Chunk {
        let reg = self.next_register();
        let mut chunk = Chunk {
            result_register: reg,
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::List,
                    a: reg,
                    b: 0,
                    c: 0,
                },
                src: Some(expr.brace.clone()),
            }],
        };
        let beggining_reg_count = self.reg_count();
        let mut max_reg_count = self.reg_count();
        for e in &expr.elements {
            let el_chunk = e.accept(self);
            if self.reg_count() > max_reg_count {
                max_reg_count = self.reg_count();
            }
            self.set_reg_count(beggining_reg_count);
            chunk.append(&mut el_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::PushList,
                    a: reg,
                    b: el_chunk.result_register,
                    c: 0,
                },
                Some(expr.brace.clone()),
            );
        }
        self.set_reg_count(max_reg_count);
        return chunk;
    }

    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Chunk {
        let reg = self.next_register();
        let mut chunk = Chunk {
            result_register: reg,
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::Dict,
                    a: reg,
                    b: 0,
                    c: 0,
                },
                src: Some(expr.curly_brace.clone()),
            }],
        };
        for i in 0..(expr.elements.len() / 2) {
            let k = &expr.elements[i * 2];
            let val = &expr.elements[i * 2 + 1];
            let k_chunk = k.accept(self);
            let val_chunk = val.accept(self);
            chunk.append(&mut k_chunk.instructions.clone());
            chunk.append(&mut val_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::PushDict,
                    a: reg,
                    b: k_chunk.result_register,
                    c: val_chunk.result_register,
                },
                Some(expr.curly_brace.clone()),
            );
        }
        return chunk;
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Chunk {
        let token_data = Some(expr.name.clone());
        if let Some(reg) = self.get_var_register(&expr.name.lexeme.to_string()) {
            let mut chunk = expr.value.accept(self);

            if let Some(access) = &expr.access {
                let obj_chunk = access.object.accept(self);
                let access_chunk = access.first.accept(self);
                chunk.append(&mut obj_chunk.instructions.clone());
                chunk.append(&mut access_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::Set,
                        a: obj_chunk.result_register,
                        b: access_chunk.result_register,
                        c: chunk.result_register,
                    },
                    token_data.clone(),
                );
            } else {
                if !chunk.instructions.is_empty() {
                    let inst = chunk.instructions.last_mut().unwrap();
                    if inst.inst.opcode == OpCode::Call {
                        inst.inst.c = reg + 1;
                    } else {
                        inst.inst.a = reg;
                    }
                }
                chunk.result_register = reg;
            }

            return chunk;
        } else if let Some(upvalue_ix) = self.get_upvalue(&expr.name.lexeme.to_string()) {
            let reg = self.next_register();
            let mut chunk = expr.value.accept(self);

            if let Some(access) = &expr.access {
                let obj_chunk = access.object.accept(self);
                let access_chunk = access.first.accept(self);
                chunk.append(&mut obj_chunk.instructions.clone());
                chunk.append(&mut access_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::Set,
                        a: obj_chunk.result_register,
                        b: access_chunk.result_register,
                        c: chunk.result_register,
                    },
                    token_data.clone(),
                );
            } else {
                if !chunk.instructions.is_empty() {
                    let inst = chunk.instructions.last_mut().unwrap();
                    if inst.inst.opcode == OpCode::Call {
                        inst.inst.c = reg + 1;
                    } else {
                        inst.inst.a = reg;
                    }
                }
                chunk.push(
                    Instruction {
                        opcode: OpCode::SetUpval,
                        a: reg,
                        b: upvalue_ix,
                        c: 0,
                    },
                    token_data.clone(),
                );
                chunk.result_register = reg;
            }

            return chunk;
        } else if self.is_global_var(expr.name.lexeme.to_string()) {
            let reg = self.next_register();
            let mut chunk = expr.value.accept(self);
            let constant_ix = self.constants.len();
            self.constants
                .push(Literal::String(expr.name.lexeme.to_string()));

            if let Some(access) = &expr.access {
                let obj_chunk = access.object.accept(self);
                let access_chunk = access.first.accept(self);
                chunk.append(&mut obj_chunk.instructions.clone());
                chunk.append(&mut access_chunk.instructions.clone());
                chunk.push(
                    Instruction {
                        opcode: OpCode::Set,
                        a: obj_chunk.result_register,
                        b: access_chunk.result_register,
                        c: chunk.result_register,
                    },
                    token_data.clone(),
                );
            } else {
                if !chunk.instructions.is_empty() {
                    let inst = chunk.instructions.last_mut().unwrap();
                    if inst.inst.opcode == OpCode::Call {
                        inst.inst.c = reg + 1;
                    } else {
                        inst.inst.a = reg;
                    }
                }
                chunk.push(
                    Instruction {
                        opcode: OpCode::SetGlobal,
                        a: reg,
                        b: (constant_ix >> 8) as u8,
                        c: constant_ix as u8,
                    },
                    token_data.clone(),
                );
                chunk.result_register = reg;
            }

            return chunk;
        }
        self.compilation_error(ERR_UNDEFINED_VAR, Some(expr.name.clone()));
        unreachable!();
    }

    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Chunk {
        return self.access_collection(expr, OpCode::Access);
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
        chunk.push(
            Instruction {
                opcode: opcode,
                a: result_register,
                b: left_chunk.result_register,
                c: right_chunk.result_register,
            },
            Some(expr.operator.clone()),
        );
        return chunk;
    }

    fn visit_call_expr(&mut self, expr: &CallExpr) -> Chunk {
        let mut chunk = expr.callee.accept(self);
        // println!("Callee: {:#?}", chunk);
        let fn_register = self.next_register();
        // println!("fn_register: {:#?}", fn_register);
        let token_data = Some(expr.paren.clone());
        chunk.push(
            Instruction {
                opcode: OpCode::Move,
                a: fn_register,
                b: chunk.result_register,
                c: 0,
            },
            token_data.clone(),
        );
        let start_reg = self.next_register();
        for _ in 1..(expr.arguments.len()) {
            self.next_register();
        }
        for (i, expr) in (&expr.arguments).iter().enumerate() {
            let arg_chunk = expr.accept(self);
            chunk
                .instructions
                .append(&mut arg_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::Move,
                    a: start_reg + i as u8,
                    b: arg_chunk.result_register,
                    c: 0,
                },
                None,
            );
        }
        chunk.result_register = self.next_register();
        chunk.push(
            Instruction {
                opcode: OpCode::Call,
                a: fn_register,
                b: expr.arguments.len() as u8 + 1,
                c: chunk.result_register + 1,
            },
            token_data.clone(),
        );
        // println!("Call chunk: {:#?}", chunk);
        return chunk;
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> Chunk {
        let mut chunk = expr.object.accept(self);
        let result_register = self.next_register();
        let constant_ix = self.constants.len() as u16;
        let val = Literal::String(expr.name.lexeme.to_string());
        self.constants.push(val);
        chunk.push(
            Instruction {
                opcode: OpCode::LoadK,
                a: result_register,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            },
            Some(expr.name.clone()),
        );
        chunk.push(
            Instruction {
                opcode: OpCode::GetObj,
                a: result_register,
                b: chunk.result_register,
                c: result_register,
            },
            Some(expr.name.clone()),
        );
        chunk.result_register = result_register;
        return chunk;
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> Chunk {
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: 0,
        };
        let value_chunk = expr.value.accept(self);
        chunk
            .instructions
            .append(&mut value_chunk.instructions.clone());
        if let Some(access) = &expr.access {
            let mut access_chunk = self.access_collection(access, OpCode::Set);
            access_chunk.instructions.last_mut().unwrap().inst.c = value_chunk.result_register;
            chunk.append(&mut access_chunk.instructions);
            chunk.result_register = access_chunk.result_register;
        } else {
            let obj_chunk = expr.object.accept(self);
            chunk.append(&mut obj_chunk.instructions.clone());
            chunk.result_register = obj_chunk.result_register;
            let tmp_register = self.next_register();
            let constant_ix = self.constants.len() as u16;
            let val = Literal::String(expr.name.lexeme.to_string());
            self.constants.push(val);
            chunk.push(
                Instruction {
                    opcode: OpCode::LoadK,
                    a: tmp_register,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                Some(expr.name.clone()),
            );
            chunk.push(
                Instruction {
                    opcode: OpCode::SetObj,
                    a: chunk.result_register,
                    b: tmp_register,
                    c: value_chunk.result_register,
                },
                Some(expr.name.clone()),
            );
        }
        return chunk;
    }

    fn visit_super_expr(&mut self, expr: &SuperExpr) -> Chunk {
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: self.next_register(),
        };
        let constant_ix = self.constants.len() as u16;
        self.constants
            .push(Literal::String(expr.method.lexeme.to_string()));
        chunk.push(
            Instruction {
                opcode: OpCode::LoadK,
                a: chunk.result_register,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            },
            Some(expr.method.clone()),
        );
        chunk.push(
            Instruction {
                opcode: OpCode::Super,
                a: chunk.result_register,
                b: chunk.result_register,
                c: 0,
            },
            Some(expr.method.clone()),
        );
        return chunk;
    }

    fn visit_grouping_expr(&mut self, expr: &GroupingExpr) -> Chunk {
        expr.expression.accept(self)
    }

    fn visit_literal_expr(&mut self, expr: &LiteralExpr) -> Chunk {
        let result_register = self.next_register();
        let constant_ix = self.constants.len() as u16;
        self.constants.push(expr.value.clone());
        let chunk = Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::LoadK,
                    a: result_register,
                    b: (constant_ix >> 8) as u8,
                    c: constant_ix as u8,
                },
                src: None,
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
            chunk.push(
                Instruction {
                    opcode: OpCode::Test,
                    a: left_chunk.result_register,
                    b: left_chunk.result_register,
                    c: 1,
                },
                Some(expr.operator.clone()),
            );
            let jump_size = (right_chunk.instructions.len() + 2) as u16;
            chunk.push(
                Instruction {
                    opcode: OpCode::Jmp,
                    a: 0,
                    b: (jump_size >> 8) as u8,
                    c: jump_size as u8,
                },
                Some(expr.operator.clone()),
            );
            chunk.append(&mut right_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::Move,
                    a: chunk.result_register,
                    b: right_chunk.result_register,
                    c: 0,
                },
                Some(expr.operator.clone()),
            );
        } else {
            chunk.push(
                Instruction {
                    opcode: OpCode::Test,
                    a: left_chunk.result_register,
                    b: left_chunk.result_register,
                    c: 0,
                },
                Some(expr.operator.clone()),
            );
            let jump_size = (right_chunk.instructions.len() + 2) as u16;
            chunk.push(
                Instruction {
                    opcode: OpCode::Jmp,
                    a: 0,
                    b: (jump_size >> 8) as u8,
                    c: jump_size as u8,
                },
                Some(expr.operator.clone()),
            );
            chunk.append(&mut right_chunk.instructions.clone());
            chunk.push(
                Instruction {
                    opcode: OpCode::Move,
                    a: chunk.result_register,
                    b: right_chunk.result_register,
                    c: 0,
                },
                Some(expr.operator.clone()),
            );
        }
        return chunk;
    }

    fn visit_this_expr(&mut self, expr: &ThisExpr) -> Chunk {
        let reg = self.next_register();
        Chunk {
            instructions: vec![InstSrc {
                inst: Instruction {
                    opcode: OpCode::This,
                    a: reg,
                    b: 0,
                    c: 0,
                },
                src: Some(expr.keyword.clone()),
            }],
            result_register: reg,
        }
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
        chunk.push(inst, Some(expr.operator.clone()));
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
            Expr::Empty => Chunk {
                instructions: vec![],
                result_register: 0,
            },
        }
    }
}
