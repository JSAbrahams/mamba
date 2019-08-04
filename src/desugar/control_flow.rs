use crate::core::construct::Core;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::node::desugar_node;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;

pub fn desugar_control_flow(
    node_pos: &ASTNodePos,
    imp: &mut Imports,
    state: &State
) -> DesugarResult {
    Ok(match &node_pos.node {
        ASTNode::IfElse { cond, then, _else } => match _else {
            Some(_else) => Core::IfElse {
                cond:  Box::from(desugar_node(cond, imp, state)?),
                then:  Box::from(desugar_node(then, imp, state)?),
                _else: Box::from(desugar_node(_else, imp, state)?)
            },
            None => Core::If {
                cond: Box::from(desugar_node(cond, imp, state)?),
                then: Box::from(desugar_node(then, imp, state)?)
            }
        },
        ASTNode::Match { cond, cases } => {
            let expr = Box::from(desugar_node(cond, imp, state)?);
            let mut core_cases = vec![];
            let mut core_defaults = vec![];

            for case in cases {
                match &case.node {
                    ASTNode::Case { cond, body } => match &cond.node {
                        ASTNode::IdType { id, .. } => match id.node {
                            ASTNode::Underscore =>
                                core_defaults.push(desugar_node(body.as_ref(), imp, state)?),
                            _ => core_cases.push(Core::KeyValue {
                                key:   Box::from(desugar_node(cond.as_ref(), imp, state)?),
                                value: Box::from(desugar_node(body.as_ref(), imp, state)?)
                            })
                        },
                        other => panic!("Expected id type as cond but was {:?}", other)
                    },
                    other => panic!("Expected case but was {:?}", other)
                }
            }

            if core_defaults.len() > 1 {
                panic!("Can't have more than one default.")
            } else if core_defaults.len() == 1 {
                let default = Box::from(Core::AnonFun {
                    args: vec![],
                    body: Box::from(core_defaults[0].clone())
                });

                imp.add_from_import("collections", "defaultdict");
                Core::DefaultDictionary { expr, cases: core_cases, default }
            } else {
                Core::Dictionary { expr, cases: core_cases }
            }
        }
        ASTNode::While { cond, body } => Core::While {
            cond: Box::from(desugar_node(cond, imp, state)?),
            body: Box::from(desugar_node(body, imp, state)?)
        },
        ASTNode::For { expr, body } => Core::For {
            expr: Box::from(desugar_node(expr, imp, state)?),
            body: Box::from(desugar_node(body, imp, state)?)
        },

        ASTNode::Break => Core::Break,
        ASTNode::Continue => Core::Continue,
        other => panic!("Expected control flow but was: {:?}.", other)
    })
}
