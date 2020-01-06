use crate::type_checker::context::type_name::TypeName;

#[derive(Clone, Debug)]
pub enum Expected {
    AnyExpression {},
    Expression { type_name: TypeName },
    NullableExpression { expected: Box<Expected> }
}
