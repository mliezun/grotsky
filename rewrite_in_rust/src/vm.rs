#![allow(dead_code)]

use crate::compiler::FnPrototype;
use crate::instruction::*;
use crate::value::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub function: Option<MutValue<FnValue>>, // Empty when main function
    pub pc: usize,                           // Return location
    pub sp: usize,                           // Stack pointer inside activation record
    pub result_register: u8,
}

#[derive(Debug, Clone)]
pub enum Record {
    Val(Value),
    Ref(MutValue<Value>),
}

impl Record {
    fn as_val(&self) -> Value {
        match self {
            Record::Val(v) => v.clone(),
            Record::Ref(v) => v.0.deref().borrow_mut().clone(),
        }
    }
}

#[derive(Debug)]
pub struct VM {
    pub instructions: Vec<Instruction>,
    pub prototypes: Vec<FnPrototype>,
    pub constants: Vec<Value>,
    pub globals: HashMap<String, Value>,
    pub stack: Vec<StackEntry>,
    pub activation_records: Vec<Record>,
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
                        Record::Val(self.constants[inst.bx() as usize].clone());
                    pc += 1;
                }
                OpCode::LoadNil => {
                    self.activation_records[sp + inst.a as usize] = Record::Val(Value::Nil);
                    pc += 1;
                }
                OpCode::Move => {
                    self.activation_records[sp + inst.a as usize] =
                        self.activation_records[sp + inst.b as usize].clone();
                    pc += 1;
                }
                OpCode::Closure => {
                    let prototype = &self.prototypes[inst.bx() as usize];
                    let fn_upvalues: Vec<MutValue<Value>> = prototype
                        .upvalues
                        .iter()
                        .map(|u| {
                            let base_sp = if u.depth == 0 {
                                sp
                            } else {
                                let ix = self.stack.len() - 1 - u.depth as usize;
                                self.stack[ix].sp
                            };
                            let rec_ix = base_sp + u.register as usize;
                            let record = match self.activation_records[rec_ix].clone() {
                                Record::Ref(v) => v,
                                Record::Val(v) => MutValue::new(v),
                            };
                            self.activation_records[rec_ix] = Record::Ref(record.clone());
                            return record;
                        })
                        .collect();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::Fn(MutValue::new(FnValue {
                            prototype: inst.bx(),
                            upvalues: fn_upvalues,
                            constants: vec![],
                        })));
                    pc += 1;
                }
                OpCode::Call => {
                    let val = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    if let Value::Fn(fn_value) = val {
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

                        // Expand activation records
                        self.activation_records.append(
                            &mut (0..prototype.register_count)
                                .map(|_| Record::Val(Value::Nil))
                                .collect(),
                        );

                        // Copy input arguments
                        for (i, reg) in ((inst.a + 1)..(inst.a + inst.b)).enumerate() {
                            self.activation_records[sp + i] =
                                self.activation_records[previous_sp + reg as usize].clone();
                        }

                        instructions = &prototype.instructions;
                        pc = 0;
                    } else {
                        panic!("Not a function");
                    }
                }
                OpCode::Return => {
                    if self.stack.len() <= 1 {
                        return;
                    }
                    let stack = self.stack.pop().unwrap();
                    if inst.b == inst.a + 2 && stack.result_register > 0 {
                        // Store return values
                        self.activation_records[stack.sp + (stack.result_register - 1) as usize] =
                            self.activation_records[sp + inst.a as usize].clone();
                    }
                    self.activation_records
                        .drain(sp..self.activation_records.len());
                    pc = stack.pc;
                    sp = stack.sp;
                    if let Some(func) = &self.stack.last().unwrap().function {
                        instructions =
                            &self.prototypes[func.0.borrow().prototype as usize].instructions;
                    } else {
                        instructions = &self.instructions;
                    }
                }

                OpCode::Jmp => {
                    // println!("{:#?}", pc);
                    pc = pc.wrapping_add(inst.sbx() as usize);
                }
                OpCode::Test => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap();
                    let bool_c = inst.c != 0;
                    if truthy(&val_b.as_val()) == bool_c {
                        self.activation_records[sp + inst.a as usize] = val_b.clone();
                    } else {
                        pc += 1;
                    }
                    pc += 1;
                }
                OpCode::Add => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().add(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Sub => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().sub(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Mul => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().mul(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Pow => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().pow(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Div => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().div(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Mod => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().modulo(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Lt => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().lt(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Lte => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().lte(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Gt => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().gt(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Gte => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().gte(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Eq => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().equal(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::Neq => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().nequal(&mut val_c.as_val()));
                    pc += 1;
                }
                OpCode::GetUpval => {
                    let current_func = self.stack.last_mut().unwrap().function.clone().unwrap();
                    let upval = &current_func.0.borrow().upvalues[inst.b as usize];
                    self.activation_records[sp + inst.a as usize] = Record::Ref(upval.clone());
                    pc += 1;
                }
                OpCode::SetUpval => {
                    let current_func = self.stack.last_mut().unwrap().function.clone().unwrap();
                    let upval = &current_func.0.borrow().upvalues[inst.b as usize];
                    upval.0.replace(
                        self.activation_records[sp + inst.a as usize]
                            .as_val()
                            .clone(),
                    );
                    pc += 1;
                }
                OpCode::List => {
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::List(MutValue::new(ListValue { elements: vec![] })));
                    pc += 1;
                }
                OpCode::PushList => {
                    let val = self.activation_records[sp + inst.a as usize]
                        .as_val()
                        .clone();
                    if let Value::List(list_val) = val {
                        let lval = list_val.0.deref();
                        lval.borrow_mut().elements.push(
                            self.activation_records[sp + inst.b as usize]
                                .as_val()
                                .clone(),
                        );
                        pc += 1;
                    } else {
                        panic!("Cannot push to non-list");
                    }
                }
                OpCode::Dict => {
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::Dict(MutValue::new(DictValue {
                            elements: HashMap::new(),
                        })));
                    pc += 1;
                }
                OpCode::PushDict => {
                    let val = self.activation_records[sp + inst.a as usize]
                        .as_val()
                        .clone();
                    if let Value::Dict(dict_val) = val {
                        let dval = dict_val.0.deref();
                        let hash_map = &mut dval.borrow_mut().elements;
                        hash_map.insert(
                            self.activation_records[sp + inst.b as usize]
                                .as_val()
                                .clone(),
                            self.activation_records[sp + inst.c as usize]
                                .as_val()
                                .clone(),
                        );
                        pc += 1;
                    } else {
                        panic!("Cannot push to non-dict");
                    }
                }
                OpCode::Slice => {
                    let val = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    if let Value::List(list) = &val {
                        let slice = SliceValue {
                            first: Rc::new(list.0.borrow().elements[0].clone()),
                            second: Rc::new(list.0.borrow().elements[1].clone()),
                            third: Rc::new(list.0.borrow().elements[2].clone()),
                        };
                        self.activation_records[sp + inst.a as usize] =
                            Record::Val(Value::Slice(slice));
                        pc += 1;
                    } else {
                        panic!("Cannot create slice from non-list")
                    }
                }
                OpCode::Access => {
                    let val = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let accessor = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    match val {
                        Value::List(list) => {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(list.0.borrow().access(accessor));
                            pc += 1;
                        }
                        Value::Dict(dict) => unimplemented!(),
                        Value::String(str) => unimplemented!(),
                        _ => panic!("Cannot access non iterable"),
                    }
                }
                OpCode::Set => {
                    let dest = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let accessor = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let val = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    match dest {
                        Value::List(list) => {
                            if let Value::Number(nval) = accessor {
                                list.0.borrow_mut().elements[nval.n as usize] = val;
                                pc += 1;
                            } else {
                                panic!("Index error");
                            }
                        }
                        Value::Dict(dict) => unimplemented!(),
                        Value::String(str) => unimplemented!(),
                        _ => panic!("Cannot access non iterable"),
                    }
                }
                OpCode::Class => {
                    let class_name = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let superclass = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let mut class = ClassValue {
                        name: "".to_string(),
                        superclass: None,
                        methods: HashMap::new(),
                        classmethods: HashMap::new(),
                    };
                    if let Value::String(name) = class_name {
                        class.name = name.s;
                    } else {
                        panic!("Class name should be string");
                    }
                    match superclass {
                        Value::Class(superclass_val) => {
                            class.superclass = Some(superclass_val);
                        }
                        Value::Nil => {}
                        _ => panic!("Cannot inherit from non-class"),
                    }
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::Class(MutValue::new(class)));
                    pc += 1;
                }
                OpCode::ClassMeth => {
                    let class = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let prop = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let method = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let meth_name = if let Value::String(s) = prop {
                        s.s
                    } else {
                        panic!("Cannot assign to non-string prop")
                    };
                    let meth_value = if let Value::Fn(m) = method {
                        m
                    } else {
                        panic!("Cannot assign method as non-function")
                    };
                    if let Value::Class(class_val) = class {
                        class_val
                            .0
                            .borrow_mut()
                            .methods
                            .insert(meth_name, meth_value);
                        pc += 1;
                    } else {
                        panic!("Cannot assign method to non-classs");
                    }
                }
                OpCode::ClassStMeth => {
                    let class = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let prop = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let method = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let meth_name = if let Value::String(s) = prop {
                        s.s
                    } else {
                        panic!("Cannot assign to non-string prop")
                    };
                    let meth_value = if let Value::Fn(m) = method {
                        m
                    } else {
                        panic!("Cannot assign method as non-function")
                    };
                    if let Value::Class(class_val) = class {
                        class_val
                            .0
                            .borrow_mut()
                            .classmethods
                            .insert(meth_name, meth_value);
                        pc += 1;
                    } else {
                        panic!("Cannot assign method to non-classs");
                    }
                }
                OpCode::GetObj => {
                    let val_b = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let val_c = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let prop = if let Value::String(s) = val_c {
                        s.s
                    } else {
                        panic!("Prop has to be a string")
                    };
                    self.activation_records[sp + inst.a as usize] = Record::Val(val_b.get(prop));
                    pc += 1;
                }
                OpCode::Addi => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    self.activation_records[sp + inst.a as usize] = Record::Val(
                        val_b
                            .as_val()
                            .add(&mut Value::Number(NumberValue { n: inst.c as f64 })),
                    );
                    pc += 1;
                }
                _ => todo!(),
            }
        }
    }
}
