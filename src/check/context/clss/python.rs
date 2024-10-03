use std::collections::HashSet;
use std::convert::TryFrom;
use std::ops::Deref;

use python_parser::ast::{Classdef, CompoundStatement, Statement};

use crate::check::context::clss;
use crate::check::context::clss::generic::GenericClass;
use crate::check::context::field::generic::GenericFields;
use crate::check::context::function::generic::GenericFunction;
use crate::check::context::function::python::INIT;
use crate::check::context::parameter::python::GenericParameters;
use crate::check::context::parent::generic::GenericParent;
use crate::check::name::string_name::StringName;
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
pub const DICT: &str = "dict";

pub const TUPLE: &str = "Tuple";
pub const CALLABLE: &str = "Callable";
pub const UNION: &str = "Union";
pub const ANY: &str = "Any";

pub const NONE: &str = "None";
pub const EXCEPTION: &str = "Exception";

/// Create a [GenericClass] from [ClassDef].
///
/// - Init is removed from function list, it is the built-in constructor
impl TryFrom<&Classdef> for GenericClass {
    type Error = Vec<TypeErr>;

    fn try_from(class_def: &Classdef) -> TypeResult<GenericClass> {
        let (mut functions, mut fields) = (HashSet::new(), HashSet::new());
        let generics = GenericParameters::from(&class_def.arguments).parameters;

        class_def.code.iter().for_each(|statement| match statement {
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
        });

        let generic_names: Vec<Name> = generics.iter().map(|g| Name::from(&g.name)).collect();
        let class = StringName::new(python_to_concrete(&class_def.name).as_str(), &generic_names);
        let functions: Vec<GenericFunction> = functions
            .into_iter()
            .map(|f| f.in_class(Some(&class), false, Position::invisible()))
            .collect::<Result<_, _>>()?;
        let args = functions
            .iter()
            .find(|f| f.name == StringName::from(INIT))
            .map_or(vec![], |f| f.arguments.clone());

        Ok(GenericClass {
            is_py_type: true,
            name: class.clone(),
            pos: Position::invisible(),
            concrete: false,
            args,
            fields: fields
                .into_iter()
                .flat_map(|f| f.in_class(Some(&class.clone()), false, Position::invisible()))
                .collect(),
            functions: functions
                .into_iter()
                .filter(|f| f.name != StringName::from(INIT))
                .map(|f| f.in_class(Some(&class), false, Position::invisible()))
                .filter_map(Result::ok)
                .collect(),
            parents: class_def
                .arguments
                .iter()
                .map(GenericParent::from)
                .filter(|parent| StringName::from(&parent.name).name != "Generic")
                .collect(),
        })
    }
}

pub fn python_to_concrete(name: &str) -> String {
    match name {
        INT_PRIMITIVE => String::from(clss::INT),
        FLOAT_PRIMITIVE => String::from(clss::FLOAT),
        STRING_PRIMITIVE => String::from(clss::STRING),
        BOOL_PRIMITIVE => String::from(clss::BOOL),
        ENUM_PRIMITIVE => String::from(clss::ENUM),
        COMPLEX_PRIMITIVE => String::from(clss::COMPLEX),

        COLLECTION => String::from(clss::COLLECTION),
        RANGE => String::from(clss::RANGE),
        SLICE => String::from(clss::SLICE),
        SET => String::from(clss::SET),
        LIST => String::from(clss::LIST),
        TUPLE => String::from(clss::TUPLE),
        DICT => String::from(clss::DICT),

        UNION => String::from(clss::UNION),
        CALLABLE => String::from(clss::CALLABLE),
        NONE => String::from(clss::NONE),
        EXCEPTION => String::from(clss::EXCEPTION),
        ANY => String::from(clss::ANY),

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
    use crate::check::name::string_name::StringName;
    use crate::check::name::true_name::TrueName;
    use crate::check::name::{Empty, Name};

    fn class_def(stmt: &Statement) -> Classdef {
        match &stmt {
            Statement::Compound(compound) => match compound.deref() {
                CompoundStatement::Classdef(classdef) => classdef.clone(),
                other => panic!("Not class def but {:?}", other),
            },
            other => panic!("Not compound statement but {:?}", other),
        }
    }

    #[test] // # See 317, #318, and #319 for why variables are after constructor
    fn from_py_fields() {
        let source = "class MyClass:\n    def __init__(self): pass\n    b: int = 10\n    a: int\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = class_def(&first);
        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, StringName::from("MyClass"));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.fields.len(), 2);
        let mut fields = generic_class.fields.iter().sorted_by_key(|f| f.name.clone()).into_iter();

        let field = fields.next().expect("field");
        assert_eq!(field.name, String::from("a"));
        assert!(field.is_py_type);
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert!(field.mutable);
        assert_eq!(field.ty, None); // See #318

        let field = fields.next().expect("field");
        assert_eq!(field.name, String::from("b"));
        assert!(field.is_py_type);
        assert_eq!(field.in_class, Some(StringName::from("MyClass")));
        assert!(field.mutable);
        assert_eq!(field.ty, Some(Name::from("Int")));
    }

    #[test]
    #[ignore] // See #311
    fn from_py_fields_in_init() {
        let source =
            "class MyClass:\n    def __init__(self, a: int): self.a=a\n    def g(x: bool): pass\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = class_def(&first);
        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, StringName::from("MyClass"));
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
        let class_def: Classdef = class_def(&first);
        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, StringName::from("MyClass"));
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
        let class_def: Classdef = class_def(&first);
        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        assert_eq!(generic_class.name, StringName::from("MyClass"));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.parents.len(), 2);
        let mut iter =
            generic_class.parents.iter().sorted_by_key(|p| p.name.variant.clone()).into_iter();

        let parent2 = iter.next().expect("parent in class");
        assert_eq!(parent2.name, TrueName::from("P2"));
        assert!(parent2.is_py_type);

        let parent = iter.next().expect("parent in class");
        assert_eq!(parent.name, TrueName::from("ParentClass"));
        assert!(parent.is_py_type);
    }

    #[test]
    fn from_class_with_generic() {
        let source = "class MyClass(Generic[T], P2):\n    pass\n";
        let (_, statements) =
            python_parser::file_input(python_parser::make_strspan(&source)).expect("parse source");

        let first = statements.first().expect("non empty statements");
        let class_def: Classdef = class_def(&first);
        let generic_class = GenericClass::try_from(&class_def).expect("generic class");

        let name = StringName::new("MyClass", &[Name::from("T")]);
        assert_eq!(generic_class.name, StringName::from(name));
        assert!(generic_class.is_py_type);

        assert_eq!(generic_class.parents.len(), 1);
        let parent = generic_class.parents.iter().next().expect("parent in class");
        assert_eq!(parent.name, TrueName::from("P2"));
        assert!(parent.is_py_type);
    }
}
