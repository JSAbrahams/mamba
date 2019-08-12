use mamba::type_checker::type_node::Ty;
use mamba::type_checker::type_node::Type;

#[test]
fn test_eq_type_all() {
    assert_eq!(Ty::Any, Ty::Any);
    assert_eq!(Ty::Any, Ty::Float);
    assert_eq!(Ty::Float, Ty::Any);
}

#[test]
fn test_eq_float() { assert_eq!(Ty::Float, Ty::Float) }

#[test]
fn test_neq_float_string() { assert_ne!(Ty::Float, Ty::String) }

#[test]
fn test_eq_range() {
    assert_eq!(Ty::Range { ty: Box::from(Type::new(&Ty::Any)) }, Ty::Range {
        ty: Box::from(Type::new(&Ty::Any))
    })
}

#[test]
fn test_eq_list() {
    assert_eq!(Ty::List { ty: Box::from(Type::new(&Ty::Any)) }, Ty::List {
        ty: Box::from(Type::new(&Ty::Any))
    })
}

#[test]
fn test_eq_set() {
    assert_eq!(Ty::Set { ty: Box::from(Type::new(&Ty::Any)) }, Ty::Set {
        ty: Box::from(Type::new(&Ty::Any))
    })
}

#[test]
fn test_eq_tuple() {
    assert_eq!(Ty::Tuple { tys: vec![Type::new(&Ty::Int), Type::new(&Ty::Float)] }, Ty::Tuple {
        tys: vec![Type::new(&Ty::Int), Type::new(&Ty::Float)]
    })
}

#[test]
fn test_neq_tuple_different_size() {
    assert_ne!(Ty::Tuple { tys: vec![Type::new(&Ty::Int)] }, Ty::Tuple {
        tys: vec![Type::new(&Ty::Int), Type::new(&Ty::Float)]
    })
}

#[test]
fn test_neq_tuple() {
    assert_ne!(Ty::Tuple { tys: vec![Type::new(&Ty::Float), Type::new(&Ty::Int)] }, Ty::Tuple {
        tys: vec![Type::new(&Ty::Int), Type::new(&Ty::Float)]
    })
}
