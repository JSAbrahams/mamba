use crate::type_checker::context::generic::type_name::GenericTypeName;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Expression;
use std::convert::TryFrom;

impl TryFrom<&Expression> for GenericTypeName {
    type Error = TypeErr;

    fn try_from(value: &Expression) -> Result<Self, Self::Error> {
        let lit = match value {
            Expression::Name(id) => id.clone(),
            _ => String::new()
        };

        Ok(GenericTypeName::Single { lit, generics: vec![] })
    }
}
