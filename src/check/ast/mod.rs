use std::ops::Deref;

use crate::check::constrain::unify::finished::Finished;
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::common::position::Position;
use crate::parse::ast::AST;
use crate::parse::ast::node_op::NodeOp;

pub mod node;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct ASTTy {
    pub pos: Position,
    pub node: NodeTy,
    pub ty: Option<Name>,
}

impl From<(&Box<AST>, &Finished)> for ASTTy {
    fn from((ast, finished): (&Box<AST>, &Finished)) -> Self {
        ASTTy::from((ast.deref(), finished))
    }
}

impl From<(Box<AST>, &Finished)> for ASTTy {
    fn from((ast, finished): (Box<AST>, &Finished)) -> Self {
        ASTTy::from((ast.deref(), finished))
    }
}

impl From<(&AST, &Finished)> for ASTTy {
    fn from((ast, finished): (&AST, &Finished)) -> Self {
        let node = NodeTy::from((&ast.node, finished));
        ASTTy { node, ty: finished.pos_to_name.get(&ast.pos).cloned(), pos: ast.pos }
    }
}

impl From<&AST> for ASTTy {
    fn from(ast: &AST) -> Self {
        let node = NodeTy::from((&ast.node, &Finished::default()));
        ASTTy { node, ty: None, pos: ast.pos }
    }
}

impl From<&Box<AST>> for ASTTy {
    fn from(ast: &Box<AST>) -> Self {
        let node = NodeTy::from((&ast.node, &Finished::default()));
        ASTTy { node, ty: None, pos: ast.pos }
    }
}

impl ASTTy {
    pub fn with_ty(self, ty: &Name) -> ASTTy {
        trace!("Annotated AST at {} with '{}'", self.pos, ty);
        ASTTy { ty: Some(ty.clone()), ..self }
    }
}

type OptASTTy = Option<Box<ASTTy>>;
type OptName = Option<Name>;

#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub enum NodeTy {
    Import { from: Option<Box<ASTTy>>, import: Vec<ASTTy>, alias: Vec<ASTTy> },
    Class { ty: StringName, args: Vec<ASTTy>, parents: Vec<ASTTy>, body: OptASTTy },
    Parent { ty: StringName, args: Vec<ASTTy> },
    Reassign { left: Box<ASTTy>, right: Box<ASTTy>, op: NodeOp },
    VariableDef { mutable: bool, var: Box<ASTTy>, ty: OptName, expr: OptASTTy, forward: Vec<ASTTy> },
    FunDef { pure: bool, id: Box<ASTTy>, args: Vec<ASTTy>, ret: OptName, raises: Vec<ASTTy>, body: OptASTTy },
    AnonFun { args: Vec<ASTTy>, body: Box<ASTTy> },
    Raises { expr_or_stmt: Box<ASTTy>, errors: Vec<ASTTy> },
    Raise { error: Box<ASTTy> },
    Handle { expr_or_stmt: Box<ASTTy>, cases: Vec<ASTTy> },
    With { resource: Box<ASTTy>, alias: Option<(Box<ASTTy>, bool, OptName)>, expr: Box<ASTTy> },
    FunctionCall { name: StringName, args: Vec<ASTTy> },
    PropertyCall { instance: Box<ASTTy>, property: Box<ASTTy> },
    Id { lit: String },
    ExpressionType { expr: Box<ASTTy>, mutable: bool, ty: OptName },
    TypeDef { ty: StringName, isa: OptName, body: OptASTTy },
    TypeAlias { ty: StringName, isa: Name, conditions: Vec<ASTTy> },
    Condition { cond: Box<ASTTy>, el: OptASTTy },
    FunArg { vararg: bool, mutable: bool, var: Box<ASTTy>, ty: OptName, default: OptASTTy },
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
    Empty,
}

#[cfg(test)]
mod test {
    use crate::{AST, ASTTy};
    use crate::check::name::Name;
    use crate::common::position::Position;
    use crate::parse::ast::Node;

    #[test]
    fn to_ty() {
        let node = Node::Pass;
        let ast = AST::new(Position::invisible(), node.clone());
        let ast_ty = ASTTy::from(&ast).with_ty(&Name::from("Dummy"));

        assert_eq!(ast_ty.ty, Some(Name::from("Dummy")));
    }
}
