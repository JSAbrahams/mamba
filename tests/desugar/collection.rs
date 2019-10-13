use mamba::common::position::EndPoint;
use mamba::common::position::Position;
use mamba::core::construct::Core;
use mamba::desugar::desugar;
use mamba::parser::ast::Node;
use mamba::parser::ast::AST;

#[test]
fn tuple_verify() {
    let elements = vec![
        to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
    ];
    let tuple = to_pos!(Node::Tuple { elements });
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
        to_pos_unboxed!(Node::Id { lit: String::from("a") }),
        to_pos_unboxed!(Node::Bool { lit: true }),
    ];
    let set = to_pos!(Node::Set { elements });
    let core = desugar(&set);

    let core_elements = match core {
        Ok(Core::Set { elements }) => elements,
        other => panic!("Expected set but got {:?}", other)
    };

    assert_eq!(core_elements[0], Core::Id { lit: String::from("a") });
    assert_eq!(core_elements[1], Core::Bool { _bool: true });
}

#[test]
fn list_verify() {
    let elements = vec![
        to_pos_unboxed!(Node::ENum { num: String::from("a"), exp: String::from("100") }),
        to_pos_unboxed!(Node::Real { lit: String::from("3000.5") }),
    ];
    let tuple = to_pos!(Node::List { elements });
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
    let item = to_pos!(Node::Id { lit: String::from("a") });
    let conditions = vec![];
    let list_builder = to_pos!(Node::SetBuilder { item, conditions });

    let desugar_result = desugar(&list_builder);
    assert!(desugar_result.is_err());
}

#[test]
fn list_builder_verify() {
    let item = to_pos!(Node::Id { lit: String::from("a") });
    let conditions = vec![];
    let list_builder = to_pos!(Node::ListBuilder { item, conditions });

    let desugar_result = desugar(&list_builder);
    assert!(desugar_result.is_err());
}
