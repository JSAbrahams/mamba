use mamba::lex::tokenize;
use mamba::parse::ast::Node;
use mamba::parse::parse_direct;

macro_rules! unwrap_func_definition {
    ($ast:expr) => {{
        let definition = match $ast.node {
            Node::Script { statements, .. } => statements.first().expect("script empty.").clone(),
            _ => panic!("ast was not script.")
        };
        match definition.node {
            Node::FunDef { id, private, pure, fun_args, ret_ty, raises, body, .. } =>
                (private, pure, id, fun_args, ret_ty, raises, body),
            other => panic!("Expected variabledef but was {:?}.", other)
        }
    }};
}

macro_rules! unwrap_definition {
    ($ast:expr) => {{
        let definition = match $ast.node {
            Node::Script { statements, .. } => statements.first().expect("script empty.").clone(),
            _ => panic!("ast was not script.")
        };
        match definition.node {
            Node::VariableDef { private, mutable, var, ty, expression, forward } =>
                (private, mutable, var, ty, expression, forward),
            other => panic!("Expected variabledef but was {:?}.", other)
        }
    }};
}

#[test]
fn empty_definition_verify() {
    let source = String::from("def a");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(expression, None);
    assert_eq!(forward, vec![]);
}

#[test]
fn definition_verify() {
    let source = String::from("def a <- 10");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, vec![]);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, Node::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn mutable_definition_verify() {
    let source = String::from("def mut a <- 10");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, false);
    assert_eq!(mutable, true);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, vec![]);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, Node::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn private_definition_verify() {
    let source = String::from("def private a <- 10");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, true);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, vec![]);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, Node::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn typed_definition_verify() {
    let source = String::from("def a: Object <- 10");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    let type_id = match _type {
        Some(_type_pos) => match _type_pos.node {
            Node::Type { id, generics: _ } => id,
            other => panic!("Expected type but was: {:?}", other)
        },
        None => panic!("Expected type but was none.")
    };
    let expr = match expression {
        Some(expr_pos) => expr_pos,
        other => panic!("Unexpected expression: {:?}", other)
    };

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(forward, vec![]);
    assert_eq!(expr.node, Node::Int { lit: String::from("10") });
    assert_eq!(type_id.node, Node::Id { lit: String::from("Object") });
}

#[test]
fn forward_empty_definition_verify() {
    let source = String::from("def a forward b, c");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(expression, None);
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].node, Node::Id { lit: String::from("b") });
    assert_eq!(forward[1].node, Node::Id { lit: String::from("c") });
}

#[test]
fn forward_definition_verify() {
    let source = String::from("def a <- MyClass forward b, c");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, id, _type, expression, forward) = unwrap_definition!(ast);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(expression.unwrap().node, Node::Id { lit: String::from("MyClass") });
    assert_eq!(forward.len(), 2);
    assert_eq!(forward[0].node, Node::Id { lit: String::from("b") });
    assert_eq!(forward[1].node, Node::Id { lit: String::from("c") });
}

#[test]
fn function_definition_verify() {
    let source = String::from("def f(b: Something, vararg c) => d");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, raises, body) = unwrap_func_definition!(ast);

    assert_eq!(private, false);
    assert!(!pure);
    assert_eq!(id.node, Node::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 2);
    assert_eq!(ret_ty, None);
    assert_eq!(raises, vec![]);

    match body {
        Some(body) => assert_eq!(body.node, Node::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }

    match (&fun_args[0].node, &fun_args[1].node) {
        (
            Node::FunArg { vararg: v1, var: id1, mutable: mut1, ty: ty1, default: d1 },
            Node::FunArg { vararg: v2, var: id2, mutable: mut2, ty: ty2, default: d2 }
        ) => {
            assert_eq!(v1.clone(), false);
            assert_eq!(v2.clone(), true);

            assert_eq!(id1.node, Node::Id { lit: String::from("b") });
            assert_eq!(id2.node, Node::Id { lit: String::from("c") });

            assert!(!mut1);
            assert!(!mut2);

            match ty1.clone().unwrap().node {
                Node::Type { id, generics } => {
                    assert_eq!(id.node, Node::Id { lit: String::from("Something") });
                    assert_eq!(generics.len(), 0);
                }
                other => panic!("Expected type for first argument: {:?}", other)
            }
            assert_eq!(ty2.clone(), None);

            assert_eq!(d1.clone(), None);
            assert_eq!(d2.clone(), None);
        }
        other => panic!("Expected two fun args: {:?}", other)
    }
}

#[test]
fn function_no_args_definition_verify() {
    let source = String::from("def f() => d");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast);

    assert_eq!(private, false);
    assert!(!pure);
    assert_eq!(id.node, Node::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 0);
    assert_eq!(ret_ty, None);

    match body {
        Some(body) => assert_eq!(body.node, Node::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn function_pure_definition_verify() {
    let source = String::from("def pure f() => d");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast);

    assert_eq!(private, false);
    assert!(pure);
    assert_eq!(id.node, Node::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 0);
    assert_eq!(ret_ty, None);

    match body {
        Some(body) => assert_eq!(body.node, Node::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn function_definition_with_literal_verify() {
    let source = String::from("def f(x, vararg mut b: Something) => d");
    let ast = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, pure, id, fun_args, ret_ty, _, body) = unwrap_func_definition!(ast);

    assert_eq!(private, false);
    assert!(!pure);
    assert_eq!(id.node, Node::Id { lit: String::from("f") });
    assert_eq!(fun_args.len(), 2);
    assert_eq!(ret_ty, None);

    match body {
        Some(body) => assert_eq!(body.node, Node::Id { lit: String::from("d") }),
        other => panic!("Unexpected expression: {:?}", other)
    }

    match (&fun_args[0].node, &fun_args[1].node) {
        (
            Node::FunArg { vararg: v1, var: id1, mutable: mut1, ty: ty1, default: d1 },
            Node::FunArg { vararg: v2, var: id2, mutable: mut2, ty: ty2, default: d2 }
        ) => {
            assert!(!v1.clone());
            assert!(v2.clone());

            assert!(!mut1.clone());
            assert!(mut2.clone());

            assert_eq!(id1.node, Node::Id { lit: String::from("x") });
            assert_eq!(id2.node, Node::Id { lit: String::from("b") });

            assert_eq!(ty1.clone(), None);
            match ty2.clone().unwrap().node {
                Node::Type { id, generics } => {
                    assert_eq!(id.node, Node::Id { lit: String::from("Something") });
                    assert_eq!(generics.len(), 0);
                }
                other => panic!("Expected type for first argument: {:?}", other)
            }

            assert_eq!(d1.clone(), None);
            assert_eq!(d2.clone(), None);
        }
        other => panic!("Expected two fun args: {:?}", other)
    }
}
