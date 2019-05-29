use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use mamba::parser::parse_direct;

#[test]
fn anon_fun_no_args_verify() {
    let source = String::from("\\ => c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (args, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::AnonFun { args, body } => (args.clone(), body.clone()),
                _ => panic!("first element script was anon fun.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(args.len(), 0);
    assert_eq!(body.node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn anon_fun_verify() {
    let source = String::from("\\a,b => c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (args, body) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::AnonFun { args, body } => (args.clone(), body.clone()),
                _ => panic!("first element script was anon fun.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(args.len(), 2);
    let (id1, id2) = match (&args[0], &args[1]) {
        (
            ASTNodePos { node: ASTNode::IdType { id: id1, _type: None, mutable: false }, .. },
            ASTNodePos { node: ASTNode::IdType { id: id2, _type: None, mutable: false }, .. }
        ) => (id1.clone(), id2.clone()),
        other => panic!("Id's of anon fun not id maybe type: {:?}", other)
    };

    assert_eq!(id1.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(id2.node, ASTNode::Id { lit: String::from("b") });

    assert_eq!(body.node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn direct_call_verify() {
    let source = String::from("a(b, c)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (name, args) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::FunctionCall { name, args } => (name.clone(), args.clone()),
                _ => panic!("first element script was anon fun.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(name.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(args[1].node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn method_call_verify() {
    let source = String::from("instance.a(b, c)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (instance, name, args) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::PropertyCall { instance, property } => match &property.node {
                    ASTNode::FunctionCall { name, args } =>
                        (instance.clone(), name.clone(), args.clone()),
                    other => panic!("not function call in property call {:?}", other)
                },
                other => panic!("first element script was property call {:?}", other)
            },
        other => panic!("ast_tree was not script {:?}", other)
    };

    assert_eq!(instance.node, ASTNode::Id { lit: String::from("instance") });

    assert_eq!(name.node, ASTNode::Id { lit: String::from("a") });

    assert_eq!(args.len(), 2);
    assert_eq!(args[0].node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(args[1].node, ASTNode::Id { lit: String::from("c") });
}

#[test]
fn call_verify() {
    let source = String::from("a b");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (name, args) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::FunctionCall { name, args } => (name.clone(), args.clone()),
                _ => panic!("first element script was anon fun.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(name.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].node, ASTNode::Id { lit: String::from("b") });
}

#[test]
fn call_right_associative_verify() {
    let source = String::from("a b c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (left, middle, right) = match ast_tree.node {
        ASTNode::Script { statements, .. } =>
            match &statements.first().expect("script empty.").node {
                ASTNode::FunctionCall { name, args } => match &args[0].node {
                    ASTNode::FunctionCall { name: middle, args } =>
                        (name.clone(), middle.clone(), args[0].clone()),
                    other => panic!("Expected nested call but was {:?}.", other)
                },
                _ => panic!("first element script was anon fun.")
            },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(left.node, ASTNode::Id { lit: String::from("a") });
    assert_eq!(middle.node, ASTNode::Id { lit: String::from("b") });
    assert_eq!(right.node, ASTNode::Id { lit: String::from("c") });
}
