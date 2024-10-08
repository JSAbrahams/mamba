use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_type;

pub fn parse_cntrl_flow_expr(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::If => parse_if(it),
            Token::Match => parse_match(it),
            _ => Err(Box::from(expected_one_of(
                &[Token::If, Token::Match],
                lex,
                "control flow expression",
            ))),
        },
        &[Token::If, Token::Match],
        "control flow expression",
    )
}

fn parse_if(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("if expression")?;
    it.eat(&Token::If, "if expressions")?;
    let cond = it.parse(&parse_expression, "if expression", start)?;
    it.eat(&Token::Then, "if expression")?;
    let then = it.parse(&parse_expr_or_stmt, "if expression", start)?;

    let el = if it.peek_if(&|lex| lex.token == Token::Else) {
        it.parse_if(&Token::Else, &parse_expr_or_stmt, "if else branch", start)?
    } else if it.peek_if_followed_by(&Token::NL, &Token::Else) {
        it.eat(&Token::NL, "if else branch")?;
        it.parse_if(&Token::Else, &parse_expr_or_stmt, "if else branch", start)?
    } else {
        None
    };

    let pos = if let Some(el) = &el {
        start.union(el.pos)
    } else {
        start.union(then.pos)
    };
    let node = Node::IfElse { cond, then, el };

    Ok(Box::from(AST::new(pos, node)))
}

fn parse_match(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("match")?;
    it.eat(&Token::Match, "match")?;
    let cond = it.parse(&parse_expression, "match", start)?;
    it.eat(&Token::NL, "match")?;
    let cases = it.parse_vec(&parse_match_cases, "match", start)?;
    let end = cases.last().cloned().map_or(cond.pos, |case| case.pos);

    let node = Node::Match { cond, cases };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_match_cases(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.eat(&Token::Indent, "match cases")?;
    let mut cases = vec![];
    it.peek_while_not_token(&Token::Dedent, &mut |it, _| {
        cases.push(*it.parse(&parse_match_case, "match case", start)?);
        it.eat_if(&Token::NL);
        Ok(())
    })?;

    it.eat(&Token::Dedent, "match cases")?;
    Ok(cases)
}

fn parse_match_case(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("match case")?;
    let cond = it.parse(&parse_expression_maybe_type, "match case", start)?;
    it.eat(&Token::BTo, "match case")?;
    let body = it.parse(&parse_expr_or_stmt, "match case", start)?;

    let node = Node::Case {
        cond,
        body: body.clone(),
    };
    Ok(Box::from(AST::new(start.union(body.pos), node)))
}

fn parse_expression_maybe_type(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("expression maybe type")?;
    let mutable = it.eat_if(&Token::Fin).is_none();

    let expr = it.parse(&parse_expression, "expression maybe type", start)?;
    let ty = it.parse_if(
        &Token::DoublePoint,
        &parse_type,
        "expression maybe type",
        start,
    )?;
    let end = ty.clone().map_or(expr.pos, |t| t.pos);

    let node = Node::ExpressionType { expr, mutable, ty };
    Ok(Box::from(AST::new(start.union(end), node)))
}

#[cfg(test)]
mod test {
    use crate::parse::ast::{Node, AST};
    use crate::parse::parse_direct;
    use crate::parse::result::ParseResult;
    use crate::test_util::resource_content;

    #[test]
    fn if_else_verify() {
        let source = String::from("if a then c else d");
        let statements = parse_direct(&source).unwrap();

        let Node::IfElse { cond, then, el } = &statements.first().expect("script empty.").node
        else {
            panic!("first element script was not if.")
        };

        assert_eq!(
            cond.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            then.node,
            Node::Id {
                lit: String::from("c")
            }
        );
        assert_eq!(
            el.as_ref().unwrap().node,
            Node::Id {
                lit: String::from("d")
            }
        );
    }

    #[test]
    fn match_verify() {
        let source = String::from("match a\n    a => b\n    c => d");
        let statements = parse_direct(&source).unwrap();

        let Node::Match { cond, cases } = &statements.first().expect("script empty.").node else {
            panic!("first element script was not match.")
        };

        assert_eq!(
            cond.node,
            Node::Id {
                lit: String::from("a")
            }
        );

        assert_eq!(cases.len(), 2);
        let (cond1, expr1, cond2, expr2) = match (&cases[0], &cases[1]) {
            (
                AST {
                    node:
                        Node::Case {
                            cond: cond1,
                            body: expr1,
                        },
                    ..
                },
                AST {
                    node:
                        Node::Case {
                            cond: cond2,
                            body: expr2,
                        },
                    ..
                },
            ) => match (&cond1.node, &cond2.node) {
                (
                    Node::ExpressionType { expr: cond1, .. },
                    Node::ExpressionType { expr: cond2, .. },
                ) => (cond1, expr1, cond2, expr2),
                other => panic!("expected expression type: {:?}", other),
            },
            _ => panic!("Cases incorrect."),
        };

        assert_eq!(
            cond1.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            expr1.node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            cond2.node,
            Node::Id {
                lit: String::from("c")
            }
        );
        assert_eq!(
            expr2.node,
            Node::Id {
                lit: String::from("d")
            }
        );
    }

    #[test]
    fn if_expression() -> ParseResult<()> {
        let source = String::from("if a then\n    b\n");
        let ast = parse_direct(&source)?;

        let Some(Node::IfElse { cond, then, el }) = ast.first().map(|a| &a.node) else {
            panic!("Expected if, got {:?}", ast)
        };

        assert_eq!(
            cond.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        let then = match &then.node {
            Node::Block { statements } => statements[0].clone(),
            _ => panic!("Expected then block, got {:?}", then),
        };
        assert_eq!(
            then.node,
            Node::Id {
                lit: String::from("b")
            }
        );

        assert!(el.is_none());
        Ok(())
    }

    #[test]
    fn if_else_expression() -> ParseResult<()> {
        let source = String::from("if a then\n    b\nelse\n    c");
        let ast = parse_direct(&source)?;

        let Some(Node::IfElse { cond, then, el }) = ast.first().map(|a| &a.node) else {
            panic!("Expected if, got {:?}", ast)
        };

        assert_eq!(
            cond.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        let then = match &then.node {
            Node::Block { statements } => statements[0].clone(),
            _ => panic!("Expected then block, got {:?}", then),
        };
        assert_eq!(
            then.node,
            Node::Id {
                lit: String::from("b")
            }
        );

        let el = match el.clone().unwrap().node {
            Node::Block { statements } => statements[0].clone(),
            _ => panic!("Expected then block, got {:?}", then),
        };
        assert_eq!(
            el.node,
            Node::Id {
                lit: String::from("c")
            }
        );
        Ok(())
    }

    #[test]
    fn if_then_missing_body() {
        let source = String::from("if a then b else");
        parse_direct(&source).unwrap_err();
    }

    #[test]
    fn match_missing_condition() {
        let source = String::from("match\n    a => b");
        parse_direct(&source).unwrap_err();
    }

    #[test]
    fn match_missing_arms() {
        let source = String::from("match a with\n    ");
        parse_direct(&source).unwrap_err();
    }

    #[test]
    fn match_missing_arms_no_newline() {
        let source = String::from("match a");
        parse_direct(&source).unwrap_err();
    }

    #[test]
    fn match_stmt() {
        let source = resource_content(true, &["control_flow"], "match_stmt.mamba");
        source.parse::<AST>().unwrap();
    }
}
