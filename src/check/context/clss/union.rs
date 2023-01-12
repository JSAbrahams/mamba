use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::{Context, TypeErr};
use crate::check::context::{clss, LookupClass};
use crate::check::context::clss::{Class, GetField, GetFun, HasParent};
use crate::check::context::field::Field;
use crate::check::context::field::union::FieldUnion;
use crate::check::context::function::Function;
use crate::check::context::function::union::FunUnion;
use crate::check::name::{Empty, Name, TupleCallable};
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

#[derive(Debug, Eq)]
pub struct ClassUnion {
    union: HashSet<Class>,
}

impl From<&ClassUnion> for Name {
    fn from(class_union: &ClassUnion) -> Self {
        let names: HashSet<Name> = class_union
            .union
            .iter()
            .map(|u| match u.name.name.as_str() { // very odd map
                clss::TUPLE => {
                    let args = u.name.generics.get(0).cloned().unwrap_or_else(|| Name::from(""));
                    let args = args.names.iter().next().map_or_else(Vec::new, |name| if name.is_tuple() {
                        name.elements(Position::invisible()).expect("Unreachable")
                    } else {
                        vec![]
                    });

                    let ret = u.name.generics.get(1).cloned().unwrap_or_else(|| Name::from(""));
                    Name::callable(args.as_slice(), &ret)
                }
                clss::CALLABLE => Name::tuple(u.name.generics.clone().as_slice()),
                _ => Name::from(&u.name)
            })
            .collect();

        Name::from(&names)
    }
}

impl GetField<FieldUnion> for ClassUnion {
    fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<FieldUnion> {
        let union: HashSet<Field> =
            self.union.iter().map(|c| c.field(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define attribute '{name}'", Name::from(self));
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(FieldUnion::from(&union))
    }
}

impl GetFun<FunUnion> for ClassUnion {
    /// Check if ClassUnion implements a function.
    fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<FunUnion> {
        let union: HashSet<Function> =
            self.union.iter().map(|c| c.fun(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define function '{name}'", Name::from(self));
            return Err(vec![TypeErr::new(pos, &msg)]);
        }
        Ok(FunUnion::from(&union))
    }
}

impl HasParent<&TrueName> for ClassUnion {
    fn has_parent(
        &self,
        name: &TrueName,
        ctx: &Context,
        pos: Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&Name> for ClassUnion {
    fn has_parent(&self, name: &Name, ctx: &Context, pos: Position) -> TypeResult<bool> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl HasParent<&StringName> for ClassUnion {
    fn has_parent(
        &self,
        name: &StringName,
        ctx: &Context,
        pos: Position,
    ) -> Result<bool, Vec<TypeErr>> {
        let res: Vec<bool> =
            self.union.iter().map(|c| c.has_parent(name, ctx, pos)).collect::<Result<_, _>>()?;
        Ok(res.iter().all(|b| *b))
    }
}

impl Display for ClassUnion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{{{}}}", comma_delm(&self.union))
    }
}

impl PartialEq for ClassUnion {
    fn eq(&self, other: &Self) -> bool {
        self.union.len() == other.union.len()
            && self.union.iter().zip(&other.union).all(|(this, that)| this == that)
    }
}

impl Hash for ClassUnion {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.union.iter().for_each(|c| c.hash(state))
    }
}

impl LookupClass<&Name, ClassUnion> for Context {
    /// Look up GenericClass and substitute generics to field a Class.
    ///
    /// # Error
    ///
    /// If NameUnion is empty.
    fn class(&self, name: &Name, pos: Position) -> TypeResult<ClassUnion> {
        if name.is_empty() {
            return Err(vec![TypeErr::new(pos, &format!("Unexpected '{name}'"))]);
        }

        let union = name.names.iter().map(|n| self.class(n, pos)).collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}

#[cfg(test)]
mod test {
    use crate::check::context::{clss, LookupClass};
    use crate::check::context::clss::HasParent;
    use crate::check::name::{Name, TupleCallable};
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn tuple_with_generics_has_collection_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::tuple(&[Name::from("Int"), Name::from("Float")]);
        let tuple = ctx.class(&name, pos).expect("Tuple");

        let collection = StringName::new(clss::COLLECTION, &[Name::from("Int"), Name::from("Float")]);
        assert!(tuple.has_parent(&TrueName::from(&collection), &ctx, pos).expect("is parent"));
        Ok(())
    }

    #[test]
    fn tuple_has_tuple_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::tuple(&[Name::from("Int"), Name::from("Float")]);
        let tuple = ctx.class(&name, pos).expect("Tuple");

        let collection = StringName::new(clss::TUPLE, &[Name::from("Int"), Name::from("Float")]);
        assert!(tuple.has_parent(&TrueName::from(&collection), &ctx, pos).expect("is parent"));
        Ok(())
    }

    #[test]
    fn tuple_not_parent_wrong_types() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::tuple(&[Name::from("Int"), Name::from("Float"), ]);
        let tuple = ctx.class(&name, pos).expect("Tuple");

        let collection = StringName::new(clss::TUPLE, &[Name::from("Int"), Name::from("String")]);
        assert!(!tuple.has_parent(&TrueName::from(&collection), &ctx, pos).expect("is parent"));
        Ok(())
    }

    #[test]
    fn callable_has_callable_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::callable(&[Name::from("Int"), Name::from("Float")],
                                  &Name::from("Float"));
        let callable = ctx.class(&name, pos).expect("Callable");

        let args = Name::from(&StringName::new(clss::TUPLE, &[Name::from("Int"), Name::from("Float")]));
        let collection = StringName::new(clss::CALLABLE, &[args, Name::from("Float")]);
        assert!(callable.has_parent(&TrueName::from(&collection), &ctx, pos).expect("Is Parent"));
        Ok(())
    }

    #[test]
    fn callable_has_callable_parent_tuple_arg() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::callable(&[Name::from("Int"), Name::from("Float")],
                                  &Name::from("Float"));
        let callable = ctx.class(&name, pos).expect("Callable");

        let args = Name::tuple(&[Name::from("Int"), Name::from("Float")]);
        let collection = StringName::new(clss::CALLABLE, &[args, Name::from("Float")]);
        assert!(callable.has_parent(&TrueName::from(&collection), &ctx, pos)?);
        Ok(())
    }

    #[test]
    fn callable_parent_wrong_ret_type() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::callable(&[Name::from("Int"), Name::from("Float")],
                                  &Name::from("Float"));
        let callable = ctx.class(&name, pos).expect("Callable");

        let args = Name::from(&StringName::new(clss::TUPLE, &[Name::from("Int"), Name::from("Float")]));
        let collection = StringName::new(clss::CALLABLE, &[args, Name::from("String")]);
        assert!(!callable.has_parent(&TrueName::from(&collection), &ctx, pos).expect("Is Parent"));
        Ok(())
    }

    #[test]
    fn callable_parent_wrong_arg_type() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::invisible();
        let name = Name::callable(&[Name::from("Int"), Name::from("Float")],
                                  &Name::from("Float"));
        let callable = ctx.class(&name, pos).expect("Callable");

        let args = Name::from(&StringName::new(clss::TUPLE, &[Name::from("String"), Name::from("Float")]));
        let collection = StringName::new(clss::CALLABLE, &[args, Name::from("Float")]);
        assert!(!callable.has_parent(&TrueName::from(&collection), &ctx, pos).expect("Is Parent"));
        Ok(())
    }
}
