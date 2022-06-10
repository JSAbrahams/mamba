pub mod union;
pub mod variant;

#[cfg(test)]
mod test {
    use crate::check::context::{clss, LookupClass};
    use crate::check::context::clss::HasParent;
    use crate::check::name::Name;
    use crate::check::name::name_variant::NameVariant;
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;
    use crate::check::result::TypeResult;
    use crate::common::position::Position;
    use crate::Context;

    #[test]
    fn tuple_has_collection_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::default();
        let tuple = ctx.class(&Name::from(clss::TUPLE), pos)?;

        let collection = StringName::new(clss::COLLECTION, &[]);
        assert!(tuple.has_parent(&TrueName::from(&collection), &ctx, pos)?);
        Ok(())
    }

    #[test]
    fn namevariant_tuple_has_collection_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::default();
        let name = Name::from(&TrueName::from(&NameVariant::Tuple(vec![
            Name::from("Int"),
            Name::from("Float"),
        ])));
        let tuple = ctx.class(&name, pos).expect("Tuple");

        let collection = StringName::new(clss::COLLECTION, &[Name::from("GENERIC"), Name::from("GENERIC")]);
        assert!(tuple.has_parent(&TrueName::from(&collection), &ctx, pos)?);
        Ok(())
    }

    #[test]
    fn namevariant_callabel_has_callable_parent() -> TypeResult<()> {
        let ctx = Context::default().into_with_std_lib()?.into_with_primitives()?;
        let pos = Position::default();
        let name = Name::from(&TrueName::from(&NameVariant::Fun(vec![
            Name::from("Int"),
            Name::from("Float"),
        ], Box::from(Name::from("Float")))));
        let callable = ctx.class(&name, pos).expect("Callable");

        let collection = StringName::new(clss::CALLABLE, &[]);
        assert!(callable.has_parent(&TrueName::from(&collection), &ctx, pos)?);
        Ok(())
    }
}
