#[derive(Debug, Clone)]
pub enum Type {
    Empty,

    Int,
    Float,
    String,
    Bool,

    Set { ty: Box<Type> },
    List { ty: Box<Type> },
    Tuple { ty: Vec<Type> },

    Range { ty: Box<Type> },
    AnonFun { arg: Vec<Type>, out: Box<Type> },

    Maybe { ty: Box<Type> },

    NA,
    Any
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (.., Type::Any) | (Type::Any, ..) => true,
            (.., Type::NA) | (Type::NA, ..) => false,

            (Type::Empty, Type::Empty)
            | (Type::Float, Type::Float)
            | (Type::Int, Type::Int)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String) => true,

            (Type::Set { ty: ty_self }, Type::Set { ty: ty_other }) => ty_self == ty_other,
            (Type::List { ty: ty_self }, Type::List { ty: ty_other }) => ty_self == ty_other,
            (Type::Tuple { ty: ty_self }, Type::Tuple { ty: ty_other }) =>
                ty_self.len() == ty_other.len()
                    && ty_self.iter().zip(ty_other).all(|(left, right)| left == right),

            (Type::Range { ty: ty_self }, Type::Range { ty: ty_other }) => ty_self == ty_other,
            (
                Type::AnonFun { arg: arg_self, out: out_self },
                Type::AnonFun { arg: arg_other, out: out_other }
            ) =>
                out_self == out_other
                    && arg_self.len() == arg_other.len()
                    && arg_self.iter().zip(arg_other).all(|(left, right)| left == right),

            (Type::Maybe { ty: lit_self }, Type::Maybe { ty: lit_other }) => lit_self == lit_other,

            _ => false
        }
    }
}
