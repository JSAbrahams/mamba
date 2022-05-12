use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_id;

pub fn parse_cntrl_flow_stmt(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::While => parse_while(it),
            Token::For => parse_for(it),
            Token::Break => {
                let end = it.eat(&Token::Break, "control flow statement")?;
                Ok(Box::from(AST::new(&lex.pos.union(&end), Node::Break)))
            }
            Token::Continue => {
                let end = it.eat(&Token::Continue, "control flow statement")?;
                Ok(Box::from(AST::new(&lex.pos.union(&end), Node::Continue)))
            }
            _ => Err(expected_one_of(
                &[Token::While, Token::For, Token::Break, Token::Continue],
                lex,
                "control flow statement",
            ))
        },
        &[Token::While, Token::For, Token::Break, Token::Continue],
        "control flow statement",
    )
}

fn parse_while(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("while statement")?;
    it.eat(&Token::While, "while statement")?;
    let cond = it.parse(&parse_expression, "while statement", &start)?;
    it.eat(&Token::Do, "while")?;
    let body = it.parse(&parse_expr_or_stmt, "while statement", &start)?;

    let node = Node::While { cond, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

fn parse_for(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("for statement")?;
    it.eat(&Token::For, "for statement")?;
    let expr = it.parse(&parse_id, "for statement", &start)?;
    it.eat(&Token::In, "for statement")?;
    let col = it.parse(&parse_expression, "for statement", &start)?;
    it.eat(&Token::Do, "for statement")?;
    let body = it.parse(&parse_expr_or_stmt, "for statement", &start)?;

    let node = Node::For { expr, col, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

#[cfg(test)]
mod test {
    use crate::parse::{parse, parse_direct};
    use crate::parse::ast::Node;
    use crate::parse::lex::tokenize;
    use crate::parse::result::ParseResult;
    use crate::test_util::resource_content;

    #[test]
    fn for_statement_verify() {
        let source = String::from("for a in c do d");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (expr, collection, body) = match &statements.first().expect("script empty.").node {
            Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
            _ => panic!("first element script was not for.")
        };

        assert_eq!(expr.node, Node::Id { lit: String::from("a") });
        assert_eq!(collection.node, Node::Id { lit: String::from("c") });
        assert_eq!(body.node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn for_range_step_verify() {
        let source = String::from("for a in c .. d .. e do f");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (expr, col, body) = match &statements.first().expect("script empty.").node {
            Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
            _ => panic!("first element script was not foreach.")
        };

        match col.node {
            Node::Range { from, to, inclusive, step } => {
                assert_eq!(from.node, Node::Id { lit: String::from("c") });
                assert_eq!(to.node, Node::Id { lit: String::from("d") });
                assert!(!inclusive);
                assert_eq!(step.clone().unwrap().node, Node::Id { lit: String::from("e") });
            }
            _ => panic!("Expected range")
        }

        assert_eq!(expr.node, Node::Id { lit: String::from("a") });
        assert_eq!(body.node, Node::Id { lit: String::from("f") });
    }

    #[test]
    fn for_range_incl_verify() {
        let source = String::from("for a in c ..= d do f");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (expr, col, body) = match &statements.first().expect("script empty.").node {
            Node::For { expr, col, body } => (expr.clone(), col.clone(), body.clone()),
            _ => panic!("first element script was not foreach.")
        };

        match col.node {
            Node::Range { from, to, inclusive, step } => {
                assert_eq!(from.node, Node::Id { lit: String::from("c") });
                assert_eq!(to.node, Node::Id { lit: String::from("d") });
                assert!(inclusive);
                assert_eq!(step, None);
            }
            _ => panic!("Expected range")
        }

        assert_eq!(expr.node, Node::Id { lit: String::from("a") });
        assert_eq!(body.node, Node::Id { lit: String::from("f") });
    }

    #[test]
    fn if_verify() {
        let source = String::from("if a then c");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (cond, then, el) = match &statements.first().expect("script empty.").node {
            Node::IfElse { cond, then, el } => (cond, then, el),
            _ => panic!("first element script was not if.")
        };

        assert_eq!(cond.node, Node::Id { lit: String::from("a") });
        assert_eq!(then.node, Node::Id { lit: String::from("c") });
        assert_eq!(el.is_none(), true);
    }

    #[test]
    fn if_with_block_verify() {
        let source = String::from("if a then\n    c\n    d");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (cond, then, el) = match &statements.first().expect("script empty.").node {
            Node::IfElse { cond, then, el } => (cond.clone(), then.clone(), el.clone()),
            _ => panic!("first element script was not if.")
        };

        assert_eq!(cond.node, Node::Id { lit: String::from("a") });
        assert_eq!(el.is_none(), true);

        let block = match then.node {
            Node::Block { statements } => statements,
            other => panic!("then of if was not block, was: {:?}", other)
        };

        assert_eq!(block.len(), 2);
        assert_eq!(block[0].node, Node::Id { lit: String::from("c") });
        assert_eq!(block[1].node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn while_verify() {
        let source = String::from("while a do d");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (cond, body) = match &statements.first().expect("script empty.").node {
            Node::While { cond, body } => (cond.clone(), body.clone()),
            _ => panic!("first element script was not while.")
        };

        assert_eq!(cond.node, Node::Id { lit: String::from("a") });
        assert_eq!(body.node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn for_missing_do() {
        let source = String::from("for a in c d");
        parse_direct(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn for_missing_body() {
        let source = String::from("for a in c");
        parse_direct(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn if_missing_then() {
        let source = String::from("if a b");
        parse_direct(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn if_missing_body() {
        let source = String::from("if a then");
        parse_direct(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn while_statements() -> ParseResult<()> {
        let source = resource_content(true, &["control_flow"], "while.mamba");
        parse(&tokenize(&source).unwrap()).map(|_| ())
    }

    #[test]
    fn assigns_and_while() {
        let source = resource_content(false, &["syntax"], "assign_and_while.mamba");
        parse(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn for_statements() {
        let source = resource_content(true, &["control_flow"], "for_statements.mamba");
        parse(&tokenize(&source).unwrap()).unwrap();
    }

    #[test]
    fn if_stmt() {
        let source = resource_content(true, &["control_flow"], "if.mamba");
        parse(&tokenize(&source).unwrap()).unwrap();
    }
}
