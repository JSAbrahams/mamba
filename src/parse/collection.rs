use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::expression::is_start_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;

pub fn parse_collection(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::LRBrack => parse_tuple(it),
            Token::LSBrack => parse_list(it),
            Token::LCBrack => parse_set(it),
            _ => Err(Box::from(expected_one_of(
                &[Token::LRBrack, Token::LSBrack, Token::LCBrack],
                lex,
                "collection",
            )))
        },
        &[Token::LRBrack, Token::LSBrack, Token::LCBrack],
        "collection",
    )
}

pub fn parse_tuple(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("tuple")?;
    it.eat(&Token::LRBrack, "tuple")?;
    let elements = it.parse_vec(&parse_expressions, "tuple", start)?;
    let end = it.eat(&Token::RRBrack, "tuple")?;

    Ok(Box::from(if elements.len() == 1 {
        elements[0].clone()
    } else {
        AST::new(start.union(end), Node::Tuple { elements })
    }))
}

fn parse_set(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("set")?;
    it.eat(&Token::LCBrack, "set")?;

    if let Some(end) = it.eat_if(&Token::RCBrack) {
        let node = Node::Set { elements: vec![] };
        return Ok(Box::from(AST::new(start.union(end), node)));
    }

    let item = it.parse(&parse_expression, "set", start)?;
    if it.eat_if(&Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions, "set", start)?;
        let end = it.eat(&Token::RCBrack, "set")?;
        let node = Node::SetBuilder { item, conditions };
        return Ok(Box::from(AST::new(start.union(end), node)));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(&Token::Comma, &parse_expressions, "set", start)?);

    let end = it.eat(&Token::RCBrack, "set")?;
    let node = Node::Set { elements };
    Ok(Box::from(AST::new(start.union(end), node)))
}

fn parse_list(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("list")?;
    it.eat(&Token::LSBrack, "list")?;

    if let Some(end) = it.eat_if(&Token::RSBrack) {
        let node = Node::List { elements: vec![] };
        return Ok(Box::from(AST::new(start.union(end), node)));
    }

    let item = it.parse(&parse_expression, "list", start)?;
    if it.eat_if(&Token::Ver).is_some() {
        let conditions = it.parse_vec(&parse_expressions, "list", start)?;
        let end = it.eat(&Token::RSBrack, "list")?;
        let node = Node::ListBuilder { item, conditions };
        return Ok(Box::from(AST::new(start.union(end), node)));
    }

    let mut elements = vec![*item];
    elements.append(&mut it.parse_vec_if(&Token::Comma, &parse_expressions, "list", start)?);

    let end = it.eat(&Token::RSBrack, "list")?;
    let node = Node::List { elements };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_expressions(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("expression")?;
    let mut expressions = vec![];
    it.peek_while_fn(&is_start_expression, &mut |it, _| {
        expressions.push(*it.parse(&parse_expression, "expressions", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(expressions)
}

#[cfg(test)]
mod test {
    use crate::parse::{parse, parse_direct};
    use crate::parse::ast::Node;
    use crate::parse::result::ParseResult;
    use crate::test_util::resource_content;

    #[test]
    fn tuple_empty_verify() {
        let source = String::from("()");
        let statements = parse_direct(&source).unwrap();

        let elements = match &statements.first().expect("script empty.").node {
            Node::Tuple { elements } => elements,
            _ => panic!("first element script was not tuple.")
        };

        assert_eq!(elements.len(), 0);
    }

    #[test]
    fn tuple_single_is_expr_verify() {
        let source = String::from("(a)");
        let statements = parse_direct(&source).unwrap();

        let lit = match &statements.first().expect("script empty.").node {
            Node::Id { lit } => lit,
            _ => panic!("first element script was not tuple.")
        };

        assert_eq!(lit.as_str(), "a");
    }

    #[test]
    fn tuple_multiple_verify() {
        let source = String::from("(d, c)");
        let statements = parse_direct(&source).unwrap();

        let elements = match &statements.first().expect("script empty.").node {
            Node::Tuple { elements } => elements,
            _ => panic!("first element script was not tuple.")
        };

        assert_eq!(elements.len(), 2);
        assert_eq!(elements[0].node, Node::Id { lit: String::from("d") });
        assert_eq!(elements[1].node, Node::Id { lit: String::from("c") });
    }

    #[test]
    fn set_verify() {
        let source = String::from("{a, b}");
        let statements = parse_direct(&source).unwrap();

        let elements = match &statements.first().expect("script empty.").node {
            Node::Set { elements } => elements,
            _ => panic!("first element script was not set.")
        };

        assert_eq!(elements[0].node, Node::Id { lit: String::from("a") });
        assert_eq!(elements[1].node, Node::Id { lit: String::from("b") });
    }

    #[test]
    fn set_builder_verify() {
        let source = String::from("{a | c, d}");
        let statements = parse_direct(&source).unwrap();

        let (items, conditions) = match &statements.first().expect("script empty.").node {
            Node::SetBuilder { item, conditions } => (item.clone(), conditions.clone()),
            _ => panic!("first element script was not set builder.")
        };

        assert_eq!(items.node, Node::Id { lit: String::from("a") });

        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].node, Node::Id { lit: String::from("c") });
        assert_eq!(conditions[1].node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn list_verify() {
        let source = String::from("[a, b]");
        let statements = parse_direct(&source).unwrap();

        let elements = match &statements.first().expect("script empty.").node {
            Node::List { elements } => elements,
            _ => panic!("first element script was not list.")
        };

        assert_eq!(elements[0].node, Node::Id { lit: String::from("a") });
        assert_eq!(elements[1].node, Node::Id { lit: String::from("b") });
    }

    #[test]
    fn list_builder_verify() {
        let source = String::from("[a | c, d]");
        let statements = parse_direct(&source).unwrap();

        let (items, conditions) = match &statements.first().expect("script empty.").node {
            Node::ListBuilder { item, conditions } => (item.clone(), conditions.clone()),
            _ => panic!("first element script was not list builder.")
        };

        assert_eq!(items.node, Node::Id { lit: String::from("a") });

        assert_eq!(conditions.len(), 2);
        assert_eq!(conditions[0].node, Node::Id { lit: String::from("c") });
        assert_eq!(conditions[1].node, Node::Id { lit: String::from("d") });
    }

    #[test]
    #[ignore]
    fn list_expression() -> ParseResult<()> {
        let source = resource_content(true, &["collection"], "list.mamba");
        parse(&source).map(|_| ())
    }

    #[test]
    #[ignore]
    fn parse_map() -> ParseResult<()> {
        let source = resource_content(true, &["collection"], "map.mamba");
        parse(&source).map(|_| ())
    }

    #[test]
    fn parse_set() -> ParseResult<()> {
        let source = resource_content(true, &["collection"], "set.mamba");
        parse(&source).map(|_| ())
    }

    #[test]
    fn parse_tuple() -> ParseResult<()> {
        let source = resource_content(true, &["collection"], "tuple.mamba");
        parse(&source).map(|_| ())
    }
}