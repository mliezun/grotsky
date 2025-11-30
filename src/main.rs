mod compiler;
mod embed;
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

#[cfg(feature = "profile")]
use pprof::protos::Message;
#[cfg(feature = "profile")]
use std::io::Write;

use std::{fs::{canonicalize, read, write}, process::exit};

use std::{env, panic};

const GENERAL_USAGE: &'static str = r##"Usage:
    grotsky [script.gr | bytecode.grc]
    grotsky compile script.gr
    grotsky embed bytecode.grc
"##;

fn main() {
    if embed::is_embedded() {
        embed::execute_embedded();
        return;
    }

    #[cfg(feature = "profile")]
    let guard = if env::var("GROTSKY_PROFILE").unwrap_or("0".to_string()) == "1" {
        Some(pprof::ProfilerGuard::new(100).unwrap())
    } else {
        None
    };

    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", GENERAL_USAGE);
        exit(1);
    }
    let content: Vec<u8>;
    let mut abs_path = if args[1] == "compile" || args[1] == "embed" {
        if args.len() != 3 {
            println!("{}", GENERAL_USAGE);
            exit(1);
        }
        canonicalize(&args[2]).unwrap()
    } else {
        canonicalize(&args[1]).unwrap()
    };
    let abs_path_str = abs_path.to_string_lossy();
    let abs_path_string = String::from(abs_path_str);
    if args[1] == "embed" {
        if abs_path.extension().unwrap_or_default() != "grc" {
            println!("Can only embed a .grc file");
            exit(1);
        }
        let mut new_abs_path = abs_path.clone();
        new_abs_path.set_extension("exe");
        embed::embed_file(abs_path_string, String::from(new_abs_path.to_string_lossy()));
        return;
    }

    interpreter::set_absolute_path(abs_path_string);
    content = read(interpreter::get_absolute_path()).unwrap();
    let grotsky_debug = env::var("GROTSKY_DEBUG").unwrap_or("0".to_string());
    if grotsky_debug != "1" && !grotsky_debug.eq_ignore_ascii_case("true") {
        // Disable rust backtrace
        panic::set_hook(Box::new(|_info| {}));
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

    #[cfg(feature = "profile")]
    if let Some(report) = guard {
        if let Ok(report) = report.report().build() {
            let file = std::fs::File::create("flamegraph.svg").unwrap();
            report.flamegraph(file).unwrap();
            
            let mut file = std::fs::File::create("profile.pb").unwrap();
            let profile = report.pprof().unwrap();
            let mut content = Vec::new();
            profile.write_to_vec(&mut content).unwrap();
            file.write_all(&content).unwrap();
        }
    }
}
