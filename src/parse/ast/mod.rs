use std::fmt::Debug;

use crate::common::position::Position;

pub mod node;

/// Wrapper of Node, and its start end end position in the source code.
/// The start and end positions can be used to generate useful error messages.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct AST {
    pub pos: Position,
    pub node: Node,
}

impl AST {
    pub fn new(pos: &Position, node: Node) -> AST { AST { pos: pos.clone(), node } }

    pub fn same_value(&self, other: &AST) -> bool { self.node.same_value(&other.node) }

    #[must_use]
    pub fn map(&self, mapping: &dyn Fn(&Node) -> Node) -> AST {
        AST {
            pos: self.pos.clone(),
            node: self.node.map(mapping),
        }
    }
}

type OptAST = Option<Box<AST>>;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Node {
    File { pure: bool, statements: Vec<AST> },
    Import { import: Vec<AST>, aliases: Vec<AST> },
    FromImport { id: Box<AST>, import: Box<AST> },
    Class { ty: Box<AST>, args: Vec<AST>, parents: Vec<AST>, body: OptAST },
    Generic { id: Box<AST>, isa: OptAST },
    Parent { ty: Box<AST>, args: Vec<AST> },
    Init,
    Reassign { left: Box<AST>, right: Box<AST> },
    VariableDef { mutable: bool, var: Box<AST>, ty: OptAST, expr: OptAST, forward: Vec<AST> },
    FunDef { pure: bool, id: Box<AST>, args: Vec<AST>, ret: OptAST, raises: Vec<AST>, body: OptAST },
    AnonFun { args: Vec<AST>, body: Box<AST> },
    Raises { expr_or_stmt: Box<AST>, errors: Vec<AST> },
    Raise { error: Box<AST> },
    Handle { expr_or_stmt: Box<AST>, cases: Vec<AST> },
    With { resource: Box<AST>, alias: Option<(Box<AST>, bool, Option<Box<AST>>)>, expr: Box<AST> },
    FunctionCall { name: Box<AST>, args: Vec<AST> },
    PropertyCall { instance: Box<AST>, property: Box<AST> },
    Id { lit: String },
    ExpressionType { expr: Box<AST>, mutable: bool, ty: OptAST },
    TypeDef { ty: Box<AST>, isa: OptAST, body: OptAST },
    TypeAlias { ty: Box<AST>, isa: Box<AST>, conditions: Vec<AST> },
    TypeTup { types: Vec<AST> },
    TypeUnion { types: Vec<AST> },
    Type { id: Box<AST>, generics: Vec<AST> },
    TypeFun { args: Vec<AST>, ret_ty: Box<AST> },
    Condition { cond: Box<AST>, el: OptAST },
    FunArg { vararg: bool, mutable: bool, var: Box<AST>, ty: OptAST, default: OptAST },
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
    Set { elements: Vec<AST> },
    SetBuilder { item: Box<AST>, conditions: Vec<AST> },
    List { elements: Vec<AST> },
    ListBuilder { item: Box<AST>, conditions: Vec<AST> },
    Tuple { elements: Vec<AST> },
    Range { from: Box<AST>, to: Box<AST>, inclusive: bool, step: OptAST },
    Slice { from: Box<AST>, to: Box<AST>, inclusive: bool, step: OptAST },
    Index { item: Box<AST>, range: Box<AST> },
    Block { statements: Vec<AST> },
    Real { lit: String },
    Int { lit: String },
    ENum { num: String, exp: String },
    Str { lit: String, expressions: Vec<AST> },
    DocStr { lit: String },
    Bool { lit: bool },
    Add { left: Box<AST>, right: Box<AST> },
    AddU { expr: Box<AST> },
    Sub { left: Box<AST>, right: Box<AST> },
    SubU { expr: Box<AST> },
    Mul { left: Box<AST>, right: Box<AST> },
    Div { left: Box<AST>, right: Box<AST> },
    FDiv { left: Box<AST>, right: Box<AST> },
    Mod { left: Box<AST>, right: Box<AST> },
    Pow { left: Box<AST>, right: Box<AST> },
    Sqrt { expr: Box<AST> },
    BAnd { left: Box<AST>, right: Box<AST> },
    BOr { left: Box<AST>, right: Box<AST> },
    BXOr { left: Box<AST>, right: Box<AST> },
    BOneCmpl { expr: Box<AST> },
    BLShift { left: Box<AST>, right: Box<AST> },
    BRShift { left: Box<AST>, right: Box<AST> },
    Le { left: Box<AST>, right: Box<AST> },
    Ge { left: Box<AST>, right: Box<AST> },
    Leq { left: Box<AST>, right: Box<AST> },
    Geq { left: Box<AST>, right: Box<AST> },
    Is { left: Box<AST>, right: Box<AST> },
    IsN { left: Box<AST>, right: Box<AST> },
    Eq { left: Box<AST>, right: Box<AST> },
    Neq { left: Box<AST>, right: Box<AST> },
    IsA { left: Box<AST>, right: Box<AST> },
    IsNA { left: Box<AST>, right: Box<AST> },
    Not { expr: Box<AST> },
    And { left: Box<AST>, right: Box<AST> },
    Or { left: Box<AST>, right: Box<AST> },
    IfElse { cond: Box<AST>, then: Box<AST>, el: OptAST },
    Match { cond: Box<AST>, cases: Vec<AST> },
    Case { cond: Box<AST>, body: Box<AST> },
    For { expr: Box<AST>, col: Box<AST>, body: Box<AST> },
    In { left: Box<AST>, right: Box<AST> },
    Step { amount: Box<AST> },
    While { cond: Box<AST>, body: Box<AST> },
    Break,
    Continue,
    Return { expr: Box<AST> },
    ReturnEmpty,
    Underscore,
    Undefined,
    Pass,
    Question { left: Box<AST>, right: Box<AST> },
    QuestionOp { expr: Box<AST> },
    Print { expr: Box<AST> },
    Comment { comment: String },
}

#[cfg(test)]
mod test {
    use crate::common::position::{CaretPos, Position};
    use crate::parse::ast::{AST, Node};

    #[test]
    fn simple_ast() {
        let pos = Position::new(&CaretPos::new(3, 403), &CaretPos::new(324, 673));
        let node = Node::Id { lit: String::from("fd") };

        let ast = AST::new(&pos, node.clone());

        assert_eq!(ast.pos, pos);
        assert_eq!(ast.node, node);
    }
}
