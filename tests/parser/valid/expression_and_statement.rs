use mamba::lexer::tokenize;
use mamba::parser::ast::Node;
use mamba::parser::parse;
use mamba::parser::parse_direct;

#[test]
fn pure_file() {
    let source = String::from("pure");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let pure = match ast_tree.node {
        Node::File { pure, .. } => pure,
        _ => panic!("ast_tree was not file.")
    };

    assert!(pure);
}

#[test]
fn range_verify() {
    let source = String::from("hello .. world");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("hello") });
    assert_eq!(to.node, Node::Id { lit: String::from("world") });
    assert!(!inclusive);
    assert_eq!(step, None);
}

#[test]
fn range_step_verify() {
    let source = String::from("hello .. world step 2");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("hello") });
    assert_eq!(to.node, Node::Id { lit: String::from("world") });
    assert!(!inclusive);
    assert_eq!(step.unwrap().node, Node::Int { lit: String::from("2") });
}

#[test]
fn range_incl_verify() {
    let source = String::from("foo ..= bar");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Range { from, to, inclusive, step } =>
                (from.clone(), to.clone(), inclusive.clone(), step.clone()),
            _ => panic!("first element script was not range inclusive.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("foo") });
    assert_eq!(to.node, Node::Id { lit: String::from("bar") });
    assert!(inclusive);
    assert_eq!(step, None);
}

#[test]
fn reassign_verify() {
    let source = String::from("id <- new_value");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Reassign { left, right } => (left.clone(), right.clone()),
            _ => panic!("first element script was not reassign.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(left.node, Node::Id { lit: String::from("id") });
    assert_eq!(right.node, Node::Id { lit: String::from("new_value") });
}

#[test]
fn print_verify() {
    let source = String::from("print some_value");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let expr = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Print { expr } => expr.clone(),
            _ => panic!("first element script was not reassign.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(expr.node, Node::Id { lit: String::from("some_value") });
}

#[test]
fn return_verify() {
    let source = String::from("return some_value");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let expr = match ast_tree.node {
        Node::Script { statements, .. } => match &statements.first().expect("script empty.").node {
            Node::Return { expr } => expr.clone(),
            _ => panic!("first element script was not reassign.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(expr.node, Node::Id { lit: String::from("some_value") });
}

#[test]
fn retry_verify() {
    let source = String::from("retry");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let node_pos = match ast_tree.node {
        Node::Script { statements, .. } => statements.first().expect("script empty.").clone(),
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(node_pos.node, Node::Retry);
}

#[test]
fn underscore_verify() {
    let source = String::from("_");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let node_pos = match ast_tree.node {
        Node::Script { statements, .. } => statements.first().expect("script empty.").clone(),
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(node_pos.node, Node::Underscore);
}

#[test]
fn pass_verify() {
    let source = String::from("pass");
    let ast_tree = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let node_pos = match ast_tree.node {
        Node::Script { statements, .. } => statements.first().expect("script empty.").clone(),
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(node_pos.node, Node::Pass);
}

#[test]
fn from_import_verify() {
    let source = String::from("from a import b");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast_tree.node {
        Node::File { imports, .. } => imports,
        _ => panic!("ast_tree was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (id, _use, _as) = match &imports[0].node {
        Node::FromImport { id, import } => match &import.node {
            Node::Import { import, _as } => (id, import, _as),
            other => panic!("Expected import but was {:?}.", other)
        },
        other => panic!("Expected from import but was {:?}.", other)
    };

    assert_eq!(id.node, Node::Id { lit: String::from("a") });
    assert_eq!(_use.len(), 1);
    assert_eq!(_use[0].node, Node::Id { lit: String::from("b") });
    assert_eq!(_as.len(), 0);
}

#[test]
fn import_verify() {
    let source = String::from("import c");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast_tree.node {
        Node::File { imports, .. } => imports,
        _ => panic!("ast_tree was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (_use, _as) = match &imports[0].node {
        Node::Import { import, _as } => (import, _as),
        other => panic!("Expected import but was {:?}.", other)
    };

    assert_eq!(_use.len(), 1);
    assert_eq!(_use[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(_as.len(), 0);
}

#[test]
fn import_as_verify() {
    let source = String::from("import a, b as c, d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast_tree.node {
        Node::File { imports, .. } => imports,
        _ => panic!("ast_tree was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (_use, _as) = match &imports[0].node {
        Node::Import { import, _as } => (import, _as),
        other => panic!("Expected import but was {:?}.", other)
    };

    assert_eq!(_use.len(), 2);
    assert_eq!(_use[0].node, Node::Id { lit: String::from("a") });
    assert_eq!(_use[1].node, Node::Id { lit: String::from("b") });
    assert_eq!(_as.len(), 2);
    assert_eq!(_as[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(_as[1].node, Node::Id { lit: String::from("d") });
}
