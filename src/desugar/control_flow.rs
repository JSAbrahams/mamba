use crate::core::construct::Core;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Context;
use crate::desugar::context::State;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;

pub fn desugar_control_flow(node: &ASTNode, ctx: &Context, state: &State) -> Core {
    match node {
        ASTNode::IfElse { cond, then, _else } => match _else {
            Some(_else) => Core::IfElse {
                cond:  Box::from(desugar_node(cond, ctx, state)),
                then:  Box::from(desugar_node(then, ctx, state)),
                _else: Box::from(desugar_node(_else, ctx, state))
            },
            None => Core::If {
                cond: Box::from(desugar_node(cond, ctx, state)),
                then: Box::from(desugar_node(then, ctx, state))
            }
        },
        ASTNode::Match { cond, cases } => Core::Match {
            cond:  Box::from(desugar_node(cond, ctx, state)),
            cases: desugar_vec(cases, ctx, state)
        },
        ASTNode::Case { cond, body } => Core::Case {
            cond: Box::from(desugar_node(cond, ctx, state)),
            body: Box::from(desugar_node(body, ctx, state))
        },
        ASTNode::While { cond, body } => Core::While {
            cond: Box::from(desugar_node(cond, ctx, state)),
            body: Box::from(desugar_node(body, ctx, state))
        },
        ASTNode::For { expr, body } => Core::For {
            expr: Box::from(desugar_node(expr, ctx, state)),
            body: Box::from(desugar_node(body, ctx, state))
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,
        other => panic!("Expected control flow but was: {:?}.", other)
    }
}
