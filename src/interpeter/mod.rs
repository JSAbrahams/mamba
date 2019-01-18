use crate::desugarer::Core;
use std::fmt;
use std::fmt::Formatter;

pub enum Value {
    Real { float: f64 },
    BigReal { int_digits: Vec<i64>, frac_digits: Vec<i64> },
    Int { int: i64 },
    BigInt { integer_digits: Vec<i64> },
    ENum { base: f64, exp: i64 },
    Str { _str: String },
    Bool { _bool: bool },
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Value::Real { float } => write!(f, "{}", float),
            Value::Int { int } => write!(f, "{}", int),
            Value::ENum { base, exp } => write!(f, "{}e{}", base, exp),
            Value::Str { _str } => write!(f, "{}", _str),
            Value::Bool { _bool } => write!(f, "{}", _bool),

            _ => unimplemented!()
        }
    }
}

pub fn interpret(core: Core) -> Value {
    return Value::Int { int: 0 };
}
