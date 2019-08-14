use crate::parser::ast::{ASTNode, ASTNodePos};
use crate::type_checker::stage_1::field::Field;
use crate::type_checker::stage_1::function::Function;
use crate::type_checker::type_node::{Ty, Type};
use crate::type_checker::util::ExtractStmtExt;

#[derive(Debug)]
pub struct Interface {
    pub id:        Type,
    pub location:  Vec<String>,
    pub fields:    Vec<Field>,
    pub functions: Vec<Function>
}

#[derive(Debug)]
pub struct Class {
    pub id:         Type,
    pub private:    bool,
    pub location:   Vec<String>,
    pub init:       Option<Function>,
    pub implements: Vec<Type>,
    pub fields:     Vec<Field>,
    pub functions:  Vec<Function>
}

impl Interface {
    pub fn new(node_pos: &ASTNodePos) -> Result<Interface, String> {
        // TODO get location
        match &node_pos.node {
            ASTNode::TypeAlias { _type, conditions: _conditions } => {
                // TODO handle conditions
                let id = Type::try_from_type(_type.clone().node)?;
                Ok(Interface { id, location: vec![], fields: vec![], functions: vec![] })
            }
            ASTNode::TypeDef { _type, body } => {
                let id = Type::try_from_type(_type.clone().node)?;
                let (fields, functions) = match body {
                    Some(body) => Interface::extract_type_def_body(&id, body)?,
                    None => (vec![], vec![])
                };
                Ok(Interface { id, location: vec![], fields, functions })
            }
            other => Err(format!("Expected type def but got {:?}", other))
        }
    }

    fn extract_type_def_body(
        class_id: &Type,
        body: &ASTNodePos
    ) -> Result<(Vec<Field>, Vec<Function>), String> {
        body.statements()?.iter().try_fold(
            (vec![], vec![]),
            |(mut fields, mut functions), node_pos| match &node_pos.node {
                ASTNode::FunDef { .. } => {
                    functions.push(Function::new(Some(class_id.clone()), node_pos)?);
                    Ok((fields, functions))
                }
                ASTNode::VarDef { .. } => {
                    fields.push(Field::new(node_pos)?);
                    Ok((fields, functions))
                }
                other => Err(format!("Expected fun or var definition but got {:?}", other))
            }
        )
    }
}

impl Class {
    pub fn new(node_pos: &ASTNodePos) -> Result<Class, String> {
        match &node_pos.node {
            ASTNode::Class { _type, args, parents, body } => {
                let id = Type::try_from_type(_type.clone().node)?;
                // TODO Add private classes to language grammar
                // TODO get location of class
                let private = false;
                let location = vec![];

                let implements = parents
                    .iter()
                    .map(|parent| match &parent.node {
                        // TODO check that arguments passed to parent are correct type
                        ASTNode::Parent { id, .. } => {
                            // TODO handle generics
                            Ok(Type::new(&Ty::Custom {
                                lit: match &id.node {
                                    ASTNode::Id { lit } => lit.clone(),
                                    other => return Err(format!("Expected id {:?}", other))
                                }
                            }))
                        }
                        other => Err(format!("Expected parent {:?}", other))
                    })
                    .collect::<Result<Vec<Type>, String>>()?;

                let mut init = None;
                let (fields, functions) = body.statements()?.iter().try_fold(
                    (vec![], vec![]),
                    |(mut fields, mut funcs), node_pos| match &node_pos.node {
                        ASTNode::VarDef { .. } => {
                            fields.push(Field::new(node_pos)?);
                            Ok((fields, funcs))
                        }
                        ASTNode::FunDef { .. } => {
                            let function = Function::new(Some(id.clone()), node_pos)?;
                            match function.id.as_ref() {
                                "init" if args.is_empty() => init = Some(function),
                                "init" =>
                                    return Err(String::from(
                                        "Explicit init function in class with arguments"
                                    )),
                                _ => funcs.push(function)
                            }
                            Ok((fields, funcs))
                        }
                        other => Err(format!("Expected var or fun def in class body {:?}", other))
                    }
                )?;

                if !args.is_empty() {
                    init = Some(Function::new_init(&id, args)?)
                }

                Ok(Class { id, private, location, implements, init, fields, functions })
            }
            other => Err(format!("Expected class but got {:?}", other))
        }
    }
}
