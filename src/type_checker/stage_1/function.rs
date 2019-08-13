use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::type_node::Ty;
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Function {
    pub id:       String,
    pub location: Vec<String>,
    pub pure:     bool,
    pub private:  bool,
    pub args:     Vec<FunctionArg>,
    pub ret:      Type,
    pub raises:   Vec<Type>
}

#[derive(Debug)]
pub struct FunctionArg {
    pub id: String,
    pub ty: Option<Type>
}

impl Function {
    pub fn new(class_id: Option<Type>, node_pos: &ASTNodePos) -> Result<Function, String> {
        match &node_pos.node {
            ASTNode::FunDef { pure, private, id_type, fun_args, ret_ty, raises, .. } => {
                let id = match &id_type.node {
                    ASTNode::IdType { id, mutable, _type } => {
                        if *mutable {
                            return Err(String::from("Function definition cannot be mutable"));
                        }
                        if _type.is_some() {
                            return Err(String::from("Function name cannot have type"));
                        }

                        match &id.node {
                            ASTNode::Id { lit } => lit.clone(),
                            other => return Err(format!("Expected id but got {:?}", other))
                        }
                    }
                    ASTNode::Id { lit } => lit.clone(),
                    ASTNode::Init => String::from("init"),
                    other => return Err(format!("Expected id but got {:?}", other.clone()))
                };
                let args: Result<Vec<_>, String> = fun_args
                    .into_iter()
                    .map(|fun_arg| FunctionArg::new(class_id.clone(), &fun_arg))
                    .collect();
                let raises: Result<Vec<_>, String> = raises
                    .into_iter()
                    .map(|raise| Type::try_from_type(raise.clone().node))
                    .collect();

                Ok(Function {
                    id,
                    // TODO store location of function
                    location: vec![],
                    pure: *pure,
                    private: *private,
                    args: args?,
                    ret: if ret_ty.is_some() {
                        Type::try_from_type(ret_ty.clone().unwrap_or_else(|| unreachable!()).node)?
                    } else {
                        // TODO what if no explicit return type in signature?
                        Type::new(&Ty::Any)
                    },
                    raises: raises?
                })
            }
            other => Err(format!("Expected function definition but got {:?}", other))
        }
    }

    pub fn new_init(class_id: &Type, args: &Vec<ASTNodePos>) -> Result<Function, String> {
        // TODO get location
        let location = vec![];
        let args: Result<Vec<_>, String> = args
            .iter()
            .map(|node_pos| FunctionArg::new(Some(class_id.clone()), node_pos))
            .collect();

        Ok(Function {
            id: String::from("init"),
            location,
            pure: true,
            private: false,
            args: args?,
            ret: class_id.clone(),
            raises: vec![]
        })
    }
}

impl FunctionArg {
    pub fn new(class_id: Option<Type>, node_pos: &ASTNodePos) -> Result<FunctionArg, String> {
        match &node_pos.node {
            // TODO do something with vararg
            ASTNode::FunArg { id_maybe_type, default, .. } => match &id_maybe_type.node {
                ASTNode::IdType { id, mutable, _type } => {
                    let id = match &id.node {
                        ASTNode::Id { lit } => lit.clone(),
                        ASTNode::_Self if default.is_some() =>
                            return Err(format!("Self argument cannot have default")),
                        ASTNode::_Self => String::from("self"),
                        other => return Err(format!("Expected id but got {:?}", other.clone()))
                    };

                    let ty = match (class_id, id.as_ref(), _type) {
                        (_, "self", Some(ty)) =>
                            return Err(format!(
                                "Currently cannot assign type to self argument: {:?}",
                                ty
                            )),
                        (None, "self", _) =>
                            return Err(format!("Cannot have self argument outside of a class")),
                        (Some(_type), "self", None) => Some(_type),

                        (_, _, Some(ty)) => Some(Type::try_from_type(ty.clone().node)?),
                        (_, _, None) => return Err(format!("Function argument must have type"))
                    };

                    Ok(FunctionArg {
                        id,
                        ty: ty.map(|ty| if *mutable { ty.into_mutable() } else { ty })
                    })
                }
                other => Err(format!("Expected id type but got {:?}", other))
            },
            other => Err(format!("Expected function argument but got {:?}", other))
        }
    }
}
