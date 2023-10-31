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
mod value;
mod vm;

use std::fs::{canonicalize, read_to_string};

use std::{env, panic};

const SOURCE: &str = "
let a = 1
while a < 1000000 {
    a = a + 1
}
";

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
    interpreter::set_absolute_path(abs_path_string);
    if args.len() > 1 {
        content = read_to_string(interpreter::get_absolute_path()).unwrap();
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
