use crate::{compiler, lexer, native, parser, state, stmt, value, vm};
use std::collections::{HashMap, HashSet};
use std::panic;

pub static mut ABSOLUTE_PATH: &'static str = "";

static mut GLOBAL_INTERPRETER: Option<Interpreter> = None;

struct Interpreter {
    vm: vm::VM,
    compiler: compiler::Compiler,
}

fn get_global_interpreter() -> &'static mut Interpreter {
    setup_global_interpreter();
    let gi = unsafe { &mut GLOBAL_INTERPRETER };
    match gi {
        Some(g) => g,
        None => panic!("No global interpreter"),
    }
}

fn setup_global_interpreter() {
    if unsafe { GLOBAL_INTERPRETER.is_none() } {
        let compiler = compiler::Compiler {
            constants: vec![],
            contexts: vec![],
            prototypes: vec![],
            globals: HashSet::new(),
        };
        let mut my_vm = vm::VM {
            instructions: vec![],
            instructions_data: vec![],
            prototypes: vec![],
            constants: vec![],
            globals: HashMap::new(),
            builtins: HashMap::new(),
            stack: vec![vm::StackEntry {
                function: None,
                pc: 0,
                sp: 0,
                result_register: 0,
                this: None,
            }],
            activation_records: vec![],
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
        my_vm.builtins.insert(
            "net".to_string(),
            value::Value::Native(native::Net::build()),
        );
        unsafe {
            GLOBAL_INTERPRETER = Some(Interpreter {
                vm: my_vm,
                compiler: compiler,
            })
        };
    }
}

pub fn parse_source_code(source: String) -> Vec<stmt::Stmt> {
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
    return state.stmts.clone();
}

pub fn run_bytecode_interpreter(source: String) {
    let interpreter = get_global_interpreter();
    let stmts = parse_source_code(source);
    interpreter.compiler.compile(stmts);
    let instructions: Vec<compiler::InstSrc> = interpreter
        .compiler
        .contexts
        .iter()
        .map(|c| c.chunks.iter())
        .flatten()
        .map(|c| c.instructions.clone())
        .flatten()
        .collect();
    interpreter.vm.instructions = instructions.iter().map(|i| i.inst.clone()).collect();
    interpreter.vm.instructions_data = instructions.iter().map(|i| i.src.clone()).collect();
    interpreter.vm.prototypes = interpreter.compiler.prototypes.clone();
    interpreter.vm.constants = interpreter.compiler.constants.clone();
    interpreter.vm.activation_records =
        (0..interpreter.compiler.contexts.last().unwrap().register_count)
            .map(|_| vm::Record::Val(value::Value::Nil))
            .collect();

    interpreter.vm.interpret();
}

pub fn import_module(source: String) -> HashMap<String, value::Value> {
    let interpreter = get_global_interpreter();

    // Store current interpreter state
    let saved_fn_contexts = interpreter.compiler.contexts.clone();
    let saved_instructions = interpreter.vm.instructions.clone();
    let saved_instructions_data = interpreter.vm.instructions_data.clone();
    let saved_activation_records = interpreter.vm.activation_records.clone();
    let saved_globals = interpreter.vm.globals.clone();
    let saved_stack = interpreter.vm.stack.clone();

    let stmts = parse_source_code(source);
    interpreter.compiler.contexts = vec![];
    interpreter.compiler.compile(stmts);

    let instructions: Vec<compiler::InstSrc> = interpreter
        .compiler
        .contexts
        .iter()
        .map(|c| c.chunks.iter())
        .flatten()
        .map(|c| c.instructions.clone())
        .flatten()
        .collect();
    interpreter.vm.instructions = instructions.iter().map(|i| i.inst.clone()).collect();
    interpreter.vm.instructions_data = instructions.iter().map(|i| i.src.clone()).collect();
    interpreter.vm.prototypes = interpreter.compiler.prototypes.clone();
    interpreter.vm.constants = interpreter.compiler.constants.clone();
    interpreter.vm.activation_records =
        (0..interpreter.compiler.contexts.last().unwrap().register_count)
            .map(|_| vm::Record::Val(value::Value::Nil))
            .collect();
    interpreter.vm.globals = HashMap::new();
    interpreter.vm.interpret();

    let module_exports = interpreter.vm.globals.clone();

    // Restore saved state
    interpreter.compiler.contexts = saved_fn_contexts;
    interpreter.vm.instructions = saved_instructions;
    interpreter.vm.instructions_data = saved_instructions_data;
    interpreter.vm.activation_records = saved_activation_records;
    interpreter.vm.globals = saved_globals;
    interpreter.vm.stack = saved_stack;

    return module_exports;
}
