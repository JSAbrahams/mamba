use crate::desugarer::Core;
use crate::desugarer::desugar;
use crate::parser::ASTNode;

pub fn desugar_function(node: ASTNode) -> Core {
    match node {
        ASTNode::FunCall(class, box ASTNode::Id(function), args) =>
            Core::FunCall(des!(class), function, des!(args)),
        ASTNode::FunCallDirect(box ASTNode::Id(function), args) =>
            Core::FunCall(Box::new(Core::Empty), function, des!(args)),

        ASTNode::FunDef(function, args, ret_type, body) =>
            Core::FunDef(des!(function), des_vec!(args), des!(ret_type), des!(body)),
        ASTNode::FunDefNoRetType(function, args, body) =>
            Core::FunDef(des!(function), des_vec!(args), Box::new(Core::Empty), des!(body)),

        ASTNode::FunArg(arg, arg_type) => Core::FunArg(des!(arg), des!(arg_type)),

        _ => panic!("")
    }
}
