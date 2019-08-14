use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Field {
    pub id:       String,
    pub location: Vec<String>,
    pub ty:       Type
}

impl Field {
    pub fn new(node_pos: &ASTNodePos) -> Result<Field, String> {
        match &node_pos.node {
            ASTNode::VarDef { ofmut, id_maybe_type, .. } => {
                // TODO get location of field
                // TODO something with private
                let location = vec![];
                // TODO do something with forward
                // TODO get type of expression during second pass
                let (id, ty) = match &id_maybe_type.node {
                    ASTNode::IdType { id, _type, mutable } => match (&id.node, _type) {
                        (ASTNode::Id { ref lit }, Some(_type)) => (lit.clone(), {
                            let mut ty = Type::try_from_type(_type.clone().node)?;
                            if *mutable {
                                ty = ty.into_mutable()
                            }
                            if *ofmut {
                                ty = ty.into_of_mutable()
                            }
                            ty
                        }),
                        (ASTNode::Id { .. }, None) =>
                            return Err(String::from(
                                "Currently cannot inference type of class fields"
                            )),
                        (other, _) => return Err(format!("Expected id {:?}", other))
                    },
                    other => return Err(format!("Expected id type {:?}", other))
                };

                Ok(Field { id, location, ty })
            }
            other => Err(format!("Expected field but got {:?}", other))
        }
    }
}
