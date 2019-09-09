use crate::type_checker::context::class::Type;
use crate::type_checker::type_result::TypeErr;
use crate::type_checker::CheckInput;

pub type InferResult<T = Option<Type>> = std::result::Result<T, Vec<TypeErr>>;

pub fn infer(_: &[CheckInput]) -> InferResult { Ok(None) }
