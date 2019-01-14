use crate::core::Core;
use crate::parser::ASTNode;

#[macro_use]
/// Desugar and box.
macro_rules! des { ($ast:expr ) => {{ Box::new(desugar(*$ast)) }} }

macro_rules! des_direct { ($ast:expr ) => {{ desugar(*$ast) }} }

macro_rules! des_vec { ($ast:expr ) => {{ panic!("not implemented") }} }

//mod expression;
//mod function;
//mod module;
//mod statement;

pub fn desugar(input: ASTNode) -> Core {
    panic!("not implemented")
}
