use crate::desugarer::Core;

pub enum Value {
    Empty
}

pub fn interpret(core: Core) -> Value {
    return Value::Empty;
}