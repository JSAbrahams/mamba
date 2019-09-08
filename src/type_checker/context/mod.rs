use crate::parser::ast::ASTNode;
use crate::type_checker::CheckInput;
use crate::type_checker::context::class::Type;
use crate::type_checker::context::field::Field;
use crate::type_checker::context::function::Function;
use crate::type_checker::context::type_name::TypeName;
use crate::type_checker::type_result::{TypeErr, TypeResult};

pub mod class;
pub mod field;
pub mod function;
pub mod type_name;

mod common;

/// A context stores all information of all identified types of the current
/// application.
///
/// Functions and fields are also stored alongside identified classes such that
/// we can also check usage of top-level fields and functions.
#[derive(Debug)]
pub struct Context {
    types: Vec<Type>,
    functions: Vec<Function>,
    fields: Vec<Field>,
}

pub trait ReturnType {
    /// Set return type.
    ///
    /// Mainly for use during the type inference stage.
    ///
    /// # Failures
    ///
    /// If function already has a return type, and the given type is not equal
    /// to said type, a [TypeErr](crate::type_checker::type_result::TypeErr)
    /// is returned.
    fn with_return_type_name(self, ty: TypeName) -> Result<Self, TypeErr>
        where
            Self: Sized;

    /// Get return type.
    ///
    /// This function should be used after the type inference stage.
    ///
    /// # Failures
    ///
    /// Fail if there is no return type. This can happen if either there was no
    /// return type in the signature, or the return type was not set during
    /// the type inference stage (unable to derive return type).
    fn get_return_type_name(&self) -> Result<TypeName, TypeErr>;
}

pub fn build_context(files: &[CheckInput]) -> TypeResult<Context> {
    let mut errs: Vec<TypeErr> = vec![];
    let mut type_res: Vec<Result<Type, Vec<TypeErr>>> = vec![];
    let mut fun_res: Vec<Result<Function, TypeErr>> = vec![];
    let mut field_res: Vec<Result<Field, TypeErr>> = vec![];

    files.iter().for_each(|(file, source, path)| match &file.node {
        ASTNode::File { pure, modules, .. } =>
            modules.iter().for_each(|module| match &module.node {
                ASTNode::Class { .. } | ASTNode::TypeDef { .. } =>
                    type_res.push(Type::try_from_node_pos(module, *pure).map_err(|errs| {
                        errs.into_iter()
                            .map(|err| err.into_with_source(source.clone(), path))
                            .collect()
                    })),
                other => if let ASTNode::Script { statements } = other {
                    statements.iter().for_each(|statement| match &statement.node {
                        ASTNode::FunDef { .. } => fun_res.push(
                            Function::try_from_node_pos(statement, *pure, false)
                                .map_err(|err| err.into_with_source(source.clone(), path))
                        ),
                        ASTNode::VariableDef { .. } => field_res.push(
                            Field::try_from_node_pos(statement)
                                .map_err(|err| err.into_with_source(source.clone(), path))
                        ),
                        _ => {}
                    })
                } else { {} }
            }),
        _ => errs.push(TypeErr::new(&file.position, "Expected file"))
    });

    let (types, type_errs): (Vec<_>, Vec<_>) = type_res.into_iter().partition(Result::is_ok);
    let (functions, fun_errs): (Vec<_>, Vec<_>) = fun_res.into_iter().partition(Result::is_ok);
    let (fields, field_errs): (Vec<_>, Vec<_>) = field_res.into_iter().partition(Result::is_ok);

    if !errs.is_empty() || !type_errs.is_empty() || !fun_errs.is_empty() || !field_errs.is_empty() {
        errs.append(&mut type_errs.into_iter().map(Result::unwrap_err).flatten().collect());
        errs.append(&mut fun_errs.into_iter().map(Result::unwrap_err).collect());
        errs.append(&mut field_errs.into_iter().map(Result::unwrap_err).collect());
        Err(errs)
    } else {
        Ok(Context {
            types: types.into_iter().map(Result::unwrap).collect(),
            functions: functions.into_iter().map(Result::unwrap).collect(),
            fields: fields.into_iter().map(Result::unwrap).collect(),
        })
    }
}
