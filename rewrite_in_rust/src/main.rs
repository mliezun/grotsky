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

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        println!("Usage:\n\tgrotsky [filename]");
        return;
    }
    let content: String;
    let source: &str;
    let abs_path = canonicalize(&args[1]).unwrap();
    let abs_path_str = abs_path.to_string_lossy();
    let abs_path_string = String::from(abs_path_str);
    interpreter::set_absolute_path(abs_path_string);
    content = read_to_string(interpreter::get_absolute_path()).unwrap();
    source = content.as_str();
    let grotsky_debug = env::var("GROTSKY_DEBUG").unwrap_or("0".to_string());
    if grotsky_debug != "1" && !grotsky_debug.eq_ignore_ascii_case("true") {
        // Disable rust backtrace
        panic::set_hook(Box::new(|_info| {}));
    }
    interpreter::run_bytecode_interpreter(String::from(source));
}
