use std::collections::HashSet;
use std::convert::TryFrom;
use std::hash::{Hash, Hasher};
use std::iter::FromIterator;
use std::ops::Deref;

use crate::check::context::arg;
use crate::check::context::arg::generic::{ClassArgument, GenericFunctionArg};
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::INIT;
use crate::check::context::parent::generic::GenericParent;
use crate::check::name::Name;
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq)]
pub struct GenericClass {
    pub is_py_type: bool,
    pub name: TrueName,
    pub pos: Position,
    pub concrete: bool,
    pub args: Vec<GenericFunctionArg>,
    pub fields: HashSet<GenericField>,
    pub functions: HashSet<GenericFunction>,
    pub parents: HashSet<GenericParent>,
}

impl PartialEq for GenericClass {
    fn eq(&self, other: &Self) -> bool { self.name == other.name }
}

impl Hash for GenericClass {
    fn hash<H: Hasher>(&self, state: &mut H) { self.name.hash(state) }
}

impl GenericClass {
    pub fn all_pure(self, pure: bool) -> TypeResult<Self> {
        Ok(GenericClass {
            functions: self.functions.iter().map(|f| f.clone().pure(pure)).collect(),
            ..self
        })
    }
}

impl TryFrom<&AST> for GenericClass {
    type Error = Vec<TypeErr>;

    fn try_from(class: &AST) -> TypeResult<GenericClass> {
        match &class.node {
            // TODO add pure classes
            Node::Class { ty, args, parents, body, .. } => {
                let name = TrueName::try_from(ty)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        Node::Block { statements } => statements.clone(),
                        _ => return Err(vec![TypeErr::new(&class.pos, "Expected block in class")])
                    }
                } else {
                    vec![]
                };

                let mut class_args = vec![];
                let mut arg_errs = vec![];
                let mut argument_fields = HashSet::new();
                args.iter().for_each(|arg| match ClassArgument::try_from(arg) {
                    Err(err) => arg_errs.push(err),
                    Ok(ClassArgument { field, fun_arg }) => if let Some(field) = field {
                        class_args.push(fun_arg);
                        argument_fields.insert(field);
                    } else {
                        class_args.push(fun_arg);
                    }
                });
                if !arg_errs.is_empty() {
                    return Err(arg_errs.into_iter().flatten().collect());
                }
                let mut class_args = if class_args.is_empty() {
                    class_args
                } else {
                    let mut new_args = vec![GenericFunctionArg {
                        is_py_type: false,
                        name: String::from(arg::SELF),
                        has_default: false,
                        pos: Default::default(),
                        vararg: false,
                        mutable: false,
                        ty: Some(Name::from(&name)),
                    }];
                    new_args.append(&mut class_args);
                    new_args
                };

                let (body_fields, functions) = get_fields_and_functions(&name, &statements, false)?;
                if let Some(function) = functions.iter().find(|f| f.name == StringName::from(INIT))
                {
                    if class_args.is_empty() {
                        class_args.append(&mut function.arguments.clone())
                    } else {
                        return Err(vec![TypeErr::new(
                            &class.pos,
                            "Cannot have constructor and class arguments",
                        )]);
                    }
                }

                if class_args.is_empty() {
                    class_args.push(GenericFunctionArg {
                        is_py_type: false,
                        name: String::from(arg::SELF),
                        pos: Default::default(),
                        has_default: false,
                        vararg: false,
                        mutable: false,
                        ty: Option::from(Name::from(&name)),
                    })
                }

                let (parents, parent_errs): (Vec<_>, Vec<_>) =
                    parents.iter().map(GenericParent::try_from).partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs
                        .into_iter()
                        .flat_map(Result::unwrap_err)
                        .collect());
                }

                Ok(GenericClass {
                    is_py_type: false,
                    name,
                    pos: class.pos.clone(),
                    args: class_args,
                    concrete: true,
                    fields: argument_fields.union(&body_fields).cloned().collect(),
                    functions,
                    parents: parents.into_iter().map(Result::unwrap).collect(),
                })
            }
            Node::TypeDef { ty, isa, body, .. } => {
                let name = TrueName::try_from(ty)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        Node::Block { statements } => statements.clone(),
                        _ => return Err(vec![TypeErr::new(&class.pos, "Expected block in class")])
                    }
                } else {
                    vec![]
                };

                let parents = if let Some(isa) = isa {
                    HashSet::from_iter(vec![GenericParent::try_from(isa.deref())?])
                } else {
                    HashSet::new()
                };

                let (fields, functions) = get_fields_and_functions(&name, &statements, true)?;
                // TODO add parents to type definitions
                Ok(GenericClass {
                    is_py_type: false,
                    name,
                    pos: class.pos.clone(),
                    args: vec![],
                    concrete: false,
                    fields,
                    functions,
                    parents,
                })
            }
            Node::TypeAlias { ty, isa, .. } => Ok(GenericClass {
                is_py_type: false,
                name: TrueName::try_from(ty)?,
                pos: class.pos.clone(),
                args: vec![],
                concrete: false,
                fields: HashSet::new(),
                functions: HashSet::new(),
                parents: HashSet::from_iter(vec![GenericParent::try_from(isa.deref())?]),
            }),
            _ => Err(vec![TypeErr::new(&class.pos, "Expected class or type definition")])
        }
    }
}

fn get_fields_and_functions(
    class: &TrueName,
    statements: &[AST],
    type_def: bool,
) -> TypeResult<(HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut fields = HashSet::new();
    let mut functions = HashSet::new();

    for statement in statements {
        match &statement.node {
            Node::FunDef { .. } => {
                let function = GenericFunction::try_from(statement)?;
                let function = function.in_class(Some(class), type_def, &statement.pos)?;
                functions.insert(function);
            }
            Node::VariableDef { .. } => {
                let stmt_fields: HashSet<GenericField> = GenericFields::try_from(statement)?
                    .fields
                    .into_iter()
                    .map(|f| f.in_class(Some(class), type_def, &statement.pos))
                    .collect::<Result<_, _>>()?;
                fields = fields.union(&stmt_fields).cloned().collect();
            }
            Node::Comment { .. } | Node::DocStr { .. } => {}
            _ =>
                return Err(vec![TypeErr::new(
                    &statement.pos,
                    "Expected function or variable definition",
                )]),
        }
    }

    Ok((fields, functions))
}
