use mamba::lexer::tokenize;
use mamba::parser::ast::Node;
use mamba::parser::ast::AST;
use mamba::parser::parse_direct;

#[test]
fn anon_fun_no_args_verify() {
    let source = String::from("\\ => c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (args, body) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::AnonFun { args, body } => (args.clone(), body.clone()),
            _ => panic!("first element script was anon fun.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(args.len(), 0);
    assert_eq!(body.node, Node::Id { lit: String::from("c") });
}

#[test]
fn anon_fun_verify() {
    let source = String::from("\\a,b => c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (args, body) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::AnonFun { args, body } => (args.clone(), body.clone()),
            _ => panic!("first element script was anon fun.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(args.len(), 2);
    let (id1, id2) = match (&args[0], &args[1]) {
        (
            AST { node: Node::IdType { id: id1, _type: None, mutable: false }, .. },
            AST { node: Node::IdType { id: id2, _type: None, mutable: false }, .. }
        ) => (id1.clone(), id2.clone()),
        other => panic!("Id's of anon fun not id maybe type: {:?}", other)
    };

    assert_eq!(id1.node, Node::Id { lit: String::from("a") });
    assert_eq!(id2.node, Node::Id { lit: String::from("b") });

    assert_eq!(body.node, Node::Id { lit: String::from("c") });
}

#[test]
fn direct_call_verify() {
    let source = String::from("a(b, c)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (name, args) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::FunctionCall { name, args } => (name.clone(), args.clone()),
            _ => panic!("first element script was anon fun.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(name.node, Node::Id { lit: String::from("a") });
    assert_eq!(args.len(), 2);
    assert_eq!(args[0].node, Node::Id { lit: String::from("b") });
    assert_eq!(args[1].node, Node::Id { lit: String::from("c") });
}

#[test]
fn method_call_verify() {
    let source = String::from("instance.a(b, c)");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (instance, name, args) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::PropertyCall { instance, property } => match &property.node {
                Node::FunctionCall { name, args } => (instance.clone(), name.clone(), args.clone()),
                other => panic!("not function call in property call {:?}", other)
            },
            other => panic!("first element script was property call {:?}", other)
        },
        other => panic!("ast_tree was not script {:?}", other)
    };

    assert_eq!(instance.node, Node::Id { lit: String::from("instance") });

    assert_eq!(name.node, Node::Id { lit: String::from("a") });

    assert_eq!(args.len(), 2);
    assert_eq!(args[0].node, Node::Id { lit: String::from("b") });
    assert_eq!(args[1].node, Node::Id { lit: String::from("c") });
}

#[test]
fn call_verify() {
    let source = String::from("a b");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (name, args) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::FunctionCall { name, args } => (name.clone(), args.clone()),
            _ => panic!("first element script was anon fun.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(name.node, Node::Id { lit: String::from("a") });
    assert_eq!(args.len(), 1);
    assert_eq!(args[0].node, Node::Id { lit: String::from("b") });
}

#[test]
fn call_right_associative_verify() {
    let source = String::from("a b c");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (left, middle, right) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::FunctionCall { name, args } => match &args[0].node {
                Node::FunctionCall { name: middle, args } =>
                    (name.clone(), middle.clone(), args[0].clone()),
                other => panic!("Expected nested call but was {:?}.", other)
            },
            _ => panic!("first element script was anon fun.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(left.node, Node::Id { lit: String::from("a") });
    assert_eq!(middle.node, Node::Id { lit: String::from("b") });
    assert_eq!(right.node, Node::Id { lit: String::from("c") });
}
