use crate::type_checker::node_type::Type;

pub type TypeResult<T = Type> = std::result::Result<T, String>;
