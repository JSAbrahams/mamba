pub enum Type {
    Empty,
    Undefined,

    Int,
    ENum,
    Float,
    String,
    Bool,

    Set { ty: Box<Type> },
    List { ty: Box<Type> },
    Map { ty: Box<Type> },
    Tuple { ty: Vec<Type> },

    Range { ty: Box<Type> },
    AnonFun { arg: Vec<Type>, out: Box<Type> },

    Custom { lit: String },
    Maybe { ty: Box<Type> },
}
