use std::collections::HashSet;
use std::fmt::{Display, Formatter};
use std::hash::{Hash, Hasher};

use crate::{Context, TypeErr};
use crate::check::context::arg::FunctionArg;
use crate::check::context::clss::{GetField, GetFun, HasParent};
use crate::check::context::clss::concrete::variant::ClassVariant;
use crate::check::context::field::concrete::union::FieldUnion;
use crate::check::context::function::concrete::union::FunUnion;
use crate::check::context::LookupClass;
use crate::check::name::{Empty, Name};
use crate::check::name::name_variant::NameVariant;
use crate::check::name::string_name::StringName;
use crate::check::name::true_name::TrueName;
use crate::check::result::TypeResult;
use crate::common::delimit::comma_delm;
use crate::common::position::Position;

#[derive(Debug, Eq)]
pub struct ClassUnion {
    union: HashSet<ClassVariant>,
}

impl From<&ClassUnion> for Name {
    fn from(class_union: &ClassUnion) -> Self {
        let names: HashSet<Name> = class_union
            .union
            .iter()
            .map(|u| match u {
                ClassVariant::Direct(set) => {
                    let names: HashSet<StringName> = set.iter().map(|c| c.name.clone()).collect();
                    let names: HashSet<Name> = names.iter().map(Name::from).collect();
                    Name::from(&names)
                }
                ClassVariant::Tuple(classes) => {
                    let tuple = NameVariant::Tuple(classes.iter().map(Name::from).collect());
                    Name::from(&tuple)
                }
                ClassVariant::Fun(args, ret) => {
                    let args = args.iter().map(Name::from).collect();
                    Name::from(&NameVariant::Fun(args, Box::from(Name::from(ret))))
                }
            })
            .collect();

        Name::from(&names)
    }
}

impl ClassUnion {
    pub fn constructor(&self, pos: Position) -> TypeResult<HashSet<Vec<FunctionArg>>> {
        let mut fun_args: HashSet<Vec<FunctionArg>> = HashSet::new();

        let res: Vec<HashSet<Vec<FunctionArg>>> = self
            .union
            .iter()
            .map(|c| match c {
                ClassVariant::Direct(set) => {
                    Ok(set.iter().map(|c| c.args.clone()).collect())
                }
                other => {
                    let msg = format!("'{}' cannot have a constructor", other);
                    Err(vec![TypeErr::new(pos, &msg)])
                }
            })
            .collect::<Result<Vec<HashSet<Vec<FunctionArg>>>, _>>()?;

        res.iter().for_each(|set| {
            set.iter().for_each(|args| {
                fun_args.insert(args.clone());
            })
        });

        Ok(fun_args)
    }
}

impl GetField<FieldUnion> for ClassUnion {
    fn field(&self, name: &str, ctx: &Context, pos: Position) -> TypeResult<FieldUnion> {
        let union: HashSet<FieldUnion> =
            self.union.iter().map(|c| c.field(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define attribute '{}'", Name::from(self), name);
            return Err(vec![TypeErr::new(pos, &msg)]);
        }

        Ok(FieldUnion::from(&union))
    }
}

impl GetFun<FunUnion> for ClassUnion {
    /// Check if ClassUnion implements a function.
    fn fun(&self, name: &StringName, ctx: &Context, pos: Position) -> TypeResult<FunUnion> {
        let union: HashSet<FunUnion> =
            self.union.iter().map(|c| c.fun(name, ctx, pos)).collect::<Result<_, _>>()?;

        if union.is_empty() {
            let msg = format!("'{}' does not define function '{}'", Name::from(self), name);
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
            return Err(vec![TypeErr::new(pos, &format!("Unexpected '{}'", name))]);
        }

        let union = name.names.iter().map(|n| self.class(n, pos)).collect::<Result<_, _>>()?;
        Ok(ClassUnion { union })
    }
}
