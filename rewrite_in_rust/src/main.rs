mod compiler;
mod errors;
mod expr;
mod instruction;
mod interpreter;
mod lexer;
mod native;
mod parser;
mod state;
mod stmt;
mod token;
mod tree_interp;
mod value;
mod vm;

use fnv::FnvHashMap;
use std::collections::{HashMap, HashSet};

use std::fs::{canonicalize, read_to_string};
use std::time::Instant;
use std::{env, panic};

const SOURCE: &str = "
let a = 1
while a < 1000000 {
    a = a + 1
}
";

fn tree_interpreter(source: String) {
    let state = &mut state::InterpreterState::new(source);
    let mut lex = lexer::Lexer::new(state);
    lex.scan();
    let mut parser = parser::Parser::new(state);
    parser.parse();
    // println!("{:#?}", state.tokens);
    // println!("{:#?}", state.stmts);
    let mut env = tree_interp::Env {
        enclosing: None,
        values: FnvHashMap::default(),
    };
    let mut exec = tree_interp::Exec::new(core::ptr::addr_of_mut!(env));
    let start = Instant::now();
    exec.interpret(&mut state.stmts);
    let duration = start.elapsed();
    // println!("Duration tree: {:?}", duration.as_secs_f64());
}

fn string_to_static_str(s: String) -> &'static str {
    Box::leak(s.into_boxed_str())
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let content: String;
    let source: &str;
    let abs_path = canonicalize(&args[1]).unwrap();
    let abs_path_str = abs_path.to_string_lossy();
    let abs_path_string = String::from(abs_path_str);
    unsafe {
        interpreter::ABSOLUTE_PATH = string_to_static_str(abs_path_string);
    }
    if args.len() > 1 {
        content = read_to_string(unsafe { interpreter::ABSOLUTE_PATH }).unwrap();
        source = content.as_str();
    } else {
        source = SOURCE;
    }
    let grotsky_debug = env::var("GROTSKY_DEBUG").unwrap_or("0".to_string());
    if grotsky_debug != "1" && !grotsky_debug.eq_ignore_ascii_case("true") {
        // Disable rust backtrace
        panic::set_hook(Box::new(|_info| {}));
    }
    interpreter::run_bytecode_interpreter(String::from(source));
}
