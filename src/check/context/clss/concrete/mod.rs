pub mod union;
pub mod variant;

#[cfg(test)]
mod test {
    use crate::check::context::{clss, LookupClass};
    use crate::check::context::clss::HasParent;
    use crate::check::name::Name;
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

        let collection = StringName::new(clss::COLLECTION, &[Name::from("GENERIC")]);
        assert!(tuple.has_parent(&TrueName::from(&collection), &ctx, pos)?);
        Ok(())
    }
}
