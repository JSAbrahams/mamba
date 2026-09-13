use crate::parse::ast::{Node, AST, SELF_TY};
use crate::parse::block::parse_set;
use crate::parse::definition::parse_fun_arg;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::{expected, expected_one_of, ParseResult};
use crate::parse::ty::{parse_id, parse_type};
use crate::parse::Lex;

/// Rewrite the type `Self` to the enclosing class, throughout a class body.
///
/// `Self` names the class as a type, in a method and in an associated function alike. Doing
/// this once here means no later stage has to know it exists, including the backend, which
/// reads annotations straight from the AST and would otherwise emit a bare `Self` into
/// Python.
///
/// Only type positions are rewritten. `Self` as a value is a separate concern.
///
/// Statements nested inside control flow are not descended into, so a `Self` annotation there
/// is left alone. Python never evaluates a local annotation, so that is inert rather than
/// wrong, but it is a known limit.
fn resolve_self_ty(ast: &AST, class: &AST) -> Box<AST> {
    fn in_ty(ty: &AST, class: &AST) -> AST {
        match &ty.node {
            Node::Type { id, generics } => match &id.node {
                Node::Id { lit } if lit == SELF_TY && generics.is_empty() => {
                    AST::new(ty.pos, class.node.clone())
                }
                _ => AST::new(
                    ty.pos,
                    Node::Type {
                        id: id.clone(),
                        generics: generics.iter().map(|g| in_ty(g, class)).collect(),
                    },
                ),
            },
            Node::TypeTup { types } => AST::new(
                ty.pos,
                Node::TypeTup {
                    types: types.iter().map(|t| in_ty(t, class)).collect(),
                },
            ),
            Node::TypeUnion { types } => AST::new(
                ty.pos,
                Node::TypeUnion {
                    types: types.iter().map(|t| in_ty(t, class)).collect(),
                },
            ),
            Node::TypeFun { args, ret_ty } => AST::new(
                ty.pos,
                Node::TypeFun {
                    args: args.iter().map(|a| in_ty(a, class)).collect(),
                    ret_ty: Box::from(in_ty(ret_ty, class)),
                },
            ),
            Node::QuestionOp { expr } => AST::new(
                ty.pos,
                Node::QuestionOp {
                    expr: Box::from(in_ty(expr, class)),
                },
            ),
            _ => ty.clone(),
        }
    }

    let node = match &ast.node {
        Node::Block { statements } => Node::Block {
            statements: statements
                .iter()
                .map(|stmt| *resolve_self_ty(stmt, class))
                .collect(),
        },
        Node::FunDef {
            pure,
            id,
            args,
            ret,
            raises,
            body,
        } => Node::FunDef {
            pure: *pure,
            id: id.clone(),
            args: args
                .iter()
                .map(|arg| *resolve_self_ty(arg, class))
                .collect(),
            ret: ret.as_ref().map(|ret| Box::from(in_ty(ret, class))),
            raises: raises.iter().map(|r| in_ty(r, class)).collect(),
            body: body.as_ref().map(|body| resolve_self_ty(body, class)),
        },
        Node::FunArg {
            vararg,
            mutable,
            var,
            ty,
            default,
        } => Node::FunArg {
            vararg: *vararg,
            mutable: *mutable,
            var: var.clone(),
            ty: ty.as_ref().map(|ty| Box::from(in_ty(ty, class))),
            default: default.clone(),
        },
        Node::VariableDef {
            mutable,
            var,
            ty,
            expr,
            forward,
        } => Node::VariableDef {
            mutable: *mutable,
            var: var.clone(),
            ty: ty.as_ref().map(|ty| Box::from(in_ty(ty, class))),
            expr: expr.clone(),
            forward: forward.clone(),
        },
        _ => ast.node.clone(),
    };

    Box::from(AST::new(ast.pos, node))
}

