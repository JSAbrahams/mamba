use crate::type_checker::type_node::Type;

pub struct Context {
    pub interfaces: Vec<Interface>,
    pub classes:    Vec<Class>
}

pub struct Interface {
    id:        String,
    fields:    Vec<Field>,
    functions: Vec<Function>
}

pub struct Class {
    id:         String,
    init:       Option<Function>,
    implements: Vec<Interface>,
    fields:     Vec<Field>,
    functions:  Vec<Function>
}

pub struct Field {
    id:      String,
    mutable: bool,
    public:  bool,
    ty:      Type
}

pub struct Function {
    id:     String,
    public: bool,
    args:   Vec<FunctionArg>,
    ret:    Type,
    raises: Interface
}

pub struct FunctionArg {
    id: String,
    ty: Type
}
