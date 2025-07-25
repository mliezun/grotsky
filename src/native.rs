use socket2::{Domain, Socket};
use std::cmp::Ordering;
use std::io::{stdin, BufRead, Read, Write};
use std::net::{Shutdown, ToSocketAddrs};
use std::ops::DerefMut;
use std::{
    cell::RefCell, collections::HashMap, env, fs::canonicalize, ops::Deref, rc::Rc,
    time::SystemTime,
};
use regex::Regex;

use crate::errors::{ERR_EXPECTED_LIST, ERR_EXPECTED_NUMBER, ERR_LIST_EMPTY};
use crate::value::{BoolValue, BytesValue, DictValue};
use crate::{
    errors::ERR_INVALID_NUMBER_ARGUMENTS,
    errors::{RuntimeErr, ERR_EXPECTED_OBJECT, ERR_EXPECTED_STRING},
    interpreter,
    value::{ListValue, MutValue, NativeBaggage, NativeValue, NumberValue, StringValue, Value},
};
use std::fs;
use std::path::Path;

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

    fn readln(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if !values.is_empty() {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let stdin = stdin();
        let line = stdin.lock()
            .lines()
            .next();
        if line.is_none() {
            return Ok(Value::Nil);
        }
        let line_unwrapped = line.unwrap();
        if line_unwrapped.is_err() {
            return Err(RuntimeErr{
                msg: "Could not read from stdin",
                signal: None,
            });
        }
        return Ok(Value::String(StringValue::new(line_unwrapped.unwrap())));
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

    fn read_file(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        match fs::read_to_string(string_value.s.as_str()) {
            Ok(content) => Ok(Value::String(StringValue::new(content))),
            Err(e) => {
                if let std::io::ErrorKind::InvalidData = e.kind() {
                    match fs::read(string_value.s.as_str()) {
                        Ok(c) => Ok(Value::Bytes(BytesValue { s: c })),
                        Err(_) => Err(RuntimeErr {
                            msg: "Cannot read file",
                            signal: None,
                        }),
                    }
                } else {
                    Err(RuntimeErr {
                        msg: "Cannot read file",
                        signal: None,
                    })
                }
            }
        }
    }

    fn write_file(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let path = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let content = match &values[1] {
            Value::String(s) => s.s.as_bytes(),
            Value::Bytes(s) => &s.s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        match fs::write(path.s.as_str(), content) {
            Ok(_) => Ok(Value::Nil),
            Err(_) => Err(RuntimeErr {
                msg: "Cannot write file",
                signal: None,
            }),
        }
    }

    fn list_dir(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let path = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        match fs::read_dir(path.s.as_str()) {
            Ok(content) => {
                let mut list = ListValue { elements: vec![] };
                for d in content {
                    let mut dict = HashMap::new();
                    let file = d.unwrap();
                    let file_metadata = file.metadata().unwrap();
                    let file_name = file.file_name().into_string().unwrap();
                    dict.insert(
                        Value::String(StringValue::new("name".to_string())),
                        Value::String(StringValue::new(file_name)),
                    );
                    dict.insert(
                        Value::String(StringValue::new("size".to_string())),
                        Value::Number(NumberValue {
                            n: file_metadata.len() as f64,
                        }),
                    );
                    dict.insert(
                        Value::String(StringValue::new("is_dir".to_string())),
                        Value::Bool(BoolValue {
                            b: file_metadata.is_dir(),
                        }),
                    );
                    list.elements
                        .push(Value::Dict(MutValue::new(DictValue { elements: dict })));
                }
                Ok(Value::List(MutValue::new(list)))
            }
            Err(_) => Err(RuntimeErr {
                msg: "Cannot read dir",
                signal: None,
            }),
        }
    }

    fn file_exists(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let path = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        match fs::metadata(&path.s) {
            Ok(_) => Ok(Value::Bool(BoolValue { b: true })),
            Err(_) => Ok(Value::Bool(BoolValue { b: false })),
        }
    }

    fn mkdir_all(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let path = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let _perm = match &values[1] {
            Value::Number(n) => n,
            _ => {
                return Err(ERR_EXPECTED_NUMBER);
            }
        };
        match fs::create_dir_all(&path.s) {
            Ok(_) => Ok(Value::Nil),
            Err(_) => Err(RuntimeErr {
                msg: "Cannot create file",
                signal: None,
            }),
        }
    }

    pub fn build() -> NativeValue {
        let mut io = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        let println = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::println),
            bind: false,
            baggage: None,
        };
        let readln = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::readln),
            bind: false,
            baggage: None,
        };
        let clock = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::clock),
            bind: false,
            baggage: None,
        };
        let read_file = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::read_file),
            bind: false,
            baggage: None,
        };
        let write_file = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::write_file),
            bind: false,
            baggage: None,
        };
        let list_dir = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::list_dir),
            bind: false,
            baggage: None,
        };
        let file_exists = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::file_exists),
            bind: false,
            baggage: None,
        };
        let mkdir_all = NativeValue {
            props: HashMap::new(),
            callable: Some(&IO::mkdir_all),
            bind: false,
            baggage: None,
        };
        io.props
            .insert("println".to_string(), Value::Native(println));
        io.props
            .insert("readln".to_string(), Value::Native(readln));
        io.props.insert("clock".to_string(), Value::Native(clock));
        io.props
            .insert("readFile".to_string(), Value::Native(read_file));
        io.props
            .insert("writeFile".to_string(), Value::Native(write_file));
        io.props
            .insert("listDir".to_string(), Value::Native(list_dir));
        io.props
            .insert("fileExists".to_string(), Value::Native(file_exists));
        io.props
            .insert("mkdirAll".to_string(), Value::Native(mkdir_all));
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
        return Ok(Value::String(StringValue::new(result)));
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
        return Ok(Value::String(StringValue::new(result)));
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
        return Ok(Value::String(StringValue::new(
            unsafe { String::from_utf8_unchecked(vec![number_value.n as u8]) },
        )));
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
            .map(|s| Value::String(StringValue::new(s.to_string())))
            .collect::<Vec<Value>>();
        return Ok(Value::List(MutValue::new(ListValue { elements: result })));
    }

    pub fn compare(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let str1 = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let str2 = match values.get(1).unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = match str1.s.cmp(&str2.s) {
            Ordering::Less => Ok(Value::Number(NumberValue { n: -1.0 })),
            Ordering::Equal => Ok(Value::Number(NumberValue { n: 0.0 })),
            Ordering::Greater => Ok(Value::Number(NumberValue { n: 1.0 })),
        };
        return result;
    }

    pub fn build() -> NativeValue {
        let mut strings = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        let to_lower = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::to_lower),
            bind: false,
            baggage: None,
        };
        let to_upper = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::to_upper),
            bind: false,
            baggage: None,
        };
        let ord = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::ord),
            bind: false,
            baggage: None,
        };
        let chr = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::chr),
            bind: false,
            baggage: None,
        };
        let as_number = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::as_number),
            bind: false,
            baggage: None,
        };
        let split = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::split),
            bind: false,
            baggage: None,
        };
        let compare = NativeValue {
            props: HashMap::new(),
            callable: Some(&Strings::compare),
            bind: false,
            baggage: None,
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
        strings
            .props
            .insert("compare".to_string(), Value::Native(compare));
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
            Value::Class(_) => Ok(Value::String(StringValue::new("class".to_string()))),
            Value::Object(_) => Ok(Value::String(StringValue::new("object".to_string()))),
            Value::Dict(_) => Ok(Value::String(StringValue::new("dict".to_string()))),
            Value::List(_) => Ok(Value::String(StringValue::new("list".to_string()))),
            Value::Fn(_) => Ok(Value::String(StringValue::new("function".to_string()))),
            Value::Native(_) => Ok(Value::String(StringValue::new("native".to_string()))),
            Value::Number(_) => Ok(Value::String(StringValue::new("number".to_string()))),
            Value::String(_) => Ok(Value::String(StringValue::new("string".to_string()))),
            Value::Bytes(_) => Ok(Value::String(StringValue::new("bytes".to_string()))),
            Value::Bool(_) => Ok(Value::String(StringValue::new("bool".to_string()))),
            Value::Slice(_) => Ok(Value::String(StringValue::new("slice".to_string()))),
            Value::Nil => Ok(Value::String(StringValue::new("nil".to_string()))),
        }
    }

    pub fn build() -> NativeValue {
        let type_fn = NativeValue {
            props: HashMap::new(),
            callable: Some(&Type::type_fn),
            bind: false,
            baggage: None,
        };
        return type_fn;
    }
}

