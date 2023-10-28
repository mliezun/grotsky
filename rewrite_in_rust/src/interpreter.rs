use crate::{compiler, lexer, native, parser, state, value, vm};
use std::collections::{HashMap, HashSet};
use std::panic;

pub static mut ABSOLUTE_PATH: &'static str = "";

pub fn run_bytecode_interpreter(source: String) -> vm::VM {
    let state = &mut state::InterpreterState::new(source);
    let mut lex = lexer::Lexer::new(state);
    lex.scan();
    if !state.errors.is_empty() {
        for err in &state.errors {
            println!("Error on line {}\n\t{}", err.line, err.message);
        }
        std::process::exit(0);
    }
    let mut parser = parser::Parser::new(state);
    let result = panic::catch_unwind(panic::AssertUnwindSafe(|| {
        parser.parse();
    }));
    if result.is_err() || !state.errors.is_empty() {
        for err in &state.errors {
            println!("Error on line {}\n\t{}", err.line, err.message);
        }
        std::process::exit(0);
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
        globals: HashSet::new(),
    };
    compiler.compile(state.stmts.clone());
    let instructions: Vec<compiler::InstSrc> = compiler
        .contexts
        .iter()
        .map(|c| c.chunks.iter())
        .flatten()
        .map(|c| c.instructions.clone())
        .flatten()
        .collect();
    let mut my_vm = vm::VM {
        instructions: instructions.iter().map(|i| i.inst.clone()).collect(),
        instructions_data: instructions.iter().map(|i| i.src.clone()).collect(),
        prototypes: compiler.prototypes,
        constants: compiler.constants,
        globals: HashMap::new(),
        builtins: HashMap::new(),
        stack: vec![vm::StackEntry {
            function: None,
            pc: 0,
            sp: 0,
            result_register: 0,
            this: None,
        }],
        activation_records: (0..compiler.contexts.last().unwrap().register_count)
            .map(|_| vm::Record::Val(value::Value::Nil))
            .collect(),
    };
    my_vm
        .builtins
        .insert("io".to_string(), value::Value::Native(native::IO::build()));
    my_vm.builtins.insert(
        "strings".to_string(),
        value::Value::Native(native::Strings::build()),
    );
    my_vm.builtins.insert(
        "type".to_string(),
        value::Value::Native(native::Type::build()),
    );
    my_vm.builtins.insert(
        "env".to_string(),
        value::Value::Native(native::Env::build()),
    );
    my_vm.builtins.insert(
        "import".to_string(),
        value::Value::Native(native::Import::build()),
    );
    my_vm.interpret();
    return my_vm;
}
