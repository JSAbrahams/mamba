use crate::core::Core;
use crate::parser::ASTNode;

#[macro_use]
/// Desugar and box.
macro_rules! des { ($ast:expr ) => {{ Box::new(desugar($ast)) }} }

mod control_flow_expr;

pub fn desugar(input: ASTNode) -> Core {
    match input {
        ASTNode::Module(module) => match *module {
            ASTNode::Script(imports, functions, body) => panic!(""),
            ASTNode::Class(name, imports, functions) => panic!(""),
            ASTNode::Util(name, imports, functions) => panic!(""),
            _ => panic!("")
        },
        _ => panic!("")
    }
}
