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

impl Value {
    fn add(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n + other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    fn lt(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: (num_val.n < other_val.n) as i32 as f64,
                });
            }
        }
        panic!("Not implemented");
    }
}

fn truthy(val: &Value) -> bool {
    if let Value::Number(num_val) = val {
        return num_val.n != 0.0;
    }
    return false;
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

    fn sbx(&self) -> i16 {
        let b = self.b as u16;
        let c = self.c as u16;
        let result = (b << 8 | c) as i16;
        // println!("sbx = {:#?}", result);
        return result;
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
    Add,
    Lt,
    Jmp,
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
                    let sbx = inst.sbx();
                    if sbx < 0 {
                        if (sbx.abs() as usize) < pc {
                            pc -= sbx.abs() as usize;
                        } else {
                            pc = 0;
                        }
                    } else {
                        pc += inst.sbx() as usize;
                    }
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
                    let result = val_b.lt(val_c);
                    if truthy(&result) {
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
                a: 0,
                b: 0,
                c: 2,
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
                c: 253,
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
            Value::Number(NumberValue { n: 10000000.0 }),
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
