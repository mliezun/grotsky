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

use std::fs::{canonicalize, read, write};

use std::{env, panic};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage:\n\tgrotsky [filename]\n\tgrotsky compile [filename]\n");
        return;
    }
    let content: Vec<u8>;
    let mut abs_path = if args[1] == "compile" {
        if args.len() != 3 {
            println!("Usage:\n\tgrotsky [filename]\n\tgrotsky compile [filename]\n");
            return;
        }
        canonicalize(&args[2]).unwrap()
    } else {
        canonicalize(&args[1]).unwrap()
    };
    let abs_path_str = abs_path.to_string_lossy();
    let abs_path_string = String::from(abs_path_str);
    interpreter::set_absolute_path(abs_path_string);
    content = read(interpreter::get_absolute_path()).unwrap();
    let grotsky_debug = env::var("GROTSKY_DEBUG").unwrap_or("0".to_string());
    if grotsky_debug != "1" && !grotsky_debug.eq_ignore_ascii_case("true") {
        // Disable rust backtrace
        panic::set_hook(Box::new(|_info| {}));
    }
    if abs_path.extension().is_none()
        || !abs_path.extension().unwrap().eq_ignore_ascii_case("gr")
        || !abs_path.extension().unwrap().eq_ignore_ascii_case("grc")
    {
        println!("Usage:\n\tgrotsky [filename]\n\tgrotsky compile [filename]");
        println!("File extension must be .gr or .grc\n");
        return;
    }
    if args[1] == "compile" {
        abs_path.set_extension("grc");
        write(
            abs_path,
            interpreter::compile_to_bytecode(
                String::from_utf8(content).expect("Invalid input file"),
            ),
        )
        .expect("Write bytecode file");
    } else if !interpreter::run_interpreter_from_bytecode(content.as_slice()) {
        interpreter::run_bytecode_interpreter(
            String::from_utf8(content).expect("Invalid input file"),
        );
    }
}
