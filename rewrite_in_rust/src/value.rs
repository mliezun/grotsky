use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub struct MutValue<T>(pub Rc<RefCell<T>>);

impl<T> MutValue<T> {
    pub fn new(obj: T) -> Self {
        MutValue::<T>(Rc::new(RefCell::new(obj)))
    }
}

#[derive(Debug, Clone)]
pub enum Value {
    Class(MutValue<ClassValue>),
    // Object(MutValue<ObjectValue>),
    // Dict(MutValue<DictValue>),
    // List(MutValue<ListValue>),
    Fn(MutValue<FnValue>),
    // Native(NativeValue),
    Number(NumberValue),
    String(StringValue),
    Bool(BoolValue),
    Nil,
}

impl Value {
    pub fn add(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n + other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn sub(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n - other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn mul(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n * other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn div(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n / other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn pow(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n.powf(other_val.n),
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn modulo(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n % other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn lt(&mut self, other: &mut Value) -> Value {
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

pub fn truthy(val: &Value) -> bool {
    if let Value::Number(num_val) = val {
        return num_val.n != 0.0;
    }
    return false;
}

#[derive(Debug, Clone)]
pub struct NumberValue {
    pub n: f64,
}

#[derive(Debug, Clone)]
pub struct BoolValue {
    pub b: bool,
}

#[derive(Debug, Clone)]
pub struct FnValue {
    pub prototype: u16,
    pub upvalues: HashMap<String, Upvalue>,
    pub constants: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct StringValue {
    pub s: String,
}

#[derive(Debug, Clone)]
pub struct ClassValue {
    name: String,
    superclass: Option<MutValue<ClassValue>>,
    methods: Vec<MutValue<FnValue>>,
    classmethods: Vec<MutValue<FnValue>>,
}

#[derive(Debug, Clone)]
pub struct Upvalue {
    value: *mut Value,
    closed_value: Option<Value>, // Only active when the upvalue is closed
}
