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
use crate::check::name::string_name::StringName;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;
use crate::parse::ast::{AST, Node};

#[derive(Debug, Clone, Eq)]
pub struct GenericClass {
    pub is_py_type: bool,
    pub name: StringName,
    pub pos: Position,
    pub concrete: bool,
    pub args: Vec<GenericFunctionArg>,
    pub fields: HashSet<GenericField>,
    pub functions: HashSet<GenericFunction>,
    pub parents: HashSet<GenericParent>,
}

impl PartialEq for GenericClass {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for GenericClass {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

impl GenericClass {
    pub fn all_pure(self, pure: bool) -> TypeResult<Self> {
        let functions = self.functions.iter().map(|f| f.clone().pure(pure)).collect();
        Ok(GenericClass { functions, ..self })
    }
}

impl TryFrom<&AST> for GenericClass {
    type Error = Vec<TypeErr>;

    fn try_from(class: &AST) -> TypeResult<GenericClass> {
        match &class.node {
            Node::Class { ty, args, parents, body } => {
                let name = StringName::try_from(ty)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        Node::Block { statements } => statements.clone(),
                        _ => return Err(vec![TypeErr::new(class.pos, "Expected block in class")]),
                    }
                } else {
                    vec![]
                };

                let mut class_args = vec![];
                let mut arg_errs = vec![];
                let mut argument_fields = HashSet::new();
                for arg in args {
                    match ClassArgument::try_from(arg) {
                        Err(err) => arg_errs.push(err),
                        Ok(ClassArgument { field, fun_arg }) => {
                            if let Some(field) = field {
                                class_args.push(fun_arg);
                                argument_fields.insert(field.in_class(
                                    Some(&name),
                                    false,
                                    arg.pos,
                                )?);
                            } else {
                                class_args.push(fun_arg);
                            }
                        }
                    }
                }

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
                        mutable: true,
                        ty: Some(Name::from(&name)),
                    }];
                    new_args.append(&mut class_args);
                    new_args
                };

                let mut temp_parents: HashSet<Node> = HashSet::new();
                let errs: Vec<(Position, String)> = parents
                    .iter()
                    .flat_map(|p| match &p.node {
                        Node::Parent { ty, .. } => match &ty.node {
                            Node::Type { id, .. } => Some((ty.pos, id.node.clone())),
                            _ => None,
                        },
                        _ => None,
                    })
                    .flat_map(|(pos, parent)| {
                        if temp_parents.contains(&parent) {
                            Some((pos, format!("Duplicate parent: {}", parent)))
                        } else {
                            temp_parents.insert(parent);
                            None
                        }
                    })
                    .collect();

                if !errs.is_empty() {
                    return Err(errs.iter().map(|(pos, msg)| TypeErr::new(*pos, msg)).collect());
                }

