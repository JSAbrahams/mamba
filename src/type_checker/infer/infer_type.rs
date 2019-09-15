use std::collections::HashSet;

use crate::type_checker::context::concrete::Type;
use crate::type_checker::environment::expression_type::ExpressionType;

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct InferType {
    pub raises:    HashSet<Type>,
    pub expr_type: Option<ExpressionType>
}

impl InferType {
    pub fn new(types: Option<Vec<Type>>) -> InferType {
        InferType { raises: HashSet::new(), expr_type: types.map(|ty| ExpressionType::new(&ty)) }
    }

    pub fn raises(self, raises: HashSet<Type>) -> InferType {
        let mut self_raises = self.raises.clone();
        raises.iter().for_each(|err| {
            self_raises.insert(err.clone());
        });
        InferType { raises: self_raises, expr_type: self.expr_type }
    }
}
