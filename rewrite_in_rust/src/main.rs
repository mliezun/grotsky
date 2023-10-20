mod compiler;
mod errors;
mod expr;
mod instruction;
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
use std::collections::HashMap;

use std::fs::read_to_string;
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

fn run_bytecode_interpreter(source: String) {
    let state = &mut state::InterpreterState::new(source);
    let mut lex = lexer::Lexer::new(state);
    lex.scan();
    if !state.errors.is_empty() {
        for err in &state.errors {
            println!("Error on line {}\n\t{}", err.line, err.message);
        }
        return;
    }
    let mut parser = parser::Parser::new(state);
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        parser.parse();
    }));
    if result.is_err() || !state.errors.is_empty() {
        for err in &state.errors {
            println!("Error on line {}\n\t{}", err.line, err.message);
        }
        return;
    }
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
    // let start = Instant::now();
    compiler.compile(state.stmts.clone());
    let instructions: Vec<compiler::InstSrc> = compiler
        .contexts
        .iter()
        .map(|c| c.chunks.iter())
        .flatten()
        .map(|c| c.instructions.clone())
        .flatten()
        .collect();
    let mut my_mv = vm::VM {
        instructions: instructions.iter().map(|i| i.inst.clone()).collect(),
        instructions_data: instructions.iter().map(|i| i.src.clone()).collect(),
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
            .map(|_| vm::Record::Val(value::Value::Nil))
            .collect(),
        pc: 0,
    };
    my_mv.activation_records[0] = vm::Record::Val(value::Value::Native(native::IO::build()));
    my_mv.activation_records[1] = vm::Record::Val(value::Value::Native(native::Strings::build()));
    my_mv.activation_records[2] = vm::Record::Val(value::Value::Native(native::Type::build()));
    my_mv.activation_records[3] = vm::Record::Val(value::Value::Native(native::Env::build()));
    // println!("{:#?}", my_mv.activation_records);
    // println!("{:#?}", my_mv.instructions);
    // println!("{:#?}", my_mv.constants);
    my_mv.interpret();
    // let duration = start.elapsed();
    // println!("{:#?}", my_mv.activation_records);
    // println!(
    //     "Duration compilation+execution: {:?}",
    //     duration.as_secs_f64()
    // );
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
    let grotsky_debug = env::var("GROTSKY_DEBUG").unwrap_or("0".to_string());
    if grotsky_debug != "1" && !grotsky_debug.eq_ignore_ascii_case("true") {
        // Disable rust backtrace
        panic::set_hook(Box::new(|_info| {}));
    }
    // tree_interpreter(String::from(source));
    run_bytecode_interpreter(String::from(source));
}
