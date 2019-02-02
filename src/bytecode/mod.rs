use crate::bytecode::instructions::Bytecode;
use crate::desugarer::Core;

mod instructions;

type BytecodeInstr = Vec<Bytecode>;

pub fn to_bytecode(core: Core) -> BytecodeInstr {
    return BytecodeInstr::new();
}
