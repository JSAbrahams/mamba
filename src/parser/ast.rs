#[derive(PartialEq, Eq, Hash, Debug, Clone)]
/// Wrapper of ASTNode, and its start end end position in the source code.
/// The start and end positions can be used to generate useful error messages.
pub struct ASTNodePos {
    pub st_line: i32,
    pub st_pos:  i32,
    pub en_line: i32,
    pub en_pos:  i32,
    pub node:    ASTNode
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum ASTNode {
    File {
        pure:      bool,
        imports:   Vec<ASTNodePos>,
        modules:   Vec<ASTNodePos>,
        type_defs: Vec<ASTNodePos>
    },
    Import {
        import: Vec<ASTNodePos>,
        _as:    Vec<ASTNodePos>
    },
    FromImport {
        id:     Box<ASTNodePos>,
        import: Box<ASTNodePos>
    },
    Class {
        _type: Box<ASTNodePos>,
        body:  Box<ASTNodePos>
    },
    Script {
        statements: Vec<ASTNodePos>
    },
    Body {
        isa:         Vec<ASTNodePos>,
        definitions: Vec<ASTNodePos>
    },
    Init,

    Reassign {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Def {
        private:    bool,
        definition: Box<ASTNodePos>
    },
    VariableDef {
        ofmut:         bool,
        id_maybe_type: Box<ASTNodePos>,
        expression:    Option<Box<ASTNodePos>>,
        forward:       Vec<ASTNodePos>
    },
    FunDef {
        pure:     bool,
        id:       Box<ASTNodePos>,
        fun_args: Vec<ASTNodePos>,
        ret_ty:   Option<Box<ASTNodePos>>,
        raises:   Option<Vec<ASTNodePos>>,
        body:     Option<Box<ASTNodePos>>
    },

    AnonFun {
        args: Vec<ASTNodePos>,
        body: Box<ASTNodePos>
    },

    Raises {
        expr_or_stmt: Box<ASTNodePos>,
        errors:       Vec<ASTNodePos>
    },
    Raise {
        error: Box<ASTNodePos>
    },
    Handle {
        expr_or_stmt: Box<ASTNodePos>,
        cases:        Vec<ASTNodePos>
    },
    Retry,
    With {
        resource: Box<ASTNodePos>,
        _as:      Option<Box<ASTNodePos>>,
        expr:     Box<ASTNodePos>
    },

    DirectCall {
        name: Box<ASTNodePos>,
        args: Vec<ASTNodePos>
    },
    MethodCall {
        instance: Box<ASTNodePos>,
        name:     Box<ASTNodePos>,
        args:     Vec<ASTNodePos>
    },
    Call {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },

    Id {
        lit: String
    },

    IdType {
        id:      Box<ASTNodePos>,
        mutable: bool,
        _type:   Option<Box<ASTNodePos>>
    },
    TypeDef {
        _type: Box<ASTNodePos>,
        body:  Option<Box<ASTNodePos>>
    },
    TypeAlias {
        _type:      Box<ASTNodePos>,
        conditions: Option<Vec<ASTNodePos>>
    },
    TypeTup {
        types: Vec<ASTNodePos>
    },
    Type {
        id:       Box<ASTNodePos>,
        generics: Vec<ASTNodePos>
    },
    TypeFun {
        _type: Box<ASTNodePos>,
        body:  Box<ASTNodePos>
    },
    Condition {
        cond:  Box<ASTNodePos>,
        _else: Option<Box<ASTNodePos>>
    },
    FunArg {
        vararg:        bool,
        id_maybe_type: Box<ASTNodePos>,
        default:       Option<Box<ASTNodePos>>
    },

    _Self,
    AddOp,
    SubOp,
    SqrtOp,
    MulOp,
    FDivOp,
    DivOp,
    PowOp,
    ModOp,
    EqOp,
    LeOp,
    GeOp,

    Set {
        elements: Vec<ASTNodePos>
    },
    SetBuilder {
        items:      Box<ASTNodePos>,
        conditions: Vec<ASTNodePos>
    },
    List {
        elements: Vec<ASTNodePos>
    },
    ListBuilder {
        items:      Box<ASTNodePos>,
        conditions: Vec<ASTNodePos>
    },
    Tuple {
        elements: Vec<ASTNodePos>
    },

    Range {
        from:      Box<ASTNodePos>,
        to:        Box<ASTNodePos>,
        inclusive: bool,
        step:      Option<Box<ASTNodePos>>
    },

    Block {
        statements: Vec<ASTNodePos>
    },

    Real {
        lit: String
    },
    Int {
        lit: String
    },
    ENum {
        num: String,
        exp: String
    },
    Str {
        lit: String
    },
    Bool {
        lit: bool
    },

    Add {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    AddU {
        expr: Box<ASTNodePos>
    },
    Sub {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    SubU {
        expr: Box<ASTNodePos>
    },
    Mul {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Div {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    FDiv {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Mod {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Pow {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Sqrt {
        expr: Box<ASTNodePos>
    },

    BAnd {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    BOr {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    BXOr {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    BOneCmpl {
        expr: Box<ASTNodePos>
    },
    BLShift {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    BRShift {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },

    Le {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Ge {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Leq {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Geq {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Is {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    IsN {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Eq {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Neq {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    IsA {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    IsNA {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Not {
        expr: Box<ASTNodePos>
    },
    And {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Or {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },

    IfElse {
        cond:  Box<ASTNodePos>,
        then:  Box<ASTNodePos>,
        _else: Option<Box<ASTNodePos>>
    },
    Match {
        cond:  Box<ASTNodePos>,
        cases: Vec<ASTNodePos>
    },
    Case {
        cond: Box<ASTNodePos>,
        body: Box<ASTNodePos>
    },
    For {
        expr: Box<ASTNodePos>,
        body: Box<ASTNodePos>
    },
    In {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Step {
        amount: Box<ASTNodePos>
    },
    While {
        cond: Box<ASTNodePos>,
        body: Box<ASTNodePos>
    },
    Break,
    Continue,

    Return {
        expr: Box<ASTNodePos>
    },
    ReturnEmpty,
    Underscore,
    Pass,

    QuestOr {
        _do:      Box<ASTNodePos>,
        _default: Box<ASTNodePos>
    },

    Print {
        expr: Box<ASTNodePos>
    },
    Comment {
        comment: String
    }
}
