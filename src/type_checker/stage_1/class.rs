use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::stage_1::field::Field;
use crate::type_checker::stage_1::function::Function;
use crate::type_checker::type_node::Ty;
use crate::type_checker::type_node::Type;

#[derive(Debug)]
pub struct Interface {
    pub id:        Type,
    pub location:  Vec<String>,
    pub fields:    Vec<Field>,
    pub functions: Vec<Function>
}

#[derive(Debug)]
pub struct Class {
    id:         Type,
    private:    bool,
    location:   Vec<String>,
    init:       Option<Function>,
    implements: Vec<Interface>,
    fields:     Vec<Field>,
    functions:  Vec<Function>
}

impl Interface {
    pub fn new(node_pos: &ASTNodePos) -> Result<Interface, String> {
        match &node_pos.node {
            ASTNode::TypeAlias { _type, conditions: _conditions } => {
                // TODO do something with conditions
                let id = match &_type.node {
                    ASTNode::Type { .. } => Type::try_from_type(_type.clone().node),
                    other => return Err(format!("Expected type but got {:?}", other))
                };

                Ok(Interface {
                    id:        id?,
                    location:  vec![],
                    fields:    vec![],
                    functions: vec![]
                })
            }
            ASTNode::TypeDef { _type, body } => {
                let id = match &_type.node {
                    ASTNode::Type { .. } => Type::try_from_type(_type.clone().node),
                    other => return Err(format!("Expected type but got {:?}", other))
                };

                let mut fields = vec![];
                let mut functions = vec![];

                if body.is_some() {
                    let body = body.clone().unwrap_or_else(|| unreachable!());
                    let statements = match body.node {
                        ASTNode::Block { statements } => statements,
                        other => return Err(format!("Expected block but got {:?}", other))
                    };

                    for statement in statements {
                        match &statement.node {
                            ASTNode::FunDef { .. } => functions.push(Function::new(&statement)?),
                            ASTNode::VariableDef { .. } => fields.push(Field::new(&statement)?),
                            other =>
                                return Err(format!(
                                    "Expected fun or variable definition but got {:?}",
                                    other
                                )),
                        }
                    }
                }

                Ok(Interface { id: id?, location: vec![], fields, functions })
            }
            other => Err(format!("Expected type def but got {:?}", other))
        }
    }
}

impl Class {
    pub fn new(node_pos: &ASTNodePos) -> Result<Class, String> {
        match &node_pos.node {
            ASTNode::Class { .. } => Ok(Class {
                id:         Type::new(&Ty::Empty),
                private:    false,
                location:   vec![],
                init:       None,
                implements: vec![],
                fields:     vec![],
                functions:  vec![]
            }),
            other => Err(format!("Expected class but got {:?}", other))
        }
    }
}
