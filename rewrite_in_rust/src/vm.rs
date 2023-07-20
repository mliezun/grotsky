#![allow(dead_code)]

use crate::compiler::FnPrototype;
use crate::instruction::*;
use crate::value::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub function: Option<MutValue<FnValue>>, // Empty when main function
    pub pc: usize,                           // Return location
    pub sp: usize,                           // Stack pointer inside activation record
    pub result_register: u8,
}

#[derive(Debug)]
pub struct VM {
    pub instructions: Vec<Instruction>,
    pub prototypes: Vec<FnPrototype>,
    pub constants: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub stack: Vec<StackEntry>,
    pub activation_records: Vec<Value>,
    pub pc: usize,
}

impl VM {
    pub fn interpret(&mut self) {
        let mut instructions = &self.instructions;
        let mut pc = self.pc;
        let mut sp = self.stack[self.stack.len() - 1].sp;
        while pc < instructions.len() {
            let inst = &instructions[pc];
            // println!("Executing {:#?}", inst);
            match inst.opcode {
                OpCode::LoadK => {
                    self.activation_records[sp + inst.a as usize] =
                        self.constants[inst.bx() as usize].clone();
                    pc += 1;
                }
                OpCode::Move => {
                    self.activation_records[sp + inst.a as usize] =
                        self.activation_records[sp + inst.b as usize].clone();
                    pc += 1;
                }
                OpCode::Closure => {
                    self.activation_records[sp + inst.a as usize] =
                        Value::Fn(MutValue::new(FnValue {
                            prototype: inst.bx(),
                            upvalues: HashMap::new(),
                            constants: vec![],
                        }));
                    // println!("Records={:#?}", self.activation_records);
                    pc += 1;
                }
                OpCode::Call => {
                    if let Value::Fn(fn_value) = &self.activation_records[sp + inst.a as usize] {
                        let stack = StackEntry {
                            function: Some(fn_value.clone()),
                            pc: pc + 1,
                            sp: sp,
                            result_register: inst.c,
                        };
                        let previous_sp = sp;
                        sp = self.activation_records.len();
                        self.stack.push(stack);
                        let prototype = &self.prototypes[fn_value.0.borrow().prototype as usize];
                        self.activation_records.append(
                            &mut (0..prototype.register_count).map(|_| Value::Nil).collect(),
                        );
                        for (i, reg) in ((inst.a + 1)..(inst.a + inst.b)).enumerate() {
                            self.activation_records[sp + i] =
                                self.activation_records[previous_sp + reg as usize].clone();
                        }
                        // println!("{:#?}", self.activation_records);
                        instructions = &prototype.instructions;
                        pc = 0;
                    } else {
                        println!("sp={}, inst.a={}", sp, inst.a);
                        println!("Registers={:#?}", self.activation_records);
                        panic!("Not a function");
                    }
                }
                OpCode::Return => {
                    if self.stack.len() <= 1 {
                        // println!("Finishing program");
                        return;
                    }
                    let stack = self.stack.pop().unwrap();
                    // println!("Popping stack: {:#?}", stack);
                    // println!("Executing inst: {:#?}", inst);
                    if inst.b == inst.a + 2 && stack.result_register > 0 {
                        // println!(
                        //     "Storing result result_register={}, stack.sp={}",
                        //     stack.result_register - 1,
                        //     stack.sp,
                        // );
                        self.activation_records[stack.sp + (stack.result_register - 1) as usize] =
                            self.activation_records[sp + inst.a as usize].clone();
                    }
                    self.activation_records
                        .drain(sp..self.activation_records.len());
                    pc = stack.pc;
                    sp = stack.sp;
                    if let Some(func) = &self.stack.last().unwrap().function {
                        // println!("There is a function {:#?}", func);
                        instructions =
                            &self.prototypes[func.0.borrow().prototype as usize].instructions;
                    } else {
                        // println!("There is no function");
                        instructions = &self.instructions;
                    }
                    // println!("{:#?}", self.stack);
                    // println!("{:#?}", instructions);
                    // println!("{:#?}", pc);
                }

                OpCode::Jmp => {
                    // println!("{:#?}", pc);
                    pc = pc.wrapping_add(inst.sbx() as usize);
                }
                OpCode::Test => {
                    let val_b = self.activation_records.get(sp + inst.b as usize).unwrap();
                    let bool_c = inst.c != 0;
                    if truthy(val_b) == bool_c {
                        self.activation_records[sp + inst.a as usize] = val_b.clone();
                    } else {
                        pc += 1;
                    }
                    pc += 1;
                }
                OpCode::Add => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.add(val_c);
                    pc += 1;
                }
                OpCode::Sub => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.sub(val_c);
                    pc += 1;
                }
                OpCode::Mul => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.mul(val_c);
                    pc += 1;
                }
                OpCode::Pow => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.pow(val_c);
                    pc += 1;
                }
                OpCode::Div => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.div(val_c);
                    pc += 1;
                }
                OpCode::Mod => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.modulo(val_c);
                    pc += 1;
                }
                OpCode::Lt => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.lt(val_c);
                    pc += 1;
                }
                OpCode::Lte => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.lte(val_c);
                    pc += 1;
                }
                OpCode::Gt => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.gt(val_c);
                    pc += 1;
                }
                OpCode::Gte => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.gte(val_c);
                    pc += 1;
                }
                OpCode::Eq => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.eq(val_c);
                    pc += 1;
                }
                OpCode::Neq => {
                    let mut val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] = val_b.neq(val_c);
                    pc += 1;
                }
                _ => todo!(),
            }
        }
    }
}

pub fn test_vm_execution() {
    // println!("{:#b}", -3);
    let dummy_fn = FnValue {
        prototype: 1,
        upvalues: HashMap::new(),
        constants: vec![],
    };
    let mut vm = VM {
        instructions: vec![
            Instruction {
                opcode: OpCode::Lt,
                a: 3,
                b: 0,
                c: 2,
            },
            Instruction {
                opcode: OpCode::Test,
                a: 3,
                b: 3,
                c: 0,
            },
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                b: 0,
                c: 3,
            },
            Instruction {
                opcode: OpCode::Add,
                a: 0,
                b: 1,
                c: 0,
            },
            Instruction {
                opcode: OpCode::Jmp,
                a: 0,
                // -3
                b: 255,
                c: 252,
            },
        ],
        prototypes: vec![],
        constants: vec![],
        globals: HashMap::new(),
        stack: vec![StackEntry {
            function: None,
            pc: 0,
            sp: 0,
            result_register: 0,
        }],
        activation_records: vec![
            Value::Number(NumberValue { n: 0.0 }),
            Value::Number(NumberValue { n: 1.0 }),
            Value::Number(NumberValue { n: 1000000.0 }),
            Value::Number(NumberValue { n: 3.0 }),
            Value::Number(NumberValue { n: 4.0 }),
            Value::Number(NumberValue { n: 5.0 }),
        ],
        pc: 0,
    };
    vm.globals
        .insert("dummy_fn".to_string(), Value::Fn(MutValue::new(dummy_fn)));
    // println!("{:#?}", vm);
    vm.interpret();
    println!("{:#?}", vm);
}
