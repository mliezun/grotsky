use std::collections::HashMap;

use crate::value::{NativeValue, Value};

pub struct IO {}

impl IO {
    fn println(values: Vec<Value>) -> Value {
        if values.is_empty() {
            panic!("Arguments should not be empty");
        }
        let text = values
            .iter()
            .map(|v| v.repr())
            .reduce(|v1, v2| v1 + " " + &v2)
            .unwrap();
        println!("{}", text);
        return Value::Nil;
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
        io.props
            .insert("println".to_string(), Value::Native(println));
        return io;
    }
}
