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
}

#[derive(Debug, Clone)]
#[repr(u8)]
pub enum OpCode {
    Move,
    LoadK,
    Call,
    Return,
    Close,
    SetUpval,
    GetUpval,
    SetGlobal,
    GetGlobal,
    Closure,
    Add,
    Lt,
    Jmp,
    Test,
}