#[derive(Debug, PartialEq, Clone)]
pub enum Core {
    Module {
        id:      String,
        imports: Vec<String>,
        body:    Box<Core>
    },
    FromImport {
        from:   Box<Core>,
        import: Box<Core>
    },
    Import {
        import: Vec<Core>
    },
    ImportAs {
        import: Vec<Core>,
        _as:    Vec<Core>
    },
    ClassDef {
        name:        Box<Core>,
        generics:    Vec<Core>,
        parents:     Vec<Core>,
        definitions: Vec<Core>
    },

    MethodCall {
        object: Box<Core>,
        method: String,
        args:   Vec<Core>
    },
    PropertyCall {
        object:   Box<Core>,
        property: String
    },

    Id {
        lit: String
    },
    Assign {
        left:  Box<Core>,
        right: Box<Core>
    },
    VarDef {
        private: bool,
        id:      Box<Core>,
        right:   Box<Core>
    },
    FunDef {
        private: bool,
        id:      Box<Core>,
        args:    Vec<Core>,
        body:    Box<Core>
    },
    FunArg {
        vararg:  bool,
        id:      Box<Core>,
        default: Box<Core>
    },
    AnonFun {
        args: Vec<Core>,
        body: Box<Core>
    },

    Block {
        statements: Vec<Core>
    },

    Float {
        float: String
    },
    Int {
        int: String
    },
    ENum {
        num: String,
        exp: String
    },
    Str {
        _str: String
    },
    Bool {
        _bool: bool
    },

    Tuple {
        elements: Vec<Core>
    },
    Set {
        elements: Vec<Core>
    },
    List {
        elements: Vec<Core>
    },

    GeOp,
    Ge {
        left:  Box<Core>,
        right: Box<Core>
    },
    GeqOp,

    Geq {
        left:  Box<Core>,
        right: Box<Core>
    },
    LeOp,
    Le {
        left:  Box<Core>,
        right: Box<Core>
    },
    LeqOp,
    Leq {
        left:  Box<Core>,
        right: Box<Core>
    },

    Not {
        expr: Box<Core>
    },
    Is {
        left:  Box<Core>,
        right: Box<Core>
    },
    IsN {
        left:  Box<Core>,
        right: Box<Core>
    },
    EqOp,
    Eq {
        left:  Box<Core>,
        right: Box<Core>
    },
    NeqOp,
    Neq {
        left:  Box<Core>,
        right: Box<Core>
    },
    IsA {
        left:  Box<Core>,
        right: Box<Core>
    },
    And {
        left:  Box<Core>,
        right: Box<Core>
    },
    Or {
        left:  Box<Core>,
        right: Box<Core>
    },

    AddOp,
    Add {
        left:  Box<Core>,
        right: Box<Core>
    },
    AddU {
        expr: Box<Core>
    },
    SubOp,
    Sub {
        left:  Box<Core>,
        right: Box<Core>
    },
    SubU {
        expr: Box<Core>
    },
    MulOp,
    Mul {
        left:  Box<Core>,
        right: Box<Core>
    },
    ModOp,
    Mod {
        left:  Box<Core>,
        right: Box<Core>
    },
    PowOp,
    Pow {
        left:  Box<Core>,
        right: Box<Core>
    },
    DivOp,
    Div {
        left:  Box<Core>,
        right: Box<Core>
    },
    Sqrt {
        expr: Box<Core>
    },

    For {
        exprs:      Vec<Core>,
        collection: Box<Core>,
        body:       Box<Core>
    },
    If {
        cond: Vec<Core>,
        then: Box<Core>
    },
    IfElse {
        cond:  Vec<Core>,
        then:  Box<Core>,
        _else: Box<Core>
    },
    Match {
        cond:  Vec<Core>,
        cases: Vec<Core>
    },
    Case {
        cond: Box<Core>,
        body: Box<Core>
    },
    While {
        cond: Vec<Core>,
        body: Box<Core>
    },
    Break,
    Continue,

    Return {
        expr: Box<Core>
    },
    Print {
        expr: Box<Core>
    },
    UnderScore,

    Pass,
    None,
    Empty,
    Comment {
        comment: String
    }
}
