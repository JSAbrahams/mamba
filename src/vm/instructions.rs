pub enum Opcode {
    HALT,
    ILLEGAL,
    LOAD,

    ADD,
    SUB,
    MOD,
    POW,
    DIV,

    JMPF,
    JMPB,

    NOT,
    EQ,
    GT,
    LT,

    CALL,
}

pub struct Instruction {
    opcode: Opcode
}

impl Instruction {
    pub fn new(opcode: Opcode) -> Instruction {
        Instruction { opcode }
    }
}

impl From<u8> for Opcode {
    fn from(v: u8) -> Self {
        match v {
            0 => Opcode::LOAD,
            2 => Opcode::HALT,
            _ => Opcode::ILLEGAL
        }
    }
}
