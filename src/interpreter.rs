use crate::vm::{StackEntry, VMFnPrototype};
use crate::{compiler, embed, lexer, native, parser, state, stmt, value, vm};
use std::collections::{HashMap, HashSet};
use std::panic;
use std::rc::Rc;

static mut ABSOLUTE_PATH: &'static str = "";

static mut GLOBAL_INTERPRETER: Option<Interpreter> = None;

struct Interpreter {
    vm: vm::VM,
    compiler: compiler::Compiler,
}

pub fn get_absolute_path() -> String {
    unsafe { (*std::ptr::addr_of!(ABSOLUTE_PATH)).to_string() }
}

pub fn set_absolute_path(path: String) {
    unsafe {
        ABSOLUTE_PATH = Box::leak(path.into_boxed_str());
    }
}

fn get_global_interpreter() -> &'static mut Interpreter {
    setup_global_interpreter();
    let gi = unsafe { &mut *std::ptr::addr_of_mut!(GLOBAL_INTERPRETER) };
    match gi {
        Some(g) => g,
        None => panic!("No global interpreter"),
    }
}

fn setup_global_interpreter() {
    if unsafe { (*std::ptr::addr_of!(GLOBAL_INTERPRETER)).is_none() } {
        let compiler = compiler::Compiler {
            constants: vec![],
            contexts: vec![],
            prototypes: vec![],
            globals: HashSet::new(),
        };
        let mut my_vm = vm::VM {
            instructions: Rc::new(vec![]),
            instructions_data: Rc::new(vec![]),
            prototypes: Rc::new(vec![]),
            constants: vec![],
            globals: HashMap::new(),
            builtins: HashMap::new(),
            frames: vec![vm::StackEntry {
                function: None,
                pc: 0,
                sp: 0,
                result_register: 0,
                caller_this: None,
                current_this: None,
                file: Some(get_absolute_path()),
            }],
            activation_records: vec![],
            catch_exceptions: vec![],
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
        my_vm.builtins.insert(
            "re".to_string(),
            value::Value::Native(native::Re::build()),
        );
        my_vm.builtins.insert(
            "process".to_string(),
            value::Value::Native(native::Process::build(embed::is_embedded())),
        );
        my_vm.builtins.insert(
            "lists".to_string(),
            value::Value::Native(native::Lists::build()),
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

pub fn compile_to_bytecode(source: String) -> Vec<u8> {
    let interpreter = get_global_interpreter();
    let stmts = parse_source_code(source);
    interpreter.compiler.compile(stmts);
    bincode::serialize(&interpreter.compiler).unwrap()
}

pub fn run_interpreter_from_bytecode(bytecode: &[u8]) -> bool {
    if let Ok(compiler) = bincode::deserialize::<compiler::Compiler>(bytecode) {
        let interpreter = get_global_interpreter();
        interpreter.compiler = compiler;
        let instructions: Vec<compiler::InstSrc> = interpreter
            .compiler
            .contexts
            .iter()
            .map(|c| c.chunks.iter())
            .flatten()
            .map(|c| c.instructions.clone())
            .flatten()
            .collect();
        interpreter.vm.instructions = Rc::new(instructions.iter().map(|i| i.inst.clone()).collect());
        interpreter.vm.instructions_data = Rc::new(instructions.iter().map(|i| i.src.clone()).collect());
        interpreter.vm.prototypes = Rc::new(interpreter.compiler.prototypes.iter().map(|p| VMFnPrototype{
            instructions: Rc::new(p.instructions.clone()),
            register_count: p.register_count,
            upvalues: p.upvalues.clone(),
            instruction_data: Rc::new(p.instruction_data.clone()),
            param_count: p.param_count,
            name: p.name.clone(),
            file_path: p.file_path.clone(),
        }).collect());
        interpreter.vm.constants = interpreter
            .compiler
            .constants
            .iter()
            .map(|c| c.into())
            .collect();
        interpreter.vm.activation_records =
            (0..interpreter.compiler.contexts.last().unwrap().register_count)
                .map(|_| vm::Record::Val(value::Value::Nil))
                .collect();

        interpreter.vm.interpret();
        return true;
    }
    return false;
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
    interpreter.vm.instructions = Rc::new(instructions.iter().map(|i| i.inst.clone()).collect());
    interpreter.vm.instructions_data = Rc::new(instructions.iter().map(|i| i.src.clone()).collect());
    interpreter.vm.prototypes = Rc::new(interpreter.compiler.prototypes.iter().map(|p| VMFnPrototype{
        instructions: Rc::new(p.instructions.clone()),
        register_count: p.register_count,
        upvalues: p.upvalues.clone(),
        instruction_data: Rc::new(p.instruction_data.clone()),
        param_count: p.param_count,
        name: p.name.clone(),
        file_path: p.file_path.clone(),
    }).collect());
    interpreter.vm.constants = interpreter
        .compiler
        .constants
        .iter()
        .map(|c| c.into())
        .collect();
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
    let saved_compiler_globals = interpreter.compiler.globals.clone();
    let saved_instructions = interpreter.vm.instructions.clone();
    let saved_instructions_data = interpreter.vm.instructions_data.clone();
    let saved_activation_records = interpreter.vm.activation_records.clone();
    let saved_globals = interpreter.vm.globals.clone();
    let saved_frames = interpreter.vm.frames.clone();

    let stmts = parse_source_code(source);
    interpreter.compiler.contexts = vec![];
    interpreter.compiler.globals = HashSet::new();
    interpreter.compiler.enter_function("".to_string());
    interpreter.compiler.enter_function("module".to_string());
    interpreter.compiler.compile(stmts);
    let module_global_context = interpreter.compiler.contexts[1].clone();
    interpreter.compiler.leave_function(0);
    interpreter.compiler.leave_function(0);

    let instructions: Vec<compiler::InstSrc> = module_global_context
        .chunks
        .iter()
        .map(|c| c.instructions.clone())
        .flatten()
        .collect();
    interpreter.vm.instructions = Rc::new(instructions.iter().map(|i| i.inst.clone()).collect());
    interpreter.vm.instructions_data = Rc::new(instructions.iter().map(|i| i.src.clone()).collect());
    interpreter.vm.prototypes = Rc::new(interpreter.compiler.prototypes.iter().map(|p| VMFnPrototype{
        instructions: Rc::new(p.instructions.clone()),
        register_count: p.register_count,
        upvalues: p.upvalues.clone(),
        instruction_data: Rc::new(p.instruction_data.clone()),
        param_count: p.param_count,
        name: p.name.clone(),
        file_path: p.file_path.clone(),
    }).collect());
    interpreter.vm.constants = interpreter
        .compiler
        .constants
        .iter()
        .map(|c| c.into())
        .collect();
    interpreter.vm.activation_records = (0..module_global_context.register_count)
        .map(|_| vm::Record::Val(value::Value::Nil))
        .collect();
    interpreter.vm.globals = HashMap::new();
    interpreter.vm.frames.push(StackEntry {
        function: None,
        pc: 0,
        sp: 0,
        result_register: 0,
        caller_this: None,
        current_this: None,
        file: Some(get_absolute_path()),
    });
    interpreter.vm.interpret();

    let module_exports = module_global_context.blocks[0]
        .locals
        .iter()
        .map(|l| {
            (
                l.var_name.clone(),
                interpreter.vm.activation_records[l.reg as usize].as_val(),
            )
        })
        .collect();

    // Restore saved state
    interpreter.compiler.contexts = saved_fn_contexts;
    interpreter.compiler.globals = saved_compiler_globals;
    interpreter.vm.instructions = saved_instructions;
    interpreter.vm.instructions_data = saved_instructions_data;
    interpreter.vm.activation_records = saved_activation_records;
    interpreter.vm.globals = saved_globals;
    interpreter.vm.frames = saved_frames;

    return module_exports;
}
