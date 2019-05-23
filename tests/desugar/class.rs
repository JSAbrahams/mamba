use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;
use std::ops::Deref;

#[test]
fn import_verify() {
    let import = vec![
        to_pos_unboxed!(ASTNode::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(ASTNode::Real { lit: String::from("3000.5") }),
    ];
    let _as = vec![];
    let import = to_pos!(ASTNode::Import { import, _as });

    let core_import = match desugar(&import) {
        Core::Import { import } => import,
        other => panic!("Expected tuple but got {:?}", other)
    };

    assert_eq!(core_import.len(), 2);
    assert_eq!(core_import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
    assert_eq!(core_import[1], Core::Float { float: String::from("3000.5") });
}

#[test]
fn import_as_verify() {
    let import = vec![
        to_pos_unboxed!(ASTNode::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(ASTNode::Real { lit: String::from("3000.5") }),
    ];
    let _as = vec![to_pos_unboxed!(ASTNode::Real { lit: String::from("0.5") })];
    let import = to_pos!(ASTNode::Import { import, _as });

    let (core_import, core_as) = match desugar(&import) {
        Core::ImportAs { import, _as } => (import, _as),
        other => panic!("Expected import but got {:?}", other)
    };

    assert_eq!(core_import.len(), 2);
    assert_eq!(core_import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
    assert_eq!(core_import[1], Core::Float { float: String::from("3000.5") });
    assert_eq!(core_as.len(), 1);
    assert_eq!(core_as[0], Core::Float { float: String::from("0.5") });
}

#[test]
fn from_import_verify() {
    let id = to_pos!(ASTNode::Id { lit: String::from("afs") });
    let import = vec![
        to_pos_unboxed!(ASTNode::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(ASTNode::Real { lit: String::from("3000.5") }),
    ];
    let import = to_pos!(ASTNode::FromImport {
        id,
        import: to_pos!(ASTNode::Import { import, _as: vec![] })
    });

    let (from, import) = match desugar(&import) {
        Core::FromImport { from, import } => match &import.deref() {
            Core::Import { import } => (from.clone(), import.clone()),
            other => panic!("Expected import but got {:?}", other)
        },
        other => panic!("Expected from import but got {:?}", other)
    };

    assert_eq!(*from, Core::Id { lit: String::from("afs") });
    assert_eq!(import.len(), 2);
    assert_eq!(import[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
    assert_eq!(import[1], Core::Float { float: String::from("3000.5") });
}
