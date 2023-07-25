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
use std::collections::HashMap;

use std::env;
use std::fs::read_to_string;
use std::time::Instant;

const SOURCE: &str = "
let a = 1
while a < 1000000 {
    a = a + 1
}
";

const SOURCE_LITERAL: &str = "
let asd = [
    1,
    2,
    3,
    \"hello\",
]
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

fn test_bytecode_compiler(source: String) {
    let state = &mut state::InterpreterState::new(source);
    let mut lex = lexer::Lexer::new(state);
    lex.scan();
    let mut parser = parser::Parser::new(state);
    parser.parse();
    let mut compiler = compiler::Compiler {
        constants: vec![],
        contexts: vec![compiler::FnContext {
            chunks: vec![],
            register_count: 0,
            name: "".to_string(),
            loop_count: 0,
            blocks: vec![compiler::Block { locals: vec![] }],
            upvalues: vec![],
        }],
        prototypes: vec![],
    };
    let start = Instant::now();
    compiler.compile(state.stmts.clone());
    let mut my_mv = vm::VM {
        instructions: compiler
            .contexts
            .iter()
            .map(|c| c.chunks.iter())
            .flatten()
            .map(|c| c.instructions.clone())
            .flatten()
            .collect(),
        prototypes: compiler.prototypes,
        constants: compiler.constants,
        globals: HashMap::new(),
        stack: vec![vm::StackEntry {
            function: None,
            pc: 0,
            sp: 0,
            result_register: 0,
        }],
        activation_records: (0..compiler.contexts.last().unwrap().register_count)
            .map(|_| value::Value::Nil)
            .collect(),
        pc: 0,
    };
    // println!("{:#?}", my_mv);
    my_mv.interpret();
    let duration = start.elapsed();
    println!("{:#?}", my_mv);
    println!(
        "Duration compilation+execution: {:?}",
        duration.as_secs_f64()
    );
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
    //vm::test_vm_execution();
    let duration = start.elapsed();
    println!("Duration bytecode: {:?}", duration.as_secs_f64());
    tree_interpreter(String::from(source));
    test_bytecode_compiler(String::from(SOURCE_LITERAL));
}
