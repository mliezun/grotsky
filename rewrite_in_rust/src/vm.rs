#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
struct MutValue<T>(Rc<RefCell<T>>);

impl<T> MutValue<T> {
    fn new(obj: T) -> Self {
        MutValue::<T>(Rc::new(RefCell::new(obj)))
    }
}

#[derive(Debug, Clone)]
enum Value {
    Class(MutValue<ClassValue>),
    // Object(MutValue<ObjectValue>),
    // Dict(MutValue<DictValue>),
    // List(MutValue<ListValue>),
    Fn(MutValue<FnValue>),
    // Native(NativeValue),
    Number(NumberValue),
    // String(StringValue),
    //Bool(BoolValue),
    Nil,
}

#[derive(Debug, Clone)]
struct NumberValue {
    n: f64,
}

#[derive(Debug, Clone)]
struct Instruction {
    opcode: OpCode,
    a: u8,
    b: u8,
    c: u8,
}

impl Instruction {
    fn bx(&self) -> u16 {
        let b = self.b as u16;
        let c = self.c as u16;
        return b << 8 | c;
    }
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Move,
    LoadK,
    Call,
    Return,
    Close,
    SetUpval,
    GetUpval,
    SetGlobal,
    GetGlobal,
    Closure,
}

#[derive(Debug, Clone)]
struct FnValue {
    prototype: u16,
    upvalues: HashMap<String, Upvalue>,
    constants: Vec<Value>,
}

#[derive(Debug, Clone)]
struct ClassValue {
    name: String,
    superclass: Option<MutValue<ClassValue>>,
    methods: Vec<MutValue<FnValue>>,
    classmethods: Vec<MutValue<FnValue>>,
}

#[derive(Debug, Clone)]
struct Upvalue {
    value: *mut Value,
    closed_value: Option<Value>, // Only active when the upvalue is closed
}

#[derive(Debug, Clone)]
struct StackEntry {
    function: Option<MutValue<FnValue>>, // Empty when main function
    pc: usize,                           // Return location
    sp: usize,                           // Stack pointer inside activation record
}

#[derive(Debug, Clone)]
struct Prototype {
    instructions: Vec<Instruction>,
}

#[derive(Debug)]
struct VM {
    instructions: Vec<Instruction>,
    prototypes: Vec<Prototype>,
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
            println!("Executing {:#?}", inst);
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
                            sp: sp,
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
                        println!("Finishing program");
                        return;
                    }
                    println!("Popping stack: {}", self.stack.len());
                    let stack = self.stack.pop().unwrap();
                    pc = stack.pc;
                    sp = stack.sp;
                    if let Some(func) = &self.stack.last().unwrap().function {
                        println!("There is a function {:#?}", func);
                        instructions =
                            &self.prototypes[func.0.borrow().prototype as usize].instructions;
                    } else {
                        println!("There is no function");
                        instructions = &self.instructions;
                    }
                    println!("{:#?}", self.stack);
                    println!("{:#?}", instructions);
                    println!("{:#?}", pc);
                }
                _ => unimplemented!(),
            }
        }
    }
}

pub fn test_vm_execution() {
    let dummy_fn = FnValue {
        prototype: 1,
        upvalues: HashMap::new(),
        constants: vec![],
    };
    let mut vm = VM {
        instructions: vec![
            Instruction {
                opcode: OpCode::Closure,
                a: 0,
                b: 0,
                c: 0,
            },
            Instruction {
                opcode: OpCode::Call,
                a: 0,
                b: 0,
                c: 0,
            },
            Instruction {
                opcode: OpCode::Return,
                a: 1,
                b: 2,
                c: 3,
            },
        ],
        prototypes: vec![Prototype {
            instructions: vec![
                Instruction {
                    opcode: OpCode::Move,
                    a: 3,
                    b: 1,
                    c: 0,
                },
                Instruction {
                    opcode: OpCode::Return,
                    a: 0,
                    b: 0,
                    c: 0,
                },
            ],
        }],
        constants: vec![],
        globals: HashMap::new(),
        stack: vec![StackEntry {
            function: None,
            pc: 0,
            sp: 0,
        }],
        activation_records: vec![
            Value::Nil,
            Value::Number(NumberValue { n: 1.0 }),
            Value::Number(NumberValue { n: 2.0 }),
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
