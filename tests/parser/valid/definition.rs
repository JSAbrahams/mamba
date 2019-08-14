use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse;

macro_rules! unwrap_func_definition {
    ($ast_tree:expr) => {{
        match $ast_tree.node {
            ASTNode::File { statements, .. } =>
                match &statements.first().expect("script empty.").node {
                    ASTNode::FunDef { id_type, private, pure, fun_args, ret_ty, raises, body } => (
                        private.clone(),
                        pure.clone(),
                        id_type.clone(),
                        fun_args.clone(),
                        ret_ty.clone(),
                        raises.clone(),
                        body.clone()
                    ),
                    other => panic!("Expected variable definition but was {:?}.", other)
                },
            _ => panic!("ast_tree was not script.")
        }
    }};
}

macro_rules! unwrap_definition {
    ($ast_tree:expr) => {{
        match $ast_tree.node {
            ASTNode::File { statements, .. } =>
                match &statements.first().expect("script empty.").node {
                    ASTNode::VarDef { ofmut, private, id_maybe_type, expression, forward } =>
                        match &id_maybe_type.node {
                            ASTNode::IdType { id, mutable, _type } => (
                                private.clone(),
                                mutable.clone(),
                                ofmut.clone(),
                                id.clone(),
                                _type.clone(),
                                expression.clone(),
                                forward.clone()
                            ),
                            other => panic!("Expected id type in variable def but was {:?}.", other)
                        },
                    other => panic!("Expected variable def but was {:?}.", other)
                },
            _ => panic!("ast_tree was not script.")
        }
    }};
}

#[test]
fn empty_definition_verify() {
    let source = String::from("def a");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(_type.is_none());
    assert!(expression.is_none());
    assert!(forward.is_empty());
}

#[test]
fn definition_verify() {
    let source = String::from("def a <- 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(_type.is_none());
    assert!(forward.is_empty());

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn mutable_definition_verify() {
    let source = String::from("def mut a <- 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(_type.is_none());
    assert!(forward.is_empty());

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn ofmut_definition_verify() {
    let source = String::from("def a ofmut <- 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(!mutable);
    assert!(ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(_type.is_none());
    assert!(forward.is_empty());

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn private_definition_verify() {
    let source = String::from("def private a <- 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(_type.is_none());
    assert!(forward.is_empty());

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn typed_definition_verify() {
    let source = String::from("def a: Object <- 10");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    let type_id = match _type {
        Some(_type_pos) => match _type_pos.node {
            ASTNode::Type { id, generics: _ } => id,
            other => panic!("Expected type but was: {:?}", other)
        },
        None => panic!("Expected type but was none.")
    };
    let expr = match expression {
        Some(expr_pos) => expr_pos,
        other => panic!("Unexpected expression: {:?}", other)
    };

    assert!(!private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(forward.is_empty());
    assert_eq!(expr.node, ASTNode::Int { lit: String::from("10") });
    assert_eq!(type_id.node, ASTNode::Id { lit: String::from("Object") });
}

#[test]
fn forward_empty_definition_verify() {
    let source = String::from("def a forward b, c");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert!(expression.is_none());
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(forward[1].node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn forward_definition_verify() {
    let source = String::from("def a <- MyClass forward b, c");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert!(!private);
    assert!(!mutable);
    assert!(!ofmut);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(expression.unwrap().node, ASTNode::Id { lit: String::from("MyClass") });
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(forward[1].node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn function_definition_verify() {
    let source = String::from("def f(b: Something, vararg c) => d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, raises, body) = unwrap_func_definition!(ast_tree);

    assert!(!private);
    assert!(!pure);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 2);
    assert!(ret_ty.is_none());
    assert!(raises.is_empty());

    match body {
        Some(body) => assert_eq!(body.node, ASTNode::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }

    match (&fun_args[0].node, &fun_args[1].node) {
        (
            ASTNode::FunArg { vararg: v1, id_maybe_type: id1, default: d1 },
            ASTNode::FunArg { vararg: v2, id_maybe_type: id2, default: d2 }
        ) => {
            assert_eq!(v1.clone(), false);
            assert_eq!(v2.clone(), true);
            assert_eq!(d1.clone(), None);
            assert_eq!(d2.clone(), None);

            match (&id1.node, &id2.node) {
                (
                    ASTNode::IdType { id: id1, _type: t1, mutable: false },
                    ASTNode::IdType { id: id2, _type: t2, mutable: false }
                ) => {
                    assert_eq!(id1.node, ASTNode::Id { lit: String::from("b") });
                    assert_eq!(id2.node, ASTNode::Id { lit: String::from("c") });
                    assert_eq!(t2.clone(), None);

                    match t1.clone().unwrap().node {
                        ASTNode::Type { id, generics } => {
                            assert_eq!(id.node, ASTNode::Id { lit: String::from("Something") });
                            assert_eq!(generics.len(), 0);
                        }
                        other => panic!("Expected type for first argument: {:?}", other)
                    }
                }
                other => panic!("Expected two id's: {:?}", other)
            }
        }
        other => panic!("Expected two fun args: {:?}", other)
    }
}

#[test]
fn function_no_args_definition_verify() {
    let source = String::from("def f() => d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast_tree);

    assert_eq!(private, false);
    assert!(!pure);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 0);
    assert!(ret_ty.is_none());

    match body {
        Some(body) => assert_eq!(body.node, ASTNode::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn function_pure_definition_verify() {
    let source = String::from("def pure f() => d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast_tree);

    assert_eq!(private, false);
    assert!(pure);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 0);
    assert!(ret_ty.is_none());

    match body {
        Some(body) => assert_eq!(body.node, ASTNode::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn function_definition_with_literal_verify() {
    let source = String::from("def f(x, vararg b: Something) => d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast_tree);

    assert_eq!(private, false);
    assert!(!pure);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 2);
    assert!(ret_ty.is_none());

    match body {
        Some(body) => assert_eq!(body.node, ASTNode::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }

    match (&fun_args[0].node, &fun_args[1].node) {
        (
            ASTNode::FunArg { vararg: v1, id_maybe_type: id1, default: d1 },
            ASTNode::FunArg { vararg: v2, id_maybe_type: id2, default: d2 }
        ) => {
            assert_eq!(v1.clone(), false);
            assert_eq!(v2.clone(), true);
            assert_eq!(d1.clone(), None);
            assert_eq!(d2.clone(), None);

            match (&id1.node, &id2.node) {
                (
                    ASTNode::IdType { id: id1, mutable: false, _type: None },
                    ASTNode::IdType { id: id2, mutable: false, _type: t2 }
                ) => {
                    assert_eq!(id1.node, ASTNode::Id { lit: String::from("x") });
                    assert_eq!(id2.node, ASTNode::Id { lit: String::from("b") });

                    match t2.clone().unwrap().node {
                        ASTNode::Type { id, generics } => {
                            assert_eq!(id.node, ASTNode::Id { lit: String::from("Something") });
                            assert_eq!(generics.len(), 0);
                        }
                        other => panic!("Expected type for first argument: {:?}", other)
                    }
                }
                other => panic!("Expected two id's: {:?}", other)
            }
        }
        other => panic!("Expected two fun args: {:?}", other)
    }
}
