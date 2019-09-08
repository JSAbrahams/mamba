use mamba::common::position::EndPoint;
use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::ASTNode;
use mamba::parser::ast::ASTNodePos;

#[test]
fn tuple_verify() {
    let elements = vec![
        to_pos_unboxed!(ASTNode::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(ASTNode::Real { lit: String::from("3000.5") }),
    ];
    let tuple = to_pos!(ASTNode::Tuple { elements });
    let core = desugar(&tuple);

    let core_elements = match core {
        Ok(Core::Tuple { elements }) => elements,
        other => panic!("Expected tuple but got {:?}", other)
    };

    assert_eq!(core_elements[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
    assert_eq!(core_elements[1], Core::Float { float: String::from("3000.5") });
}

#[test]
fn set_verify() {
    let elements = vec![
        to_pos_unboxed!(ASTNode::IdType {
            id:      to_pos!(ASTNode::Id { lit: String::from("a") }),
            mutable: false,
            _type:   Some(to_pos!(ASTNode::Type {
                id:       to_pos!(ASTNode::Id { lit: String::from("some_type") }),
                generics: vec![]
            }))
        }),
        to_pos_unboxed!(ASTNode::Bool { lit: true }),
    ];
    let tuple = to_pos!(ASTNode::Set { elements });
    let core = desugar(&tuple);

    let core_elements = match core {
        Ok(Core::Set { elements }) => elements,
        other => panic!("Expected tuple but got {:?}", other)
    };

    assert_eq!(core_elements[0], Core::Id { lit: String::from("a") });
    assert_eq!(core_elements[1], Core::Bool { _bool: true });
}

#[test]
fn list_verify() {
    let elements = vec![
        to_pos_unboxed!(ASTNode::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(ASTNode::Real { lit: String::from("3000.5") }),
    ];
    let tuple = to_pos!(ASTNode::List { elements });
    let core = desugar(&tuple);

    let core_elements = match core {
        Ok(Core::List { elements }) => elements,
        other => panic!("Expected tuple but got {:?}", other)
    };

    assert_eq!(core_elements[0], Core::ENum { num: String::from("a"), exp: String::from("100") });
    assert_eq!(core_elements[1], Core::Float { float: String::from("3000.5") });
}

#[test]
fn set_builder_verify() {
    let item = to_pos!(ASTNode::Id { lit: String::from("a") });
    let conditions = vec![];
    let list_builder = to_pos!(ASTNode::SetBuilder { item, conditions });

    let desugar_result = desugar(&list_builder);
    assert!(desugar_result.is_err());
}

#[test]
fn list_builder_verify() {
    let item = to_pos!(ASTNode::Id { lit: String::from("a") });
    let conditions = vec![];
    let list_builder = to_pos!(ASTNode::ListBuilder { item, conditions });

    let desugar_result = desugar(&list_builder);
    assert!(desugar_result.is_err());
}
