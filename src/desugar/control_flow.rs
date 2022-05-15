use crate::desugar::ast::node::Core;
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


#[cfg(test)]
mod tests {
    use crate::common::position::Position;
    use crate::desugar::ast::node::Core;
    use crate::desugar::desugar;
    use crate::parse::ast::AST;
    use crate::parse::ast::Node;

    macro_rules! to_pos_unboxed {
        ($node:expr) => {{
            AST { pos: Position::default(), node: $node }
        }};
    }

    macro_rules! to_pos {
        ($node:expr) => {{
            Box::from(to_pos_unboxed!($node))
        }};
    }

    #[test]
    fn if_verify() {
        let cond = to_pos!(Node::Id { lit: String::from("cond") });
        let then = to_pos!(Node::Id { lit: String::from("then") });
        let if_stmt = to_pos!(Node::IfElse { cond, then, el: None });

        let (core_cond, core_then) = match desugar(&if_stmt) {
            Ok(Core::If { cond, then }) => (cond, then),
            other => panic!("Expected reassign but was {:?}", other)
        };

        assert_eq!(*core_cond, Core::Id { lit: String::from("cond") });
        assert_eq!(*core_then, Core::Id { lit: String::from("then") });
    }

    #[test]
    fn if_else_verify() {
        let cond = to_pos!(Node::Id { lit: String::from("cond") });
        let then = to_pos!(Node::Id { lit: String::from("then") });
        let el = to_pos!(Node::Id { lit: String::from("else") });
        let if_stmt = to_pos!(Node::IfElse { cond, then, el: Some(el) });

        let (core_cond, core_then, core_else) = match desugar(&if_stmt) {
            Ok(Core::IfElse { cond, then, el }) => (cond, then, el),
            other => panic!("Expected reassign but was {:?}", other)
        };

        assert_eq!(*core_cond, Core::Id { lit: String::from("cond") });
        assert_eq!(*core_then, Core::Id { lit: String::from("then") });
        assert_eq!(*core_else, Core::Id { lit: String::from("else") });
    }

    #[test]
    fn while_verify() {
        let cond = to_pos!(Node::Id { lit: String::from("cond") });
        let body = to_pos!(Node::ENum { num: String::from("num"), exp: String::from("") });
        let while_stmt = to_pos!(Node::While { cond, body });

        let (core_cond, core_body) = match desugar(&while_stmt) {
            Ok(Core::While { cond, body }) => (cond, body),
            other => panic!("Expected reassign but was {:?}", other)
        };

        assert_eq!(*core_cond, Core::Id { lit: String::from("cond") });
        assert_eq!(*core_body, Core::ENum { num: String::from("num"), exp: String::from("0") });
    }

    #[test]
    fn for_verify() {
        let expr = to_pos!(Node::Id { lit: String::from("expr_1") });
        let col = to_pos!(Node::Id { lit: String::from("col") });
        let body = to_pos!(Node::Id { lit: String::from("body") });
        let for_stmt = to_pos!(Node::For { expr, col, body });

        let (core_expr, core_col, core_body) = match desugar(&for_stmt) {
            Ok(Core::For { expr, col, body }) => (expr, col, body),
            other => panic!("Expected for but was {:?}", other)
        };

        assert_eq!(*core_expr, Core::Id { lit: String::from("expr_1") });
        assert_eq!(*core_col, Core::Id { lit: String::from("col") });
        assert_eq!(*core_body, Core::Id { lit: String::from("body") });
    }

    #[test]
    fn range_verify() {
        let from = to_pos!(Node::Id { lit: String::from("a") });
        let to = to_pos!(Node::Id { lit: String::from("b") });
        let range = to_pos!(Node::Range { from, to, inclusive: false, step: None });

        let (from, to, step) = match desugar(&range) {
            Ok(Core::Range { from, to, step }) => (from, to, step),
            other => panic!("Expected range but was {:?}", other)
        };

        assert_eq!(*from, Core::Id { lit: String::from("a") });
        assert_eq!(*to, Core::Id { lit: String::from("b") });
        assert_eq!(*step, Core::Int { int: String::from("1") });
    }

    #[test]
    fn range_incl_verify() {
        let from = to_pos!(Node::Id { lit: String::from("a") });
        let to = to_pos!(Node::Id { lit: String::from("b") });
        let range = to_pos!(Node::Range { from, to, inclusive: true, step: None });

        let (from, to, step) = match desugar(&range) {
            Ok(Core::Range { from, to, step }) => (from, to, step),
            other => panic!("Expected range but was {:?}", other)
        };

        assert_eq!(*from, Core::Id { lit: String::from("a") });
        assert_eq!(*to, Core::Add {
            left: Box::from(Core::Id { lit: String::from("b") }),
            right: Box::from(Core::Int { int: String::from("1") }),
        });
        assert_eq!(*step, Core::Int { int: String::from("1") });
    }

    #[test]
    fn range_step_verify() {
        let from = to_pos!(Node::Id { lit: String::from("a") });
        let to = to_pos!(Node::Id { lit: String::from("b") });
        let step = Some(to_pos!(Node::Id { lit: String::from("c") }));
        let range = to_pos!(Node::Range { from, to, inclusive: false, step });

        let (from, to, step) = match desugar(&range) {
            Ok(Core::Range { from, to, step }) => (from, to, step),
            other => panic!("Expected range but was {:?}", other)
        };

        assert_eq!(*from, Core::Id { lit: String::from("a") });
        assert_eq!(*to, Core::Id { lit: String::from("b") });
        assert_eq!(*step, Core::Id { lit: String::from("c") });
    }
}
