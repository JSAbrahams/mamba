use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::block::parse_block;
use crate::parse::control_flow_expr::parse_match_cases;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::ParseResult;
use crate::parse::statement::{is_start_statement, parse_reassignment};
use crate::parse::statement::parse_statement;
use crate::parse::ty::parse_generics;

pub fn parse_expr_or_stmt(it: &mut LexIterator) -> ParseResult {
    let result = it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::NL => {
                it.eat(&Token::NL, "expression or statement")?;
                it.parse(&parse_block, "expression or statement", &lex.pos)
            }
            token =>
                if is_start_statement(token) {
                    parse_statement(it)
                } else {
                    parse_expression(it)
                },
        },
        &[],
        "expression or statement",
    )?;

    it.peek(
        &|it, lex| match lex.token {
            Token::Raise => parse_raise(*result.clone(), it),
            Token::Handle => parse_handle(*result.clone(), it),
            Token::Assign
            | Token::AddAssign
            | Token::SubAssign
            | Token::MulAssign
            | Token::DivAssign
            | Token::PowAssign
            | Token::BLShiftAssign
            | Token::BRShiftAssign => parse_reassignment(&*result, it),
            _ => Ok(result.clone())
        },
        Ok(result.clone()),
    )
}

pub fn parse_raise(expr_or_stmt: AST, it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("raise")?;
    it.eat(&Token::Raise, "raise")?;

    it.eat(&Token::LSBrack, "raise")?;
    let errors = it.parse_vec(&parse_generics, "raise", start)?;
    it.eat(&Token::RSBrack, "raise")?;
    it.eat_if(&Token::RSBrack);
    let end = errors.last().map_or(start, |stmt| &stmt.pos);

    let node = Node::Raises { expr_or_stmt: Box::from(expr_or_stmt), errors: errors.clone() };
    Ok(Box::from(AST::new(&start.union(end), node)))
}

pub fn parse_handle(expr_or_stmt: AST, it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("handle")?;
    it.eat(&Token::Handle, "handle")?;
    it.eat(&Token::NL, "handle")?;

    let cases = it.parse_vec(&parse_match_cases, "handle", start)?;
    let end = cases.last().map_or(start, |stmt| &stmt.pos);

    let node = Node::Handle { expr_or_stmt: Box::from(expr_or_stmt), cases: cases.clone() };
    Ok(Box::from(AST::new(&start.union(end), node)))
}

#[cfg(test)]
mod test {
    use crate::parse::{parse, parse_direct};
    use crate::parse::ast::Node;
    use crate::parse::ast::node_op::NodeOp;
    use crate::parse::result::ParseResult;
    use crate::test_util::resource_content;

    #[test]
    fn range_verify() {
        let source = String::from("hello .. world");
        let statements = parse_direct(&source).unwrap();

        let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range.")
        };

