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
    Tuple { ty: Vec<Type> },

    Range { ty: Box<Type> },
    AnonFun { arg: Vec<Type>, out: Box<Type> },

    Custom { lit: String },
    Maybe { ty: Box<Type> },

    NA,
    Any
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
                Ok(Type::Tuple { ty: types? })
            }
            ASTNode::Type { id, generics } => {
                let id: Result<Type, String> = from_id(id.node);
                // TODO do something with generics
                let _generics: Result<Vec<Type>, String> =
                    generics.iter().map(|node_pos| from_id(node_pos.clone().node)).collect();

                id
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
            other => Type::Custom { lit: String::from(other) }
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
            (Type::Tuple { ty: ty_self }, Type::Tuple { ty: ty_other }) =>
                ty_self.len() == ty_other.len()
                    && ty_self.iter().zip(ty_other).all(|(left, right)| left == right),

            (Type::Range { ty: ty_self }, Type::Range { ty: ty_other }) => ty_self == ty_other,
            (
                Type::AnonFun { arg: arg_self, out: out_self },
                Type::AnonFun { arg: arg_other, out: out_other }
            ) =>
                out_self == out_other
                    && arg_self.len() == arg_other.len()
                    && arg_self.iter().zip(arg_other).all(|(left, right)| left == right),

            (Type::Custom { lit: lit_self }, Type::Custom { lit: lit_other }) =>
                lit_self == lit_other,
            (Type::Maybe { ty: ty_self }, Type::Maybe { ty: ty_other }) => ty_self == ty_other,

            _ => false
        }
    }
}
