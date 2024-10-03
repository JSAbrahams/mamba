use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use itertools::{EitherOrBoth, Itertools};

use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::Class;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::{arg, Context, LookupFunction};
use crate::check::name::string_name::StringName;
use crate::check::name::Name;
use crate::check::name::{Empty, IsSuperSet, Substitute};
use crate::check::result::{TypeErr, TypeResult};
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

pub const PRINT: &str = "print";

pub const SQRT: &str = "sqrt";

pub mod generic;
pub mod python;
pub mod union;

/// A Function, which may either be top-level, or optionally within a class.
///
/// May return any Name within ret_ty.
/// May raise any Name within raises.
#[derive(Debug, Clone, Eq)]
pub struct Function {
    pub is_py_type: bool,
    pub name: StringName,
    pub self_mutable: Option<bool>,
    pub pure: bool,
    pub arguments: Vec<FunctionArg>,
    pub raises: Name,
    pub in_class: Option<StringName>,
    pub ret_ty: Name,
}

impl LookupFunction<&StringName, Function> for Context {
    /// Look up a function and substitutes generics to yield a Function.
    ///
    /// If function does not exist, treat function as constructor and see if
    /// there exists a class with the same true_name.
    fn function(&self, function: &StringName, pos: Position) -> TypeResult<Function> {
        let generics = HashMap::new();

        if let Some(generic_fun) = self.functions.iter().find(|c| &c.name == function) {
            Function::try_from((generic_fun, &generics, pos))
        } else if let Some(generic_class) = self.classes.iter().find(|c| &c.name == function) {
            let class = Class::try_from((generic_class, &generics, pos))?;
            Ok(class.constructor(true))
        } else {
            let msg = format!("Function {function} is undefined.");
            Err(vec![TypeErr::new(pos, &msg)])
        }
    }
}

impl Hash for Function {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
        self.ret_ty.hash(state)
    }
}

impl PartialEq for Function {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments && self.ret_ty == other.ret_ty
    }
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let ret = if self.ret_ty.is_empty() {
            String::new()
        } else {
            format!(" -> {}", self.ret_ty)
        };
        let raises = if self.raises.is_empty() {
            String::new()
        } else {
            format!(" raises [{}]", &self.raises)
        };
        write!(
            f,
            "{: >8} : ({}){ret}{raises}",
            self.name,
            comma_delm(&self.arguments)
        )
    }
}

impl TryFrom<(&GenericFunction, &HashMap<Name, Name>, Position)> for Function {
    type Error = Vec<TypeErr>;

    fn try_from(
        (fun, generics, pos): (&GenericFunction, &HashMap<Name, Name>, Position),
    ) -> Result<Self, Self::Error> {
        let arguments: Vec<FunctionArg> = fun
            .arguments
            .iter()
            .map(|arg| FunctionArg::try_from((arg, generics, pos)))
            .collect::<Result<_, _>>()?;

        Ok(Function {
            is_py_type: fun.is_py_type,
            name: fun.name.substitute(generics, pos)?,
            self_mutable: {
                let function_arg = arguments.iter().find_map(|a| {
                    if a.name == arg::SELF {
                        Some(a.clone())
                    } else {
                        None
                    }
                });
                function_arg.map(|a| a.mutable)
            },
            pure: fun.pure,
            arguments,
            raises: fun.raises.substitute(generics, pos)?,
            in_class: match &fun.in_class {
                Some(in_class) => Some(in_class.substitute(generics, pos)?),
                None => None,
            },
            ret_ty: match &fun.ret_ty {
                Some(ty) => ty.substitute(generics, pos)?,
                None => Name::empty(),
            },
        })
    }
}

impl Function {
    pub fn args_compatible(&self, args: &[Name], ctx: &Context, pos: Position) -> TypeResult<()> {
        for pair in self.arguments.iter().zip_longest(args) {
            match pair {
                EitherOrBoth::Both(fun_param, arg) => {
                    if let Some(arg_ty) = &fun_param.ty {
                        if !arg_ty.is_superset_of(arg, ctx, pos)? {
                            let msg = format!(
                                "'{arg}' given to argument {fun_param}, which expected a '{arg_ty}'"
                            );
                            return Err(vec![TypeErr::new(pos, &msg)]);
                        }
                    } else {
                        let msg = format!("Type of function parameter {fun_param} unknown.");
                        return Err(vec![TypeErr::new(pos, &msg)]);
                    }
                }
                EitherOrBoth::Left(fun_param) => {
                    if !fun_param.has_default {
                        let msg = format!("Expected an argument for {fun_param}.");
                        return Err(vec![TypeErr::new(pos, &msg)]);
                    }
                }
                EitherOrBoth::Right(_) => {
                    let msg = format!(
                        "{} arguments given to {self}\nExpected at most {} arguments.",
                        args.len(),
                        self.arguments.len()
                    );
                    return Err(vec![TypeErr::new(pos, &msg)]);
                }
            }
        }
        Ok(())
    }

    pub fn simple_fun(
        name: &StringName,
        self_arg: &Name,
        ret_ty: &Name,
        pos: Position,
    ) -> TypeResult<Function> {
        if self_arg.is_empty() {
            let msg = format!("'{}' self argument of '{name}' cannot be empty", arg::SELF);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(Function {
            is_py_type: false,
            name: name.clone(),
            self_mutable: None,
            pure: false,
            arguments: vec![FunctionArg {
                is_py_type: false,
                name: String::from(arg::SELF),
                has_default: false,
                vararg: false,
                mutable: false,
                ty: Some(self_arg.clone()),
            }],
            raises: Name::empty(),
            in_class: None,
            ret_ty: ret_ty.clone(),
        })
    }
}