        assert_eq!(from.node, Node::Id { lit: String::from("hello") });
        assert_eq!(to.node, Node::Id { lit: String::from("world") });
        assert!(!inclusive);
        assert_eq!(step, None);
    }

    #[test]
    fn range_step_verify() {
        let source = String::from("hello .. world .. 2");
        let statements = parse_direct(&source).unwrap();

        let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range.")
        };

        assert_eq!(from.node, Node::Id { lit: String::from("hello") });
        assert_eq!(to.node, Node::Id { lit: String::from("world") });
        assert!(!inclusive);
        assert_eq!(step.unwrap().node, Node::Int { lit: String::from("2") });
    }

    #[test]
    fn range_incl_verify() {
        let source = String::from("foo ..= bar");
        let statements = parse_direct(&source).unwrap();

        let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range inclusive.")
        };

        assert_eq!(from.node, Node::Id { lit: String::from("foo") });
        assert_eq!(to.node, Node::Id { lit: String::from("bar") });
        assert!(inclusive);
        assert_eq!(step, None);
    }

    #[test]
    fn reassign_verify() {
        let source = String::from("id := new_value");
        let statements = parse_direct(&source).unwrap();

        let (left, right) = match &statements.first().expect("script empty.").node {
            Node::Reassign { left, right, op } => {
                assert_eq!(*op, NodeOp::Assign);
                (left.clone(), right.clone())
            }
            _ => panic!("first element script was not reassign.")
        };

        assert_eq!(left.node, Node::Id { lit: String::from("id") });
        assert_eq!(right.node, Node::Id { lit: String::from("new_value") });
    }

    #[test]
    fn return_verify() {
        let source = String::from("return some_value");
        let statements = parse_direct(&source).unwrap();

        let expr = match &statements.first().expect("script empty.").node {
            Node::Return { expr } => expr.clone(),
            _ => panic!("first element script was not reassign.")
        };

        assert_eq!(expr.node, Node::Id { lit: String::from("some_value") });
    }

    #[test]
    fn underscore_verify() {
        let source = String::from("_");
        let statements = parse_direct(&source).unwrap();

        let ast = statements.first().expect("script empty.").clone();
        assert_eq!(ast.node, Node::Underscore);
    }

    #[test]
    fn pass_verify() {
        let source = String::from("pass");
        let statements = parse_direct(&source).unwrap();

        let ast = statements.first().expect("script empty.").clone();
        assert_eq!(ast.node, Node::Pass);
    }

    #[test]
    fn import_verify() {
        let source = String::from("import c");
        let ast = parse(&source).unwrap();

        let imports = match ast.node {
            Node::File { statements: modules, .. } => modules,
            _ => panic!("ast was not file.")
        };

        assert_eq!(imports.len(), 1);
        let (from, import, alias) = match &imports[0].node {
            Node::Import { from, import, alias } => (from, import, alias),
            other => panic!("Expected import but was {:?}.", other)
        };

        assert_eq!(*from, None);
        assert_eq!(import[0].node, Node::Id { lit: String::from("c") });
        assert_eq!(alias.len(), 0);
    }

    #[test]
    fn import_as_verify() {
        let source = String::from("import a, b as c, d");
        let ast = parse(&source).unwrap();

        let imports = match ast.node {
            Node::File { statements: modules, .. } => modules,
            _ => panic!("ast was not file.")
        };

        assert_eq!(imports.len(), 1);
        let (from, import, alias) = match &imports[0].node {
            Node::Import { from, import, alias } => (from, import, alias),
            other => panic!("Expected import but was {:?}.", other)
        };

        assert_eq!(*from, None);
        assert_eq!(import.len(), 2);
        assert_eq!(import[0].node, Node::Id { lit: String::from("a") });
        assert_eq!(import[1].node, Node::Id { lit: String::from("b") });
        assert_eq!(alias.len(), 2);
        assert_eq!(alias[0].node, Node::Id { lit: String::from("c") });
        assert_eq!(alias[1].node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn range_missing_from() {
        let source = String::from(".. b");
        parse(&source).unwrap_err();
    }

    #[test]
    fn range_inc_missing_from() {
        let source = String::from("..= b");
        parse(&source).unwrap_err();
    }

    #[test]
    fn range_missing_to() {
        let source = String::from("a ..");
        parse(&source).unwrap_err();
    }

    #[test]
    fn range_incl_missing_to() {
        let source = String::from("a ..=");
        parse(&source).unwrap_err();
    }

    #[test]
    fn reassign_missing_value() {
        let source = String::from("a :=");
        parse(&source).unwrap_err();
    }

    #[test]
    fn quest_or_missing_alternative() {
        let source = String::from("a ?or");
        parse(&source).unwrap_err();
    }

    #[test]
    fn quest_or_on_nothing() {
        let source = String::from("?or");
        parse(&source).unwrap_err();
    }


    #[test]
    fn handle_verify() -> ParseResult<()> {
        let source = resource_content(true, &["error"], "handle.mamba");
        parse(&source).map(|_| ())
    }

    #[test]
    fn raises_verify() -> ParseResult<()> {
        let source = resource_content(true, &["error"], "raise.mamba");
        parse(&source).map(|_| ())
    }

    #[test]
    fn with_verify() -> ParseResult<()> {
        let source = resource_content(true, &["error"], "with.mamba");
        parse(&source).map(|_| ())
    }
}
