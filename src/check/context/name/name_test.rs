// TODO integrate into name restructuring

#[cfg(test)]
mod tests {
    use crate::check::context::clss::{BOOL_PRIMITIVE, FLOAT_PRIMITIVE, INT_PRIMITIVE, STRING_PRIMITIVE};
    use crate::check::context::Context;
    use crate::check::context::name::{IsSuperSet, NameUnion};
    use crate::check::context::name::Name;
    use crate::common::position::Position;

    #[test]
    fn test_is_superset_numbers() {
        let names = vec![
            Name::from(BOOL_PRIMITIVE),
            Name::from(STRING_PRIMITIVE),
            Name::from(INT_PRIMITIVE),
            Name::from(FLOAT_PRIMITIVE)];
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(union_1.is_superset_of(&union_2, &ctx, &Position::default()).unwrap())
    }

    #[test]
    fn test_is_superset_does_not_contain() {
        let names = vec![
            Name::from(BOOL_PRIMITIVE),
            Name::from(STRING_PRIMITIVE)];
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_1.is_superset_of(&union_2, &ctx, &Position::default()).unwrap())
    }

    #[test]
    fn test_superset_wrong_way() {
        let names = vec![
            Name::from(BOOL_PRIMITIVE),
            Name::from(STRING_PRIMITIVE),
            Name::from(INT_PRIMITIVE),
            Name::from(FLOAT_PRIMITIVE)];
        let union_1 = NameUnion::new(&names);
        let union_2 = NameUnion::from(INT_PRIMITIVE);

        let ctx = Context::default().into_with_primitives().unwrap();
        assert!(!union_2.is_superset_of(&union_1, &ctx, &Position::default()).unwrap())
    }
}
