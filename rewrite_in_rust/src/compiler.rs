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
            upvalues: vec![],
        });
    }

    fn leave_function(&mut self) -> u16 {
        let current_context = self.contexts.pop().unwrap();
        let prototype_ix = self.prototypes.len();
        let mut instructions: Vec<Instruction> = current_context
            .chunks
            .iter()
            .map(|c| c.instructions.clone())
            .flatten()
            .collect();
        instructions.push(Instruction {
            opcode: OpCode::Return,
            a: 0,
            b: 0,
            c: 0,
        });
        self.prototypes.push(FnPrototype {
            instructions: instructions,
            register_count: current_context.register_count,
            upvalues: current_context.upvalues,
        });
        return prototype_ix as u16;
    }

    fn result_register(&self) -> u8 {
        let current_context = self.contexts.last().unwrap();
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

    fn get_upvalue(&self, var_name: &String) -> Option<UpvalueRef> {
        for (depth, context) in self.contexts.iter().rev().skip(1).enumerate() {
            // println!("Looking upvalue {} {}", var_name, depth);
            if let Some(reg) = self.get_var_register_by_context(context, var_name) {
                return Some(UpvalueRef {
                    depth: depth as u8,
                    register: reg,
                });
            }
        }
        return None;
    }

    pub fn add_upvalue(&mut self, up_ref: UpvalueRef) -> u8 {
        self.contexts.last_mut().unwrap().upvalues.push(up_ref);
        return (self.contexts.last().unwrap().upvalues.len() - 1) as u8;
    }

    pub fn compile(&mut self, stmts: Vec<Stmt>) {
        let io_reg = self.next_register();
        self.allocate_register("io".to_string(), io_reg);
        for stmt in stmts {
            let chunk = stmt.accept(self);
            self.add_chunk(chunk);
        }
    }
}

#[derive(Debug, Clone)]
pub struct UpvalueRef {
    pub depth: u8,
    pub register: u8,
}

#[derive(Debug, Clone)]
pub struct FnPrototype {
    pub instructions: Vec<Instruction>,
    pub register_count: u8,
    pub upvalues: Vec<UpvalueRef>,
}

