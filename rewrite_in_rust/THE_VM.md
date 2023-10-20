# The VM

I want to do a register based VM. Im not experienced in this matter. So I'll be trying to copy from Lua's VM: https://www.lua.org/doc/jucs05.pdf.

For the implementation in Rust, I'll be borrowing from here: https://github.com/rust-hosted-langs/book.

## LuaVM

The instructions in Lua’s virtual machine take 32 bits divided into three or four fields.
- The OP field identifies the instruction and takes 6 bits.
- Field A is always present and takes 8 bits.
- Fields B and C take 9 bits each.
    - They can be combined into an 18-bit field: Bx (unsigned) and sBx (signed).

They have this instructions:

```
MOVE (A, B) R(A) := R(B)
LOADK (A, Bx) R(A) := K(Bx)
LOADBOOL (A, B, C) R(A) := (Bool)B; if (C) PC++
LOADNIL (A, B) R(A) := ... := R(B) := nil
GETUPVAL (A, B) R(A) := U[B]
GETGLOBAL (A, Bx) R(A) := G[K(Bx)]
GETTABLE (A, B, C) R(A) := R(B)[RK(C)]
SETGLOBAL (A, Bx) G[K(Bx)] := R(A)
SETUPVAL (A, B) U[B] := R(A)
SETTABLE (A, B, C) R(A)[RK(B)] := RK(C)
NEWTABLE (A, B, C) R(A) := {} (size = B,C)
SELF (A, B, C) R(A+1) := R(B); R(A) := R(B)[RK(C)]
ADD (A, B, C) R(A) := RK(B) + RK(C)
SUB (A, B, C) R(A) := RK(B) - RK(C)
MUL (A, B, C) R(A) := RK(B) * RK(C)
DIV (A, B, C) R(A) := RK(B) / RK(C)
POW (A, B, C) R(A) := RK(B) ^ RK(C)
UNM (A, B) R(A) := -R(B)
NOT (A, B) R(A) := not R(B)
CONCAT (A, B, C) R(A) := R(B) .. ... .. R(C)
JMP (sBx) PC += sBx
EQ (A, B, C) if ((RK(B) == RK(C)) ~= A) then PC++
LT (A, B, C) if ((RK(B) < RK(C)) ~= A) then PC++
LE (A, B, C) if ((RK(B) <= RK(C)) ~= A) then PC++
TEST (A, B, C) if (R(B) <=> C) then R(A) := R(B) else PC++
CALL (A, B, C) R(A), ... ,R(A+C-2) := R(A)(R(A+1), ... ,R(A+B-1))
TAILCALL (A, B, C) return R(A)(R(A+1), ... ,R(A+B-1))
RETURN (A, B) return R(A), ... ,R(A+B-2) (see note)
FORLOOP (A, sBx) R(A)+=R(A+2); if R(A) <?= R(A+1) then PC+= sBx
TFORLOOP (A, C) R(A+2), ... ,R(A+2+C) := R(A)(R(A+1), R(A+2));
TFORPREP (A, sBx) if type(R(A)) == table then R(A+1):=R(A), R(A):=next;
SETLIST (A, Bx) R(A)[Bx-Bx%FPF+i] := R(A+i), 1 <= i <= Bx%FPF+1
SETLISTO (A, Bx)
CLOSE (A) close stack variables up to R(A)
CLOSURE (A, Bx) R(A) := closure(KPROTO[Bx], R(A), ... ,R(A+n))
```

For function calls, Lua uses a kind of register window.
It evaluates the call arguments in successive registers, starting with the first unused register.
When it performs the call, those registers become part of the activation record of the called function, which therefore can access its parameters as regular local variables.
When this function returns, those registers are put back into the activation record of the caller.

When Lua compiles a function it generates a prototype containing the virtual machine instructions for the function, its constant values (numbers, literal strings, etc.), and some debug information.
At run time, whenever Lua executes a `function...end` expression, it creates a new closure.
Each closure has a reference to its corresponding prototype, a reference to its environment (a table wherein it looks for global variables), and an array of references to upvalues, which are used to access outer local variables.

This register-based machine also uses a stack, for allocating activation records, wherein the registers live.
When Lua enters a function, it preallocates from the stack an activation record large enough to hold all the function registers. All local variables are allocated in registers. As a consequence, access to local variables is specially efficient.

Lua uses two parallel stacks for function calls.
- One stack has one entry for each active function. 
  - Stores the function being called
  - The return address when the function does a call
  - A base index, which points to the activation record of the function
  
The other stack is simply a large array of Lua values that keeps those activation records. Each activation record keeps all temporary values of the function (parameters, local variables, etc.). Actually, we can see each entry in the second stack as a variable-size part of a corresponding entry in the first stack.

The list of operations are very similar to what Grotsky does. There are no mention of classes in the list, but we can emulate those using a table.

## Grotsky (naive) VM

This is a naive design for the VM, doesn't take into account nothing of the memory ownership and garbage collection. Basically it's just a high-level modelling using rust structures.

### Instruction format

Using bitfields in Rust is more complicated. Also, I don't want to mess too much with bitwise operations, so I want a simpler instruction format for Grotsky.

```rust
#[repr(u8)]
enum OpCode {
  ...
}

struct Instruction {
  opcode: OpCode,
  a: u8,
  b: u8,
  c: u8,
}
```

