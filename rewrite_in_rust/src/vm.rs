#![allow(dead_code)]

use crate::compiler::FnPrototype;
use crate::errors::*;
use crate::instruction::*;
use crate::token::TokenData;
use crate::value::*;
use std::collections::HashMap;
use std::ops::Deref;
use std::rc::Rc;

macro_rules! make_call {
    ( $self:expr, $fn_value:expr, $current_this:expr, $bind_to:expr, $instructions:expr, $pc:expr, $sp:expr, $result_register:expr, $input_range:expr ) => {{
        let stack = StackEntry {
            function: Some($fn_value.clone()),
            pc: $pc + 1,
            sp: $sp,
            result_register: $result_register,
            this: $current_this.clone(),
        };
        let previous_sp = $sp;
        $sp = $self.activation_records.len();
        $self.stack.push(stack);

        let prototype = &$self.prototypes[$fn_value.0.borrow().prototype as usize];

        // Expand activation records
        $self.activation_records.append(
            &mut (0..prototype.register_count)
                .map(|_| Record::Val(Value::Nil))
                .collect(),
        );

        // Copy input arguments
        if $input_range.len() != prototype.param_count {
            $self.exception(
                ERR_INVALID_NUMBER_ARGUMENTS,
                $self.instructions_data[$pc].clone(),
            );
        }
        for (i, reg) in $input_range.enumerate() {
            $self.activation_records[$sp + i + 1] =
                $self.activation_records[previous_sp + reg as usize].clone();
        }

        // Set current object
        $current_this = $bind_to;

        // Jump to new section of code
        $instructions = prototype.instructions.clone();
        $pc = 0;
        // Point to new instructions metadata
        // TODO: use reference instead of cloning
        $self.instructions_data = prototype.instruction_data.clone();
    }};
}

#[derive(Debug, Clone)]
pub struct StackEntry {
    pub function: Option<MutValue<FnValue>>, // Empty when main function
    pub pc: usize,                           // Return location
    pub sp: usize,                           // Stack pointer inside activation record
    pub result_register: u8,
    pub this: Option<MutValue<ObjectValue>>,
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
    pub builtins: HashMap<String, Value>,
    pub stack: Vec<StackEntry>,
    pub activation_records: Vec<Record>,
    pub instructions_data: Vec<Option<TokenData>>,
}

