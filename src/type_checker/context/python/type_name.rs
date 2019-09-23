use crate::type_checker::context::generic::type_name::GenericTypeName;
use python_parser::ast::Expression;

impl From<&Expression> for GenericTypeName {
    fn from(value: &Expression) -> GenericTypeName {
        let lit = match value {
            Expression::Name(id) => id.clone(),
            _ => String::new()
        };

        GenericTypeName::Single { lit, generics: vec![] }
    }
}