This means that we can have up to 256 registers/locals per function.

### Stack and function format

This is a "pseudocode" implementation of the data structures.

```rust
struct Upvalue {
  value: *mut Value,
  closed_value: Option<Value>, // Only active when the upvalue is closed
}

struct Function {
  prototype: Vec<Instruction>,
  upvalues: HashMap<String, Upvalue>,
  constants: Vec<Constant>,
}

struct StackEntry {
  function: Function,
  pc: usize, // Return location
  sp: usize, // Stack pointer inside activation record
}
```


### VM format


```rust
struct VM {
  instructions: Vec<Instruction>,
  globals: HashMap<String, Value>,
  stack: Vec<StackEntry>,
  activation_records: Vec<Value>,
  pc: usize,
}
```

### The Value interface

The most challenging part of the implementation is designing a Value interface that is performant and can handle all the mutability cases needed for Grotsky.

The possible values are:

- Class
- Object (class instance)
- Dict
- List
- Function
- Native
- Number (float64)
- String
- Bool
- nil

Let's separate between immutable and mutable types:

| Type     | Mutable |
| -------- | ------- |
| Class    | Yes     |
| Object   | Yes     |
| Dict     | Yes     |
| List     | Yes     |
| Function | Yes     |
| Native   | No      |
| Number   | No      |
| String   | No      |
| Bool     | No      |
| nil      | No      |


We need to create a type to support the mutability of different objects:

```rust
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug)]
struct MutValue<T>(Rc<RefCell<T>>);

impl<T> MutValue<T> {
    fn new(obj: T) -> Self {
        MutValue::<T>(Rc::new(RefCell::new(obj)))
    }
}

#[derive(Debug,Clone)]
enum Value {
    Class(MutValue<ClassValue>),
    Object(MutValue<ObjectValue>),
    Dict(MutValue<DictValue>),
    List(MutValue<ListValue>),
    Fn(MutValue<FnValue>),
    Native(NativeValue),
    Number(NumberValue),
    String(StringValue),
    Bool(BoolValue),
    Nil,
}
```


### Clojures and Classes

This are the most *difficult* values to implement. I would like to start doing a runtime implementation of those to cover the hardest thing first, and then build upon that.

```rust
#[repr(u8)]
pub enum OpCode {
  Call,
  Return,
  Close,
  SetUpval,
  GetUpval,
  SetGlobal,
  GetGlobal,
}

struct FnValue {
  prototype: Vec<Instruction>,
  upvalues: HashMap<String, Upvalue>,
  constants: Vec<Constant>,
}

struct ClassValue {
  name: String,
  superclass: Option<MutValue<ClassValue>>,
  methods: Vec<MutValue<FnValue>>,
  classmethods: Vec<MutValue<FnValue>>,
}
```


### Our Opcodes

```
Move (A, B) R(A) := R(B)
LoadK (A, Bx) R(A) := K[Bx]
LoadNil (A) R(A) := nil
Closure (A, Bx) R(A) := closure(KPROTO[Bx], R(A), ..., R(A+n))
Call (A, B, C) R(C-1) := R(A)(R(A+1), ... ,R(A+B-1))
Return (A, B) return R(A), ... ,R(A+B-2)
Test (A, B, C) if (R(B) <=> C) then R(A) := R(B) else PC++
Jmp (sBx) PC += sBx
Add (A, B, C) R(A) := R(B) + R(C)
Sub (A, B, C) R(A) := R(B) - R(C)
Div (A, B, C) R(A) := R(B) / R(C)
Mod (A, B, C) R(A) := R(B) % R(C)
Mul (A, B, C) R(A) := R(B) * R(C)
Pow (A, B, C) R(A) := R(B) ^ R(C)
Lt (A, B, C) R(A) := R(B) < R(C)
Lte (A, B, C) R(A) := R(B) <= R(C)
Gt (A, B, C) R(A) := R(B) > R(C)
Gte (A, B, C) R(A) := R(B) >= R(C)
Eq (A, B) R(A) := R(B) == R(C)
Neq (A, B) R(A) := R(B) != R(C)
Not (A, B) R(A) := !R(B)
Neg (A, B) R(A) := -R(B)
GetUpval (A, B) R(A) := U[B]
SetUpval (A, B) U[B] := R(A)
List (A) R(A) := []
PushList (A) R(A)[] := R(B)
Dict R(A) := {}
PushDict R(A)[R(B)] := R(C)
Slice (A, B) R(A) := slice(R(B))
Access (A, B, C) R(A) := R(B)[R(C)]
Set (A, B, C) R(A)[R(B)] := R(C)
Class (A, B) R(A) := class(name: R(B), superclass: R(C))
ClassMeth (A, B, C) R(A).R(B) := R(C)
ClassStMeth (A, B, C) R(A).R(B) := R(C)
GetObj (A, B, C) R(A) := R(B).R(C)
SetObj (A, B, C) R(A).R(B) := R(C)
Addi (A, B, Imm) R(A) := R(B) + Imm
GetIter (A, B, C) R(A) := R(B)[R(C)]
GetIteri (A, B, Imm) R(A) := R(B)[Imm]
Length (A, B) R(A) := R(B).length
```
