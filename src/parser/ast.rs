use crate::common::position::Position;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
/// Wrapper of Node, and its start end end position in the source code.
/// The start and end positions can be used to generate useful error messages.
pub struct AST {
    pub pos:  Position,
    pub node: Node
}

impl AST {
    pub fn new(pos: &Position, node: Node) -> AST { AST { pos: pos.clone(), node } }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Node {
    File {
        pure:    bool,
        modules: Vec<AST>
    },
    Import {
        import: Vec<AST>,
        _as:    Vec<AST>
    },
    FromImport {
        id:     Box<AST>,
        import: Box<AST>
    },
    Class {
        _type:   Box<AST>,
        args:    Vec<AST>,
        parents: Vec<AST>,
        body:    Option<Box<AST>>
    },
    Generic {
        id:  Box<AST>,
        isa: Option<Box<AST>>
    },
    Parent {
        id:       Box<AST>,
        generics: Vec<AST>,
        args:     Vec<AST>
    },
    Script {
        statements: Vec<AST>
    },
    Init,

    Reassign {
        left:  Box<AST>,
        right: Box<AST>
    },
    VariableDef {
        private:    bool,
        mutable:    bool,
        var:        Box<AST>,
        ty:         Option<Box<AST>>,
        expression: Option<Box<AST>>,
        forward:    Vec<AST>
    },
    FunDef {
        pure:     bool,
        private:  bool,
        id:       Box<AST>,
        fun_args: Vec<AST>,
        ret_ty:   Option<Box<AST>>,
        raises:   Vec<AST>,
        body:     Option<Box<AST>>
    },

    AnonFun {
        args: Vec<AST>,
        body: Box<AST>
    },

    Raises {
        expr_or_stmt: Box<AST>,
        errors:       Vec<AST>
    },
    Raise {
        error: Box<AST>
    },
    Handle {
        expr_or_stmt: Box<AST>,
        cases:        Vec<AST>
    },
    With {
        resource: Box<AST>,
        _as:      Option<Box<AST>>,
        expr:     Box<AST>
    },

    ConstructorCall {
        name: Box<AST>,
        args: Vec<AST>
    },
    FunctionCall {
        name: Box<AST>,
        args: Vec<AST>
    },
    PropertyCall {
        instance: Box<AST>,
        property: Box<AST>
    },

    Id {
        lit: String
    },
    ExpressionType {
        expr:    Box<AST>,
        mutable: bool,
        ty:      Option<Box<AST>>
    },

    TypeDef {
        _type: Box<AST>,
        isa:   Option<Box<AST>>,
        body:  Option<Box<AST>>
    },
    TypeAlias {
        _type:      Box<AST>,
        isa:        Box<AST>,
        conditions: Vec<AST>
    },
    TypeTup {
        types: Vec<AST>
    },
    TypeUnion {
        types: Vec<AST>
    },
    Type {
        id:       Box<AST>,
        generics: Vec<AST>
    },
    TypeFun {
        args:   Vec<AST>,
        ret_ty: Box<AST>
    },
    Condition {
        cond:  Box<AST>,
        _else: Option<Box<AST>>
    },
    FunArg {
        vararg:  bool,
        mutable: bool,
        var:     Box<AST>,
        ty:      Option<Box<AST>>,
        default: Option<Box<AST>>
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
        elements: Vec<AST>
    },
    SetBuilder {
        item:       Box<AST>,
        conditions: Vec<AST>
    },
    List {
        elements: Vec<AST>
    },
    ListBuilder {
        item:       Box<AST>,
        conditions: Vec<AST>
    },
    Tuple {
        elements: Vec<AST>
    },

    Range {
        from:      Box<AST>,
        to:        Box<AST>,
        inclusive: bool,
        step:      Option<Box<AST>>
    },

    Block {
        statements: Vec<AST>
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
        lit:         String,
        expressions: Vec<AST>
    },
    DocStr {
        lit: String
    },
    Bool {
        lit: bool
    },

    Add {
        left:  Box<AST>,
        right: Box<AST>
    },
    AddU {
        expr: Box<AST>
    },
    Sub {
        left:  Box<AST>,
        right: Box<AST>
    },
    SubU {
        expr: Box<AST>
    },
    Mul {
        left:  Box<AST>,
        right: Box<AST>
    },
    Div {
        left:  Box<AST>,
        right: Box<AST>
    },
    FDiv {
        left:  Box<AST>,
        right: Box<AST>
    },
    Mod {
        left:  Box<AST>,
        right: Box<AST>
    },
    Pow {
        left:  Box<AST>,
        right: Box<AST>
    },
    Sqrt {
        expr: Box<AST>
    },

    BAnd {
        left:  Box<AST>,
        right: Box<AST>
    },
    BOr {
        left:  Box<AST>,
        right: Box<AST>
    },
    BXOr {
        left:  Box<AST>,
        right: Box<AST>
    },
    BOneCmpl {
        expr: Box<AST>
    },
    BLShift {
        left:  Box<AST>,
        right: Box<AST>
    },
    BRShift {
        left:  Box<AST>,
        right: Box<AST>
    },

    Le {
        left:  Box<AST>,
        right: Box<AST>
    },
    Ge {
        left:  Box<AST>,
        right: Box<AST>
    },
    Leq {
        left:  Box<AST>,
        right: Box<AST>
    },
    Geq {
        left:  Box<AST>,
        right: Box<AST>
    },
    Is {
        left:  Box<AST>,
        right: Box<AST>
    },
    IsN {
        left:  Box<AST>,
        right: Box<AST>
    },
    Eq {
        left:  Box<AST>,
        right: Box<AST>
    },
    Neq {
        left:  Box<AST>,
        right: Box<AST>
    },
    IsA {
        left:  Box<AST>,
        right: Box<AST>
    },
    IsNA {
        left:  Box<AST>,
        right: Box<AST>
    },
    Not {
        expr: Box<AST>
    },
    And {
        left:  Box<AST>,
        right: Box<AST>
    },
    Or {
        left:  Box<AST>,
        right: Box<AST>
    },

    IfElse {
        cond:  Box<AST>,
        then:  Box<AST>,
        _else: Option<Box<AST>>
    },
    Match {
        cond:  Box<AST>,
        cases: Vec<AST>
    },
    Case {
        cond: Box<AST>,
        body: Box<AST>
    },
    For {
        expr: Box<AST>,
        col:  Box<AST>,
        body: Box<AST>
    },
    In {
        left:  Box<AST>,
        right: Box<AST>
    },
    Step {
        amount: Box<AST>
    },
    While {
        cond: Box<AST>,
        body: Box<AST>
    },
    Break,
    Continue,

    Return {
        expr: Box<AST>
    },
    ReturnEmpty,
    Underscore,
    Undefined,
    Pass,

    Question {
        left:  Box<AST>,
        right: Box<AST>
    },
    QuestionOp {
        expr: Box<AST>
    },

    Print {
        expr: Box<AST>
    },
    Comment {
        comment: String
    }
}
