#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Core {
    FromImport {
        from:   Box<Core>,
        import: Box<Core>
    },
    Import {
        imports: Vec<Core>
    },
    ImportAs {
        imports: Vec<Core>,
        alias:   Vec<Core>
    },
    ClassDef {
        name:         Box<Core>,
        parent_names: Vec<Core>,
        body:         Box<Core>
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
    Type {
        lit:      String,
        generics: Vec<Core>
    },
    ExpressionType {
        expr: Box<Core>,
        ty:   Box<Core>
    },
    Assign {
        left:  Box<Core>,
        right: Box<Core>
    },
    VarDef {
        var:  Box<Core>,
        ty:   Option<Box<Core>>,
        expr: Option<Box<Core>>
    },
    FunDef {
        id:   Box<Core>,
        arg:  Vec<Core>,
        ty:   Option<Box<Core>>,
        body: Box<Core>
    },
    FunArg {
        vararg:  bool,
        var:     Box<Core>,
        ty:      Option<Box<Core>>,
        default: Option<Box<Core>>
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
    DocStr {
        string: String
    },
    Str {
        string: String
    },
    FStr {
        string: String
    },
    Bool {
        boolean: bool
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
    Range {
        from: Box<Core>,
        to:   Box<Core>,
        step: Box<Core>
    },
    Slice {
        from: Box<Core>,
        to:   Box<Core>,
        step: Box<Core>
    },
    Index {
        item:  Box<Core>,
        range: Box<Core>
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
        col:  Box<Core>,
        body: Box<Core>
    },
    If {
        cond: Box<Core>,
        then: Box<Core>
    },
    IfElse {
        cond: Box<Core>,
        then: Box<Core>,
        el:   Box<Core>
    },
    Ternary {
        cond: Box<Core>,
        then: Box<Core>,
        el:   Box<Core>
    },
    Dictionary {
        expr:  Box<Core>,
        cases: Vec<Core>
    },
    DefaultDictionary {
        expr:    Box<Core>,
        cases:   Vec<Core>,
        default: Box<Core>
    },
    KeyValue {
        key:   Box<Core>,
        value: Box<Core>
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
        setup:   Option<Box<Core>>,
        attempt: Box<Core>,
        except:  Vec<Core>
    },
    Except {
        id:    Box<Core>,
        class: Option<Box<Core>>,
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
        alias:    Box<Core>,
        expr:     Box<Core>
    }
}
