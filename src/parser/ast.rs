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
        imports:   Vec<ASTNodePos>,
        modules:   Vec<ASTNodePos>,
        type_defs: Vec<ASTNodePos>
    },
    Import {
        id:   Box<ASTNodePos>,
        _use: Vec<ASTNodePos>,
        all:  bool,
        _as:  Option<Box<ASTNodePos>>
    },
    Stateful {
        _type: Box<ASTNodePos>,
        body:  Box<ASTNodePos>
    },
    Stateless {
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

    ModName {
        name: String
    },
    ModNameIsA {
        name: String,
        isa:  Vec<String>
    },

    ReAssign {
        left:  Box<ASTNodePos>,
        right: Box<ASTNodePos>
    },
    Def {
        private:    bool,
        definition: Box<ASTNodePos>
    },
    VariableDef {
        mutable:       bool,
        ofmut:         bool,
        id_maybe_type: Box<ASTNodePos>,
        expression:    Option<Box<ASTNodePos>>,
        forward:       Option<Vec<ASTNodePos>>
    },
    FunDef {
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
    Handle {
        expr_or_stmt: Box<ASTNodePos>,
        cases:        Vec<ASTNodePos>
    },
    Retry,

    FunctionCall {
        namespace: Box<ASTNodePos>,
        name:      Box<ASTNodePos>,
        args:      Vec<ASTNodePos>
    },
    FunctionCallDirect {
        name: Box<ASTNodePos>,
        args: Vec<ASTNodePos>
    },
    MethodCall {
        instance: Box<ASTNodePos>,
        name:     Box<ASTNodePos>,
        args:     Vec<ASTNodePos>
    },
    Call {
        instance_or_met: Box<ASTNodePos>,
        met_or_arg:      Box<ASTNodePos>
    },

    Id {
        lit: String
    },

    IdType {
        id:    Box<ASTNodePos>,
        _type: Option<Box<ASTNodePos>>
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
        from: Box<ASTNodePos>,
        to:   Box<ASTNodePos>
    },
    RangeIncl {
        from: Box<ASTNodePos>,
        to:   Box<ASTNodePos>
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
        cond:  Vec<ASTNodePos>,
        then:  Box<ASTNodePos>,
        _else: Option<Box<ASTNodePos>>
    },
    Match {
        cond:  Vec<ASTNodePos>,
        cases: Vec<ASTNodePos>
    },
    Case {
        cond: Box<ASTNodePos>,
        expr: Box<ASTNodePos>
    },
    For {
        expr:       Vec<ASTNodePos>,
        collection: Box<ASTNodePos>,
        body:       Box<ASTNodePos>
    },
    While {
        cond: Vec<ASTNodePos>,
        body: Box<ASTNodePos>
    },
    Break,
    Continue,

    Return {
        expr: Box<ASTNodePos>
    },
    ReturnEmpty,
    UnderScore,

    QuestOr {
        _do:      Box<ASTNodePos>,
        _default: Box<ASTNodePos>
    },

    Print {
        expr: Box<ASTNodePos>
    }
}
