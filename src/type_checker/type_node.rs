use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::{Function, FunctionArg};
use crate::type_checker::context::type_name::TypeName;

#[derive(Debug, Clone)]
pub struct Type {
    pub name:      String,
    pub args:      Vec<FunctionArg>,
    pub generics:  Vec<TypeName>,
    pub concrete:  bool,
    pub fields:    Vec<Field>,
    pub functions: Vec<Function>,
    pub parents:   Vec<TypeName>
}
