use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Function {
    id:       Type,
    location: Vec<String>,
    private:  bool,
    pure:     bool,
    args:     Vec<FunctionArg>,
    ret:      Type,
    raises:   Vec<Type>
}

#[derive(Debug)]
pub struct FunctionArg {
    id: String,
    ty: Option<Type>
}

impl Function {
    pub fn new(node_pos: &ASTNodePos) -> Result<Function, String> {
        match &node_pos.node {
            ASTNode::FunDef { pure, private, id_type, fun_args, ret_ty, raises, body } => {
                let args: Result<Vec<_>, String> =
                    fun_args.into_iter().map(|fun_arg| FunctionArg::new(&fun_arg)).collect();
                let raises: Result<Vec<_>, String> = raises
                    .into_iter()
                    .map(|raise| Type::try_from_node(raise.clone().node))
                    .collect();

                Ok(Function {
                    id: Type::try_from_node(id_type.clone().node)?
                    // TODO store location of function
                    location: vec![],
                    private:  *private,
                    pure:     *pure,
                    args:     args?,
                    ret:      if ret_ty.is_some() {
                        Type::try_from_node(ret_ty.clone().unwrap_or_else(|| unreachable!()).node)?
                    } else {
                        // TODO what if no explicit return type in signature?
                        Type::Any
                    },
                    raises:   raises?
                })
            }
            other => Err(format!("Expected function definition but got {:?}", other))
        }
    }
}

impl FunctionArg {
    pub fn new(node_pos: &ASTNodePos) -> Result<FunctionArg, String> {
        match &node_pos.node {
            ASTNode::FunArg { vararg, id_maybe_type, default } => match &id_maybe_type.node {
                ASTNode::IdType { id, mutable, _type } => {
                    let id = match &id.node {
                        ASTNode::Id { lit } => lit.clone(),
                        ASTNode::_Self => {
                            if default.is_some() {
                                return Err(format!("Self argument cannot have default"));
                            }
                            String::from("self")
                        }
                        other => return Err(format!("Expected id but got {:?}", other.clone()))
                    };

                    if _type.clone().is_none() && id != "self" {
                        return Err(format!(
                            "If function argument not self, must have type: {:?}",
                            _type.clone()
                        ));
                    }

                    Ok(FunctionArg {
                        id,
                        ty: match _type {
                            Some(_type) => Some(Type::try_from_node(_type.clone().node)?),
                            _ => None
                        }
                    })
                }
                other => Err(format!("Expected id type but got {:?}", other))
            },
            other => Err(format!("Expected function argument but got {:?}", other))
        }
    }
}
