use python_parser::ast::Funcdef;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::context::function::generic::GenericFunction;
use crate::check::name::string_name::StringName;
use crate::check::name::{Empty, Name};
use crate::common::position::Position;

pub const INIT: &str = "__init__";

pub const ADD: &str = "__add__";
pub const DIV: &str = "__truediv__";
pub const EQ: &str = "__eq__";
pub const NEQ: &str = "__ne__";
pub const FDIV: &str = "__floordiv__";
pub const GE: &str = "__gt__";
pub const GEQ: &str = "__ge__";
pub const LE: &str = "__lt__";
pub const LEQ: &str = "__le__";
pub const MOD: &str = "__mod__";
pub const MUL: &str = "__mul__";
pub const POW: &str = "__pow__";
pub const SUB: &str = "__sub__";

pub const STR: &str = "__str__";
pub const TRUTHY: &str = "__bool__";
pub const NEXT: &str = "__next__";
pub const ITER: &str = "__iter__";
pub const CONTAINS: &str = "__contains__";

pub const SUPER: &str = "super";

pub const GET_ITEM: &str = "__getitem__";

impl From<&Funcdef> for GenericFunction {
    fn from(func_def: &Funcdef) -> GenericFunction {
        GenericFunction {
            is_py_type: true,
            name: StringName::from(func_def.name.as_str()),
            pure: false,
            pos: Position::invisible(),
            arguments: func_def
                .parameters
                .positional_args
                .iter()
                .map(|(name, ty, expr)| GenericFunctionArg::from((name, ty, expr)))
                .collect(),
            raises: Name::empty(),
            in_class: None,
            ret_ty: func_def.return_type.as_ref().map(Name::from),
        }
    }
}

#[cfg(test)]
mod test {
    use std::ops::Deref;

    use python_parser::ast::{CompoundStatement, Funcdef, Statement};

    use crate::check::context::function::generic::GenericFunction;
    use crate::check::name::string_name::StringName;
    use crate::check::name::{Empty, Name};

    fn fun_def(stmt: &Statement) -> Funcdef {
        match &stmt {
            Statement::Compound(compound) => match compound.deref() {
                CompoundStatement::Funcdef(funcdef) => funcdef.clone(),
                other => panic!("Not func def but {:?}", other),
            },
            other => panic!("Not compound statement but {:?}", other),
        }
    }

    #[test]
    fn from_py() {
        let source = "def f(a: int, b, c: Str, d: Str = 'default') -> complex: pass";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let funcdef: Funcdef = fun_def(&first);
        let generic_function = GenericFunction::from(&funcdef);

        assert!(generic_function.is_py_type);
        assert_eq!(generic_function.name, StringName::from("f"));
        assert!(!generic_function.pure);
        assert_eq!(generic_function.raises, Name::empty());
        assert!(generic_function.in_class.is_none());
        assert_eq!(generic_function.ret_ty, Some(Name::from("Complex")));

        assert_eq!(generic_function.arguments[0].name, String::from("a"));
        assert_eq!(generic_function.arguments[0].ty, Some(Name::from("Int")));
        assert!(!generic_function.arguments[0].has_default);
        assert!(generic_function.arguments[0].mutable);

        assert_eq!(generic_function.arguments[1].name, String::from("b"));
        assert_eq!(generic_function.arguments[1].ty, None);
        assert!(!generic_function.arguments[1].has_default);
        assert!(generic_function.arguments[1].mutable);

        assert_eq!(generic_function.arguments[2].name, String::from("c"));
        assert_eq!(generic_function.arguments[2].ty, Some(Name::from("Str")));
        assert!(!generic_function.arguments[2].has_default);
        assert!(generic_function.arguments[2].mutable);

        assert_eq!(generic_function.arguments[3].name, String::from("d"));
        assert_eq!(generic_function.arguments[3].ty, Some(Name::from("Str")));
        assert!(generic_function.arguments[3].has_default);
        assert!(generic_function.arguments[3].mutable);
    }
}
