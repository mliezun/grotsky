#![allow(dead_code)]

use crate::instruction::*;
use crate::value::*;
use std::collections::HashMap;

#[derive(Debug, Clone)]
struct StackEntry {
    function: Option<MutValue<FnValue>>, // Empty when main function
    pc: usize,                           // Return location
    sp: usize,                           // Stack pointer inside activation record
}

#[derive(Debug, Clone)]
struct FnPrototype {
    instructions: Vec<Instruction>,
    nlocals: u8,
}

#[derive(Debug)]
struct VM {
    instructions: Vec<Instruction>,
    prototypes: Vec<FnPrototype>,
    constants: Vec<Value>,
    globals: HashMap<String, Value>,
    stack: Vec<StackEntry>,
    activation_records: Vec<Value>,
    pc: usize,
}

impl VM {
    fn interpret(&mut self) {
        let mut instructions = &self.instructions;
        let mut pc = self.pc;
        let mut sp = self.stack[self.stack.len() - 1].sp;
        while pc < instructions.len() {
            let inst = &instructions[pc];
            // println!("Executing {:#?}", inst);
            match inst.opcode {
                OpCode::Move => {
                    self.activation_records
                        .swap(sp + inst.a as usize, sp + inst.b as usize);
                    pc += 1;
                }
                OpCode::Closure => {
                    self.activation_records[sp + inst.a as usize] =
                        Value::Fn(MutValue::new(FnValue {
                            prototype: inst.bx(),
                            upvalues: HashMap::new(),
                            constants: vec![],
                        }));
                    pc += 1;
                }
                OpCode::Call => {
                    if let Value::Fn(fn_value) = &self.activation_records[sp + inst.a as usize] {
                        let stack = StackEntry {
                            function: Some(fn_value.clone()),
                            pc: pc + 1,
                            sp: self.activation_records.len(),
                        };
                        instructions =
                            &self.prototypes[fn_value.0.borrow().prototype as usize].instructions;
                        self.stack.push(stack);
                        pc = 0;
                        // sp +=
                    }
                }
                OpCode::Return => {
                    if self.stack.len() <= 1 {
                        // println!("Finishing program");
                        return;
                    }
                    // println!("Popping stack: {}", self.stack.len());
                    let stack = self.stack.pop().unwrap();
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
                OpCode::Jmp => {
                    // println!("{:#?}", pc);
                    pc = pc.wrapping_add(inst.sbx() as usize);
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
                OpCode::Test => {
                    let val_b = self.activation_records.get(sp + inst.b as usize).unwrap();
                    let imm_c = &Value::Number(NumberValue {
                        n: inst.c as i32 as f64,
                    });
                    if truthy(val_b) == truthy(imm_c) {
                        self.activation_records[sp + inst.a as usize] = val_b.clone();
                    } else {
                        pc += 1;
                    }
                    pc += 1;
                }
                _ => unimplemented!(),
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
