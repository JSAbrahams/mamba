use crate::core::construct::Core;
use crate::desugar::node::desugar_node;
use crate::desugar::result::DesugarResult;
use crate::desugar::result::UnimplementedErr;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::parse::ast::AST;
use crate::parse::ast::Node;

#[allow(clippy::comparison_chain)]
pub fn desugar_control_flow(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &ast.node {
        Node::IfElse { cond, then, el } => match el {
            Some(el) => Core::IfElse {
                cond: Box::from(desugar_node(cond, imp, state)?),
                then: Box::from(desugar_node(then, imp, state)?),
                el: Box::from(desugar_node(el, imp, state)?),
            },
            None => Core::If {
                cond: Box::from(desugar_node(cond, imp, state)?),
                then: Box::from(desugar_node(then, imp, state)?),
            }
        },
        Node::Match { cond, cases } => {
            let expr = Box::from(desugar_node(cond, imp, state)?);
            let mut core_cases = vec![];
            let mut core_defaults = vec![];

            for case in cases {
                match &case.node {
                    Node::Case { cond, body } => match &cond.node {
                        Node::ExpressionType { expr, .. } => match expr.node {
                            Node::Underscore =>
                                core_defaults.push(desugar_node(body.as_ref(), imp, state)?),
                            _ => core_cases.push(Core::KeyValue {
                                key: Box::from(desugar_node(cond.as_ref(), imp, state)?),
                                value: Box::from(desugar_node(body.as_ref(), imp, state)?),
                            })
                        },
                        _ =>
                            return Err(UnimplementedErr::new(
                                ast,
                                "match case expression as condition (pattern matching)",
                            )),
                    },
                    other => panic!("Expected case but was {:?}", other)
                }
            }

            if core_defaults.len() > 1 {
                panic!("Can't have more than one default.")
            } else if core_defaults.len() == 1 {
                let default = Box::from(Core::AnonFun {
                    args: vec![],
                    body: Box::from(core_defaults[0].clone()),
                });

                imp.add_from_import("collections", "defaultdict");
                Core::DefaultDictionary { expr, cases: core_cases, default }
            } else {
                Core::Dictionary { expr, cases: core_cases }
            }
        }
        Node::While { cond, body } => Core::While {
            cond: Box::from(desugar_node(cond, imp, state)?),
            body: Box::from(desugar_node(body, imp, state)?),
        },
        Node::For { expr, col, body } => Core::For {
            expr: Box::from(desugar_node(expr, imp, state)?),
            col: Box::from(desugar_node(col, imp, state)?),
            body: Box::from(desugar_node(body, imp, state)?),
        },

        Node::Break => Core::Break,
        Node::Continue => Core::Continue,
        other => panic!("Expected control flow but was: {:?}.", other)
    })
}
