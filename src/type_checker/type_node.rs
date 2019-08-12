use std::fmt;
use std::fmt::Display;

use crate::parser::ast::ASTNode;

#[derive(Debug, Clone)]
pub enum Type {
    Empty,

    Int,
    Float,
    String,
    Bool,

    Set { ty: Box<Type> },
    List { ty: Box<Type> },
    Tuple { tys: Vec<Type> },

    Range { ty: Box<Type> },
    AnonFun { args: Vec<Type>, out: Box<Type> },

    Custom { lit: String, gens: Vec<Type> },

    Maybe { ty: Box<Type> },
    Mutable { ty: Box<Type> },

    NA,
    Any
}

impl Type {
    fn copy_as_mutable(ty: &Type) -> Type { Type::Mutable { ty: Box::from(ty.clone()) } }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.clone() {
            Type::Empty => String::from("Empty"),
            Type::Int => String::from("Int"),
            Type::Float => String::from("Float"),
            Type::String => String::from("String"),
            Type::Bool => String::from("Bool"),

            Type::Set { ty } => format!("Set<{}>", ty),
            Type::List { ty } => format!("List<{}>", ty),
            Type::Tuple { tys } => format!("({})", comma_separated(tys)),

            Type::Range { ty } => format!("{}..{}", ty, ty),
            Type::AnonFun { args, out } => format!("{} => {}", comma_separated(args), out),

            Type::Custom { lit, gens } => format!("{}<{}>", lit, comma_separated(gens)),
            Type::Maybe { ty } => format!("{}?", ty),
            Type::Mutable { ty } => format!("mut {}", ty),

            Type::NA => String::new(),
            Type::Any => String::from("Any")
        })
    }
}

fn comma_separated(types: Vec<Type>) -> String {
    let mut res = String::new();
    for ty in types {
        res.push_str(format!("{}", ty).as_ref());
        res.push(',');
    }

    if !res.is_empty() {
        res.remove(res.len() - 1);
    }
    res
}

impl Type {
    pub fn try_from_node(node: ASTNode) -> Result<Self, String> {
        match node {
            ASTNode::TypeDef { .. } | ASTNode::TypeAlias { .. } => Ok(Type::NA),
            ASTNode::TypeTup { types } => {
                let types: Result<Vec<Type>, String> = types
                    .iter()
                    .map(|node_pos| Type::try_from_node(node_pos.clone().node))
                    .collect();
                Ok(Type::Tuple { tys: types? })
            }
            ASTNode::Type { id, generics } => {
                let id: Result<Type, String> = from_id(id.node);
                let generics: Result<Vec<Type>, String> =
                    generics.iter().map(|node_pos| from_id(node_pos.clone().node)).collect();

                match (id?, generics.clone()?.first()) {
                    (Type::String, None) => Ok(Type::String),
                    (Type::Int, None) => Ok(Type::Int),
                    (Type::Float, None) => Ok(Type::Float),
                    (Type::Bool, None) => Ok(Type::Bool),
                    (Type::Any, None) => Ok(Type::Any),
                    (Type::Custom { lit, .. }, first) => match (lit.as_ref(), first) {
                        ("List", Some(ty)) => Ok(Type::List { ty: Box::from(ty.clone()) }),
                        ("Set", Some(ty)) => Ok(Type::Set { ty: Box::from(ty.clone()) }),
                        ("List", None) => Err(String::from("List cannot have more than one type")),
                        ("Set", None) => Err(String::from("Set cannot have more than one type")),
                        _ => Ok(Type::Custom { lit, gens: generics? })
                    },
                    (other, _) => Err(format!("Type {} cannot have generics", other))
                }
            }
            _ => Ok(Type::NA)
        }
    }
}

fn from_id(node: ASTNode) -> Result<Type, String> {
    match node {
        ASTNode::Id { lit } => Ok(match lit.as_ref() {
            "String" => Type::String,
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "Any" => Type::Any,
            other => Type::Custom { lit: String::from(other), gens: vec![] }
        }),
        other => Err(format!("Expected type but got {:?}", other))
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (.., Type::Any) | (Type::Any, ..) => true,
            (.., Type::NA) | (Type::NA, ..) => false,

            (Type::Empty, Type::Empty)
            | (Type::Float, Type::Float)
            | (Type::Int, Type::Int)
            | (Type::Bool, Type::Bool)
            | (Type::String, Type::String) => true,

            (Type::Set { ty: ty_self }, Type::Set { ty: ty_other }) => ty_self == ty_other,
            (Type::List { ty: ty_self }, Type::List { ty: ty_other }) => ty_self == ty_other,
            (Type::Tuple { tys: ty_self }, Type::Tuple { tys: ty_other }) =>
                ty_self.len() == ty_other.len()
                    && ty_self.iter().zip(ty_other).all(|(left, right)| left == right),

            (Type::Range { ty: ty_self }, Type::Range { ty: ty_other }) => ty_self == ty_other,
            (
                Type::AnonFun { args: arg_self, out: out_self },
                Type::AnonFun { args: arg_other, out: out_other }
            ) =>
                out_self == out_other
                    && arg_self.len() == arg_other.len()
                    && arg_self.iter().zip(arg_other).all(|(left, right)| left == right),

            (
                Type::Custom { lit: lit_self, gens: gens_self },
                Type::Custom { lit: lit_other, gens: gens_other }
            ) => lit_self == lit_other && gens_self == gens_other,

            (Type::Maybe { ty: ty_self }, Type::Maybe { ty: ty_other }) => ty_self == ty_other,
            (Type::Mutable { ty: ty_self }, Type::Mutable { ty: ty_other }) => ty_self == ty_other,

            _ => false
        }
    }
}
