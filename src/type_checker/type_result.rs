use crate::type_checker::type_node::Type;

// TODO create type error (which could become rather complex but we'll see)
pub type TypeResult<T = Type> = std::result::Result<T, String>;