pub struct Env {}

impl Env {
    pub fn get(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let result = env::var(string_value.s.as_str()).unwrap_or("".to_string());
        return Ok(Value::String(StringValue::new(result)));
    }

    pub fn set(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let env_var = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let val = match values.get(1).unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        env::set_var(env_var.s.as_str(), val.s.as_str());
        return Ok(Value::Nil);
    }

    pub fn build() -> NativeValue {
        let mut env_mod = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        let get = NativeValue {
            props: HashMap::new(),
            callable: Some(&Env::get),
            bind: false,
            baggage: None,
        };
        let set = NativeValue {
            props: HashMap::new(),
            callable: Some(&Env::set),
            bind: false,
            baggage: None,
        };
        env_mod.props.insert("get".to_string(), Value::Native(get));
        env_mod.props.insert("set".to_string(), Value::Native(set));
        return env_mod;
    }
}

pub struct Import {}

impl Import {
    pub fn import(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let current_abs_path = interpreter::get_absolute_path();
        let path = Path::new(&string_value.s);
        let full_path = if path.is_absolute() {
            string_value.s.clone()
        } else {
            let mut path_buf_abs = canonicalize(&current_abs_path).unwrap();
            path_buf_abs.pop();
            path_buf_abs.push(string_value.s.as_str());
            String::from(path_buf_abs.canonicalize().unwrap().to_string_lossy())
        };
        let source = match fs::read_to_string(&full_path) {
            Ok(s) => s,
            Err(_) => {
                return Err(RuntimeErr {
                    msg: "Cannot open file",
                    signal: None,
                });
            }
        };
        interpreter::set_absolute_path(full_path);
        let result = Ok(Value::Native(NativeValue {
            props: interpreter::import_module(source),
            callable: None,
            bind: false,
            baggage: None,
        }));
        interpreter::set_absolute_path(current_abs_path);
        return result;
    }

