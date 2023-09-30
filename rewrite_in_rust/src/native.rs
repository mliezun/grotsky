use std::{
    collections::HashMap,
    time::{Instant, SystemTime},
};

use crate::value::{NativeValue, NumberValue, Value};

pub struct IO {}

impl IO {
    fn println(values: Vec<Value>) -> Value {
        if values.is_empty() {
            panic!("Arguments should not be empty");
        }
        let text = values
            .iter()
            .map(|v| v.string())
            .reduce(|v1, v2| v1 + " " + &v2)
            .unwrap();
        println!("{}", text);
        return Value::Nil;
    }

    fn clock(values: Vec<Value>) -> Value {
        if !values.is_empty() {
            panic!("Should be empty");
        }
        return Value::Number(NumberValue {
            n: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs_f64(),
        });
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
