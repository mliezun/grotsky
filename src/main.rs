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

use std::{fs::{canonicalize, read, write}, process::exit, ptr};
use const_random::const_random;

use std::{env, panic};

#[repr(C)]
struct Marker {
    magic_pattern: [u8; 512],
    is_embedded: u8,
}

const fn new_marker() -> Marker {
    Marker{
        magic_pattern: const_random!([u8; 512]),
        is_embedded: 0,
    }
}

static EMBEDDED_MARKER: Marker = new_marker();

fn is_embedded() -> bool {
    let embedded_indicator = &EMBEDDED_MARKER.is_embedded as *const u8;
    unsafe {
        // Need to perform this trick to read the actual memory location.
        // Otherwise during compilation Rust does static analysis and assumes
        // this function always returns the same value.
        return ptr::read_volatile(embedded_indicator) != 0;
    }
}

fn magic_pattern() -> &'static[u8; 512] {
    &EMBEDDED_MARKER.magic_pattern
}

fn find_position(haystack: &Vec<u8>, needle: &[u8; 512]) -> Option<usize> {
    if haystack.len() < needle.len() {
        return None;
    }
    for i in 0..=haystack.len() - needle.len() {
        if &haystack[i..i + needle.len()] == needle.as_ref() {
            return Some(i);
        }
    }
    None
}

fn embed_file(compiled_script: String, output_binary: String) {
    let exe_path = env::current_exe().unwrap();
    let mut exe_contents = read(exe_path).unwrap();
    let pattern = magic_pattern();
    if let Some(pos) = find_position(&exe_contents, pattern) {
        exe_contents[pos+512] = 1;
        let mut compiled_content = read(compiled_script).unwrap();
        for i in 0..512 {
            exe_contents.push(pattern[i]);
        }
        exe_contents.append(&mut compiled_content);
        write(output_binary, exe_contents).unwrap();
    }
}

fn execute_embedded() {
    let exe_path = env::current_exe().unwrap();
    interpreter::set_absolute_path(exe_path.clone().to_str().unwrap().to_string());

    let exe_contents = read(exe_path).unwrap();
    let pattern = magic_pattern();
    let offset: usize = 512;
    let first_match = find_position(&exe_contents, pattern).unwrap();
    let remaining = &exe_contents[first_match+offset..].to_vec();
    let pos = find_position(remaining, pattern).unwrap();
    let compiled_content = &remaining[pos+offset..];
    
    if !interpreter::run_interpreter_from_bytecode(&compiled_content) {
        println!("Could not read embedded script");
        exit(1);
    }
}

const GENERAL_USAGE: &'static str = r##"Usage:
    grotsky [script.gr | bytecode.grc]
    grotsky compile script.gr
    grotsky embed bytecode.grc
"##;

fn main() {
    if is_embedded() {
        execute_embedded();
        return;
    }
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
        embed_file(abs_path_string, String::from(new_abs_path.to_string_lossy()));
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
}
