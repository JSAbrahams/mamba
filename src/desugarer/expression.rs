use crate::core::Core;
use crate::desugarer::desugar;
use crate::parser::ASTNode;

pub fn desugar_expression(node: ASTNode) -> Core {
    match node {
        ASTNode::If(cond, then) => Core::IfElse(des!(cond), des!(then), Box::new(Core::Empty)),
        ASTNode::IfElse(cond, then, other) => Core::IfElse(Box::new(Core::Not(des!(cond))), des!(then), des!(other)),
        ASTNode::Unless(cond, then) => Core::IfElse(des!(cond), des!(then), Box::new(Core::Empty)),
        ASTNode::UnlessElse(cond, then, other) => Core::IfElse(Box::new(Core::Not(des!(cond))), des!(then), des!(other)),

        ASTNode::Block(statements) => Core::Block(des_vec!(statements)),

        ASTNode::Eq(left, right) => Core::Eq(des!(left), des!(right)),
        ASTNode::Is(left, right) => Core::Is(des!(left), des!(right)),
        ASTNode::Neq(left, right) => Core::Not(Box::new(Core::Eq(des!(left), des!(right)))),
        ASTNode::IsN(left, right) => Core::Not(Box::new(Core::Is(des!(left), des!(right)))),

        ASTNode::Ge(left, right) => Core::Ge(des!(left), des!(right)),
        ASTNode::Le(left, right) => Core::Le(des!(left), des!(right)),
        ASTNode::Geq(left, right) => Core::Geq(des!(left), des!(right)),
        ASTNode::Leq(left, right) => Core::Leq(des!(left), des!(right)),

        ASTNode::And(left, right) => Core::And(des!(left), des!(right)),
        ASTNode::Or(left, right) => Core::Or(des!(left), des!(right)),
        ASTNode::Not(ast) => Core::Not(des!(ast)),

        ASTNode::Add(left, right) => Core::Add(des!(left), des!(right)),
        ASTNode::Sub(left, right) => Core::Sub(des!(left), des!(right)),
        ASTNode::Mul(left, right) => Core::Mul(des!(left), des!(right)),
        ASTNode::Div(left, right) => Core::Div(des!(left), des!(right)),
        ASTNode::Mod(left, right) => Core::Mod(des!(left), des!(right)),
        ASTNode::AddU(ast) => des_direct!(ast),
        ASTNode::SubU(ast) => Core::SubU(des!(ast)),

        ASTNode::Id(id) => Core::Id(id),
        ASTNode::Int(integer) => Core::Int(vec![0]),
        ASTNode::Real(real) => Core::Real(vec![0.0]),
        ASTNode::ENum(num, exp) => Core::ENum(vec![0], vec![0]),
        ASTNode::Str(string) => Core::Str(string),
        ASTNode::Bool(boolean) => Core::Bool(boolean),

        _ => panic!("")
    }
}
