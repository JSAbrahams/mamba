use crate::lex::token::Token;
use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::definition::parse_fun_arg;
use crate::parse::expression::parse_inner_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;

pub fn parse_reassignment(pre: &AST, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("reassignment")?;
    it.eat(&Token::Assign, "reassignment")?;
    let right = it.parse(&parse_expression, "reassignment", &start)?;

    let node = Node::Reassign { left: Box::new(pre.clone()), right: right.clone() };
    Ok(Box::from(AST::new(&start.union(&right.pos), node)))
}

pub fn parse_anon_fun(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("anonymous function")?;
    it.eat(&Token::BSlash, "anonymous function")?;

    let mut args: Vec<AST> = vec![];
    it.peek_while_not_token(&Token::BTo, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "anonymous function", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function", &start)?;
    let node = Node::AnonFun { args, body: body.clone() };
    Ok(Box::from(AST::new(&start.union(&body.pos), node)))
}

// TODO re-add postfix function calling
pub fn parse_call(pre: &AST, it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, ast| match ast.token {
            Token::Point => {
                it.eat(&Token::Point, "call")?;
                let property = it.parse(&parse_inner_expression, "call", &pre.pos)?;
                let node = Node::PropertyCall {
                    instance: Box::from(pre.clone()),
                    property: property.clone(),
                };
                Ok(Box::from(AST::new(&pre.pos.union(&property.pos), node)))
            }
            Token::LRBrack => {
                it.eat(&Token::LRBrack, "direct call")?;
                let args = it.parse_vec(&parse_arguments, "direct call", &pre.pos)?;
                let end = it.eat(&Token::RRBrack, "direct call")?;
                let node = Node::FunctionCall { name: Box::from(pre.clone()), args };
                Ok(Box::from(AST::new(&pre.pos.union(&end), node)))
            }
            _ => Err(expected_one_of(&[Token::Point, Token::LRBrack], ast, "function call"))
        },
        &[Token::Point, Token::LRBrack],
        "function call",
    )
}

fn parse_arguments(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("arguments")?;
    let mut arguments = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        arguments.push(*it.parse(&parse_expression, "arguments", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(arguments)
}

#[cfg(test)]
mod test {
    use crate::lex::tokenize;
    use crate::parse::{parse, parse_direct};
    use crate::parse::ast::{AST, Node};

    #[test]
    fn anon_fun_no_args_verify() {
        let source = String::from("\\ => c");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (args, body) = match &statements.first().expect("script empty.").node {
            Node::AnonFun { args, body } => (args.clone(), body.clone()),
            _ => panic!("first element script was anon fun.")
        };

        assert_eq!(args.len(), 0);
        assert_eq!(body.node, Node::Id { lit: String::from("c") });
    }

    #[test]
    fn anon_fun_verify() {
        let source = String::from("\\a,b => c");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (args, body) = match &statements.first().expect("script empty.").node {
            Node::AnonFun { args, body } => (args.clone(), body.clone()),
            _ => panic!("first element script was anon fun.")
        };

        assert_eq!(args.len(), 2);
        let (id1, id2) = match (&args[0], &args[1]) {
            (
                AST { node: Node::FunArg { var: id1, ty: None, mutable: true, .. }, .. },
                AST { node: Node::FunArg { var: id2, ty: None, mutable: true, .. }, .. }
            ) => (id1.clone(), id2.clone()),
            other => panic!("Id's of anon fun not expression type: {:?}", other)
        };

        assert_eq!(id1.node, Node::Id { lit: String::from("a") });
        assert_eq!(id2.node, Node::Id { lit: String::from("b") });

        assert_eq!(body.node, Node::Id { lit: String::from("c") });
    }

    #[test]
    fn direct_call_verify() {
        let source = String::from("a(b, c)");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (name, args) = match &statements.first().expect("script empty.").node {
            Node::FunctionCall { name, args } => (name.clone(), args.clone()),
            _ => panic!("first element script was anon fun.")
        };

        assert_eq!(name.node, Node::Id { lit: String::from("a") });
        assert_eq!(args.len(), 2);
        assert_eq!(args[0].node, Node::Id { lit: String::from("b") });
        assert_eq!(args[1].node, Node::Id { lit: String::from("c") });
    }

    #[test]
    fn method_call_verify() {
        let source = String::from("instance.a(b, c)");
        let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

        let (instance, name, args) = match &statements.first().expect("script empty.").node {
            Node::PropertyCall { instance, property } => match &property.node {
                Node::FunctionCall { name, args } => (instance.clone(), name.clone(), args.clone()),
                other => panic!("not function call in property call {:?}", other)
            },
            other => panic!("first element script was property call {:?}", other)
        };

        assert_eq!(instance.node, Node::Id { lit: String::from("instance") });
        assert_eq!(name.node, Node::Id { lit: String::from("a") });

        assert_eq!(args.len(), 2);
        assert_eq!(args[0].node, Node::Id { lit: String::from("b") });
        assert_eq!(args[1].node, Node::Id { lit: String::from("c") });
    }

    #[test]
    fn direct_call_missing_closing_bracket() {
        let source = String::from("a(b");
        parse(&tokenize(&source).unwrap()).unwrap_err();
    }

    #[test]
    fn regular_call_missing_closing_bracket() {
        let source = String::from("instance.a(b");
        parse(&tokenize(&source).unwrap()).unwrap_err();
    }
}
