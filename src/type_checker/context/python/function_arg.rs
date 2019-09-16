use crate::type_checker::context::generic::function_arg::GenericFunctionArg;
use crate::type_checker::type_result::TypeErr;
use python_parser::ast::Argument;
use std::convert::TryFrom;

impl TryFrom<&Argument> for GenericFunctionArg {
    type Error = Vec<TypeErr>;

    fn try_from(arg: &Argument) -> Result<Self, Self::Error> {
        match arg {
            Argument::Positional(_) => {}
            Argument::Starargs(_) => {}
            Argument::Keyword(..) => {}
            Argument::Kwargs(_) => {}
            _ => unimplemented!()
        }

        unimplemented!()
    }
}
