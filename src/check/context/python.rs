use std::collections::HashSet;
use std::convert::TryFrom;

use ruff_python_ast::{PythonVersion, Stmt};
use ruff_python_parser::{Mode, ParseOptions};

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::{GenericField, GenericFields};
use crate::check::context::function::generic::GenericFunction;
use crate::check::result::{TypeErr, TypeResult};

/// The Python version the stub files, and the generated output, are parsed as.
///
/// Must match the interpreter Devbox pins, and the one `tests_util::PYTHON` shells out to.
pub const PYTHON_VERSION: PythonVersion = PythonVersion::PY314;

/// Parse Python source into its top-level statements.
///
/// Unlike its predecessor this rejects source it cannot parse, instead of silently truncating the
/// statement list at the first thing it does not understand.
pub fn python_stmts(python_src: &str) -> TypeResult<Vec<Stmt>> {
    let options = ParseOptions::from(Mode::Module).with_target_version(PYTHON_VERSION);
    let parsed = ruff_python_parser::parse(python_src, options)
        .map_err(|err| TypeErr::new_no_pos(&format!("Unable to parse python file: {err}")))?;

    let module = parsed
        .try_into_module()
        .ok_or_else(|| vec![TypeErr::new_no_pos("Python source is not a module")])?;
    Ok(module.into_suite().into_iter().collect())
}

/// Collect the classes, fields and functions of the given Python stubs.
///
/// Each stub is a file name, used only to tell them apart, and its source.
pub fn python_files(
    stubs: &[(&str, &str)],
) -> TypeResult<(
    HashSet<GenericClass>,
    HashSet<GenericField>,
    HashSet<GenericFunction>,
)> {
    let mut types = HashSet::new();
    let (mut fields, mut functions) = (HashSet::new(), HashSet::new());

    for (_, python_src) in stubs {
        for statement in python_stmts(python_src)? {
            match &statement {
                Stmt::Assign(assign) => GenericFields::from((assign.targets.as_slice(), None))
                    .fields
                    .into_iter()
                    .for_each(|field| {
                        fields.insert(field);
                    }),
                Stmt::AnnAssign(assign) => GenericFields::from((
                    std::slice::from_ref(assign.target.as_ref()),
                    Some(assign.annotation.as_ref()),
                ))
                .fields
                .into_iter()
                .for_each(|field| {
                    fields.insert(field);
                }),
                Stmt::FunctionDef(func_def) => {
                    functions.insert(GenericFunction::from(func_def));
                }
                Stmt::ClassDef(class_def) => {
                    types.insert(GenericClass::try_from(class_def)?);
                }
                _ => {}
            }
        }
    }

    Ok((types, fields, functions))
}