    pub fn build() -> NativeValue {
        let import_mod = NativeValue {
            props: HashMap::new(),
            callable: Some(&Self::import),
            bind: false,
            baggage: None,
        };
        return import_mod;
    }
}

pub struct Net {}

impl Net {
    fn conn_address(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_OBJECT);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        let address_str = match baggage.borrow_mut().deref() {
            NativeBaggage::TcpSocket(socket) => {
                socket.peer_addr().unwrap().as_socket().unwrap().to_string()
            }
            _ => return Err(ERR_EXPECTED_OBJECT),
        };
        return Ok(Value::String(StringValue::new(address_str)));
    }

    fn conn_read(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_OBJECT);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        let mut buf: Vec<u8> = vec![0; 4096];
        let size;
        match baggage.borrow_mut().deref_mut() {
            NativeBaggage::TcpSocket(socket) => {
                size = socket.read(&mut buf).expect("Read from connection");
            }
            _ => return Err(ERR_EXPECTED_OBJECT),
        };
        let read_from_connection = &buf[0..size];
        return Ok(Value::String(StringValue::new(
            String::from_utf8(read_from_connection.to_vec()).expect("Read as string"),
        )));
    }

    fn conn_write(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let content = match &values[1] {
            Value::String(s) => s.s.as_bytes(),
            Value::Bytes(s) => s.s.as_slice(),
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        match baggage.borrow_mut().deref_mut() {
            NativeBaggage::TcpSocket(socket) => {
                socket.write_all(content).expect("Write to conn");
            }
            _ => return Err(ERR_EXPECTED_OBJECT),
        };
        return Ok(Value::Number(NumberValue {
            n: content.len() as f64,
        }));
    }

    fn address(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_OBJECT);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        let address_str = match baggage.borrow_mut().deref() {
            NativeBaggage::TcpSocket(socket) => socket
                .local_addr()
                .unwrap()
                .as_socket()
                .unwrap()
                .to_string(),
            _ => return Err(ERR_EXPECTED_OBJECT),
        };
        return Ok(Value::String(StringValue::new(address_str)));
    }

    fn close(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_OBJECT);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        match baggage.borrow_mut().deref() {
            NativeBaggage::TcpSocket(socket) => {
                socket.shutdown(Shutdown::Both).unwrap_or(());
            }
            _ => return Err(ERR_EXPECTED_OBJECT),
        }
        return Ok(Value::Nil);
    }

    fn accept(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let native_value = match &values[0] {
            Value::Native(n) => n,
            _ => {
                return Err(ERR_EXPECTED_OBJECT);
            }
        };
        let baggage = native_value.baggage.as_ref().unwrap();
        let conn = match baggage.borrow_mut().deref() {
            NativeBaggage::TcpSocket(socket) => {
                let mut conn_obj = NativeValue {
                    props: HashMap::new(),
                    callable: None,
                    bind: false,
                    baggage: None,
                };
                let baggage = match socket.accept() {
                    Err(_) => {
                        return Err(RuntimeErr {
                            msg: "Cannot accept connection",
                            signal: None,
                        });
                    }
                    Ok((conn, _)) => Some(Rc::new(RefCell::new(NativeBaggage::TcpSocket(conn)))),
                };
                conn_obj.props.insert(
                    "address".to_string(),
                    Value::Native(NativeValue {
                        props: HashMap::new(),
                        callable: Some(&Self::conn_address),
                        bind: true,
                        baggage: baggage.clone(),
                    }),
                );
                conn_obj.props.insert(
                    "close".to_string(),
                    Value::Native(NativeValue {
                        props: HashMap::new(),
                        callable: Some(&Self::close),
                        bind: true,
                        baggage: baggage.clone(),
                    }),
                );
                conn_obj.props.insert(
                    "read".to_string(),
                    Value::Native(NativeValue {
                        props: HashMap::new(),
                        callable: Some(&Self::conn_read),
                        bind: true,
                        baggage: baggage.clone(),
                    }),
                );
                conn_obj.props.insert(
                    "write".to_string(),
                    Value::Native(NativeValue {
                        props: HashMap::new(),
                        callable: Some(&Self::conn_write),
                        bind: true,
                        baggage: baggage.clone(),
                    }),
                );
                conn_obj
            }
            _ => return Err(ERR_EXPECTED_OBJECT),
        };
        return Ok(Value::Native(conn));
    }

    fn listen_tcp(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 1 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let string_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let mut listen_tcp_module = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: true,
            baggage: None,
        };

        let socket = match Socket::new(Domain::IPV4, socket2::Type::STREAM, None) {
            Ok(s) => s,
            Err(_) => {
                return Err(RuntimeErr {
                    msg: "Cannot create a new socket",
                    signal: None,
                });
            }
        };
        let bind_to_address = if string_value.s.starts_with(":") {
            let mut ip = "0.0.0.0".to_string();
            ip.push_str(string_value.s.as_str());
            ip
        } else {
            string_value.s.clone()
        };
        let mut address = bind_to_address.to_socket_addrs();
        let address = match &mut address {
            Ok(a) => a.next().unwrap(),
            Err(_) => {
                return Err(RuntimeErr {
                    msg: "Cannot parse bind address",
                    signal: None,
                });
            }
        };
        match socket.bind(&address.into()) {
            Err(_) => {
                return Err(RuntimeErr {
                    msg: "Cannot bind port",
                    signal: None,
                })
            }
            _ => {}
        }
        match socket.listen(128) {
            Err(_) => {
                return Err(RuntimeErr {
                    msg: "Cannot listen on port",
                    signal: None,
                })
            }
            _ => {}
        }
        let baggage = Some(Rc::new(RefCell::new(NativeBaggage::TcpSocket(socket))));
        listen_tcp_module.props.insert(
            "address".to_string(),
            Value::Native(NativeValue {
                props: HashMap::new(),
                callable: Some(&Self::address),
                bind: true,
                baggage: baggage.clone(),
            }),
        );
        listen_tcp_module.props.insert(
            "close".to_string(),
            Value::Native(NativeValue {
                props: HashMap::new(),
                callable: Some(&Self::close),
                bind: true,
                baggage: baggage.clone(),
            }),
        );
        listen_tcp_module.props.insert(
            "accept".to_string(),
            Value::Native(NativeValue {
                props: HashMap::new(),
                callable: Some(&Self::accept),
                bind: true,
                baggage: baggage.clone(),
            }),
        );
        return Ok(Value::Native(listen_tcp_module));
    }

    pub fn build() -> NativeValue {
        let mut net = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        net.props.insert(
            "listenTcp".to_string(),
            Value::Native(NativeValue {
                props: HashMap::new(),
                callable: Some(&Self::listen_tcp),
                bind: false,
                baggage: None,
            }),
        );
        return net;
    }
}