                let (body_fields, functions) = get_fields_and_functions(&name, &statements, false)?;
                if let Some(function) = functions.iter().find(|f| f.name == StringName::from(INIT))
                {
                    if class_args.is_empty() {
                        class_args.append(&mut function.arguments.clone())
                    } else {
                        return Err(vec![TypeErr::new(
                            class.pos,
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
                        mutable: true,
                        ty: Option::from(Name::from(&name)),
                    })
                }

                let (parents, parent_errs): (Vec<_>, Vec<_>) =
                    parents.iter().map(GenericParent::try_from).partition(Result::is_ok);
                if !parent_errs.is_empty() {
                    return Err(parent_errs.into_iter().flat_map(Result::unwrap_err).collect());
                }

                Ok(GenericClass {
                    is_py_type: false,
                    name,
                    pos: class.pos,
                    args: class_args,
                    concrete: true,
                    fields: argument_fields.union(&body_fields).cloned().collect(),
                    functions,
                    parents: parents.into_iter().map(Result::unwrap).collect(),
                })
            }
            Node::TypeDef { ty, isa, body, .. } => {
                let name = StringName::try_from(ty)?;
                let statements = if let Some(body) = body {
                    match &body.node {
                        Node::Block { statements } => statements.clone(),
                        _ => return Err(vec![TypeErr::new(class.pos, "Expected block in class")]),
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
                Ok(GenericClass {
                    is_py_type: false,
                    name,
                    pos: class.pos,
                    args: vec![GenericFunctionArg {
                        is_py_type: false,
                        name: String::from(arg::SELF),
                        pos: Default::default(),
                        has_default: false,
                        vararg: false,
                        mutable: true,
                        ty: Option::from(Name::try_from(ty)?),
                    }],
                    concrete: false,
                    fields,
                    functions,
                    parents,
                })
            }
            Node::TypeAlias { ty, isa, .. } => Ok(GenericClass {
                is_py_type: false,
                name: StringName::try_from(ty)?,
                pos: class.pos,
                args: vec![GenericFunctionArg {
                    is_py_type: false,
                    name: String::from(arg::SELF),
                    pos: Default::default(),
                    has_default: false,
                    vararg: false,
                    mutable: true,
                    ty: Option::from(Name::try_from(ty)?),
                }],
                concrete: false,
                fields: HashSet::new(),
                functions: HashSet::new(),
                parents: HashSet::from_iter(vec![GenericParent::try_from(isa.deref())?]),
            }),
            _ => Err(vec![TypeErr::new(class.pos, "Expected class or type definition")]),
        }
    }
}

fn get_fields_and_functions(
    class: &StringName,
    statements: &[AST],
    type_def: bool,
) -> TypeResult<(HashSet<GenericField>, HashSet<GenericFunction>)> {
    let mut fields = HashSet::new();
    let mut functions = HashSet::new();

    for statement in statements {
        match &statement.node {
            Node::FunDef { .. } => {
                let function = GenericFunction::try_from(statement)?;
                let function = function.in_class(Some(class), type_def, statement.pos)?;
                functions.insert(function);
            }
            Node::VariableDef { .. } => {
                let stmt_fields: HashSet<GenericField> = GenericFields::try_from(statement)?
                    .fields
                    .into_iter()
                    .map(|f| f.in_class(Some(class), type_def, statement.pos))
                    .collect::<Result<_, _>>()?;

                for generic_field in &stmt_fields {
                    if generic_field.ty.is_none() {
                        let msg =
                            format!("Class field '{}' was not assigned a type", generic_field.name);
                        return Err(vec![TypeErr::new(generic_field.pos, &msg)]);
                    }
                }

                fields = fields.union(&stmt_fields).cloned().collect();
            }
            Node::Comment { .. } | Node::DocStr { .. } => {}
            _ => {
                let msg = "Expected function or variable definition";
                return Err(vec![TypeErr::new(statement.pos, msg)]);
            }
        }
    }

    Ok((fields, functions))
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;

    use itertools::Itertools;

    use crate::check::context::clss::generic::GenericClass;
    use crate::check::name::Name;
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;
    use crate::parse::parse_direct;
    use crate::TypeErr;

    #[test]
    fn from_class_inline_args() -> Result<(), Vec<TypeErr>> {
        let source = "class MyClass(def fin a: Int, b: Int): Parent(b)\n    def c: Int := a + b\n";
        let ast = parse_direct(source)
            .expect("valid class syntax")
            .into_iter()
            .next()
            .expect("class AST");

        let generic_class = GenericClass::try_from(&ast)?;

        assert_eq!(generic_class.name, StringName::from("MyClass"));
        assert!(!generic_class.is_py_type);
        assert!(generic_class.concrete);

        assert_eq!(generic_class.parents.len(), 1);
        let parent = generic_class.parents.iter().next().expect("Parent");
        assert_eq!(parent.name, TrueName::from("Parent"));
        assert!(!parent.is_py_type);

        assert_eq!(generic_class.args.len(), 3);
        assert_eq!(generic_class.args[0].name, String::from("self"));
        assert_eq!(generic_class.args[0].ty, Some(Name::from("MyClass")));
        assert!(!generic_class.args[0].is_py_type);
        assert!(!generic_class.args[0].vararg);
        assert!(generic_class.args[0].mutable);
        assert!(!generic_class.args[0].has_default);

        assert_eq!(generic_class.args[1].name, String::from("a"));
        assert_eq!(generic_class.args[1].ty, Some(Name::from("Int")));
        assert!(!generic_class.args[1].vararg);
        assert!(!generic_class.args[1].mutable);
        assert!(!generic_class.args[1].is_py_type);
        assert!(!generic_class.args[1].has_default);

        assert_eq!(generic_class.args[2].name, String::from("b"));
        assert_eq!(generic_class.args[2].ty, Some(Name::from("Int")));
        assert!(!generic_class.args[2].vararg);
        assert!(generic_class.args[2].mutable);
        assert!(!generic_class.args[2].is_py_type);
        assert!(!generic_class.args[2].has_default);

        assert_eq!(generic_class.fields.len(), 2);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "a");
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert_eq!(field.ty, Some(Name::from("Int")));
        assert!(!field.is_py_type);
        assert!(!field.mutable);

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "c");
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert_eq!(field.ty, Some(Name::from("Int")));
        assert!(!field.is_py_type);
        assert!(field.mutable);

