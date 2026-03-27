use crate::parse::ast::{Node, AST};
use crate::parse::block::{parse_code_block, parse_code_set};
use crate::parse::control_flow_expr::parse_match_cases;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::{Lex, Token};
use crate::parse::operation::parse_expression;
use crate::parse::result::ParseResult;
use crate::parse::statement::parse_statement;
use crate::parse::statement::{is_start_statement, parse_reassignment};

pub fn parse_expr_or_stmt(it: &mut LexIterator) -> ParseResult {
    let expr_or_stmt = it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::Where => it.parse(&parse_code_set, "statement", lex.pos),
            Token::Do => it.parse(&parse_code_block, "statement", lex.pos),
            _ if is_start_statement(lex) => parse_statement(it),
            _ => parse_expression(it),
        },
        &[],
        "expression or statement",
    )?;

    // if expression/statement followed by newline and indent, we are dealing with a handle block
    if it.peek_if(&|lex: &Lex| lex.token == Token::Raise) {
        // parse handle cases if indentation block after
        let cases = it.parse_vec(&parse_match_cases, "handle cases", expr_or_stmt.pos)?;
        let end = cases.last().map_or(expr_or_stmt.pos, |stmt| stmt.pos);

        return Ok(Box::from(AST::new(
            expr_or_stmt.pos.union(end),
            Node::Handle {
                expr_or_stmt,
                cases,
            },
        )));
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Assign
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign
            | Token::PowAssign => parse_reassignment(&expr_or_stmt, it),
            _ => Ok(expr_or_stmt.clone()),
        },
        Ok(expr_or_stmt.clone()),
    )
}

#[cfg(test)]
mod test {
    use crate::parse::ast::node_op::NodeOp;
    use crate::parse::ast::{Node, AST};

    #[test]
    fn range_verify() {
        let source = String::from("hello .. world");
        let ast: AST = source.parse().unwrap();

        let (from, to, inclusive, step) = match &ast.node {
            Node::Range {
                from,
                to,
                inclusive,
                step,
            } => (from.clone(), to.clone(), *inclusive, step.clone()),
            _ => panic!("first element script was not range."),
        };

        assert_eq!(
            from.node,
            Node::Id {
                lit: String::from("hello")
            }
        );
        assert_eq!(
            to.node,
            Node::Id {
                lit: String::from("world")
            }
        );
        assert!(!inclusive);
        assert_eq!(step, None);
    }

    #[test]
    fn range_step_verify() {
        let source = String::from("hello .. world .. 2");
        let ast: AST = source.parse().unwrap();

        let (from, to, inclusive, step) = match &ast.node {
            Node::Range {
                from,
                to,
                inclusive,
                step,
            } => (from.clone(), to.clone(), *inclusive, step.clone()),
            _ => panic!("first element script was not range."),
        };

        assert_eq!(
            from.node,
            Node::Id {
                lit: String::from("hello")
            }
        );
        assert_eq!(
            to.node,
            Node::Id {
                lit: String::from("world")
            }
        );
        assert!(!inclusive);
        assert_eq!(
            step.unwrap().node,
            Node::Int {
                lit: String::from("2")
            }
        );
    }

    #[test]
    fn range_incl_verify() {
        let source = String::from("foo ..= bar");
        let ast: AST = source.parse().unwrap();

        let (from, to, inclusive, step) = match &ast.node {
            Node::Range {
                from,
                to,
                inclusive,
                step,
            } => (from.clone(), to.clone(), *inclusive, step.clone()),
            _ => panic!("first element script was not range inclusive."),
        };

        assert_eq!(
            from.node,
            Node::Id {
                lit: String::from("foo")
            }
        );
        assert_eq!(
            to.node,
            Node::Id {
                lit: String::from("bar")
            }
        );
        assert!(inclusive);
        assert_eq!(step, None);
    }

    #[test]
    fn reassign_verify() {
        let source = String::from("id := new_value");
        let ast: AST = source.parse().unwrap();

        let (left, right) = match &ast.node {
            Node::Reassign { left, right, op } => {
                assert_eq!(*op, NodeOp::Assign);
                (left.clone(), right.clone())
            }
            _ => panic!("first element script was not reassign."),
        };

        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("id")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("new_value")
            }
        );
    }

    #[test]
    fn return_verify() {
        let source = String::from("return some_value");
        let ast: AST = source.parse().unwrap();

        let expr = match &ast.node {
            Node::Return { expr } => expr.clone(),
            _ => panic!("first element script was not reassign."),
        };

        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("some_value")
            }
        );
    }

    #[test]
    fn return_stmt_with_comment_verify() {
        let source = String::from("return some_value # comment");
        let ast: AST = source.parse().unwrap();

        let expr = match &ast.node {
            Node::Return { expr } => expr.clone(),
            _ => panic!("first element script was not reassign."),
        };

        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("some_value")
            }
        );
    }

    #[test]
    fn literal_expr_with_comment_verify() {
        let source = String::from("10 # comment");
        let ast: AST = source.parse().unwrap();

        let lit = match &ast.node {
            Node::Int { lit } => lit.clone(),
            _ => panic!("first element script was not reassign."),
        };

        assert_eq!(lit, String::from("10"));
    }

    #[test]
    fn underscore_verify() {
        let source = String::from("_");
        let ast: AST = source.parse().unwrap();
        assert_eq!(ast.node, Node::Underscore);
    }

    #[test]
    fn import_verify() {
        let source = String::from("import c");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match &ast.node {
            Node::Import {
                from,
                import,
                alias,
            } => (from, import, alias),
            other => panic!("Expected import but was {other:?}."),
        };

        assert_eq!(*from, None);
        assert_eq!(
            import[0].node,
            Node::Id {
                lit: String::from("c")
            }
        );
        assert_eq!(alias.len(), 0);
    }

    #[test]
    fn import_as_verify() {
        let source = String::from("import a, b as c, d");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match &ast.node {
            Node::Import {
                from,
                import,
                alias,
            } => (from, import, alias),
            other => panic!("Expected import but was {other:?}."),
        };

        assert_eq!(*from, None);
        assert_eq!(import.len(), 2);
        assert_eq!(
            import[0].node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            import[1].node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(alias.len(), 2);
        assert_eq!(
            alias[0].node,
            Node::Id {
                lit: String::from("c")
            }
        );
        assert_eq!(
            alias[1].node,
            Node::Id {
                lit: String::from("d")
            }
        );
    }

    #[test]
    fn range_missing_from() {
        let source = String::from(".. b");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn range_inc_missing_from() {
        let source = String::from("..= b");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn range_missing_to() {
        let source = String::from("a ..");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn range_incl_missing_to() {
        let source = String::from("a ..=");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn reassign_missing_value() {
        let source = String::from("a :=");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn quest_or_missing_alternative() {
        let source = String::from("a ?or");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn quest_or_on_nothing() {
        let source = String::from("?or");
        source.parse::<AST>().unwrap_err();
    }
}
