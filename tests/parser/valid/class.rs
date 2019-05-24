use crate::common::*;
use mamba::lexer::tokenize;
use mamba::parser::ast::ASTNode;
use mamba::parser::parse;

#[test]
fn parse_class() {
    let source = resource_content(true, &["class"], "types.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn parse_imports_class() {
    let source = resource_content(true, &["class"], "import.mamba");
    parse(&tokenize(&source).unwrap()).unwrap();
}

#[test]
fn import_verify() {
    let source = String::from("import d");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (import, _as) = match ast_tree.node {
        ASTNode::File { imports, .. } => match &imports.first().expect("script empty.").node {
            ASTNode::Import { import, _as } => (import.clone(), _as.clone()),
            _ => panic!("first element script was not list.")
        },
        _ => panic!("ast_tree was not script.")
    };

    assert_eq!(import.len(), 1);
    assert!(_as.is_empty());
    assert_eq!(import[0].node, ASTNode::Id { lit: String::from("d") });
}

#[test]
fn import_as_verify() {
    let source = String::from("import d as e");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (import, _as) = match ast_tree.node {
        ASTNode::File { imports, .. } => match &imports.first().expect("script empty.").node {
            ASTNode::Import { import, _as } => (import.clone(), _as.clone()),
            other => panic!("first element script was not import: {:?}.", other)
        },
        other => panic!("ast_tree was not script: {:?}", other)
    };

    assert_eq!(import.len(), 1);
    assert_eq!(_as.len(), 1);
    assert_eq!(import[0].node, ASTNode::Id { lit: String::from("d") });
    assert_eq!(_as[0].node, ASTNode::Id { lit: String::from("e") });
}

#[test]
fn from_import_as_verify() {
    let source = String::from("from c import d,f as e,g");
    let ast_tree = parse(&tokenize(&source).unwrap()).unwrap();

    let (from, import, _as) = match ast_tree.node {
        ASTNode::File { imports, .. } => match &imports.first().expect("script empty.").node {
            ASTNode::FromImport { id, import } => match &import.node {
                ASTNode::Import { import, _as } => (id.clone(), import.clone(), _as.clone()),
                other => panic!("not import: {:?}.", other)
            },
            other => panic!("first element script was not from: {:?}.", other)
        },
        other => panic!("ast_tree was not script: {:?}", other)
    };

    assert_eq!(from.node, ASTNode::Id { lit: String::from("c") });
    assert_eq!(import.len(), 2);
    assert_eq!(_as.len(), 2);
    assert_eq!(import[0].node, ASTNode::Id { lit: String::from("d") });
    assert_eq!(import[1].node, ASTNode::Id { lit: String::from("f") });
    assert_eq!(_as[0].node, ASTNode::Id { lit: String::from("e") });
    assert_eq!(_as[1].node, ASTNode::Id { lit: String::from("g") });
}
