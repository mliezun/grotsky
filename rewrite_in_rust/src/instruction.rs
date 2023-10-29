#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum OpCode {
    Move,
    LoadK,
    LoadNil,
    Jmp,
    Test,
    Add,
    Sub,
    Div,
    Mod,
    Mul,
    Pow,
    Lt,
    Gt,
    Lte,
    Gte,
    Eq,
    Neq,
    Not,
    Neg,
    Call,
    Return,
    SetUpval,
    GetUpval,
    Closure,
    List,
    PushList,
    Dict,
    PushDict,
    Slice,
    Access,
    Set,
    Class,
    ClassMeth,
    ClassStMeth,
    GetObj,
    SetObj,
    Addi,
    GetIter,
    GetIterk,
    GetIteri,
    Length,
    Super,
    This,
    GetGlobal,
    SetGlobal,
    GetCurrentFunc,
    GetBuiltin,
    RegisterTryCatch,
    DeregisterTryCatch,
    GetExcept,
}

// To indicate if the JMP is a continue or break inside
// a loop we asssign to the unused R(A) register one of
// the following values
pub const JMP_CONTINUE: u8 = 1;
pub const JMP_BREAK: u8 = 2;

#[derive(Debug, Clone)]
pub struct Instruction {
    pub opcode: OpCode,
    pub a: u8,
    pub b: u8,
    pub c: u8,
}

impl Instruction {
    pub fn bx(&self) -> u16 {
        let b = self.b as u16;
        let c = self.c as u16;
        return b << 8 | c;
    }

    pub fn sbx(&self) -> i16 {
        let b = self.b as u16;
        let c = self.c as u16;
        let result = (b << 8 | c) as i16;
        // println!("sbx = {:#?}", result);
        return result;
    }

    pub fn is_continue(&mut self) -> bool {
        self.opcode == OpCode::Jmp && self.a == JMP_CONTINUE
    }

    pub fn is_break(&self) -> bool {
        self.opcode == OpCode::Jmp && self.a == JMP_BREAK
    }
}
