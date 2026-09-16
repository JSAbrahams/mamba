use std::collections::HashSet;
use std::convert::TryFrom;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

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

pub fn python_files(
    python_dir: &Path,
) -> TypeResult<(
    HashSet<GenericClass>,
    HashSet<GenericField>,
    HashSet<GenericFunction>,
)> {
    let mut types = HashSet::new();
    let (mut fields, mut functions) = (HashSet::new(), HashSet::new());

    let entries = fs::read_dir(python_dir)
        .map_err(|io_err| TypeErr::new_no_pos(io_err.to_string().as_str()))?;

    for entry in entries {
        let path = entry
            .map_err(|err| TypeErr::new_no_pos(err.to_string().as_str()))?
            .path();
        // A stub directory also holds things that are not Python source: a `__pycache__`
        // directory the moment anything runs the interpreter over it, for one.
        if path.extension().is_none_or(|extension| extension != "py") {
            continue;
        }
        let python_src_path = path
            .as_os_str()
            .to_str()
            .ok_or_else(|| TypeErr::new_no_pos("Unable to build context for python resource"))?;

        let mut python_src = String::new();
        match File::open(python_src_path) {
            Ok(mut path) => path.read_to_string(&mut python_src).map_err(|err| {
                TypeErr::new_no_pos(&format!("Unable to read python file: {err:?}"))
            })?,
            Err(_) => return Err(vec![TypeErr::new_no_pos("primitive does not exist")]),
        };

        for statement in python_stmts(&python_src)? {
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
