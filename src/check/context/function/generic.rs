use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::ops::Deref;

use crate::check::context::arg::generic::GenericFunctionArg;
use crate::check::name::Name;
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq)]
pub struct GenericFunction {
    pub is_py_type: bool,
    pub name: StringName,
    pub pure: bool,
    pub pos: Position,
    pub arguments: Vec<GenericFunctionArg>,
    pub raises: Name,
    pub in_class: Option<StringName>,
    pub ret_ty: Option<Name>,
}

impl Hash for GenericFunction {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.arguments.hash(state);
        self.ret_ty.hash(state);
    }
}

impl PartialEq for GenericFunction {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.arguments == other.arguments && self.ret_ty == other.ret_ty
    }
}

impl GenericFunction {
    pub fn pure(self, pure: bool) -> Self {
        GenericFunction { pure: self.pure || pure, ..self }
    }

    pub fn in_class(self,
                    in_class: Option<&StringName>,
                    _type_def: bool,
                    pos: Position,
    ) -> TypeResult<GenericFunction> {
        if let Some(in_class) = in_class {
            Ok(GenericFunction {
                in_class: Some(in_class.clone()),
                arguments: self
                    .arguments
                    .iter()
                    .map(|arg| arg.clone().in_class(Some(in_class)))
                    .collect::<Result<_, _>>()?,
                ..self
            })
        } else {
            Err(Vec::from(TypeErr::new(pos, &String::from("Function must be in class."))))
        }
    }
}

impl TryFrom<&AST> for GenericFunction {
    type Error = Vec<TypeErr>;

    /// Build a function signature from a [AST](crate::parser::ast::AST).
    ///
    /// # Failures
    ///
    /// If [AST](crate::parser::ast::AST)'s node is not the
    /// [FunDef](crate::parser::ast::Node::FunDef) variant of the [Node](crate::parser::ast::Node).
    fn try_from(ast: &AST) -> TypeResult<GenericFunction> {
        match &ast.node {
            Node::FunDef { pure, id, args: fun_args, ret: ret_ty, raises, .. } => {
                Ok(GenericFunction {
                    is_py_type: false,
                    name: function_name(id.deref())?,
                    pure: *pure,
                    pos: ast.pos,
                    arguments: {
                        let args: Vec<GenericFunctionArg> = fun_args
                            .iter()
                            .map(GenericFunctionArg::try_from)
                            .collect::<Result<_, _>>()?;

                        let mut has_default = false;
                        for arg in args.clone() {
                            if has_default && !arg.has_default {
                                return Err(vec![TypeErr::new(
                                    arg.pos,
                                    "Cannot have argument with default followed by argument with \
                                     no default.",
                                )]);
                            }
                            has_default = arg.has_default;
                        }

                        args
                    },
                    ret_ty: match ret_ty {
                        Some(ty) => Some(Name::try_from(ty.as_ref())?),
                        None => None,
                    },
                    in_class: None,
                    raises: Name::try_from(raises)?,
                })
            }
            _ => Err(vec![TypeErr::new(ast.pos, "Expected function definition")]),
        }
    }
}

pub fn function_name(ast: &AST) -> TypeResult<StringName> {
    match &ast.node {
        Node::Id { lit } => Ok(StringName::from(lit.as_str())),
        _ => Err(vec![TypeErr::new(ast.pos, "Expected function true_name")]),
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use crate::{AST, TypeErr};
    use crate::check::context::function::generic::GenericFunction;
    use crate::check::name::Name;
    use crate::check::name::string_name::StringName;
    use crate::common::position::Position;
    use crate::parse::ast::Node;
    use crate::parse::parse_direct;

    #[test]
    fn from_non_fundef_node() {
        let ast = AST::new(Position::default(), Node::Pass);
        assert!(GenericFunction::try_from(&ast).is_err())
    }

    #[test]
    fn from_fundef() -> Result<(), Vec<TypeErr>> {
        let source = "def f(fin a: Int, b: String := \"a\") -> String raise [E] => pass";
        let ast = parse_direct(source).expect("valid function syntax").into_iter().next().expect("function AST");

        let generic_function = GenericFunction::try_from(&ast)?;

        assert_eq!(generic_function.name, StringName::from("f"));
        assert_eq!(generic_function.in_class, None);
        assert!(!generic_function.is_py_type);
        assert_eq!(generic_function.raises, Name::from("E"));
        assert!(!generic_function.pure);
        assert_eq!(generic_function.ret_ty, Some(Name::from("String")));
        assert_eq!(generic_function.arguments.len(), 2);

        assert_eq!(generic_function.arguments[0].name, String::from("a"));
        assert_eq!(generic_function.arguments[0].ty, Some(Name::from("Int")));
        assert!(!generic_function.arguments[0].has_default);
        assert!(!generic_function.arguments[0].is_py_type);
        assert!(!generic_function.arguments[0].mutable);
        assert!(!generic_function.arguments[0].vararg);

        assert_eq!(generic_function.arguments[1].name, String::from("b"));
        assert_eq!(generic_function.arguments[1].ty, Some(Name::from("String")));
        assert!(generic_function.arguments[1].has_default);
        assert!(!generic_function.arguments[1].is_py_type);
        assert!(generic_function.arguments[1].mutable);
        assert!(!generic_function.arguments[1].vararg);

        Ok(())
    }

    #[test]
    fn from_fundef_no_ret() -> Result<(), Vec<TypeErr>> {
        let source = "def f() => pass";
        let ast = parse_direct(source).expect("valid function syntax").into_iter().next().expect("function AST");

        let generic_function = GenericFunction::try_from(&ast)?;
        assert_eq!(generic_function.ret_ty, None);
        Ok(())
    }
}
