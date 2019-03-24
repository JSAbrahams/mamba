use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse_direct;

macro_rules! unwrap_definition {
    ($ast_tree:expr) => {{
        let (private, definition) = match $ast_tree.node {
            ASTNode::Script { statements, .. } =>
                match &statements.first().expect("script empty.").node {
                    ASTNode::Def { private, definition } => (private.clone(), definition.clone()),
                    _ => panic!("first element script was not foreach.")
                },
            _ => panic!("ast_tree was not script.")
        };

        let (mutable, ofmut, id, _type, expression, forward) = match definition.node {
            ASTNode::VariableDef { mutable, ofmut, id_maybe_type, expression, forward } =>
                match id_maybe_type.node {
                    ASTNode::IdType { id, _type } =>
                        (mutable, ofmut, id, _type, expression, forward),
                    other => panic!("Expected id type in variable def but was {:?}.", other)
                },
            other => panic!("Expected variabledef but was {:?}.", other)
        };

        (private, mutable, ofmut, id, _type, expression, forward)
    }};
}

#[test]
fn empty_definition_verify() {
    let source = String::from("def a");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(expression, None);
    assert_eq!(forward, None);
}

#[test]
fn definition_verify() {
    let source = String::from("def a <- 10");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, None);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn mutable_definition_verify() {
    let source = String::from("def mut a <- 10");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert_eq!(private, false);
    assert_eq!(mutable, true);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, None);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn ofmut_definition_verify() {
    let source = String::from("def a ofmut <- 10");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, true);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, None);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn private_definition_verify() {
    let source = String::from("def private a <- 10");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    assert_eq!(private, true);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(_type, None);
    assert_eq!(forward, None);

    match expression {
        Some(expr_pos) => assert_eq!(expr_pos.node, ASTNode::Int { lit: String::from("10") }),
        other => panic!("Unexpected expression: {:?}", other)
    }
}

#[test]
fn typed_definition_verify() {
    let source = String::from("def a: Object <- 10");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
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

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(forward, None);
    assert_eq!(expr.node, ASTNode::Int { lit: String::from("10") });
    assert_eq!(type_id.node, ASTNode::Id { lit: String::from("Object") });
}

#[test]
fn forward_definition_verify() {
    let source = String::from("def a forward b, c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();
    let (private, mutable, ofmut, id, _type, expression, forward) = unwrap_definition!(ast_tree);

    let forwarded = match forward {
        Some(forward) => forward,
        None => panic!("Expected type but was none.")
    };

    assert_eq!(private, false);
    assert_eq!(mutable, false);
    assert_eq!(ofmut, false);
    assert_eq!(id.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(expression, None);
    assert_eq!(forwarded.len(), 2);
    assert_eq!(forwarded[0].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(forwarded[1].node, ASTNode::Id { lit: String::from("c") });
}