        Ok(())
    }

    #[test]
    fn from_class() -> Result<(), Vec<TypeErr>> {
        let source = "class MyClass\n    def c: Int := a + b\n";
        let ast = parse_direct(source)
            .expect("valid class syntax")
            .into_iter()
            .next()
            .expect("class AST");

        let generic_class = GenericClass::try_from(&ast)?;

        assert_eq!(generic_class.name, StringName::from("MyClass"));
        assert!(!generic_class.is_py_type);
        assert!(generic_class.concrete);

        assert!(generic_class.parents.is_empty());
        assert_eq!(generic_class.args.len(), 1);
        assert_eq!(generic_class.args[0].name, String::from("self"));
        assert_eq!(generic_class.args[0].ty, Some(Name::from("MyClass")));
        assert!(!generic_class.args[0].is_py_type);
        assert!(!generic_class.args[0].vararg);
        assert!(generic_class.args[0].mutable);
        assert!(!generic_class.args[0].has_default);

        assert_eq!(generic_class.fields.len(), 1);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "c");
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert_eq!(field.ty, Some(Name::from("Int")));
        assert!(!field.is_py_type);
        assert!(field.mutable);

        Ok(())
    }

    #[test]
    fn from_class_with_generic() -> Result<(), Vec<TypeErr>> {
        let source = "class MyClass[T]\n    def c: T\n";
        let ast =
            parse_direct(source).expect("valid type syntax").into_iter().next().expect("type AST");

        let generic_class = GenericClass::try_from(&ast)?;

        let name = StringName::new("MyClass", &[Name::from("T")]);
        assert_eq!(generic_class.name, name.clone());
        assert!(!generic_class.is_py_type);
        assert!(generic_class.concrete);

        assert!(generic_class.parents.is_empty());
        assert_eq!(generic_class.args.len(), 1);
        assert_eq!(generic_class.args[0].name, String::from("self"));
        assert_eq!(generic_class.args[0].ty, Some(Name::from(&name)));
        assert!(!generic_class.args[0].is_py_type);
        assert!(!generic_class.args[0].vararg);
        assert!(generic_class.args[0].mutable);
        assert!(!generic_class.args[0].has_default);

        assert_eq!(generic_class.fields.len(), 1);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "c");
        assert_eq!(field.in_class, Some(name));
        assert_eq!(field.ty, Some(Name::from("T")));
        assert!(!field.is_py_type);
        assert!(field.mutable);

        Ok(())
    }

    #[test]
    fn from_type_with_generic() -> Result<(), Vec<TypeErr>> {
        let source = "type MyType[T]\n    def c: T\n";
        let ast =
            parse_direct(source).expect("valid type syntax").into_iter().next().expect("type AST");

        let generic_class = GenericClass::try_from(&ast)?;

        let name = StringName::new("MyType", &[Name::from("T")]);
        assert_eq!(generic_class.name, name.clone());
        assert!(!generic_class.is_py_type);
        assert!(!generic_class.concrete);

        assert!(generic_class.parents.is_empty());
        assert_eq!(generic_class.args.len(), 1);
        assert_eq!(generic_class.args[0].name, String::from("self"));
        assert_eq!(generic_class.args[0].ty, Some(Name::from(&name)));
        assert!(!generic_class.args[0].is_py_type);
        assert!(!generic_class.args[0].vararg);
        assert!(generic_class.args[0].mutable);
        assert!(!generic_class.args[0].has_default);

        assert_eq!(generic_class.fields.len(), 1);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "c");
        assert_eq!(field.in_class, Some(name));
        assert_eq!(field.ty, Some(Name::from("T")));
        assert!(!field.is_py_type);
        assert!(field.mutable);

        Ok(())
    }

    #[test]
    fn from_type_def() -> Result<(), Vec<TypeErr>> {
        let source = "type MyType\n    def c: String\n";
        let ast =
            parse_direct(source).expect("valid type syntax").into_iter().next().expect("type AST");

        let generic_class = GenericClass::try_from(&ast)?;

        assert_eq!(generic_class.name, StringName::from("MyType"));
        assert!(!generic_class.is_py_type);
        assert!(!generic_class.concrete);

        assert!(generic_class.parents.is_empty());
        assert_eq!(generic_class.args.len(), 1);
        assert_eq!(generic_class.args[0].name, String::from("self"));
        assert_eq!(generic_class.args[0].ty, Some(Name::from("MyType")));
        assert!(!generic_class.args[0].is_py_type);
        assert!(!generic_class.args[0].vararg);
        assert!(generic_class.args[0].mutable);
        assert!(!generic_class.args[0].has_default);

        assert_eq!(generic_class.fields.len(), 1);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("Field");
        assert_eq!(field.name, "c");
        assert_eq!(field.in_class, Some(StringName::from("MyType")));
        assert_eq!(field.ty, Some(Name::from("String")));
        assert!(!field.is_py_type);
        assert!(field.mutable);

        Ok(())
    }
}
