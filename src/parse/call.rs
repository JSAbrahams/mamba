use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::definition::parse_fun_arg;
use crate::parse::expression::parse_inner_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::{expected_one_of, ParseResult};

pub fn parse_anon_fun(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("anonymous function")?;
    it.eat(&Token::BSlash, "anonymous function")?;

    let mut args: Vec<AST> = vec![];
    it.peek_while_not_token(&Token::BTo, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "anonymous function", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::BTo, "anonymous function")?;

    let body = it.parse(&parse_expression, "anonymous function", start)?;
    let node = Node::AnonFun {
        args,
        body: body.clone(),
    };
    Ok(Box::from(AST::new(start.union(body.pos), node)))
}

pub fn parse_call(pre: &AST, it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, ast| match ast.token {
            Token::Point => {
                it.eat(&Token::Point, "call")?;
                let property = it.parse(&parse_inner_expression, "call", pre.pos)?;
                let node = Node::PropertyCall {
                    instance: Box::from(pre.clone()),
                    property: property.clone(),
                };
                Ok(Box::from(AST::new(pre.pos.union(property.pos), node)))
            }
            Token::LRBrack => {
                it.eat(&Token::LRBrack, "direct call")?;
                let args = it.parse_vec(&parse_arguments, "direct call", pre.pos)?;
                let end = it.eat(&Token::RRBrack, "direct call")?;
                let node = Node::FunctionCall {
                    name: Box::from(pre.clone()),
                    args,
                };
                Ok(Box::from(AST::new(pre.pos.union(end), node)))
            }
            _ => Err(Box::from(expected_one_of(
                &[Token::Point, Token::LRBrack],
                ast,
                "function call",
            ))),
        },
        &[Token::Point, Token::LRBrack],
        "function call",
    )
}

fn parse_arguments(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("arguments")?;
    let mut arguments = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        arguments.push(*it.parse(&parse_expression, "arguments", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    Ok(arguments)
}

#[cfg(test)]
mod test {
    use crate::parse::ast::node_op::NodeOp;
    use crate::parse::ast::{Node, AST};
    use crate::parse::parse_direct;

    #[test]
    fn op_assign() {
        let source = String::from("a:=1\nb+=2\nc-=3\nd*=4\ne/=5\nf^=6\ng<<=7\nh>>=8\n");
        let statements = parse_direct(&source).unwrap();

        let ops: Vec<NodeOp> = statements
            .iter()
            .map(|ast| match &ast.node {
                Node::Reassign { op, .. } => op.clone(),
                other => panic!("Expected reassign {:?}", other),
            })
            .collect();

        assert_eq!(ops[0], NodeOp::Assign);
        assert_eq!(ops[1], NodeOp::Add);
        assert_eq!(ops[2], NodeOp::Sub);
        assert_eq!(ops[3], NodeOp::Mul);
        assert_eq!(ops[4], NodeOp::Div);
        assert_eq!(ops[5], NodeOp::Pow);
        assert_eq!(ops[6], NodeOp::BLShift);
        assert_eq!(ops[7], NodeOp::BRShift);
    }

    #[test]
    fn anon_fun_no_args_verify() {
        let source = String::from("\\ => c");
        let statements = parse_direct(&source).unwrap();

        let Node::AnonFun { args, body } = &statements.first().expect("script empty.").node else {
            panic!("first element script was anon fun.")
        };

        assert_eq!(args.len(), 0);
        assert_eq!(
            body.node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn anon_fun_verify() {
        let source = String::from("\\a,b => c");
        let statements = parse_direct(&source).unwrap();

        let Node::AnonFun { args, body } = &statements.first().expect("script empty.").node else {
            panic!("first element script was anon fun.")
        };

        assert_eq!(args.len(), 2);
        let (id1, id2) = match (&args[0], &args[1]) {
            (
                AST {
                    node:
                        Node::FunArg {
                            var: id1,
                            ty: None,
                            mutable: true,
                            ..
                        },
                    ..
                },
                AST {
                    node:
                        Node::FunArg {
                            var: id2,
                            ty: None,
                            mutable: true,
                            ..
                        },
                    ..
                },
            ) => (id1.clone(), id2.clone()),
            other => panic!("Id's of anon fun not expression type: {:?}", other),
        };

        assert_eq!(
            id1.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            id2.node,
            Node::Id {
                lit: String::from("b")
            }
        );

        assert_eq!(
            body.node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn direct_call_verify() {
        let source = String::from("a(b, c)");
        let statements = parse_direct(&source).unwrap();

        let Node::FunctionCall { name, args } = &statements.first().expect("script empty.").node
        else {
            panic!("first element script was anon fun.")
        };

        assert_eq!(
            name.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0].node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            args[1].node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn method_call_verify() {
        let source = String::from("instance.a(b, c)");
        let statements = parse_direct(&source).unwrap();

        let (instance, name, args) = match &statements.first().expect("script empty.").node {
            Node::PropertyCall { instance, property } => match &property.node {
                Node::FunctionCall { name, args } => (instance.clone(), name.clone(), args.clone()),
                other => panic!("not function call in property call {:?}", other),
            },
            other => panic!("first element script was property call {:?}", other),
        };

        assert_eq!(
            instance.node,
            Node::Id {
                lit: String::from("instance")
            }
        );
        assert_eq!(
            name.node,
            Node::Id {
                lit: String::from("a")
            }
        );

        assert_eq!(args.len(), 2);
        assert_eq!(
            args[0].node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            args[1].node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn direct_call_missing_closing_bracket() {
        let source = String::from("a(b");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn regular_call_missing_closing_bracket() {
        let source = String::from("instance.a(b");
        source.parse::<AST>().unwrap_err();
    }
}
