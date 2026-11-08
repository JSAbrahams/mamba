use crate::backend::cranelift::result::{BackendErr, BackendResult};
use crate::check::ast::{ASTTy, NodeTy};

/// The identifier an `Id` node holds, e.g. a `FunDef`'s name or a `FunArg`'s bound variable.
pub(super) fn fun_name(id: &ASTTy) -> BackendResult<String> {
    match &id.node {
        NodeTy::Id { lit } => Ok(lit.clone()),
        other => Err(BackendErr::unimplemented(
            id,
            &format!("{} as a name", construct(other)),
        )),
    }
}

/// A short name for the construct `node` is, such as "a match expression", for an error message.
pub(super) fn construct(node: &NodeTy) -> &'static str {
    match node {
        NodeTy::Import { .. } => "an import",
        NodeTy::Class { .. } => "a class definition",
        NodeTy::Trait { .. } => "a trait definition",
        NodeTy::Reassign { .. } => "a reassignment",
        NodeTy::VariableDef { .. } => "a variable definition",
        NodeTy::FunDef { .. } => "a function definition",
        NodeTy::AnonFun { .. } => "an anonymous function",
        NodeTy::Raise { .. } => "a raise",
        NodeTy::Handle { .. } => "an error handler",
        NodeTy::Using { .. } => "a using block",
        NodeTy::FunctionCall { is_index: true, .. } => "an index",
        NodeTy::FunctionCall { .. } => "a function call",
        NodeTy::PropertyCall { .. } => "a method call or field access",
        NodeTy::Id { .. } => "an identifier",
        NodeTy::Dict { .. } => "a map",
        NodeTy::DictBuilder { .. } => "a map builder",
        NodeTy::Set { .. } => "a set",
        NodeTy::SetBuilder { .. } => "a set builder",
        NodeTy::List { .. } => "a list",
        NodeTy::ListBuilder { .. } => "a list builder",
        NodeTy::Tuple { .. } => "a tuple",
        NodeTy::Range { .. } => "a range",
        NodeTy::Str { .. } => "a string",
        NodeTy::Mul { .. } => "a multiplication",
        NodeTy::Div { .. } => "a division",
        NodeTy::FDiv { .. } => "a floor division",
        NodeTy::Mod { .. } => "a `mod` operation",
        NodeTy::Pow { .. } => "a `^` operation",
        NodeTy::Not { .. } => "a `not` operation",
        NodeTy::And { .. } => "an `and` operation",
        NodeTy::Or { .. } => "an `or` operation",
        NodeTy::IfElse { .. } => "an if expression",
        NodeTy::Match { .. } => "a match expression",
        NodeTy::For { .. } => "a for loop",
        NodeTy::In { .. } => "an `in` test",
        NodeTy::While { .. } => "a while loop",
        NodeTy::Question { .. } => "a `?` operation",
        NodeTy::Undefined => "`None`",
        _ => "this construct",
    }
}
