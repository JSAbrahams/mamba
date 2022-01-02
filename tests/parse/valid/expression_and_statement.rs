use mamba::lex::tokenize;
use mamba::parse::ast::Node;
use mamba::parse::parse;

use crate::parse::util::parse_direct;

#[test]
fn range_verify() {
    let source = String::from("hello .. world");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
        Node::Range { from, to, inclusive, step } =>
            (from.clone(), to.clone(), inclusive.clone(), step.clone()),
        _ => panic!("first element script was not range.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("hello") });
    assert_eq!(to.node, Node::Id { lit: String::from("world") });
    assert!(!inclusive);
    assert_eq!(step, None);
}

#[test]
fn range_step_verify() {
    let source = String::from("hello .. world step 2");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
        Node::Range { from, to, inclusive, step } =>
            (from.clone(), to.clone(), inclusive.clone(), step.clone()),
        _ => panic!("first element script was not range.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("hello") });
    assert_eq!(to.node, Node::Id { lit: String::from("world") });
    assert!(!inclusive);
    assert_eq!(step.unwrap().node, Node::Int { lit: String::from("2") });
}

#[test]
fn range_incl_verify() {
    let source = String::from("foo ..= bar");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (from, to, inclusive, step) = match &statements.first().expect("script empty.").node {
        Node::Range { from, to, inclusive, step } =>
            (from.clone(), to.clone(), inclusive.clone(), step.clone()),
        _ => panic!("first element script was not range inclusive.")
    };

    assert_eq!(from.node, Node::Id { lit: String::from("foo") });
    assert_eq!(to.node, Node::Id { lit: String::from("bar") });
    assert!(inclusive);
    assert_eq!(step, None);
}

#[test]
fn reassign_verify() {
    let source = String::from("id := new_value");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let (left, right) = match &statements.first().expect("script empty.").node {
        Node::Reassign { left, right } => (left.clone(), right.clone()),
        _ => panic!("first element script was not reassign.")
    };

    assert_eq!(left.node, Node::Id { lit: String::from("id") });
    assert_eq!(right.node, Node::Id { lit: String::from("new_value") });
}

#[test]
fn print_verify() {
    let source = String::from("print some_value");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let expr = match &statements.first().expect("script empty.").node {
        Node::Print { expr } => expr.clone(),
        _ => panic!("first element script was not reassign.")
    };

    assert_eq!(expr.node, Node::Id { lit: String::from("some_value") });
}

#[test]
fn return_verify() {
    let source = String::from("return some_value");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let expr = match &statements.first().expect("script empty.").node {
        Node::Return { expr } => expr.clone(),
        _ => panic!("first element script was not reassign.")
    };

    assert_eq!(expr.node, Node::Id { lit: String::from("some_value") });
}

#[test]
fn underscore_verify() {
    let source = String::from("_");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let ast = statements.first().expect("script empty.").clone();
    assert_eq!(ast.node, Node::Underscore);
}

#[test]
fn pass_verify() {
    let source = String::from("pass");
    let statements = parse_direct(&tokenize(&source).unwrap()).unwrap();

    let ast = statements.first().expect("script empty.").clone();
    assert_eq!(ast.node, Node::Pass);
}

#[test]
fn from_import_verify() {
    let source = String::from("from a import b");
    let ast = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast.node {
        Node::File { statements: modules, .. } => modules,
        _ => panic!("ast was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (id, _use, _as) = match &imports[0].node {
        Node::FromImport { id, import } => match &import.node {
            Node::Import { import, aliases: _as } => (id, import, _as),
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
    let ast = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast.node {
        Node::File { statements: modules, .. } => modules,
        _ => panic!("ast was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (_use, _as) = match &imports[0].node {
        Node::Import { import, aliases: _as } => (import, _as),
        other => panic!("Expected import but was {:?}.", other)
    };

    assert_eq!(_use.len(), 1);
    assert_eq!(_use[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(_as.len(), 0);
}

#[test]
fn import_as_verify() {
    let source = String::from("import a, b as c, d");
    let ast = parse(&tokenize(&source).unwrap()).unwrap();

    let imports = match ast.node {
        Node::File { statements: modules, .. } => modules,
        _ => panic!("ast was not file.")
    };

    assert_eq!(imports.len(), 1);
    let (_use, _as) = match &imports[0].node {
        Node::Import { import, aliases: _as } => (import, _as),
        other => panic!("Expected import but was {:?}.", other)
    };

    assert_eq!(_use.len(), 2);
    assert_eq!(_use[0].node, Node::Id { lit: String::from("a") });
    assert_eq!(_use[1].node, Node::Id { lit: String::from("b") });
    assert_eq!(_as.len(), 2);
    assert_eq!(_as[0].node, Node::Id { lit: String::from("c") });
    assert_eq!(_as[1].node, Node::Id { lit: String::from("d") });
}
