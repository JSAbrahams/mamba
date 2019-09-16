use crate::type_checker::context::generic::field::GenericField;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Expression;
use std::convert::TryFrom;

impl TryFrom<&(&Expression, &Vec<Expression>)> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from((id, values): &(&Expression, &Vec<Expression>)) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}

impl TryFrom<&(&Expression, &Expression, &Expression)> for GenericField {
    type Error = Vec<TypeErr>;

    fn try_from(
        (id, ty, values): &(&Expression, &Expression, &Expression)
    ) -> Result<Self, Self::Error> {
        unimplemented!()
    }
}