pub struct Re {}

impl Re {
    pub fn regex_find(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let regex_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let string_value = match values.last().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let re = Regex::new(regex_value.s.as_str()).unwrap();
        let result: Vec<Value> = re.find_iter(string_value.s.as_str()).map(|e| Value::String(StringValue::new(String::from(e.as_str())))).collect();
        return Ok(Value::List(MutValue::new(ListValue{elements: result})));
    }

    pub fn regex_match(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let regex_value = match values.first().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let string_value = match values.last().unwrap() {
            Value::String(s) => s,
            _ => {
                return Err(ERR_EXPECTED_STRING);
            }
        };
        let re = Regex::new(regex_value.s.as_str()).unwrap();
        Ok(Value::Bool(BoolValue { b: re.is_match(string_value.s.as_str()) }))
    }

    pub fn build() -> NativeValue {
        let mut re = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        re.props.insert("find".to_string(), Value::Native(NativeValue{
            props: HashMap::new(),
            callable: Some(&Self::regex_find),
            bind: false,
            baggage: None,
        }));
        re.props.insert("match".to_string(), Value::Native(NativeValue{
            props: HashMap::new(),
            callable: Some(&Self::regex_match),
            bind: false,
            baggage: None,
        }));
        return re;
    }
}

pub struct Process {}

