mod compiler;
mod expr;
mod instruction;
mod lexer;
mod parser;
mod state;
mod stmt;
mod token;
mod tree_interp;
mod value;
mod vm;

use fnv::FnvHashMap;

use std::env;
use std::fs::read_to_string;
use std::time::Instant;

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
    println!("Duration tree: {:?}", duration.as_secs_f64());
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let content: String;
    let source: &str;
    if args.len() > 1 {
        content = read_to_string(&args[1]).unwrap();
        source = content.as_str();
    } else {
        source = SOURCE;
    }
    let start = Instant::now();
    vm::test_vm_execution();
    let duration = start.elapsed();
    println!("Duration bytecode: {:?}", duration.as_secs_f64());
    tree_interpreter(String::from(source));
}