impl VM {
    pub fn interpret(&mut self) {
        let mut pc = self.stack[self.stack.len() - 1].pc;
        let mut sp = self.stack[self.stack.len() - 1].sp;
        let mut this: Option<MutValue<ObjectValue>> = None;
        let original_instructions_data = self.instructions_data.clone();
        let original_instructions = self.instructions.clone();
        while pc < self.instructions.len() {
            let inst = self.instructions[pc].clone();
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
                            if u.is_local {
                                let rec_ix = sp + u.index as usize;
                                let record = match self.activation_records[rec_ix].clone() {
                                    Record::Ref(v) => v,
                                    Record::Val(v) => MutValue::new(v),
                                };
                                self.activation_records[rec_ix] = Record::Ref(record.clone());
                                return record;
                            } else {
                                let current_func =
                                    self.stack.last_mut().unwrap().function.clone().unwrap();
                                let upval = &current_func.0.borrow().upvalues[u.index as usize];
                                return upval.clone();
                            }
                        })
                        .collect();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::Fn(MutValue::new(FnValue {
                            prototype: inst.bx(),
                            upvalues: fn_upvalues,
                            constants: vec![],
                            this: this.clone(),
                            name: prototype.name.clone(),
                        })));
                    pc += 1;
                }
                OpCode::Call => {
                    let val = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    match &val {
                        Value::Fn(fn_value) => {
                            make_call!(
                                self,
                                fn_value,
                                this,
                                fn_value.0.borrow().this.clone(),
                                self.instructions,
                                pc,
                                sp,
                                inst.c,
                                (inst.a + 1)..(inst.a + inst.b)
                            );
                        }
                        Value::Native(n) => {
                            if let Some(callable) = n.callable {
                                let mut args: Vec<Value> = vec![];
                                if n.bind {
                                    args.push(val.clone());
                                }
                                for reg in (inst.a + 1)..(inst.a + inst.b) {
                                    let val = match &self.activation_records[sp + reg as usize] {
                                        Record::Ref(v) => v.0.borrow().clone(),
                                        Record::Val(v) => v.clone(),
                                    };
                                    args.push(val);
                                }
                                let result = callable(args);
                                match result {
                                    Ok(v) => {
                                        if inst.c > 0 {
                                            self.activation_records[sp + inst.c as usize - 1] =
                                                Record::Val(v.clone());
                                        }
                                    }
                                    Err(e) => {
                                        self.exception(e, self.instructions_data[pc].clone());
                                    }
                                }
                                pc += 1;
                            } else {
                                self.exception(
                                    ERR_ONLY_FUNCTION,
                                    self.instructions_data[pc].clone(),
                                );
                            }
                        }
                        Value::Class(c) => {
                            let object_value = MutValue::new(ObjectValue {
                                class: c.clone(),
                                fields: HashMap::new(),
                            });
                            self.activation_records[sp + inst.c as usize - 1] =
                                Record::Val(Value::Object(object_value.clone()));
                            if let Some(fn_value) = c.0.borrow().methods.get(&"init".to_string()) {
                                let cloned_fn_value = fn_value.clone();
                                make_call!(
                                    self,
                                    cloned_fn_value,
                                    this,
                                    Some(object_value),
                                    self.instructions,
                                    pc,
                                    sp,
                                    0,
                                    (inst.a + 1)..(inst.a + inst.b)
                                );
                            } else {
                                pc += 1;
                            }
                        }
                        _ => {
                            self.exception(ERR_ONLY_FUNCTION, self.instructions_data[pc].clone());
                        }
                    }
                }
                OpCode::Return => {
                    if self.stack.len() <= 1 {
                        return;
                    }
                    let stack = self.stack.pop().unwrap();

                    // Store return values
                    if inst.b == inst.a + 2 && stack.result_register > 0 {
                        self.activation_records[stack.sp + (stack.result_register - 1) as usize] =
                            self.activation_records[sp + inst.a as usize].clone();
                    }

                    // Drop current stack frame
                    self.activation_records
                        .drain(sp..self.activation_records.len());

                    // Restore previous object
                    this = stack.this;

                    // Restore pointers to previous section of code
                    pc = stack.pc;
                    sp = stack.sp;
                    if let Some(func) = &self.stack.last().unwrap().function {
                        let proto = &self.prototypes[func.0.borrow().prototype as usize];
                        // TODO: use reference instead of cloning
                        self.instructions = proto.instructions.clone();
                        self.instructions_data = proto.instruction_data.clone();
                    } else {
                        // TODO: use reference instead of cloning
                        self.instructions = original_instructions.clone();
                        self.instructions_data = original_instructions_data.clone();
                    }
                }

                OpCode::Jmp => {
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
                    match val_b.as_val().add(&mut val_c.as_val()) {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                            pc += 1;
                        }
                        Err(e) => {
                            if let Some(signal) = e.signal {
                                if let Value::Fn(fn_value) = signal {
                                    make_call!(
                                        self,
                                        fn_value,
                                        this,
                                        fn_value.0.borrow().this.clone(),
                                        self.instructions,
                                        pc,
                                        sp,
                                        inst.a + 1,
                                        (inst.c)..(inst.c + 1)
                                    );
                                } else {
                                    self.exception(
                                        ERR_EXPECTED_FUNCTION,
                                        self.instructions_data[pc].clone(),
                                    );
                                }
                            } else {
                                self.exception(e, self.instructions_data[pc].clone());
                            }
                        }
                    }
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
                    match val_b.as_val().sub(&mut val_c.as_val()) {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => {
                            self.exception(e, self.instructions_data[pc].clone());
                        }
                    }

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
                    match val_b.as_val().mul(&mut val_c.as_val()) {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => {
                            self.exception(e, self.instructions_data[pc].clone());
                        }
                    }
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
                    match val_b.as_val().lte(&mut val_c.as_val()) {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => {
                            self.exception(e, self.instructions_data[pc].clone());
                        }
                    }
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
                OpCode::Neg => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    match val_b.as_val().neg() {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => self.exception(e, self.instructions_data[pc].clone()),
                    }
                    pc += 1;
                }
                OpCode::Not => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(val_b.as_val().not());
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
                            match list.0.borrow().access(accessor) {
                                Ok(v) => {
                                    self.activation_records[sp + inst.a as usize] = Record::Val(v);
                                }
                                Err(e) => {
                                    self.exception(e, self.instructions_data[pc].clone());
                                }
                            }
                            pc += 1;
                        }
                        Value::String(str) => {
                            match str.access(accessor) {
                                Ok(v) => {
                                    self.activation_records[sp + inst.a as usize] = Record::Val(v);
                                }
                                Err(e) => {
                                    self.exception(e, self.instructions_data[pc].clone());
                                }
                            }
                            pc += 1;
                        }
                        Value::Dict(dict) => {
                            match dict.0.borrow().access(accessor) {
                                Ok(v) => {
                                    self.activation_records[sp + inst.a as usize] = Record::Val(v);
                                }
                                Err(e) => {
                                    self.exception(e, self.instructions_data[pc].clone());
                                }
                            }
                            pc += 1;
                        }
                        _ => {
                            self.exception(ERR_INVALID_ACCESS, self.instructions_data[pc].clone());
                        }
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
                        Value::Dict(dict) => {
                            dict.0.borrow_mut().elements.insert(accessor, val);
                            pc += 1;
                        }
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
                        _ => {
                            self.exception(ERR_EXPECTED_CLASS, self.instructions_data[pc].clone());
                            unreachable!();
                        }
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
                        self.exception(ERR_EXPECTED_STRING, self.instructions_data[pc].clone());
                        unreachable!();
                    };
                    match val_b.get(prop) {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => {
                            self.exception(e, self.instructions_data[pc].clone());
                        }
                    }
                    pc += 1;
                }
                OpCode::SetObj => {
                    let val_a = match &self.activation_records[sp + inst.a as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let val_b = match &self.activation_records[sp + inst.b as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    let val_c: Value = match &self.activation_records[sp + inst.c as usize] {
                        Record::Ref(v) => v.0.borrow().clone(),
                        Record::Val(v) => v.clone(),
                    };
                    match val_a {
                        Value::Number(n) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::Bool(b) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::String(s) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::List(l) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::Dict(d) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::Native(n) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::Class(c) => {
                            self.exception(ERR_READ_ONLY, self.instructions_data[pc].clone());
                        }
                        Value::Object(o) => {
                            if let Value::String(s) = val_b {
                                o.0.borrow_mut().fields.insert(s.s.clone(), val_c);
                            } else {
                                self.exception(
                                    ERR_EXPECTED_STRING,
                                    self.instructions_data[pc].clone(),
                                );
                            }
                        }
                        _ => {
                            self.exception(ERR_EXPECTED_OBJECT, self.instructions_data[pc].clone());
                        }
                    };
                    pc += 1;
                }
                OpCode::Addi => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    match val_b
                        .as_val()
                        .add(&mut Value::Number(NumberValue { n: inst.c as f64 }))
                    {
                        Ok(v) => {
                            self.activation_records[sp + inst.a as usize] = Record::Val(v);
                        }
                        Err(e) => {
                            self.exception(e, self.instructions_data[pc].clone());
                        }
                    }
                    pc += 1;
                }
                OpCode::GetIter => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap()
                        .clone();
                    let n = if let Value::Number(n) = val_c.as_val() {
                        n.n as usize
                    } else {
                        self.exception(ERR_EXPECTED_NUMBER, self.instructions_data[pc].clone());
                        0
                    };
                    match val_b.as_val() {
                        Value::Dict(d) => {
                            let dict = d.0.borrow();
                            let mut iter = dict.elements.iter().skip(n).peekable();
                            let elms = iter.peek().unwrap();
                            let mut dict_value = DictValue {
                                elements: HashMap::new(),
                            };
                            dict_value.elements.insert(
                                Value::String(StringValue {
                                    s: "key".to_string(),
                                }),
                                elms.0.clone(),
                            );
                            dict_value.elements.insert(
                                Value::String(StringValue {
                                    s: "value".to_string(),
                                }),
                                elms.1.clone(),
                            );
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(Value::Dict(MutValue::new(dict_value)));
                        }
                        Value::List(l) => {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(l.0.borrow().elements[n].clone())
                        }
                        _ => {
                            self.exception(
                                ERR_EXPECTED_COLLECTION,
                                self.instructions_data[pc].clone(),
                            );
                        }
                    }
                    pc += 1;
                }
                OpCode::GetIterk => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let val_c = self
                        .activation_records
                        .get_mut(sp + inst.c as usize)
                        .unwrap()
                        .clone();
                    let n = if let Value::Number(n) = val_c.as_val() {
                        n.n as usize
                    } else {
                        self.exception(ERR_EXPECTED_NUMBER, self.instructions_data[pc].clone());
                        0
                    };
                    match val_b.as_val() {
                        Value::Dict(d) => {
                            let dict = d.0.borrow();
                            let mut iter = dict.elements.iter().skip(n).peekable();
                            let elms = iter.peek().unwrap();
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(elms.0.clone());
                        }
                        Value::List(l) => {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(l.0.borrow().elements[n].clone())
                        }
                        _ => {
                            self.exception(
                                ERR_EXPECTED_COLLECTION,
                                self.instructions_data[pc].clone(),
                            );
                        }
                    }
                    pc += 1;
                }
                OpCode::GetIteri => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let n = inst.c as usize;
                    match val_b.as_val() {
                        Value::Dict(d) => {
                            let k = Value::String(StringValue {
                                s: "key".to_string(),
                            });
                            let v = Value::String(StringValue {
                                s: "value".to_string(),
                            });
                            if n == 0 {
                                self.activation_records[sp + inst.a as usize] =
                                    Record::Val(d.0.borrow().elements.get(&k).unwrap().clone());
                            } else if n == 1 {
                                self.activation_records[sp + inst.a as usize] =
                                    Record::Val(d.0.borrow().elements.get(&v).unwrap().clone());
                            } else {
                                self.exception(
                                    ERR_EXPECTED_IDENTIFIERS_DICT,
                                    self.instructions_data[pc].clone(),
                                );
                            }
                        }
                        Value::List(l) => {
                            if let Some(e) = l.0.borrow().elements.get(n) {
                                self.activation_records[sp + inst.a as usize] =
                                    Record::Val(e.clone());
                            } else {
                                self.exception(
                                    ERR_WRONG_NUMBER_OF_VALUES,
                                    self.instructions_data[pc].clone(),
                                );
                            }
                        }
                        _ => {
                            self.exception(ERR_CANNOT_UNPACK, self.instructions_data[pc].clone());
                        }
                    }
                    pc += 1;
                }
                OpCode::Length => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    match val_b.as_val() {
                        Value::Dict(d) => {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(Value::Number(NumberValue {
                                    n: d.0.borrow().elements.len() as f64,
                                }));
                        }
                        Value::List(l) => {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(Value::Number(NumberValue {
                                    n: l.0.borrow().elements.len() as f64,
                                }));
                        }
                        _ => {
                            self.exception(
                                ERR_EXPECTED_COLLECTION,
                                self.instructions_data[pc].clone(),
                            );
                        }
                    }
                    pc += 1;
                }
                OpCode::Super => {
                    let val_b = self
                        .activation_records
                        .get_mut(sp + inst.b as usize)
                        .unwrap()
                        .clone();
                    let prop = if let Value::String(s) = val_b.as_val() {
                        s.s
                    } else {
                        self.exception(ERR_EXPECTED_STRING, self.instructions_data[pc].clone());
                        unreachable!();
                    };
                    let _obj = this.as_ref().unwrap().clone();
                    let o = _obj.0.borrow();
                    let cls = o.class.0.borrow();
                    if let Some(supercls) = &cls.superclass {
                        let scls = supercls.0.borrow();
                        if let Some(m) = scls.find_method(prop) {
                            self.activation_records[sp + inst.a as usize] =
                                Record::Val(m.0.borrow().bind(_obj.clone()));
                        } else {
                            self.exception(ERR_METHOD_NOT_FOUND, self.instructions_data[pc].clone())
                        }
                    } else {
                        self.exception(ERR_EXPECTED_SUPERCLASS, self.instructions_data[pc].clone());
                    }
                    pc += 1;
                }
                OpCode::This => {
                    self.activation_records[sp + inst.a as usize] =
                        Record::Val(Value::Object(this.as_ref().unwrap().clone()));
                    pc += 1;
                }
                OpCode::GetGlobal => {
                    if let Value::String(s) = &self.constants[inst.bx() as usize] {
                        if let Some(g) = self.globals.get(&s.s) {
                            self.activation_records[sp + inst.a as usize] = Record::Val(g.clone());
                        } else {
                            self.exception(ERR_UNDEFINED_VAR, self.instructions_data[pc].clone());
                        }
                    } else {
                        self.exception(ERR_EXPECTED_STRING, self.instructions_data[pc].clone());
                    }
                    pc += 1;
                }
                OpCode::SetGlobal => {
                    if let Value::String(s) = &self.constants[inst.bx() as usize] {
                        self.globals.insert(
                            s.s.clone(),
                            self.activation_records[sp + inst.a as usize].as_val(),
                        );
                    } else {
                        self.exception(ERR_EXPECTED_STRING, self.instructions_data[pc].clone());
                    }
                    pc += 1;
                }
                OpCode::GetCurrentFunc => {
                    self.activation_records[sp + inst.a as usize] = Record::Val(Value::Fn(
                        self.stack.last().unwrap().function.clone().unwrap(),
                    ));
                    pc += 1;
                }
                OpCode::GetBuiltin => {
                    if let Value::String(s) = &self.constants[inst.bx() as usize] {
                        self.activation_records[sp + inst.a as usize] =
                            Record::Val(self.builtins.get(&s.s).unwrap().clone());
                    } else {
                        self.exception(ERR_EXPECTED_STRING, self.instructions_data[pc].clone());
                    }
                    pc += 1;
                }
            }
        }
    }

    // pub fn extern_call(&self, prototype: u16, values: Vec<Value>) -> Result<Value, RuntimeErr> {}

    pub fn exception(&self, error: RuntimeErr, token: Option<TokenData>) {
        match token {
            Some(tk) => {
                print!(
                    "Runtime Error on line {}\n\t{}: {}\n",
                    tk.line, error.msg, tk.lexeme
                );
            }
            None => {
                print!("Runtime Error\n\t{}\n", error.msg);
            }
        }
        std::process::exit(0);
    }
}
