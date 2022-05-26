use std::ops::Deref;

use crate::check::name::Name;
use crate::common::position::Position;
use crate::parse::ast::AST;
use crate::parse::ast::node_op::NodeOp;

pub mod node;
pub mod pos_name;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ASTTy {
    pub pos: Position,
    pub node: NodeTy,
    pub ty: Option<Name>,
}

impl From<&AST> for ASTTy {
    fn from(ast: &AST) -> Self {
        ASTTy { pos: ast.pos.clone(), node: NodeTy::from(&ast.node), ty: None }
    }
}

impl From<Box<AST>> for ASTTy {
    fn from(ast: Box<AST>) -> Self {
        ASTTy::from(ast.deref())
    }
}

impl From<&Box<AST>> for ASTTy {
    fn from(ast: &Box<AST>) -> Self {
        ASTTy::from(ast.deref())
    }
}

impl ASTTy {
    pub fn with_ty(self, ty: &Name) -> ASTTy {
        trace!("Annotated AST at {} with '{}'", self.pos, ty);
        ASTTy { ty: Some(ty.clone()), ..self }
    }
}

type OptASTTy = Option<Box<ASTTy>>;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeTy {
    File { pure: bool, statements: Vec<ASTTy> },
    Import { from: Option<Box<ASTTy>>, import: Vec<ASTTy>, alias: Vec<ASTTy> },
    Class { ty: Box<ASTTy>, args: Vec<ASTTy>, parents: Vec<ASTTy>, body: OptASTTy },
    Generic { id: Box<ASTTy>, isa: OptASTTy },
    Parent { ty: Box<ASTTy>, args: Vec<ASTTy> },
    Reassign { left: Box<ASTTy>, right: Box<ASTTy>, op: NodeOp },
    VariableDef { mutable: bool, var: Box<ASTTy>, ty: OptASTTy, expr: OptASTTy, forward: Vec<ASTTy> },
    FunDef { pure: bool, id: Box<ASTTy>, args: Vec<ASTTy>, ret: OptASTTy, raises: Vec<ASTTy>, body: OptASTTy },
    AnonFun { args: Vec<ASTTy>, body: Box<ASTTy> },
    Raises { expr_or_stmt: Box<ASTTy>, errors: Vec<ASTTy> },
    Raise { error: Box<ASTTy> },
    Handle { expr_or_stmt: Box<ASTTy>, cases: Vec<ASTTy> },
    With { resource: Box<ASTTy>, alias: Option<(Box<ASTTy>, bool, Option<Box<ASTTy>>)>, expr: Box<ASTTy> },
    FunctionCall { name: Box<ASTTy>, args: Vec<ASTTy> },
    PropertyCall { instance: Box<ASTTy>, property: Box<ASTTy> },
    Id { lit: String },
    ExpressionType { expr: Box<ASTTy>, mutable: bool, ty: OptASTTy },
    TypeDef { ty: Box<ASTTy>, isa: OptASTTy, body: OptASTTy },
    TypeAlias { ty: Box<ASTTy>, isa: Box<ASTTy>, conditions: Vec<ASTTy> },
    TypeTup { types: Vec<ASTTy> },
    TypeUnion { types: Vec<ASTTy> },
    Type { id: Box<ASTTy>, generics: Vec<ASTTy> },
    TypeFun { args: Vec<ASTTy>, ret_ty: Box<ASTTy> },
    Condition { cond: Box<ASTTy>, el: OptASTTy },
    FunArg { vararg: bool, mutable: bool, var: Box<ASTTy>, ty: OptASTTy, default: OptASTTy },
    Set { elements: Vec<ASTTy> },
    SetBuilder { item: Box<ASTTy>, conditions: Vec<ASTTy> },
    List { elements: Vec<ASTTy> },
    ListBuilder { item: Box<ASTTy>, conditions: Vec<ASTTy> },
    Tuple { elements: Vec<ASTTy> },
    Range { from: Box<ASTTy>, to: Box<ASTTy>, inclusive: bool, step: OptASTTy },
    Slice { from: Box<ASTTy>, to: Box<ASTTy>, inclusive: bool, step: OptASTTy },
    Index { item: Box<ASTTy>, range: Box<ASTTy> },
    Block { statements: Vec<ASTTy> },
    Real { lit: String },
    Int { lit: String },
    ENum { num: String, exp: String },
    Str { lit: String, expressions: Vec<ASTTy> },
    DocStr { lit: String },
    Bool { lit: bool },
    Add { left: Box<ASTTy>, right: Box<ASTTy> },
    AddU { expr: Box<ASTTy> },
    Sub { left: Box<ASTTy>, right: Box<ASTTy> },
    SubU { expr: Box<ASTTy> },
    Mul { left: Box<ASTTy>, right: Box<ASTTy> },
    Div { left: Box<ASTTy>, right: Box<ASTTy> },
    FDiv { left: Box<ASTTy>, right: Box<ASTTy> },
    Mod { left: Box<ASTTy>, right: Box<ASTTy> },
    Pow { left: Box<ASTTy>, right: Box<ASTTy> },
    Sqrt { expr: Box<ASTTy> },
    BAnd { left: Box<ASTTy>, right: Box<ASTTy> },
    BOr { left: Box<ASTTy>, right: Box<ASTTy> },
    BXOr { left: Box<ASTTy>, right: Box<ASTTy> },
    BOneCmpl { expr: Box<ASTTy> },
    BLShift { left: Box<ASTTy>, right: Box<ASTTy> },
    BRShift { left: Box<ASTTy>, right: Box<ASTTy> },
    Le { left: Box<ASTTy>, right: Box<ASTTy> },
    Ge { left: Box<ASTTy>, right: Box<ASTTy> },
    Leq { left: Box<ASTTy>, right: Box<ASTTy> },
    Geq { left: Box<ASTTy>, right: Box<ASTTy> },
    Is { left: Box<ASTTy>, right: Box<ASTTy> },
    IsN { left: Box<ASTTy>, right: Box<ASTTy> },
    Eq { left: Box<ASTTy>, right: Box<ASTTy> },
    Neq { left: Box<ASTTy>, right: Box<ASTTy> },
    IsA { left: Box<ASTTy>, right: Box<ASTTy> },
    IsNA { left: Box<ASTTy>, right: Box<ASTTy> },
    Not { expr: Box<ASTTy> },
    And { left: Box<ASTTy>, right: Box<ASTTy> },
    Or { left: Box<ASTTy>, right: Box<ASTTy> },
    IfElse { cond: Box<ASTTy>, then: Box<ASTTy>, el: OptASTTy },
    Match { cond: Box<ASTTy>, cases: Vec<ASTTy> },
    Case { cond: Box<ASTTy>, body: Box<ASTTy> },
    For { expr: Box<ASTTy>, col: Box<ASTTy>, body: Box<ASTTy> },
    In { left: Box<ASTTy>, right: Box<ASTTy> },
    While { cond: Box<ASTTy>, body: Box<ASTTy> },
    Break,
    Continue,
    Return { expr: Box<ASTTy> },
    ReturnEmpty,
    Underscore,
    Undefined,
    Pass,
    Question { left: Box<ASTTy>, right: Box<ASTTy> },
    QuestionOp { expr: Box<ASTTy> },
    Comment { comment: String },
}

#[cfg(test)]
mod test {
    use crate::{AST, ASTTy};
    use crate::check::ast::NodeTy;
    use crate::check::name::Name;
    use crate::common::position::{CaretPos, Position};
    use crate::parse::ast::Node;

    #[test]
    fn from_ast() {
        let node = Node::Pass;
        let pos = Position::from(&CaretPos::new(4, 8));
        let ast = AST::new(&pos, node.clone());

        let ast_ty = ASTTy::from(&ast);
        let ast_ty2 = ASTTy::from(&Box::from(ast));

        assert_eq!(ast_ty, ast_ty2);
        assert_eq!(ast_ty.node, NodeTy::from(&node));
        assert_eq!(ast_ty.pos, pos);
    }

    #[test]
    fn to_ty() {
        let node = Node::Pass;
        let ast = AST::new(&Position::default(), node.clone());
        let ast_ty = ASTTy::from(&ast).with_ty(&Name::from("Dummy"));

        assert_eq!(ast_ty.ty, Some(Name::from("Dummy")));
    }
}
