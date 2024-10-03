use crate::parse::ast::node_op::NodeOp;
use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::custom;
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_expression_type;
use crate::parse::ty::parse_id;
use crate::parse::ty::parse_type;

pub fn parse_definition(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("definition")?;
    it.eat(&Token::Def, "definition")?;
    let pure = it.eat_if(&Token::Pure).is_some();

    macro_rules! op {
        ($it:expr, $op:ident) => {{
            let end = $it.eat(&Token::$op, "definition")?;
            let node = Node::Id {
                lit: format!("{}", NodeOp::$op),
            };
            let ast = AST::new(start.union(end), node);
            parse_fun_def(&ast, pure, $it)
        }};
    }

    let res = it.peek_or_err(
        &|it, lex| match lex.token {
            Token::LRBrack | Token::LCBrack | Token::LSBrack if !pure => parse_variable_def(it),

            Token::Add => op!(it, Add),
            Token::Sub => op!(it, Sub),
            Token::Sqrt => op!(it, Sqrt),
            Token::Mul => op!(it, Mul),
            Token::FDiv => op!(it, FDiv),
            Token::Div => op!(it, Div),
            Token::Pow => op!(it, Pow),
            Token::Mod => op!(it, Mod),
            Token::Eq => op!(it, Eq),
            Token::Ge => op!(it, Ge),
            Token::Le => op!(it, Le),
            _ => parse_var_or_fun_def(it, pure),
        },
        &[
            Token::Id(String::new()),
            Token::LRBrack,
            Token::LCBrack,
            Token::LSBrack,
            Token::Add,
            Token::Sub,
            Token::Sqrt,
            Token::Mul,
            Token::FDiv,
            Token::Div,
            Token::Pow,
            Token::Mod,
            Token::Eq,
            Token::Ge,
            Token::Le,
        ],
        "definition",
    )?;

    Ok(Box::new(AST {
        pos: res.pos.union(start),
        node: res.node.clone(),
    }))
}

fn parse_var_or_fun_def(it: &mut LexIterator, pure: bool) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let id = *it.parse(
        &parse_expression_type,
        "variable or function definition",
        start,
    )?;

    match &id.node {
        Node::ExpressionType { ty: Some(_), .. } | Node::TypeTup { .. } if !pure => {
            parse_variable_def_id(&id, it)
        }
        Node::ExpressionType { expr, ty, mutable } if ty.is_none() => it.peek(
            &|it, lex| match lex.token {
                Token::LRBrack => parse_fun_def(&id, pure, it),
                _ if !pure => parse_variable_def_id(&id, it),
                _ => {
                    let msg = format!("Definition cannot have {} identifier", Token::Pure);
                    Err(Box::from(custom(&msg, id.pos)))
                }
            },
            {
                let node = Node::VariableDef {
                    mutable: *mutable,
                    var: expr.clone(),
                    ty: None,
                    expr: None,
                    forward: vec![],
                };
                Ok(Box::from(AST::new(id.pos.union(id.pos), node)))
            },
        ),
        _ => Err(Box::from(custom(
            "definition must start with id type",
            id.pos,
        ))),
    }
}