pub fn parse_class(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("class")?;
    it.eat(&Token::Class, "class")?;
    let ty = it.parse(&parse_type, "class", start)?;

    // Class arguments are always fields, never `def`-prefixed.
    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
            args.push(*it.parse(&parse_fun_arg, "constructor argument", start)?);
            it.eat_if(&Token::Comma);
            Ok(())
        })?;
        it.eat(&Token::RRBrack, "class arguments")?;
    }

    let mut parents = vec![];
    if it.eat_if(&Token::DoublePoint).is_some() {
        it.peek_while_not_tokens(&[Token::NL, Token::Where], &mut |it, lex| match lex.token {
            Token::Id(_) | Token::LRBrack => {
                parents.push(*it.parse(&parse_parent, "parents", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(Box::from(expected(
                &Token::Id(String::new()),
                &lex.clone(),
                "parents",
            ))),
        })?;
    }

    let (body, pos) = if it.peek_if(&|lex: &Lex| lex.token == Token::Where) {
        let body = it.parse(&parse_set, "class body", start)?;
        (Some(body.clone()), start.union(body.pos))
    } else {
        (None, start)
    };

    let body = body.map(|body| resolve_self_ty(&body, &ty));

    let node = Node::Class {
        ty,
        args,
        parents,
        body,
    };
    Ok(Box::from(AST::new(pos, node)))
}

pub fn parse_parent(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("parent")?;
    let ty = it.parse(&parse_type, "parent", start)?;

    let mut args = vec![];
    let end = if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, lex| match &lex.token {
            Token::Id { .. } => {
                args.push(*it.parse(&parse_id, "parent arguments", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            Token::Str { .. } => {
                args.push(*it.parse(&parse_expression, "parent arguments", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(Box::from(expected_one_of(
                &[
                    Token::Id(String::new()),
                    Token::Str(String::new(), vec![]),
                    Token::Int(String::new()),
                    Token::Real(String::new()),
                    Token::ENum(String::new(), String::new()),
                ],
                lex,
                "parent arguments",
            ))),
        })?;
        it.eat(&Token::RRBrack, "parent arguments")?
    } else {
        ty.pos
    };

    let node = Node::Parent { ty, args };
    Ok(Box::from(AST::new(start.union(end), node)))
}

/// Parse a trait definition: `trait <id> [: <parent>] [where <defs> end]`.
///
/// Unlike `type` (see `parse_type_def`), a trait has no `when <conditions>` form: traits are interface-like building blocks (as in Java/Rust),
/// not type refinement, so there is nothing to attach a runtime condition to.
pub fn parse_trait_def(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("trait definition")?;
    it.eat(&Token::Trait, "trait definition")?;
    let ty = it.parse(&parse_type, "trait definition", start)?;
    let isa = it.parse_if(&Token::DoublePoint, &parse_parent, "trait parent", start)?;

    if it.peek_if(&|lex: &Lex| lex.token == Token::Where) {
        let body = it.parse(&parse_set, "trait definition", start)?;
        // In a trait `Self` really means the implementing type, which Python cannot express.
        // Resolving it to the trait is an approximation, but it keeps the emitted annotation
        // a name that exists.
        let node = Node::Trait {
            ty: ty.clone(),
            isa,
            body: Some(resolve_self_ty(&body, &ty)),
        };
        Ok(Box::from(AST::new(start.union(body.pos), node)))
    } else {
        let node = Node::Trait {
            ty: ty.clone(),
            isa,
            body: None,
        };
        Ok(Box::from(AST::new(start.union(ty.pos), node)))
    }
}

#[cfg(test)]
mod test {
    use crate::common::result::WithSource;
    use crate::parse::ast::{Node, AST};
    use crate::parse::result::ParseErr;

    #[test]
    fn import_verify() {
        let source = String::from("import d");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block {
                statements: modules,
                ..
            } => match &modules.first().expect("script empty.").node {
                Node::Import {
                    from,
                    import,
                    alias,
                } => (from.clone(), import.clone(), alias.clone()),
                _ => panic!("first element script was not list."),
            },
            _ => panic!("ast was not script."),
        };

        assert_eq!(from, None);
        assert_eq!(import.len(), 1);
        assert!(alias.is_empty());
        assert_eq!(
            import[0].node,
            Node::Id {
                lit: String::from("d")
            }
        );
    }

    #[test]
    fn import_as_verify() {
        let source = String::from("import d as e");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block {
                statements: modules,
                ..
            } => match &modules.first().expect("script empty.").node {
                Node::Import {
                    from,
                    import,
                    alias,
                } => (from.clone(), import.clone(), alias.clone()),
                other => panic!("first element script was not import: {other:?}."),
            },
            other => panic!("ast was not script: {other:?}"),
        };

        assert_eq!(from, None);
        assert_eq!(import.len(), 1);
        assert_eq!(alias.len(), 1);
        assert_eq!(
            import[0].node,
            Node::Id {
                lit: String::from("d")
            }
        );
        assert_eq!(
            alias[0].node,
            Node::Id {
                lit: String::from("e")
            }
        );
    }

    #[test]
    fn from_import_as_verify() {
        let source = String::from("from c import d,f as e,g");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block {
                statements: modules,
                ..
            } => match &modules.first().expect("script empty.").node {
                Node::Import {
                    from,
                    import,
                    alias,
                } => (from.clone(), import.clone(), alias.clone()),
                other => panic!("first element script was not from: {other:?}."),
            },
            other => panic!("ast was not script: {other:?}"),
        };

        assert_eq!(
            from.unwrap().node,
            Node::Id {
                lit: String::from("c")
            }
        );
        assert_eq!(import.len(), 2);
        assert_eq!(alias.len(), 2);
        assert_eq!(
            import[0].node,
            Node::Id {
                lit: String::from("d")
            }
        );
        assert_eq!(
            import[1].node,
            Node::Id {
                lit: String::from("f")
            }
        );
        assert_eq!(
            alias[0].node,
            Node::Id {
                lit: String::from("e")
            }
        );
        assert_eq!(
            alias[1].node,
            Node::Id {
                lit: String::from("g")
            }
        );
    }

    #[test]
    fn parse_class_alias() {
        let source = String::from("class MyErr1: Exception(\"Something went wrong\")");
        let ast = source.parse::<AST>().unwrap();

        let (ty, args, parents, body) = match ast.node {
            Node::Block {
                statements: modules,
                ..
            } => match &modules.first().expect("script empty.").node {
                Node::Class {
                    ty,
                    args,
                    parents,
                    body,
                } => (ty.clone(), args.clone(), parents.clone(), body.clone()),
                other => panic!("Was not class: {other:?}."),
            },
            other => panic!("Ast was not script: {other:?}"),
        };

        match ty.node {
            Node::Type { id, generics } => {
                assert_eq!(
                    id.node,
                    Node::Id {
                        lit: String::from("MyErr1")
                    }
                );
                assert_eq!(generics.len(), 0);
            }
            _ => panic!("Expected type: {:?}", ty.node),
        };

        assert_eq!(args.len(), 0);
        assert_eq!(body, None);

        assert_eq!(parents.len(), 1);
        let parent = parents.first().unwrap();
        match &parent.node {
            Node::Parent { ty, args } => {
                match &ty.node {
                    Node::Type { id, generics } => {
                        assert_eq!(
                            id.node,
                            Node::Id {
                                lit: String::from("Exception")
                            }
                        );
                        assert_eq!(generics.len(), 0);
                    }
                    _ => panic!("Expected type: {:?}", ty.node),
                }
                assert_eq!(args.len(), 1);
                let arg = args.first().unwrap();
                assert_eq!(
                    arg.node,
                    Node::Str {
                        lit: String::from("Something went wrong"),
                        expressions: vec![]
                    }
                )
            }
            _ => panic!("Expected parent: {:?}", parent.node),
        }
    }

    #[test]
    fn single_line_class() {
        let source = String::from("class MyClass");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn two_classes_no_newline_after() {
        let source = String::from("class MyClass\nclass MyClass1");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn two_classes_newline_after() {
        let source = String::from("class MyClass\nclass MyClass1\n");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn class_with_single_line_body_no_newline() -> Result<(), Box<ParseErr>> {
        let source = "class MyClass\n    def var := 10";
        source
            .parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map_err(Box::new)
            .map(|_| ())
    }

    #[test]
    fn class_with_single_line_body_newline() -> Result<(), Box<ParseErr>> {
        let source = "class MyClass\n    def var := 10\n";
        source
            .parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map_err(Box::new)
            .map(|_| ())
    }

    #[test]
    fn class_with_body_class_right_after() -> Result<(), Box<ParseErr>> {
        let source = "class MyClass\n    def var := 10\nclass MyClass1\n";
        source
            .parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map_err(Box::new)
            .map(|_| ())
    }
}