#[derive(Debug)]
pub struct FnContext {
    pub name: String,
    pub loop_count: u8,
    pub register_count: u8,
    pub chunks: Vec<Chunk>,
    pub blocks: Vec<Block>,
    pub upvalues: Vec<UpvalueRef>,
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
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (loop_size >> 8) as u8,
            c: loop_size as u8,
        });
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
            } else if inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
            }
        }
        return chunk;
    }

    fn visit_enhanced_for_stmt(&mut self, stmt: &EnhancedForStmt) -> Chunk {
        let counter_reg = self.next_register();
        let mut chunk = Chunk {
            result_register: 0,
            instructions: vec![],
        };
        let cond_chunk = Chunk {
            result_register: counter_reg,
            instructions: vec![Instruction {
                opcode: OpCode::Lt,
                a: counter_reg,
                b: 0,
                c: 0,
            }],
        };
        let mut body_chunk = stmt.body.accept(self);
        let inc_chunk = Chunk {
            result_register: counter_reg,
            instructions: vec![Instruction {
                opcode: OpCode::Addi,
                a: counter_reg,
                b: counter_reg,
                c: 1,
            }],
        };
        body_chunk
            .instructions
            .append(&mut inc_chunk.instructions.clone());
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
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (loop_size >> 8) as u8,
            c: loop_size as u8,
        });
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
            } else if inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
            }
        }
        return chunk;
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
        let loop_size =
            -((body_chunk.instructions.len() + cond_chunk.instructions.len() + 2) as i16);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (loop_size >> 8) as u8,
            c: loop_size as u8,
        });
        // Patch continue and break
        let chunk_size = chunk.instructions.len();
        for (i, inst) in chunk.instructions.iter_mut().enumerate() {
            if inst.is_continue() {
                let jump_offset = -(i as i64);
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
            } else if inst.is_break() {
                let jump_offset = chunk_size - i;
                inst.a = 0;
                inst.b = (jump_offset >> 8) as u8;
                inst.c = jump_offset as u8;
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
            chunk.instructions.push(Instruction {
                opcode: OpCode::Return,
                a: val_chunk.result_register,
                b: val_chunk.result_register + 2,
                c: 0,
            });
        } else {
            chunk.instructions.push(Instruction {
                opcode: OpCode::Return,
                a: 0,
                b: 0,
                c: 0,
            });
        }
        return chunk;
    }

    fn visit_break_stmt(&mut self, stmt: &BreakStmt) -> Chunk {
        return Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::Jmp,
                a: JMP_BREAK,
                b: 0,
                c: 0,
            }],
            result_register: 0,
        };
    }

    fn visit_continue_stmt(&mut self, stmt: &ContinueStmt) -> Chunk {
        return Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::Jmp,
                a: JMP_CONTINUE,
                b: 0,
                c: 0,
            }],
            result_register: 0,
        };
    }

    fn visit_if_stmt(&mut self, stmt: &IfStmt) -> Chunk {
        let extra_jmp_insts: usize = 3;
        let mut total_body_size: usize = 0;
        let mut if_cond_chunk = stmt.condition.accept(self);
        let mut if_body: Vec<Instruction> = stmt
            .then_branch
            .iter()
            .map(|c| c.accept(self).instructions)
            .flatten()
            .collect();
        total_body_size += if_cond_chunk.instructions.len();
        total_body_size += if_body.len();
        total_body_size += extra_jmp_insts;
        let mut elifs_cond_chunks: Vec<Chunk> = vec![];
        let mut elifs_body: Vec<Vec<Instruction>> = vec![];
        for e in &stmt.elifs {
            let elif_cond_chunk = e.condition.accept(self);
            let elif_body: Vec<Instruction> = e
                .then_branch
                .iter()
                .map(|c| c.accept(self).instructions)
                .flatten()
                .collect();
            total_body_size += elif_cond_chunk.instructions.len();
            total_body_size += elifs_body.len();
            total_body_size += extra_jmp_insts;
            elifs_cond_chunks.push(elif_cond_chunk);
            elifs_body.push(elif_body);
        }
        let else_body: Vec<Instruction> = stmt
            .else_branch
            .iter()
            .map(|c| c.accept(self).instructions)
            .flatten()
            .collect();
        total_body_size += else_body.len();

        // Add Test and Jump instructions
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: 0,
        };
        chunk.instructions.append(&mut if_cond_chunk.instructions);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Test,
            a: if_cond_chunk.result_register,
            b: if_cond_chunk.result_register,
            c: 0,
        });
        let mut jmp_offset = (if_body.len() + 2) as u16;
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (jmp_offset >> 8) as u8,
            c: jmp_offset as u8,
        });
        chunk.instructions.append(&mut if_body);
        jmp_offset = (total_body_size - chunk.instructions.len() + 1) as u16;
        chunk.instructions.push(Instruction {
            opcode: OpCode::Jmp,
            a: 0,
            b: (jmp_offset >> 8) as u8,
            c: jmp_offset as u8,
        });
        for (i, elif_cond_chunk) in elifs_cond_chunks.iter().enumerate() {
            chunk
                .instructions
                .append(&mut elif_cond_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::Test,
                a: elif_cond_chunk.result_register,
                b: elif_cond_chunk.result_register,
                c: 0,
            });
            let elif_body = &elifs_body[i];
            jmp_offset = (elif_body.len() + 2) as u16;
            chunk.instructions.push(Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jmp_offset >> 8) as u8,
                c: jmp_offset as u8,
            });
            chunk.instructions.append(&mut elif_body.clone());
            jmp_offset = (total_body_size - chunk.instructions.len() + 1) as u16;
            chunk.instructions.push(Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: (jmp_offset >> 8) as u8,
                c: jmp_offset as u8,
            });
        }
        chunk.instructions.append(&mut else_body.clone());
        return chunk;
    }

    fn visit_fn_stmt(&mut self, stmt: &FnStmt) -> Chunk {
        let result_register: u8 = self.next_register();
        self.allocate_register(stmt.name.lexeme.to_string(), result_register);
        self.enter_function(stmt.name.lexeme.to_string());
        // Register name inside function
        let self_fn_name_reg = self.next_register();
        self.allocate_register(stmt.name.lexeme.to_string(), self_fn_name_reg);
        self.add_chunk(Chunk {
            instructions: vec![Instruction {
                opcode: OpCode::Closure,
                a: self_fn_name_reg,
                b: 0,
                c: 0,
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
            let chunk = s.accept(self);
            self.add_chunk(chunk);
        }
        self.leave_block();
        let prototype_ix = self.leave_function();
        let first_instruction = self.prototypes[prototype_ix as usize]
            .instructions
            .first_mut()
            .unwrap();
        first_instruction.b = (prototype_ix >> 8) as u8;
        first_instruction.c = prototype_ix as u8;
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
        let result_register: u8 = self.next_register();
        self.allocate_register(
            stmt.name.clone().unwrap().lexeme.to_string(),
            result_register,
        );
        let mut chunk = Chunk {
            instructions: vec![],
            result_register: result_register,
        };
        let tmp_reg = self.next_register();
        let constant_ix = self.constants.len() as u16;
        self.constants.push(Value::String(StringValue {
            s: stmt.name.clone().unwrap().lexeme.to_string(),
        }));
        chunk.instructions.push(Instruction {
            opcode: OpCode::LoadK,
            a: tmp_reg,
            b: (constant_ix >> 8) as u8,
            c: constant_ix as u8,
        });
        let superclass_reg = if let Some(superclass) = &stmt.superclass {
            let var_chunk = self.visit_variable_expr(superclass);
            chunk
                .instructions
                .append(&mut var_chunk.instructions.clone());
            var_chunk.result_register
        } else {
            let nil_reg = self.next_register();
            chunk.instructions.push(Instruction {
                opcode: OpCode::LoadNil,
                a: nil_reg,
                b: 0,
                c: 0,
            });
            nil_reg
        };
        chunk.instructions.push(Instruction {
            opcode: OpCode::Class,
            a: chunk.result_register,
            b: tmp_reg,
            c: superclass_reg,
        });
        self.enter_block();
        for met in &stmt.methods {
            let met_chunk = self.visit_fn_stmt(met);
            chunk
                .instructions
                .append(&mut met_chunk.instructions.clone());
            let constant_ix = self.constants.len() as u16;
            self.constants.push(Value::String(StringValue {
                s: met.name.lexeme.to_string(),
            }));
            chunk.instructions.push(Instruction {
                opcode: OpCode::LoadK,
                a: tmp_reg,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            });
            chunk.instructions.push(Instruction {
                opcode: OpCode::ClassMeth,
                a: chunk.result_register,
                b: tmp_reg,
                c: met_chunk.result_register,
            });
        }
        for met in &stmt.static_methods {
            let met_chunk = self.visit_fn_stmt(met);
            chunk
                .instructions
                .append(&mut met_chunk.instructions.clone());
            let constant_ix = self.constants.len() as u16;
            self.constants.push(Value::String(StringValue {
                s: met.name.lexeme.to_string(),
            }));
            chunk.instructions.push(Instruction {
                opcode: OpCode::LoadK,
                a: tmp_reg,
                b: (constant_ix >> 8) as u8,
                c: constant_ix as u8,
            });
            chunk.instructions.push(Instruction {
                opcode: OpCode::ClassStMeth,
                a: chunk.result_register,
                b: tmp_reg,
                c: met_chunk.result_register,
            });
        }
        self.leave_block();
        return chunk;
    }
}

impl ExprVisitor<Chunk> for Compiler {
    fn visit_function_expr(&mut self, expr: &FnExpr) -> Chunk {
        let result_register: u8 = self.next_register();
        self.enter_function("".to_string());
        for p in expr.params.iter() {
            let reg = self.next_register();
            self.allocate_register(p.lexeme.to_string(), reg);
        }
        self.enter_block();
        for s in &expr.body {
            let chunk = s.accept(self);
            self.add_chunk(chunk);
        }
        self.leave_block();
        let prototype_ix = self.leave_function();
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

    fn visit_variable_expr(&mut self, expr: &VarExpr) -> Chunk {
        let var_name = expr.name.clone().unwrap().lexeme.to_string();
        if let Some(reg) = self.get_var_register(&var_name) {
            return Chunk {
                instructions: vec![],
                result_register: reg,
            };
        } else if let Some(up_ref) = self.get_upvalue(&var_name) {
            let reg = self.next_register();
            let upvalue_ix = self.add_upvalue(up_ref);
            return Chunk {
                instructions: vec![Instruction {
                    opcode: OpCode::GetUpval,
                    a: reg,
                    b: upvalue_ix,
                    c: 0,
                }],
                result_register: reg,
            };
        }
        panic!("Variable not found!");
    }

    fn visit_list_expr(&mut self, expr: &ListExpr) -> Chunk {
        let reg = self.next_register();
        let mut chunk = Chunk {
            result_register: reg,
            instructions: vec![Instruction {
                opcode: OpCode::List,
                a: reg,
                b: 0,
                c: 0,
            }],
        };
        for e in &expr.elements {
            let el_chunk = e.accept(self);
            chunk
                .instructions
                .append(&mut el_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::PushList,
                a: reg,
                b: el_chunk.result_register,
                c: 0,
            });
        }
        return chunk;
    }

    fn visit_dictionary_expr(&mut self, expr: &DictionaryExpr) -> Chunk {
        let reg = self.next_register();
        let mut chunk = Chunk {
            result_register: reg,
            instructions: vec![Instruction {
                opcode: OpCode::Dict,
                a: reg,
                b: 0,
                c: 0,
            }],
        };
        for i in 0..(expr.elements.len() / 2) {
            let k = &expr.elements[i * 2];
            let val = &expr.elements[i * 2 + 1];
            let k_chunk = k.accept(self);
            let val_chunk = val.accept(self);
            chunk.instructions.append(&mut k_chunk.instructions.clone());
            chunk
                .instructions
                .append(&mut val_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::PushDict,
                a: reg,
                b: k_chunk.result_register,
                c: val_chunk.result_register,
            });
        }
        return chunk;
    }

    fn visit_assign_expr(&mut self, expr: &AssignExpr) -> Chunk {
        if let Some(reg) = self.get_var_register(&expr.name.lexeme.to_string()) {
            let mut chunk = expr.value.accept(self);

            if let Some(access) = &expr.access {
                let obj_chunk = access.object.accept(self);
                let access_chunk = access.first.accept(self);
                chunk
                    .instructions
                    .append(&mut obj_chunk.instructions.clone());
                chunk
                    .instructions
                    .append(&mut access_chunk.instructions.clone());
                chunk.instructions.push(Instruction {
                    opcode: OpCode::Set,
                    a: obj_chunk.result_register,
                    b: access_chunk.result_register,
                    c: chunk.result_register,
                });
            } else {
                if !chunk.instructions.is_empty() {
                    let inst = chunk.instructions.last_mut().unwrap();
                    if inst.opcode == OpCode::Call {
                        inst.c = reg + 1;
                    } else {
                        inst.a = reg;
                    }
                }
                chunk.result_register = reg;
            }

            return chunk;
        } else if let Some(up_ref) = self.get_upvalue(&expr.name.lexeme.to_string()) {
            let reg = self.next_register();
            let mut chunk = expr.value.accept(self);

            if let Some(access) = &expr.access {
                let obj_chunk = access.object.accept(self);
                let access_chunk = access.first.accept(self);
                chunk
                    .instructions
                    .append(&mut obj_chunk.instructions.clone());
                chunk
                    .instructions
                    .append(&mut access_chunk.instructions.clone());
                chunk.instructions.push(Instruction {
                    opcode: OpCode::Set,
                    a: obj_chunk.result_register,
                    b: access_chunk.result_register,
                    c: chunk.result_register,
                });
            } else {
                if !chunk.instructions.is_empty() {
                    let inst = chunk.instructions.last_mut().unwrap();
                    if inst.opcode == OpCode::Call {
                        inst.c = reg + 1;
                    } else {
                        inst.a = reg;
                    }
                }
                let upvalue_ix = self.add_upvalue(up_ref);
                chunk.instructions.push(Instruction {
                    opcode: OpCode::SetUpval,
                    a: reg,
                    b: upvalue_ix,
                    c: 0,
                });
                chunk.result_register = reg;
            }

            return chunk;
        }
        panic!("Var doesn't exist!");
    }

    fn visit_access_expr(&mut self, expr: &AccessExpr) -> Chunk {
        // println!("{:#?}", expr);
        let mut chunk = Chunk {
            result_register: self.next_register(),
            instructions: vec![],
        };
        let obj_chunk = expr.object.accept(self);
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
            chunk.instructions.push(Instruction {
                opcode: OpCode::Access,
                a: chunk.result_register,
                b: obj_chunk.result_register,
                c: first_chunk.result_register,
            });
        } else {
            let list_register = self.next_register();
            chunk.instructions.push(Instruction {
                opcode: OpCode::List,
                a: list_register,
                b: 0,
                c: 0,
            });
            let nil_register = self.next_register();
            chunk.instructions.push(Instruction {
                opcode: OpCode::LoadNil,
                a: nil_register,
                b: 0,
                c: 0,
            });
            if !expr.first.is_empty() {
                let first_chunk = expr.first.accept(self);
                chunk
                    .instructions
                    .append(&mut first_chunk.instructions.clone());
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: first_chunk.result_register,
                    c: 0,
                });
            } else {
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: nil_register,
                    c: 0,
                });
            }
            if !expr.second.is_empty() {
                let second_chunk = expr.second.accept(self);
                chunk
                    .instructions
                    .append(&mut second_chunk.instructions.clone());
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: second_chunk.result_register,
                    c: 0,
                });
            } else {
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: nil_register,
                    c: 0,
                });
            }
            if !expr.third.is_empty() {
                let third_chunk = expr.third.accept(self);
                chunk
                    .instructions
                    .append(&mut third_chunk.instructions.clone());
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: third_chunk.result_register,
                    c: 0,
                });
            } else {
                chunk.instructions.push(Instruction {
                    opcode: OpCode::PushList,
                    a: list_register,
                    b: nil_register,
                    c: 0,
                });
            }
            chunk.instructions.push(Instruction {
                opcode: OpCode::Slice,
                a: list_register,
                b: list_register,
                c: 0,
            });
            chunk.instructions.push(Instruction {
                opcode: OpCode::Access,
                a: chunk.result_register,
                b: obj_chunk.result_register,
                c: list_register,
            });
        }
        return chunk;
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
        let mut chunk = expr.callee.accept(self);
        // println!("Callee: {:#?}", chunk);
        let fn_register = self.next_register();
        // println!("fn_register: {:#?}", fn_register);
        chunk.instructions.push(Instruction {
            opcode: OpCode::Move,
            a: fn_register,
            b: chunk.result_register,
            c: 0,
        });
        let start_reg = self.next_register();
        for _ in 1..(expr.arguments.len()) {
            self.next_register();
        }
        for (i, expr) in (&expr.arguments).iter().enumerate() {
            let arg_chunk = expr.accept(self);
            chunk
                .instructions
                .append(&mut arg_chunk.instructions.clone());
            chunk.instructions.push(Instruction {
                opcode: OpCode::Move,
                a: start_reg + i as u8,
                b: arg_chunk.result_register,
                c: 0,
            });
        }
        chunk.result_register = self.next_register();
        chunk.instructions.push(Instruction {
            opcode: OpCode::Call,
            a: fn_register,
            b: expr.arguments.len() as u8 + 1,
            c: chunk.result_register + 1,
        });
        // println!("Call chunk: {:#?}", chunk);
        return chunk;
    }

    fn visit_get_expr(&mut self, expr: &GetExpr) -> Chunk {
        let mut chunk = expr.object.accept(self);
        let result_register = self.next_register();
        let constant_ix = self.constants.len() as u16;
        let val = Value::String(StringValue {
            s: expr.name.lexeme.to_string(),
        });
        self.constants.push(val);
        chunk.instructions.push(Instruction {
            opcode: OpCode::LoadK,
            a: result_register,
            b: (constant_ix >> 8) as u8,
            c: constant_ix as u8,
        });
        chunk.instructions.push(Instruction {
            opcode: OpCode::GetObj,
            a: result_register,
            b: chunk.result_register,
            c: result_register,
        });
        chunk.result_register = result_register;
        return chunk;
    }

    fn visit_set_expr(&mut self, expr: &SetExpr) -> Chunk {
        let mut chunk = expr.object.accept(self);
        // TODO: handle expr.access
        let value_chunk = expr.value.accept(self);
        chunk
            .instructions
            .append(&mut value_chunk.instructions.clone());
        let tmp_register = self.next_register();
        let constant_ix = self.constants.len() as u16;
        let val = Value::String(StringValue {
            s: expr.name.lexeme.to_string(),
        });
        self.constants.push(val);
        chunk.instructions.push(Instruction {
            opcode: OpCode::LoadK,
            a: tmp_register,
            b: (constant_ix >> 8) as u8,
            c: constant_ix as u8,
        });
        chunk.instructions.push(Instruction {
            opcode: OpCode::SetObj,
            a: chunk.result_register,
            b: tmp_register,
            c: value_chunk.result_register,
        });
        return chunk;
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
            let jump_size = (right_chunk.instructions.len() + 2) as u16;
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
            let jump_size = (right_chunk.instructions.len() + 2) as u16;
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
            Expr::Empty => Chunk {
                instructions: vec![],
                result_register: 0,
            },
        }
    }
}
