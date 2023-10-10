use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;

use crate::errors::{
    RuntimeErr, ERR_EXPECTED_DICT, ERR_EXPECTED_LIST, ERR_EXPECTED_NUMBER, ERR_EXPECTED_OBJECT,
    ERR_EXPECTED_STEP, ERR_EXPECTED_STRING, ERR_ONLY_NUMBERS, ERR_UNDEFINED_OP, ERR_UNDEFINED_PROP,
};

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
    Native(NativeValue),
    Number(NumberValue),
    String(StringValue),
    Bool(BoolValue),
    Slice(SliceValue),
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
    pub fn repr(&self) -> String {
        match self {
            Value::String(s) => format!("{:?}", s.s),
            _ => self.string(),
        }
    }

    pub fn string(&self) -> String {
        match self {
            Value::String(s) => s.s.clone(),
            Value::Number(n) => n.n.to_string(),
            Value::Bool(b) => b.b.to_string(),
            Value::List(l) => {
                format!(
                    "[{}]",
                    l.0.borrow()
                        .elements
                        .iter()
                        .map(|x| x.repr())
                        .reduce(|acc, e| acc + ", " + &e)
                        .unwrap_or("".to_string())
                )
            }
            Value::Dict(d) => {
                format!(
                    "{{{}}}",
                    d.0.borrow()
                        .elements
                        .iter()
                        .map(|(k, v)| format!("{}: {}", k.repr(), v.repr()))
                        .reduce(|acc, e| acc + ", " + &e)
                        .unwrap_or("".to_string())
                )
            }
            Value::Fn(f) => "<fn anonymous>".to_string(),
            Value::Nil => "<nil>".to_string(),
            _ => unimplemented!(),
        }
    }

    pub fn get(&self, prop: String) -> Result<Value, RuntimeErr> {
        if let Value::Number(n) = self {
            return Err(ERR_UNDEFINED_PROP);
        }
        if let Value::List(l) = self {
            if prop == "length" {
                return Ok(Value::Number(NumberValue {
                    n: l.0.borrow().elements.len() as f64,
                }));
            } else {
                return Err(ERR_UNDEFINED_PROP);
            }
        }
        if let Value::String(s) = self {
            if prop == "length" {
                return Ok(Value::Number(NumberValue {
                    n: s.s.len() as f64,
                }));
            } else {
                return Err(ERR_UNDEFINED_PROP);
            }
        }
        if let Value::Dict(d) = self {
            if prop == "length" {
                return Ok(Value::Number(NumberValue {
                    n: d.0.borrow().elements.len() as f64,
                }));
            } else {
                return Err(ERR_UNDEFINED_PROP);
            }
        }
        if let Value::Native(n) = self {
            return Ok(n.props.get(&prop).unwrap().clone());
        }
        return Err(ERR_EXPECTED_OBJECT);
    }

    pub fn add(&mut self, other: &mut Value) -> Result<Value, RuntimeErr> {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Ok(Value::Number(NumberValue {
                    n: num_val.n + other_val.n,
                }));
            } else {
                return Err(ERR_EXPECTED_NUMBER);
            }
        }
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Ok(Value::String(StringValue {
                    s: str_val.s.clone() + &other_val.s,
                }));
            } else {
                return Err(ERR_EXPECTED_STRING);
            }
        }
        if let Value::List(list_val) = self {
            match other {
                Value::List(other_val) => {
                    let mut elements = vec![];
                    for e in list_val.0.borrow().elements.iter() {
                        elements.push(e.clone());
                    }
                    for e in other_val.0.borrow().elements.iter() {
                        elements.push(e.clone());
                    }
                    return Ok(Value::List(MutValue::new(ListValue { elements: elements })));
                }
                Value::Nil => {
                    return Err(ERR_UNDEFINED_OP);
                }
                _ => {
                    return Err(ERR_EXPECTED_LIST);
                }
            }
        }
        if let Value::Dict(dict_val) = self {
            if let Value::Dict(other_val) = other {
                let mut elements = HashMap::new();
                for (k, v) in dict_val.0.borrow().elements.iter() {
                    elements.insert(k.clone(), v.clone());
                }
                for (k, v) in other_val.0.borrow().elements.iter() {
                    elements.insert(k.clone(), v.clone());
                }
                return Ok(Value::Dict(MutValue::new(DictValue { elements: elements })));
            } else {
                return Err(ERR_EXPECTED_DICT);
            }
        }
        return Err(ERR_UNDEFINED_OP);
    }
    pub fn sub(&mut self, other: &mut Value) -> Result<Value, RuntimeErr> {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Ok(Value::Number(NumberValue {
                    n: num_val.n - other_val.n,
                }));
            } else {
                return Err(ERR_EXPECTED_NUMBER);
            }
        }
        if let Value::List(list_val) = self {
            if let Value::List(other_val) = other {
                let mut elements = HashSet::new();
                for e in list_val.0.borrow().elements.iter() {
                    elements.insert(e.clone());
                }
                for e in other_val.0.borrow().elements.iter() {
                    elements.remove(e);
                }
                return Ok(Value::List(MutValue::new(ListValue {
                    elements: elements.into_iter().collect(),
                })));
            }
        }
        return Err(ERR_UNDEFINED_OP);
    }
    pub fn mul(&mut self, other: &mut Value) -> Result<Value, RuntimeErr> {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Ok(Value::Number(NumberValue {
                    n: num_val.n * other_val.n,
                }));
            }
        }
        return Err(ERR_UNDEFINED_OP);
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
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Value::Bool(BoolValue {
                    b: str_val.s < other_val.s,
                });
            }
        }
        panic!("Not implemented");
    }
    pub fn lte(&mut self, other: &mut Value) -> Result<Value, RuntimeErr> {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Ok(Value::Bool(BoolValue {
                    b: num_val.n <= other_val.n,
                }));
            }
        }
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Ok(Value::Bool(BoolValue {
                    b: str_val.s <= other_val.s,
                }));
            }
        }
        return Err(ERR_UNDEFINED_OP);
    }
    pub fn gt(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n > other_val.n,
                });
            }
        }
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Value::Bool(BoolValue {
                    b: str_val.s > other_val.s,
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
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Value::Bool(BoolValue {
                    b: str_val.s >= other_val.s,
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
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Value::Bool(BoolValue {
                    b: str_val.s == other_val.s,
                });
            }
        }
        if let Value::Bool(bool_val) = self {
            if let Value::Bool(other_val) = other {
                return Value::Bool(BoolValue {
                    b: bool_val.b == other_val.b,
                });
            }
        }
        if let Value::List(list_val) = self {
            if let Value::List(other_val) = other {
                let elements = &list_val.0.borrow().elements;
                let other_elements = &other_val.0.borrow().elements;
                let result = std::ptr::eq(elements.as_ptr(), other_elements.as_ptr());
                return Value::Bool(BoolValue { b: result });
            }
        }
        if let Value::Dict(dict_val) = self {
            if let Value::Dict(other_val) = other {
                let elements = &dict_val.0.borrow().elements;
                let other_elements = &other_val.0.borrow().elements;
                let result = std::ptr::eq(
                    core::ptr::addr_of!(elements),
                    core::ptr::addr_of!(other_elements),
                );
                return Value::Bool(BoolValue { b: result });
            }
        }
        return match self {
            Value::Nil => match other {
                Value::Nil => Value::Bool(BoolValue { b: true }),
                _ => Value::Bool(BoolValue { b: false }),
            },
            _ => Value::Bool(BoolValue { b: false }),
        };
    }
    pub fn nequal(&mut self, other: &mut Value) -> Value {
        if let Value::Number(num_val) = self {
            if let Value::Number(other_val) = other {
                return Value::Bool(BoolValue {
                    b: num_val.n != other_val.n,
                });
            }
        }
        if let Value::String(str_val) = self {
            if let Value::String(other_val) = other {
                return Value::Bool(BoolValue {
                    b: str_val.s != other_val.s,
                });
            }
        }
        if let Value::Bool(bool_val) = self {
            if let Value::Bool(other_val) = other {
                return Value::Bool(BoolValue {
                    b: bool_val.b != other_val.b,
                });
            }
        }
        if let Value::List(list_val) = self {
            if let Value::List(other_val) = other {
                let elements = &list_val.0.borrow().elements;
                let other_elements = &other_val.0.borrow().elements;
                let result = !std::ptr::eq(elements.as_ptr(), other_elements.as_ptr());
                return Value::Bool(BoolValue { b: result });
            }
        }
        if let Value::Dict(dict_val) = self {
            if let Value::Dict(other_val) = other {
                let elements = &dict_val.0.borrow().elements;
                let other_elements = &other_val.0.borrow().elements;
                let result = !std::ptr::eq(
                    core::ptr::addr_of!(elements),
                    core::ptr::addr_of!(other_elements),
                );
                return Value::Bool(BoolValue { b: result });
            }
        }
        return match self {
            Value::Nil => match other {
                Value::Nil => Value::Bool(BoolValue { b: false }),
                _ => Value::Bool(BoolValue { b: true }),
            },
            _ => Value::Bool(BoolValue { b: true }),
        };
    }
    pub fn neg(&mut self) -> Result<Value, RuntimeErr> {
        if let Value::Number(num_val) = self {
            return Ok(Value::Number(NumberValue { n: -num_val.n }));
        }
        if let Value::List(list_val) = self {
            let mut elements: HashSet<Value> = HashSet::new();
            for element in list_val.0.borrow().elements.iter() {
                elements.insert(element.clone());
            }
            return Ok(Value::List(MutValue::new(ListValue {
                elements: elements.into_iter().collect(),
            })));
        }
        return Err(ERR_UNDEFINED_OP);
    }
    pub fn not(&mut self) -> Value {
        return match self {
            Value::Bool(bool_val) => Value::Bool(BoolValue { b: !bool_val.b }),
            _ => Value::Bool(BoolValue { b: !truthy(self) }),
        };
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

impl StringValue {
    pub fn access(&self, accesor: Value) -> Result<Value, RuntimeErr> {
        match accesor {
            Value::Number(val) => Ok(Value::String(StringValue {
                s: String::from(self.s.get((val.n as usize)..(1 + val.n as usize)).unwrap()),
            })),
            Value::Slice(val) => {
                let mut result_str = "".to_string();
                match val.as_range() {
                    Ok((range, step)) => {
                        if step > self.s.len() {
                            return Ok(Value::String(StringValue { s: result_str }));
                        }
                        for i in range.step_by(step) {
                            if i >= self.s.len() {
                                break;
                            }
                            result_str.push_str(self.s.get(i..i + 1).unwrap());
                        }
                        Ok(Value::String(StringValue { s: result_str }))
                    }
                    Err(e) => Err(e),
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ClassValue {
    pub name: String,
    pub superclass: Option<MutValue<ClassValue>>,
    pub methods: HashMap<String, MutValue<FnValue>>,
    pub classmethods: HashMap<String, MutValue<FnValue>>,
}

#[derive(Debug, Clone)]
pub struct ListValue {
    pub elements: Vec<Value>,
}

impl ListValue {
    pub fn access(&self, accesor: Value) -> Result<Value, RuntimeErr> {
        // println!("accessor = {:#?}", accesor);
        match accesor {
            Value::Number(val) => {
                if self.elements.is_empty() {
                    return Ok(Value::List(MutValue::new(ListValue { elements: vec![] })));
                }
                return Ok(self.elements[val.n as usize].clone());
            }
            Value::Slice(slice) => {
                let mut elements = vec![];
                match slice.as_range() {
                    Ok((range, step)) => {
                        if step > self.elements.len() {
                            return Ok(Value::List(MutValue::new(ListValue {
                                elements: elements,
                            })));
                        }
                        for i in range.step_by(step) {
                            if i >= self.elements.len() {
                                break;
                            }
                            elements.push(self.elements[i].clone());
                        }
                        return Ok(Value::List(MutValue::new(ListValue { elements: elements })));
                    }
                    Err(e) => Err(e),
                }
            }
            _ => unimplemented!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DictValue {
    pub elements: HashMap<Value, Value>,
}

impl DictValue {
    pub fn access(&self, accesor: Value) -> Value {
        if self.elements.is_empty() {
            return Value::Dict(MutValue::new(DictValue {
                elements: HashMap::new(),
            }));
        }
        self.elements.get(&accesor).unwrap().clone()
    }
}

#[derive(Debug, Clone)]
pub struct SliceValue {
    pub first: Rc<Value>,
    pub second: Rc<Value>,
    pub third: Rc<Value>,
}

impl SliceValue {
    fn as_range(&self) -> Result<(Range<usize>, usize), RuntimeErr> {
        let first = match &*self.first {
            Value::Number(val) => Some(val.n as usize),
            Value::Nil => None,
            _ => return Err(ERR_ONLY_NUMBERS),
        };
        let second = match &*self.second {
            Value::Number(val) => Some(val.n as usize),
            Value::Nil => None,
            _ => return Err(ERR_ONLY_NUMBERS),
        };
        let mut third = match &*self.third {
            Value::Number(val) => val.n as usize,
            Value::Nil => return Err(ERR_EXPECTED_STEP),
            _ => return Err(ERR_ONLY_NUMBERS),
        };
        let range = if first.is_some() && second.is_some() {
            first.unwrap()..second.unwrap()
        } else if first.is_some() && second.is_none() {
            first.unwrap()..usize::MAX
        } else if first.is_none() && second.is_some() {
            0..second.unwrap()
        } else {
            0..usize::MAX
        };
        if third <= 1 {
            third = 1;
        }
        return Ok((range, third));
    }
}

#[derive(Clone)]
pub struct NativeValue {
    pub props: HashMap<String, Value>,
    pub callable: Option<&'static dyn Fn(Vec<Value>) -> Value>,
}

impl core::fmt::Debug for NativeValue {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        if self.callable.is_some() {
            write!(f, "<native fn>")
        } else {
            write!(f, "NativeValue({:#?})", self.props)
        }
    }
}