impl Process {
    pub fn build(is_embedded: bool) -> NativeValue {
        let mut process = NativeValue{
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        let mut argv: Vec<String> = env::args().collect();
        if !is_embedded && argv.len() >= 2 {
            argv.drain(0..1);
        }
        let argv_values = Value::List(MutValue::new(ListValue{
            elements: argv.iter().map(|a| Value::String(StringValue::new(a.clone()))).collect(),
        }));
        process.props.insert("argv".to_string(), argv_values);
        return process;
    }
}

pub struct Lists {}

impl Lists {
    pub fn push(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let list_value = match values.first().unwrap() {
            Value::List(l) => l,
            _ => {
                return Err(ERR_EXPECTED_LIST);
            }
        };
        list_value.0.borrow_mut().elements.push(values.last().unwrap().clone());
        Ok(Value::List(list_value.clone()))
    }

    pub fn pop(values: Vec<Value>) -> Result<Value, RuntimeErr> {
        if values.len() != 2 {
            return Err(ERR_INVALID_NUMBER_ARGUMENTS);
        }
        let list_value = match values.first().unwrap() {
            Value::List(l) => l,
            _ => {
                return Err(ERR_EXPECTED_LIST);
            }
        };
        let last_element = list_value.0.borrow_mut().elements.pop();
        if last_element.is_none() {
            return Err(ERR_LIST_EMPTY);
        }
        Ok(last_element.unwrap())
    }

    pub fn build() -> NativeValue {
        let mut list = NativeValue {
            props: HashMap::new(),
            callable: None,
            bind: false,
            baggage: None,
        };
        list.props.insert("push".to_string(), Value::Native(NativeValue {
            props: HashMap::new(),
            callable: Some(&Self::push),
            bind: false,
            baggage: None,
        }));
        list.props.insert("pop".to_string(), Value::Native(NativeValue {
            props: HashMap::new(),
            callable: Some(&Self::pop),
            bind: false,
            baggage: None,
        }));
        return list;
    }
}