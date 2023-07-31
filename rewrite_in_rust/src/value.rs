use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
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
    Dict(MutValue<DictValue>),
    List(MutValue<ListValue>),
    Fn(MutValue<FnValue>),
    // Native(NativeValue),
    Number(NumberValue),
    String(StringValue),
    Bool(BoolValue),
    Nil,
}

impl Hash for Value {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Value::Number(val) => state.write_u64(val.n as u64),
            Value::String(val) => val.s.hash(state),
            Value::Bool(val) => state.write_u8(val.b as u8),
            _ => unimplemented!(),
        };
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        let _self = &mut self.clone();
        return truthy(&_self.equal(&mut other.clone()));
    }
}

impl Eq for Value {}

impl Value {
    pub fn add(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Number(NumberValue {
                    n: num_val.n + other_val.n,
                });
            }
        }
        println!("{:#?} + {:#?}", self, other);
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
                return Value::Bool(BoolValue {
                    b: num_val.n < other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn lte(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n <= other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn gt(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n > other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn gte(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n >= other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn equal(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n == other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn nequal(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n != other_val.n,
                });
            }
        }
        panic!("Not implemented");
    }
}

pub fn truthy(val: &Value) -> bool {
    match val {
        Value::Number(val) => val.n != 0.0,
        Value::Bool(val) => val.b,
        Value::String(val) => !val.s.is_empty(),
        Value::Nil => false,
        _ => true,
    }
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
    pub upvalues: Vec<MutValue<Value>>,
    pub constants: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct StringValue {
    pub s: String,
}

#[derive(Debug, Clone)]
pub struct ClassValue {
    pub name: String,
    pub superclass: Option<MutValue<ClassValue>>,
    pub methods: Vec<MutValue<FnValue>>,
    pub classmethods: Vec<MutValue<FnValue>>,
}

#[derive(Debug, Clone)]
pub struct ListValue {
    pub elements: Vec<Value>,
}

#[derive(Debug, Clone)]
pub struct DictValue {
    pub elements: HashMap<Value, Value>,
}

#[derive(Debug, Clone)]
pub struct UpValue {
    pub rec: usize,
    pub value: Option<Value>,
}
