use mamba::type_checker::node_type::Type;

#[test]
fn test_eq_type_all() {
    assert_eq!(Type::Any, Type::Any);
    assert_eq!(Type::Any, Type::Float);
    assert_eq!(Type::Float, Type::Any);
}

#[test]
fn test_eq_float() { assert_eq!(Type::Float, Type::Float) }

#[test]
fn test_neq_float_string() { assert_ne!(Type::Float, Type::String) }

#[test]
fn test_eq_maybe() {
    assert_eq!(Type::Maybe { ty: Box::from(Type::Any) }, Type::Maybe { ty: Box::from(Type::Any) })
}

#[test]
fn test_eq_range() {
    assert_eq!(Type::Range { ty: Box::from(Type::Any) }, Type::Range { ty: Box::from(Type::Any) })
}

#[test]
fn test_eq_list() {
    assert_eq!(Type::List { ty: Box::from(Type::Any) }, Type::List { ty: Box::from(Type::Any) })
}

#[test]
fn test_eq_set() {
    assert_eq!(Type::Set { ty: Box::from(Type::Any) }, Type::Set { ty: Box::from(Type::Any) })
}

#[test]
fn test_eq_tuple() {
    assert_eq!(Type::Tuple { ty: vec![Type::Int, Type::Float] }, Type::Tuple {
        ty: vec![Type::Int, Type::Float]
    })
}

#[test]
fn test_neq_tuple_different_size() {
    assert_ne!(Type::Tuple { ty: vec![Type::Int] }, Type::Tuple {
        ty: vec![Type::Int, Type::Float]
    })
}

#[test]
fn test_neq_tuple() {
    assert_ne!(Type::Tuple { ty: vec![Type::Float, Type::Int] }, Type::Tuple {
        ty: vec![Type::Int, Type::Float]
    })
}
