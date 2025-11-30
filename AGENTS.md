# AGENTS.md - Instructions for LLM Agents

This document provides comprehensive instructions for LLM agents working on the Grotsky project. It covers the project structure, architecture, coding conventions, and testing procedures.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture](#architecture)
3. [Project Structure](#project-structure)
4. [Key Concepts](#key-concepts)
5. [Coding Guidelines](#coding-guidelines)
6. [Testing](#testing)
7. [Common Patterns](#common-patterns)
8. [Important Implementation Details](#important-implementation-details)

## Project Overview

**Grotsky** is a toy programming language implemented in Rust as a bytecode register-based VM interpreter. It was inspired by the book "Crafting Interpreters" and draws design inspiration from Lua's VM, Go, Python, and JavaScript.

### Key Features

- **Syntax**: C-like style with curly braces but no semicolons
- **Collections**: Lists and dictionaries (dicts)
- **Networking**: TCP socket listening capabilities
- **Modules**: Import system for code organization
- **Environment Variables**: Access to system environment
- **Classes**: Object-oriented programming with inheritance
- **Closures**: First-class functions with closure support
- **Compilation**: Scripts can be compiled to bytecode (`.grc` files)
- **Embedding**: Bytecode can be embedded into distributable binaries

### Language Characteristics

- Dynamically typed
- Garbage collected (via Rust's `Rc<RefCell<T>>` pattern)
- Register-based VM (not stack-based)
- Uses activation records for function calls (currently not very efficient, but kept for now)

## Architecture

### Compilation Pipeline

```
Source Code (.gr) 
  → Lexer (token.rs)
  → Parser (parser.rs) 
  → Compiler (compiler.rs)
  → Bytecode (.grc) [optional]
  → VM (vm.rs)
  → Execution
```

### Core Components

1. **Lexer** (`src/lexer.rs`): Tokenizes source code
2. **Parser** (`src/parser.rs`): Builds AST from tokens
3. **Compiler** (`src/compiler.rs`): Generates bytecode instructions
4. **VM** (`src/vm.rs`): Executes bytecode instructions
5. **Value System** (`src/value.rs`): Represents runtime values
6. **Native Functions** (`src/native.rs`): Built-in standard library

### VM Design

The VM is **register-based** (not stack-based), inspired by Lua's VM design. Key characteristics:

- **Instruction Format**: Each instruction has an opcode and three 8-bit operands (A, B, C)
- **Registers**: Up to 256 registers per function (u8 limit)
- **Activation Records**: Each function call creates a new activation record in a flat array
- **Call Stack**: Separate stack tracks function calls, return addresses, and stack pointers

**Important Note**: The current implementation uses activation records that are appended to a flat array each time a function is called. This is not very efficient, but it's the current design and should be kept for now.

## Project Structure

```
grotsky/
├── src/                    # Main source code
│   ├── main.rs            # Entry point, CLI handling
│   ├── lexer.rs           # Tokenizer
│   ├── parser.rs          # AST construction
│   ├── compiler.rs        # Bytecode generation
│   ├── vm.rs              # Virtual machine execution
│   ├── value.rs           # Runtime value types
│   ├── instruction.rs     # Bytecode instruction definitions
│   ├── native.rs          # Standard library (io, strings, net, etc.)
│   ├── interpreter.rs     # Interpreter orchestration
│   ├── errors.rs          # Error definitions
│   ├── token.rs           # Token definitions
│   ├── expr.rs            # Expression AST nodes
│   ├── stmt.rs            # Statement AST nodes
│   ├── state.rs           # Global interpreter state
│   └── embed.rs           # Bytecode embedding utilities
├── test/                  # Test files
│   ├── integration/       # Integration tests
│   └── benchmark/        # Performance benchmarks
├── examples/              # Example Grotsky scripts
├── archive/               # Legacy Go implementation (for reference)
├── tool/                  # Build and test utilities
├── Cargo.toml            # Rust dependencies
├── Makefile              # Build commands
├── syntax.ebnf           # Grammar specification
└── readme.md             # User documentation
```

## Key Concepts

### Activation Records

**Activation records** are the mechanism by which the VM manages function-local storage. Each function call creates a new activation record:

- Activation records are stored in a flat `Vec<Record>` called `activation_records`
- Each function has a `register_count` that determines how many registers it needs
- When a function is called, new registers are appended to the activation records array
- The stack pointer (`sp`) points to the start of the current function's activation record
- When a function returns, its activation records are removed from the array

**Current Limitation**: This design is not very efficient because it requires appending/removing from a vector on every function call/return. However, this is the current implementation and should be maintained for now.

### Value System

Values in Grotsky use Rust's `Rc<RefCell<T>>` pattern for shared mutable ownership:

```rust
pub enum Value {
    Class(MutValue<ClassValue>),
    Object(MutValue<ObjectValue>),
    Dict(MutValue<DictValue>),
    List(MutValue<ListValue>),
    Fn(MutValue<FnValue>),
    Native(NativeValue),
    Number(NumberValue),
    String(StringValue),
    Bytes(BytesValue),
    Bool(BoolValue),
    Slice(SliceValue),
    Nil,
}
```

**MutValue** wraps `Rc<RefCell<T>>` to provide shared mutable access. This is necessary because:
- Values can be shared across multiple references (closures, objects, etc.)
- Values need to be mutable (lists, dicts, objects can be modified)
- Rust's ownership system requires this pattern for dynamic languages

### Register Allocation

The compiler allocates registers for local variables:

- Registers are allocated sequentially starting from 0
- Each function has its own register space
- Parameters are stored in registers starting from register 1 (register 0 is reserved)
- The compiler tracks register usage in `FnContext`

### Instruction Format

All instructions follow this format:

```rust
struct Instruction {
    opcode: OpCode,
    a: u8,  // First operand (destination register, etc.)
    b: u8,  // Second operand
    c: u8,  // Third operand
}
```

Some instructions combine B and C into a 16-bit value:
- `bx()`: Unsigned 16-bit (B << 8 | C)
- `sbx()`: Signed 16-bit (for jumps)

### Function Prototypes

Each function is compiled into a `FnPrototype` containing:
- `instructions`: The bytecode for the function
- `register_count`: Number of registers needed
- `param_count`: Number of parameters
- `upvalues`: References to closed-over variables
- `instruction_data`: Source location information for debugging

## Coding Guidelines

### Rust Best Practices

1. **Performance**: Write code with performance in mind. This is a VM interpreter where performance matters.
2. **Borrow Checker**: Ensure all code compiles without borrow checker errors. Use `Rc<RefCell<T>>` pattern where shared mutable ownership is needed.
3. **Error Handling**: Use the error constants from `errors.rs`. Throw exceptions using the `throw_exception!` macro in `vm.rs`.
4. **Cloning**: Be mindful of cloning. Values are often cloned, but avoid unnecessary clones in hot paths.

### Code Style

- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)
- Use meaningful variable names
- Add comments for complex logic, especially in the VM
- Keep functions focused and reasonably sized

### File Organization

- Each major component has its own file
- Keep related functionality together
- The VM (`vm.rs`) is large but should remain in one file for now

### Error Messages

- Use constants from `errors.rs` for runtime errors
- Error messages should be clear and helpful
- All errors are represented as `RuntimeErr` structs

## Testing

### Running Tests

The project uses a Makefile for common tasks:

```bash
# Build the Rust interpreter
make grotsky-rs

# Run integration tests
make test_integration

# Run benchmarks
make benchmark_loop
make benchmark_fib
make benchmark_objects
```

### Integration Tests

Integration tests are in `test/integration/`:

- **blog.py**: Tests the interpreter by cloning and running a real blog project
- Run with: `make test_integration` or `python test/integration/blog.py`

The integration test:
1. Clones the blog repository (mliezun.github.io)
2. Runs the site generation script with grotsky-rs
3. Verifies successful execution

### Benchmarks

Benchmarks are in `test/benchmark/`:

- **loop.gr**: Simple loop performance test
- **fib.gr**: Recursive Fibonacci test
- **objects.gr**: Object creation and method calls test

Run benchmarks with:
```bash
make benchmark_loop    # Compare with Go version
make benchmark_fib
make benchmark_objects
```

Or directly:
```bash
python tool/benchmark.py build/grotsky-rs loop
```

### Manual Testing

You can test scripts directly:

```bash
# Run a script
./build/grotsky-rs examples/test.gr

# Compile to bytecode
./build/grotsky-rs compile examples/test.gr

# Run bytecode
./build/grotsky-rs examples/test.grc

# Embed bytecode
./build/grotsky-rs embed examples/test.grc
```

### Example Scripts

The `examples/` directory contains various test scripts:
- `grep.gr`: A grep-like utility
- `http.gr`: HTTP server example
- `test.gr`: General test cases
- And more...

## Common Patterns

### Adding a New Opcode

1. Add the opcode to `instruction.rs` in the `OpCode` enum
2. Add handling in `vm.rs` in the `interpret()` method's match statement
3. Add compilation logic in `compiler.rs` if needed
4. Update `THE_VM.md` if documenting new instructions

### Adding a Native Function

1. Add the function to the appropriate module in `native.rs`
2. Register it in the `get_builtins()` function
3. The function signature should be: `fn(vm: &mut VM, args: &[Value]) -> Result<Value, RuntimeErr>`

### Accessing Activation Records

When accessing registers in the VM:

```rust
// Read from register
let value = self.activation_records[sp + register_index].as_val();

// Write to register
self.activation_records[sp + register_index] = Record::Val(value);

// Note: sp is the stack pointer (start of current function's activation record)
```

### Function Calls

Function calls use the `make_call!` macro in `vm.rs`:

1. Validates argument count
2. Checks recursion depth
3. Creates a new stack frame
4. Expands activation records
5. Copies arguments to the new activation record
6. Updates program counter and instructions

### Error Handling

Errors are thrown using the `throw_exception!` macro:

```rust
throw_exception!(
    self,
    current_this,
    original_instructions,
    original_instructions_data,
    pc,
    sp,
    ERR_SOME_ERROR
);
```

This will:
1. Unwind the stack to the nearest catch block
2. Pop frames and activation records
3. Set the exception value
4. Jump to the catch block

## Important Implementation Details

### Activation Records Design

**Current Implementation**: Each function call appends new registers to a flat `Vec<Record>`. This means:
- Function calls: `activation_records.append(new_registers)`
- Function returns: `activation_records.truncate(old_length)`

**Why it's inefficient**: Vector appends/truncates can cause reallocations. A more efficient design would use a proper stack or pre-allocate space.

**Why we keep it**: This is the current design. Don't change it unless explicitly asked.

### Register Indexing

- Register 0 is typically reserved or unused in some contexts
- Parameters start at register 1
- Local variables are allocated sequentially
- Stack pointer (`sp`) points to the start of the current function's activation record
- Register access: `activation_records[sp + register_index]`

### Upvalues (Closures)

Closures capture variables from outer scopes:
- Upvalues are stored in the function prototype
- `GetUpval` and `SetUpval` instructions access upvalues
- Upvalues can be "closed" when the outer function returns

### Try-Catch

Exception handling:
- `RegisterTryCatch` marks the start of a try block
- `DeregisterTryCatch` marks the end
- Exceptions are stored in `catch_exceptions` stack
- When an exception is thrown, the stack unwinds to the nearest catch block

### Module System

Modules are loaded via the `import()` function:
- Each module is compiled separately
- Module exports are stored in the module's global context
- Modules can import other modules
- Currently, embedding only supports a single file (limitation)

### Bytecode Format

Bytecode is serialized using `bincode`:
- The entire `Compiler` struct is serialized
- Includes prototypes, constants, and instruction data
- Can be embedded into the binary using `embed.rs`

## Debugging

### Debug Mode

Set environment variable for detailed output:
```bash
GROTSKY_DEBUG=1 ./build/grotsky-rs script.gr
RUST_BACKTRACE=full ./build/grotsky-rs script.gr
```

### Common Issues

1. **Borrow Checker Errors**: Use `Rc<RefCell<T>>` pattern for shared mutable data
2. **Register Overflow**: Maximum 255 registers per function (u8 limit)
3. **Recursion Depth**: Maximum 1024 frames (`MAX_FRAMES`)
4. **Activation Record Access**: Always use `sp + register_index` for register access

## References

- **Grammar**: See `syntax.ebnf` for the language grammar
- **VM Design**: See `THE_VM.md` for VM design notes
- **User Guide**: See `readme.md` for language documentation
- **Lua VM Paper**: The VM is inspired by Lua's VM design (see `THE_VM.md`)

## Summary for Quick Reference

- **Language**: Rust
- **VM Type**: Register-based (not stack-based)
- **Function Calls**: Use activation records (flat array, not very efficient)
- **Values**: Use `Rc<RefCell<T>>` pattern for shared mutable ownership
- **Testing**: Use `make test_integration` for integration tests
- **Benchmarks**: Use `make benchmark_*` commands
- **Error Handling**: Use constants from `errors.rs` and `throw_exception!` macro
- **Register Limit**: 255 per function (u8)
- **Recursion Limit**: 1024 frames

When making changes:
1. Ensure code compiles without borrow checker errors
2. Follow Rust best practices for performance
3. Test with integration tests
4. Keep activation records design as-is unless explicitly asked to change
5. Use existing error constants and patterns

