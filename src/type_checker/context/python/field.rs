use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Expression;
use std::convert::TryFrom;

impl TryFrom<(&Vec<Expression>, &Vec<Vec<Expression>>)> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(
        (id, values): (&Vec<Expression>, &Vec<Vec<Expression>>)
    ) -> Result<Self, Self::Error> {
        Ok(GenericField {
            name:    "".to_string(),
            pos:     Default::default(),
            private: false,
            mutable: false,
            ty:      None
        })
    }
}

impl TryFrom<(&Vec<Expression>, &Expression, &Vec<Expression>)> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(
        (id, ty, values): (&Vec<Expression>, &Expression, &Vec<Expression>)
    ) -> Result<Self, Self::Error> {
        Ok(GenericField {
            name:    "".to_string(),
            pos:     Default::default(),
            private: false,
            mutable: false,
            ty:      None
        })
    }
}
