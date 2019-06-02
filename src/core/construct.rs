#[derive(Debug, PartialEq, Clone)]
pub enum Core {
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
        doc:         Option<String>,
        parents:     Vec<Core>,
        definitions: Vec<Core>
    },

    File {
        doc:        Option<String>,
        statements: Vec<Core>
    },

    FunctionCall {
        function: Box<Core>,
        args:     Vec<Core>
    },
    PropertyCall {
        object:   Box<Core>,
        property: Box<Core>
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
        doc:     Option<String>,
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
    FDivOp,
    FDiv {
        left:  Box<Core>,
        right: Box<Core>
    },
    Sqrt {
        expr: Box<Core>
    },

    BAnd {
        left:  Box<Core>,
        right: Box<Core>
    },
    BOr {
        left:  Box<Core>,
        right: Box<Core>
    },
    BXOr {
        left:  Box<Core>,
        right: Box<Core>
    },
    BOneCmpl {
        expr: Box<Core>
    },
    BLShift {
        left:  Box<Core>,
        right: Box<Core>
    },
    BRShift {
        left:  Box<Core>,
        right: Box<Core>
    },

    For {
        expr: Box<Core>,
        body: Box<Core>
    },
    Range {
        from: Box<Core>,
        to:   Box<Core>,
        step: Box<Core>
    },
    If {
        cond: Box<Core>,
        then: Box<Core>
    },
    IfElse {
        cond:  Box<Core>,
        then:  Box<Core>,
        _else: Box<Core>
    },
    Match {
        cond:  Box<Core>,
        cases: Vec<Core>
    },
    Case {
        cond: Box<Core>,
        body: Box<Core>
    },
    While {
        cond: Box<Core>,
        body: Box<Core>
    },
    In {
        left:  Box<Core>,
        right: Box<Core>
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
    },

    TryExcept {
        _try:   Box<Core>,
        except: Vec<Core>
    },
    Except {
        id:    Box<Core>,
        class: Box<Core>,
        body:  Box<Core>
    },
    Raise {
        error: Box<Core>
    },

    With {
        resource: Box<Core>,
        expr:     Box<Core>
    },
    WithAs {
        resource: Box<Core>,
        _as:      Box<Core>,
        expr:     Box<Core>
    }
}
