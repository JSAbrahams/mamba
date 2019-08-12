use std::fmt;
use std::fmt::Display;

use crate::parser::ast::ASTNode;

#[derive(Debug, Clone)]
pub struct Type {
    pub optional: bool,
    pub mutable:  bool,
    pub ty:       Ty
}

impl Type {
    pub fn new(ty: &Ty) -> Type { Type { optional: false, mutable: false, ty: ty.clone() } }

    pub fn try_from_type(node: ASTNode) -> Result<Self, String> {
        Ok(Type { optional: false, mutable: false, ty: Ty::try_from_type(node)? })
    }
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool { self.optional == other.optional && self.ty == other.ty }
}

impl Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}{}", self.ty, if self.optional { "?" } else { "" })
    }
}

#[derive(Debug, Clone)]
pub enum Ty {
    Empty,
    Any,

    Custom { lit: String, gens: Vec<Type> },

    Int,
    Float,
    String,
    Bool,

    Set { ty: Box<Type> },
    List { ty: Box<Type> },
    Tuple { tys: Vec<Type> },

    Range { ty: Box<Type> },
    AnonFun { args: Vec<Type>, out: Box<Type> },

    NA
}

impl Display for Ty {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", match self.clone() {
            Ty::Empty => String::from("Empty"),
            Ty::Any => String::from("Any"),

            Ty::Custom { lit, gens } => format!("{}<{}>", lit, comma_separated(gens)),

            Ty::Int => String::from("Int"),
            Ty::Float => String::from("Float"),
            Ty::String => String::from("String"),
            Ty::Bool => String::from("Bool"),

            Ty::Set { ty } => format!("Set<{}>", ty),
            Ty::List { ty } => format!("List<{}>", ty),
            Ty::Tuple { tys } => format!("({})", comma_separated(tys)),

            Ty::Range { ty } => format!("{}..{}", ty, ty),
            Ty::AnonFun { args, out } => format!("{} => {}", comma_separated(args), out),

            Ty::NA => String::new()
        })
    }
}

fn comma_separated(types: Vec<Type>) -> String {
    let mut res = String::new();
    for ty in types {
        res.push_str(format!("{}", ty.ty).as_ref());
        res.push(',');
        res.push(' ');
    }

    if !res.is_empty() {
        res.remove(res.len() - 2);
    }
    res
}

impl Ty {
    pub fn try_from_type(node: ASTNode) -> Result<Self, String> {
        match node {
            ASTNode::TypeDef { .. } | ASTNode::TypeAlias { .. } => Ok(Ty::NA),
            ASTNode::TypeTup { types } => {
                let types: Result<Vec<Type>, String> = types
                    .iter()
                    .map(|node_pos| Type::try_from_type(node_pos.clone().node))
                    .collect();
                Ok(Ty::Tuple { tys: types? })
            }
            ASTNode::Type { id, generics } => {
                let id: Result<Type, String> = from_id(id.node);
                let generics: Result<Vec<Type>, String> =
                    generics.iter().map(|node_pos| from_id(node_pos.clone().node)).collect();

                match (id?.ty, generics.clone()?.first()) {
                    (Ty::String, None) => Ok(Ty::String),
                    (Ty::Int, None) => Ok(Ty::Int),
                    (Ty::Float, None) => Ok(Ty::Float),
                    (Ty::Bool, None) => Ok(Ty::Bool),
                    (Ty::Any, None) => Ok(Ty::Any),
                    (Ty::Custom { lit, .. }, first) => match (lit.as_ref(), first) {
                        ("List", Some(ty)) => Ok(Ty::List { ty: Box::from(ty.clone()) }),
                        ("Set", Some(ty)) => Ok(Ty::Set { ty: Box::from(ty.clone()) }),
                        ("List", None) => Err(String::from("List cannot have more than one type")),
                        ("Set", None) => Err(String::from("Set cannot have more than one type")),
                        _ => Ok(Ty::Custom { lit, gens: generics? })
                    },
                    (other, _) => Err(format!("Type {} cannot have generics", other))
                }
            }
            _ => Ok(Ty::NA)
        }
    }
}

fn from_id(node: ASTNode) -> Result<Type, String> {
    match node {
        ASTNode::Id { lit } => Ok(match lit.as_ref() {
            "String" => Type::new(&Ty::String),
            "Int" => Type::new(&Ty::Int),
            "Float" => Type::new(&Ty::Float),
            "Bool" => Type::new(&Ty::Bool),
            "Any" => Type::new(&Ty::Any),
            other => Type::new(&Ty::Custom { lit: String::from(other), gens: vec![] })
        }),
        other => Err(format!("Expected type but got {:?}", other))
    }
}

impl PartialEq for Ty {
    fn eq(&self, other: &Self) -> bool {
        match (&self, other) {
            (.., Ty::Any) | (Ty::Any, ..) => true,
            (.., Ty::NA) | (Ty::NA, ..) => false,

            (Ty::Empty, Ty::Empty)
            | (Ty::Float, Ty::Float)
            | (Ty::Int, Ty::Int)
            | (Ty::Bool, Ty::Bool)
            | (Ty::String, Ty::String) => true,

            (Ty::Set { ty: ty_self }, Ty::Set { ty: ty_other }) => ty_self == ty_other,
            (Ty::List { ty: ty_self }, Ty::List { ty: ty_other }) => ty_self == ty_other,
            (Ty::Tuple { tys: ty_self }, Ty::Tuple { tys: ty_other }) =>
                ty_self.len() == ty_other.len()
                    && ty_self.iter().zip(ty_other).all(|(left, right)| left == right),

            (Ty::Range { ty: ty_self }, Ty::Range { ty: ty_other }) => ty_self == ty_other,
            (
                Ty::AnonFun { args: arg_self, out: out_self },
                Ty::AnonFun { args: arg_other, out: out_other }
            ) =>
                out_self == out_other
                    && arg_self.len() == arg_other.len()
                    && arg_self.iter().zip(arg_other).all(|(left, right)| left == right),

            (
                Ty::Custom { lit: lit_self, gens: gens_self },
                Ty::Custom { lit: lit_other, gens: gens_other }
            ) => lit_self == lit_other && gens_self == gens_other,

            _ => false
        }
    }
}
