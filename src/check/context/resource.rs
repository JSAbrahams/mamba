use std::collections::HashSet;

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericField;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::python::python_files;
use crate::check::context::Context;
use crate::check::result::TypeResult;

impl Context {
    /// Loads pre-defined Python primitives into Context.
    pub fn into_with_primitives(self) -> TypeResult<Self> {
        let (py_types, py_fields, py_functions) = python_files(PRIMITIVE)?;

        let classes: HashSet<GenericClass> = self.classes.union(&py_types).cloned().collect();
        let functions: HashSet<GenericFunction> =
            self.functions.union(&py_functions).cloned().collect();
        let fields: HashSet<GenericField> = self.fields.union(&py_fields).cloned().collect();

        let (py_types, py_fields, py_functions) = python_files(STD)?;

        let classes: HashSet<GenericClass> = classes.union(&py_types).cloned().collect();
        let functions: HashSet<GenericFunction> = functions.union(&py_functions).cloned().collect();
        let fields: HashSet<GenericField> = fields.union(&py_fields).cloned().collect();

        Ok(Context {
            classes,
            functions,
            fields,
        })
    }

    /// Loads pre-defined Python standard library into Context.
    pub fn into_with_std_lib(self) -> TypeResult<Self> {
        let (py_types, py_fields, py_functions) = python_files(STD)?;

        let types = self.classes.union(&py_types).cloned().collect();
        let functions = self.functions.union(&py_functions).cloned().collect();
        let fields = self.fields.union(&py_fields).cloned().collect();
        Ok(Context {
            classes: types,
            functions,
            fields,
        })
    }
}

/// The Python stubs of the primitives, embedded in the binary so that checking needs no filesystem.
///
/// Each entry is a file name and its source.
const PRIMITIVE: &[(&str, &str)] = &[
    ("bool.py", include_str!("../resource/primitive/bool.py")),
    (
        "complex.py",
        include_str!("../resource/primitive/complex.py"),
    ),
    ("float.py", include_str!("../resource/primitive/float.py")),
    ("int.py", include_str!("../resource/primitive/int.py")),
    ("string.py", include_str!("../resource/primitive/string.py")),
];

/// The Python stubs of the standard library, embedded like [PRIMITIVE].
///
/// The extensionless `std/input` file is left out, as it always was: only `.py` files are stubs.
const STD: &[(&str, &str)] = &[
    ("builtins.py", include_str!("../resource/std/builtins.py")),
    ("callable.py", include_str!("../resource/std/callable.py")),
    (
        "collection.py",
        include_str!("../resource/std/collection.py"),
    ),
    ("divmod.py", include_str!("../resource/std/divmod.py")),
    ("exception.py", include_str!("../resource/std/exception.py")),
    ("input.py", include_str!("../resource/std/input.py")),
    ("optional.py", include_str!("../resource/std/optional.py")),
    ("range.py", include_str!("../resource/std/range.py")),
    ("typing.py", include_str!("../resource/std/typing.py")),
];

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{PRIMITIVE, STD};

    /// A stub added to a resource directory must also be added to its table, or it is silently ignored.
    #[test_case::test_case("primitive", PRIMITIVE)]
    #[test_case::test_case("std", STD)]
    fn table_lists_every_stub(dir: &str, table: &[(&str, &str)]) {
        let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("check")
            .join("resource")
            .join(dir);
        let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|extension| extension == "py"))
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        on_disk.sort();

        let mut listed: Vec<String> = table.iter().map(|(name, _)| name.to_string()).collect();
        listed.sort();
        assert_eq!(listed, on_disk);
    }
}
