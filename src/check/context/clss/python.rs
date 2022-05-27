use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericFields;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::INIT;
use crate::check::context::parameter::python::GenericParameters;
use crate::check::context::parent::generic::GenericParent;
use crate::check::context::{clss, function};
use crate::check::name::stringname::StringName;
use crate::check::name::truename::TrueName;
use crate::check::name::Name;
use crate::check::result::{TypeErr, TypeResult};
use crate::common::position::Position;

pub const INT_PRIMITIVE: &str = "int";
pub const FLOAT_PRIMITIVE: &str = "float";
pub const STRING_PRIMITIVE: &str = "str";
pub const BOOL_PRIMITIVE: &str = "bool";
pub const ENUM_PRIMITIVE: &str = "enum";
pub const COMPLEX_PRIMITIVE: &str = "complex";

pub const COLLECTION: &str = "collection";
pub const RANGE: &str = "range";
pub const SLICE: &str = "slice";
pub const SET: &str = "set";
pub const LIST: &str = "list";

pub const NONE: &str = "None";
pub const EXCEPTION: &str = "Exception";

/// Create a [GenericClass] from [ClassDef].
///
/// - Init is removed from function list, it is the built-in constructor
impl TryFrom<&Classdef> for GenericClass {
    type Error = Vec<TypeErr>;

    fn try_from(class_def: &Classdef) -> TypeResult<GenericClass> {
        let mut functions = HashSet::new();
        let mut fields = HashSet::new();
        let generics = GenericParameters::from(&class_def.arguments).parameters;

        for statement in &class_def.code {
            match statement {
                Statement::Assignment(variables, _) => {
                    let gen_fields = GenericFields::from((variables, &None)).fields;
                    fields = fields.union(&gen_fields).cloned().collect();
                }
                Statement::TypedAssignment(variables, ty, _) => {
                    let gen_fields = GenericFields::from((variables, &Some(ty.clone()))).fields;
                    fields = fields.union(&gen_fields).cloned().collect();
                }
                Statement::Compound(compound) => {
                    if let CompoundStatement::Funcdef(func_def) = compound.deref() {
                        functions.insert(GenericFunction::from(func_def));
                    }
                }
                _ => {}
            }
        }

        let generic_names: Vec<Name> = generics.iter().map(|g| Name::from(&g.name)).collect();
        let class = TrueName::new(python_to_concrete(&class_def.name).as_str(), &generic_names);
        let functions: Vec<GenericFunction> = functions
            .into_iter()
            .map(|f| f.in_class(Some(&class), false, &Position::default()))
            .collect::<Result<_, _>>()?;
        let args = functions
            .iter()
            .find(|f| f.name == StringName::from(function::INIT))
            .map_or(vec![], |f| f.arguments.clone());

        Ok(GenericClass {
            is_py_type: true,
            name: class.clone(),
            pos: Position::default(),
            concrete: false,
            args,
            fields: fields
                .into_iter()
                .flat_map(|f| f.in_class(Some(&class.clone()), false, &Position::default()))
                .collect(),
            functions: functions
                .into_iter()
                .filter(|f| f.name != StringName::from(INIT))
                .map(|f| f.in_class(Some(&class), false, &Position::default()))
                .filter_map(Result::ok)
                .collect(),
            parents: class_def.arguments.iter().map(GenericParent::from).collect(),
        })
    }
}

pub fn python_to_concrete(name: &str) -> String {
    match name {
        INT_PRIMITIVE => String::from(clss::INT_PRIMITIVE),
        FLOAT_PRIMITIVE => String::from(clss::FLOAT_PRIMITIVE),
        STRING_PRIMITIVE => String::from(clss::STRING_PRIMITIVE),
        BOOL_PRIMITIVE => String::from(clss::BOOL_PRIMITIVE),
        ENUM_PRIMITIVE => String::from(clss::ENUM_PRIMITIVE),
        COMPLEX_PRIMITIVE => String::from(clss::COMPLEX_PRIMITIVE),

        RANGE => String::from(clss::RANGE),
        SLICE => String::from(clss::SLICE),
        SET => String::from(clss::SET),
        LIST => String::from(clss::LIST),

        NONE => String::from(clss::NONE),
        EXCEPTION => String::from(clss::EXCEPTION),

        other => String::from(other),
    }
}

#[cfg(test)]
mod test {
    use std::convert::TryFrom;
    use std::ops::Deref;

    use itertools::Itertools;
    use python_parser::ast::{Classdef, CompoundStatement, Statement};

    use crate::check::context::clss::generic::GenericClass;
    use crate::check::name::namevariant::NameVariant;
    use crate::check::name::stringname::StringName;
    use crate::check::name::truename::TrueName;
    use crate::check::name::Name;

    #[test]
    #[ignore] // See #311
    fn from_py_fields_in_init() {
        let source =
            "class MyClass:\n    def __init__(self, a: int): self.a=a\n    def g(x: bool): pass\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = match &first {
            Statement::Compound(compound) => match compound.deref() {
                CompoundStatement::Classdef(classdef) => classdef.clone(),
                other => panic!("Not class def but {:?}", other),
            },
            other => panic!("Not compound statement but {:?}", other),
        };

        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, TrueName::from("MyClass"));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.fields.len(), 1);
        let field = generic_class.fields.iter().next().expect("field in class");
        assert_eq!(field.name, String::from("a"));
        assert!(field.is_py_type);
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert!(field.mutable);
        assert_eq!(field.ty, Some(Name::from("Int")));
    }

    #[test]
    fn from_py_functions() {
        let source =
            "class MyClass:\n    def __init__(self, a: int): self.a=a\n    def g(x: bool): pass\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = match &first {
            Statement::Compound(compound) => match compound.deref() {
                CompoundStatement::Classdef(classdef) => classdef.clone(),
                other => panic!("Not class def but {:?}", other),
            },
            other => panic!("Not compound statement but {:?}", other),
        };

        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, TrueName::from("MyClass"));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.functions.len(), 1);
        let function = generic_class.functions.iter().next().expect("function in class");
        assert_eq!(function.name, StringName::from("g"));
        assert_eq!(function.in_class, Some(StringName::from("MyClass")));
        assert!(function.is_py_type);
        assert!(!function.pure);
        assert_eq!(function.raises, Name::empty());

        assert_eq!(function.arguments.len(), 1);
        let argument = function.arguments.iter().next().expect("function argument");
        assert_eq!(argument.name, String::from("x"));
        assert_eq!(argument.ty, Some(Name::from("Bool")));
        assert!(!argument.has_default);
    }

    #[test]
    fn from_py_parents() {
        let source = "class MyClass(ParentClass, P2):\n    pass\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = match &first {
            Statement::Compound(compound) => match compound.deref() {
                CompoundStatement::Classdef(classdef) => classdef.clone(),
                other => panic!("Not class def but {:?}", other),
            },
            other => panic!("Not compound statement but {:?}", other),
        };

        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, TrueName::from("MyClass"));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.parents.len(), 2);
        let mut iter = generic_class
            .parents
            .iter()
            .sorted_by_key(|p| match &p.name.variant {
                NameVariant::Single(string_name) => string_name.name.clone(),
                other => panic!("Expected Single, was {:?}", other),
            })
            .into_iter();

        let parent2 = iter.next().expect("parent in class");
        assert_eq!(parent2.name, TrueName::from("P2"));
        assert!(parent2.is_py_type);

        let parent = iter.next().expect("parent in class");
        assert_eq!(parent.name, TrueName::from("ParentClass"));
        assert!(parent.is_py_type);
    }
}