fn parse_fun_def(id: &AST, pure: bool, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let fun_args = it.parse_vec(&parse_fun_args, "function definition", start)?;

    let id = match &id.node {
        Node::ExpressionType { expr, mutable, ty } => match (mutable, ty) {
            (_, None) => expr.clone(),
            (_, Some(_)) => {
                return Err(Box::from(custom(
                    "Function identifier cannot have type",
                    expr.pos,
                )))
            }
        },
        Node::Id { .. } => Box::from(id.clone()),

        _ => {
            return Err(Box::from(custom(
                "Function definition not given id or operator",
                id.pos,
            )))
        }
    };

    let ret_ty = it.parse_if(&Token::To, &parse_type, "function return type", start)?;
    let raises = it.parse_vec_if(&Token::Raise, &parse_raises, "raises", start)?;
    let body = it.parse_if(&Token::BTo, &parse_expr_or_stmt, "function body", start)?;

    let end = match (&ret_ty, &raises.last(), &body) {
        (_, _, Some(b)) => b.pos,
        (_, Some(b), _) => b.pos,
        (Some(b), ..) => b.pos,
        _ => id.pos,
    };

    let node = Node::FunDef {
        id,
        pure,
        args: fun_args,
        ret: ret_ty,
        raises,
        body,
    };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_raises(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.eat(&Token::LSBrack, "raises")?;
    let mut raises: Vec<AST> = Vec::new();
    it.peek_while_not_token(&Token::RSBrack, &mut |it, _| {
        raises.push(*it.parse(&parse_type, "raises", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;
    it.eat(&Token::RSBrack, "raises")?;
    Ok(raises)
}

pub fn parse_fun_args(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.eat(&Token::LRBrack, "function arguments")?;
    let mut args = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "function arguments", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::RRBrack, "function arguments")?;
    Ok(args)
}

pub fn parse_fun_arg(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("function argument")?;
    let vararg = it.eat_if(&Token::Vararg).is_some();

    let expression_type = it.parse(&parse_expression_type, "function argument", start)?;
    let (mutable, var, ty) = match &expression_type.node {
        Node::ExpressionType { expr, mutable, ty } => (*mutable, expr.clone(), ty.clone()),
        _ => {
            return Err(Box::from(custom(
                "Expected expression type in function argument",
                expression_type.pos,
            )));
        }
    };
    let default = it.parse_if(
        &Token::Assign,
        &parse_expression,
        "function argument default",
        start,
    )?;

    let end = default.clone().map_or(expression_type.pos, |def| def.pos);
    let node = Node::FunArg {
        vararg,
        mutable,
        var,
        ty,
        default,
    };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_forward(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("forward")?;
    let mut forwarded: Vec<AST> = vec![];
    it.peek_while_not_token(&Token::NL, &mut |it, _| {
        forwarded.push(*it.parse(&parse_id, "forward", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(forwarded)
}

fn parse_variable_def_id(id: &AST, it: &mut LexIterator) -> ParseResult {
    let expression = it.parse_if(&Token::Assign, &parse_expression, "definition body", id.pos)?;
    let forward = it.parse_vec_if(&Token::Forward, &parse_forward, "definition raises", id.pos)?;
    let (mutable, var, ty) = match &id.node {
        Node::ExpressionType { expr, mutable, ty } => (*mutable, expr.clone(), ty.clone()),
        _ => {
            return Err(Box::from(custom(
                "Expected expression type in variable definition",
                id.pos,
            )))
        }
    };

    let end = match (&expression, &forward.last()) {
        (_, Some(expr)) => expr.pos,
        (Some(expr), _) => expr.pos,
        _ => id.pos,
    };
    let node = Node::VariableDef {
        mutable,
        var,
        ty,
        expr: expression,
        forward,
    };
    Ok(Box::from(AST::new(id.pos.union(end), node)))
}

fn parse_variable_def(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("variable definition")?;
    let id = it.parse(&parse_expression_type, "variable definition", start)?;
    parse_variable_def_id(&id, it)
}

#[cfg(test)]
mod test {
    use crate::parse::ast::{Node, AST};
    use crate::parse::parse_direct;
    use crate::parse::result::ParseResult;
    use crate::test_util::resource_content;

    macro_rules! unwrap_func_definition {
        ($ast:expr) => {{
            let definition = $ast.first().expect("script empty.").clone();
            match definition.node {
                Node::FunDef {
                    id,
                    pure,
                    args,
                    ret,
                    raises,
                    body,
                    ..
                } => (pure, id, args, ret, raises, body),
                other => panic!("Expected variabledef but was {:?}.", other),
            }
        }};
    }

    macro_rules! unwrap_definition {
        ($ast:expr) => {{
            let definition = $ast.first().expect("script empty.").node.clone();
            match definition {
                Node::VariableDef {
                    mutable,
                    var,
                    ty,
                    expr,
                    forward,
                } => (mutable, var, ty, expr, forward),
                other => panic!("Expected variabledef but was {:?}.", other),
            }
        }};
    }

    #[test]
    fn empty_definition_verify() {
        let source = String::from("def a");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, _type, expression, forward) = unwrap_definition!(ast);

        assert_eq!(mutable, true);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(_type, None);
        assert_eq!(expression, None);
        assert_eq!(forward, vec![]);
    }

    #[test]
    fn definition_verify() {
        let source = String::from("def a := 10");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        assert_eq!(mutable, true);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(ty, None);
        assert_eq!(forward, vec![]);

        match expression {
            Some(expr_pos) => assert_eq!(
                expr_pos.node,
                Node::Int {
                    lit: String::from("10")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }
    }

    #[test]
    fn mutable_definition_verify() {
        let source = String::from("def fin a := 10");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        assert_eq!(mutable, false);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(ty, None);
        assert_eq!(forward, vec![]);

        match expression {
            Some(expr_pos) => assert_eq!(
                expr_pos.node,
                Node::Int {
                    lit: String::from("10")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }
    }

    #[test]
    fn private_definition_verify() {
        let source = String::from("def a := 10");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        assert_eq!(mutable, true);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(ty, None);
        assert_eq!(forward, vec![]);

        match expression {
            Some(expr_pos) => assert_eq!(
                expr_pos.node,
                Node::Int {
                    lit: String::from("10")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }
    }

    #[test]
    fn typed_definition_verify() {
        let source = String::from("def a: Object := 10");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        let type_id = match ty {
            Some(_type_pos) => match _type_pos.node {
                Node::Type { id, generics: _ } => id,
                other => panic!("Expected type but was: {:?}", other),
            },
            None => panic!("Expected type but was none."),
        };
        let expr = match expression {
            Some(expr_pos) => expr_pos,
            other => panic!("Unexpected expression: {:?}", other),
        };

        assert_eq!(mutable, true);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(forward, vec![]);
        assert_eq!(
            expr.node,
            Node::Int {
                lit: String::from("10")
            }
        );
        assert_eq!(
            type_id.node,
            Node::Id {
                lit: String::from("Object")
            }
        );
    }

    #[test]
    fn forward_empty_definition_verify() {
        let source = String::from("def a forward b, c");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        assert!(mutable);
        assert_eq!(ty, None);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(expression, None);
        assert_eq!(forward.len(), 2);
        assert_eq!(
            forward[0].node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            forward[1].node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn forward_definition_verify() {
        let source = String::from("def a := MyClass forward b, c");
        let ast = parse_direct(&source).unwrap();
        let (mutable, id, ty, expression, forward) = unwrap_definition!(ast);

        assert!(mutable);
        assert_eq!(ty, None);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            expression.unwrap().node,
            Node::Id {
                lit: String::from("MyClass")
            }
        );
        assert_eq!(forward.len(), 2);
        assert_eq!(
            forward[0].node,
            Node::Id {
                lit: String::from("b")
            }
        );
        assert_eq!(
            forward[1].node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn function_definition_verify() {
        let source = String::from("def f(fin b: Something, vararg c) => d");
        let ast = parse_direct(&source).unwrap();
        let (pure, id, fun_args, ret, raises, body) = unwrap_func_definition!(ast);

        assert!(!pure);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("f")
            }
        );
        assert_eq!(fun_args.len(), 2);
        assert_eq!(ret, None);
        assert_eq!(raises, vec![]);

        match body {
            Some(body) => assert_eq!(
                body.node,
                Node::Id {
                    lit: String::from("d")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }

        match (&fun_args[0].node, &fun_args[1].node) {
            (
                Node::FunArg {
                    vararg: v1,
                    var: id1,
                    mutable: mut1,
                    ty: ty1,
                    default: d1,
                },
                Node::FunArg {
                    vararg: v2,
                    var: id2,
                    mutable: mut2,
                    ty: ty2,
                    default: d2,
                },
            ) => {
                assert_eq!(v1.clone(), false);
                assert_eq!(v2.clone(), true);

                assert_eq!(
                    id1.node,
                    Node::Id {
                        lit: String::from("b")
                    }
                );
                assert_eq!(
                    id2.node,
                    Node::Id {
                        lit: String::from("c")
                    }
                );

                assert!(!mut1);
                assert!(mut2);

                match ty1.clone().unwrap().node {
                    Node::Type { id, generics } => {
                        assert_eq!(
                            id.node,
                            Node::Id {
                                lit: String::from("Something")
                            }
                        );
                        assert_eq!(generics.len(), 0);
                    }
                    other => panic!("Expected type for first argument: {:?}", other),
                }
                assert_eq!(ty2.clone(), None);

                assert_eq!(d1.clone(), None);
                assert_eq!(d2.clone(), None);
            }
            other => panic!("Expected two fun args: {:?}", other),
        }
    }

    #[test]
    fn function_no_args_definition_verify() {
        let source = String::from("def f() => d");
        let ast = parse_direct(&source).unwrap();
        let (pure, id, args, ret, _, body) = unwrap_func_definition!(ast);

        assert!(!pure);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("f")
            }
        );
        assert_eq!(args.len(), 0);
        assert_eq!(ret, None);

        match body {
            Some(body) => assert_eq!(
                body.node,
                Node::Id {
                    lit: String::from("d")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }
    }

    #[test]
    fn function_pure_definition_verify() {
        let source = String::from("def pure f() => d");
        let ast = parse_direct(&source).unwrap();
        let (pure, id, args, ret, _, body) = unwrap_func_definition!(ast);

        assert!(pure);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("f")
            }
        );
        assert_eq!(args.len(), 0);
        assert_eq!(ret, None);

        match body {
            Some(body) => assert_eq!(
                body.node,
                Node::Id {
                    lit: String::from("d")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }
    }

    #[test]
    fn function_definition_with_literal_verify() {
        let source = String::from("def f(x, vararg b: Something) => d");
        let ast = parse_direct(&source).unwrap();
        let (pure, id, fun_args, ret, _, body) = unwrap_func_definition!(ast);

        assert!(!pure);
        assert_eq!(
            id.node,
            Node::Id {
                lit: String::from("f")
            }
        );
        assert_eq!(fun_args.len(), 2);
        assert_eq!(ret, None);

        match body {
            Some(body) => assert_eq!(
                body.node,
                Node::Id {
                    lit: String::from("d")
                }
            ),
            other => panic!("Unexpected expression: {:?}", other),
        }

        match (&fun_args[0].node, &fun_args[1].node) {
            (
                Node::FunArg {
                    vararg: v1,
                    var: id1,
                    mutable: mut1,
                    ty: ty1,
                    default: d1,
                },
                Node::FunArg {
                    vararg: v2,
                    var: id2,
                    mutable: mut2,
                    ty: ty2,
                    default: d2,
                },
            ) => {
                assert!(!v1.clone());
                assert!(v2.clone());

                assert!(mut1.clone());
                assert!(mut2.clone());

                assert_eq!(
                    id1.node,
                    Node::Id {
                        lit: String::from("x")
                    }
                );
                assert_eq!(
                    id2.node,
                    Node::Id {
                        lit: String::from("b")
                    }
                );

                assert_eq!(ty1.clone(), None);
                match ty2.clone().unwrap().node {
                    Node::Type { id, generics } => {
                        assert_eq!(
                            id.node,
                            Node::Id {
                                lit: String::from("Something")
                            }
                        );
                        assert_eq!(generics.len(), 0);
                    }
                    other => panic!("Expected type for first argument: {:?}", other),
                }

                assert_eq!(d1.clone(), None);
                assert_eq!(d2.clone(), None);
            }
            other => panic!("Expected two fun args: {:?}", other),
        }
    }

    #[test]
    fn def_mut_private_wrong_order() {
        let source = String::from("def mut private a ");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn def_missing_id() {
        let source = String::from("def");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn def_fun_no_closing_brack() {
        let source = String::from("def f(a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn def_fun_missing_arrow() {
        let source = String::from("def f(a) a * 10");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn def_fun_missing_brackets() {
        let source = String::from("def f => print a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn handle_no_branches() {
        let source = String::from("def a handle");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn handle_no_indentation() {
        let source = String::from("def a handle\nerr: Err => b");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn function_definitions() -> ParseResult<()> {
        let source = resource_content(true, &["function"], "definition.mamba");
        source.parse::<AST>().map(|_| ())
    }

    #[test]
    fn function_calling() -> ParseResult<()> {
        let source = resource_content(true, &["function"], "calls.mamba");
        source.parse::<AST>().map(|_| ())
    }

    #[test]
    fn type_annotation_in_tuple() {
        let source = resource_content(false, &["syntax"], "type_annotation_in_tuple.mamba");
        source.parse::<AST>().unwrap_err();
    }
}
