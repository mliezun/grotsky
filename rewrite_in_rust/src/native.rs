use std::{collections::HashMap, time::SystemTime};

use crate::{
    errors::ERR_INVALID_NUMBER_ARGUMENTS,
    errors::{RuntimeErr, ERR_EXPECTED_STRING},
    value::{ListValue, MutValue, NativeValue, NumberValue, StringValue, Value},
};

pub struct IO {}

impl IO {
    fn println(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.is_empty() {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let text = values
            .iter()
            .map(|v| v.string())
            .reduce(|v1, v2| v1 + " " + &v2)
            .unwrap();
        println!("{}", text);
        return Ok(Value::Nil);
    }

    fn clock(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if !values.is_empty() {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        return Ok(Value::Number(NumberValue {
            n: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        }));
    }

    pub fn build() -> NativeValue {
        let mut io = NativeValue {
            props: HashMap::new(),
            callable: None,
        };
        let println = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::println),
        };
        let clock = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::clock),
        };
        io.props
            .insert("println".to_string(), Value::Native(println));
        io.props.insert("clock".to_string(), Value::Native(clock));
        return io;
    }
}

pub struct Strings {}

impl Strings {
    pub fn to_lower(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = string_value.s.to_lowercase();
        return Ok(Value::String(StringValue { s: result }));
    }

    pub fn to_upper(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = string_value.s.to_uppercase();
        return Ok(Value::String(StringValue { s: result }));
    }

    pub fn ord(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = string_value.s.as_bytes()[0];
        return Ok(Value::Number(NumberValue { n: result as f64 }));
    }

    pub fn chr(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let number_value = match values.first().unwrap() {
            Value::Number(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        return Ok(Value::String(StringValue {
            s: unsafe { String::from_utf8_unchecked(vec![number_value.n as u8]) },
        }));
    }

    pub fn as_number(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        match string_value.s.parse::<f64>() {
            Ok(n) => Ok(Value::Number(NumberValue { n })),
            Err(_) => Ok(Value::Nil),
        }
    }

    pub fn split(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let str = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let sep = match values.get(1).unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = str
            .s
            .split(sep.s.as_str())
            .map(|s| Value::String(StringValue { s: s.to_string() }))
            .collect::<Vec<Value>>();
        return Ok(Value::List(MutValue::new(ListValue { elements: result })));
    }

    pub fn build() -> NativeValue {
        let mut strings = NativeValue {
            props: HashMap::new(),
            callable: None,
        };
        let to_lower = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::to_lower),
        };
        let to_upper = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::to_upper),
        };
        let ord = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::ord),
        };
        let chr = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::chr),
        };
        let as_number = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::as_number),
        };
        let split = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::split),
        };
        strings
            .props
            .insert("toLower".to_string(), Value::Native(to_lower));
        strings
            .props
            .insert("toUpper".to_string(), Value::Native(to_upper));
        strings.props.insert("ord".to_string(), Value::Native(ord));
        strings.props.insert("chr".to_string(), Value::Native(chr));
        strings
            .props
            .insert("asNumber".to_string(), Value::Native(as_number));
        strings
            .props
            .insert("split".to_string(), Value::Native(split));
        return strings;
    }
}

pub struct Type {}

impl Type {
    pub fn type_fn(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        match values[0] {
            Value::Class(_) => Ok(Value::String(StringValue {
                s: "class".to_string(),
            })),
            Value::Dict(_) => Ok(Value::String(StringValue {
                s: "dict".to_string(),
            })),
            Value::List(_) => Ok(Value::String(StringValue {
                s: "list".to_string(),
            })),
            Value::Fn(_) => Ok(Value::String(StringValue {
                s: "function".to_string(),
            })),
            Value::Native(_) => Ok(Value::String(StringValue {
                s: "native".to_string(),
            })),
            Value::Number(_) => Ok(Value::String(StringValue {
                s: "number".to_string(),
            })),
            Value::String(_) => Ok(Value::String(StringValue {
                s: "string".to_string(),
            })),
            Value::Bool(_) => Ok(Value::String(StringValue {
                s: "bool".to_string(),
            })),
            Value::Slice(_) => Ok(Value::String(StringValue {
                s: "slice".to_string(),
            })),
            Value::Nil => Ok(Value::String(StringValue {
                s: "nil".to_string(),
            })),
        }
    }

    pub fn build() -> NativeValue {
        let type_fn = NativeValue {
            props: HashMap::new(),
            callable: Some(&Type::type_fn),
        };
        return type_fn;
    }
}
