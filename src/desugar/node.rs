use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::class::desugar_class;
use crate::desugar::common::desugar_vec;
use crate::desugar::context::Imports;
use crate::desugar::context::State;
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::desugar_result::UnimplementedErr;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;

pub fn desugar_node(node_pos: &ASTNodePos, imp: &mut Imports, state: &State) -> DesugarResult {
    Ok(match &node_pos.node {
        ASTNode::Import { import, _as } => match _as.len() {
            0 => Core::Import { imports: desugar_vec(import, imp, state)? },
            _ => Core::ImportAs {
                imports: desugar_vec(import, imp, state)?,
                _as:     desugar_vec(_as, imp, state)?
            }
        },
        ASTNode::FromImport { id, import } => Core::FromImport {
            from:   Box::from(desugar_node(id, imp, state)?),
            import: Box::from(desugar_node(import, imp, state)?)
        },

        ASTNode::Def { .. } => desugar_definition(node_pos, imp, state)?,
        ASTNode::Reassign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        ASTNode::Block { statements } =>
            Core::Block { statements: desugar_vec(statements, imp, state)? },

        ASTNode::Int { lit } => Core::Int { int: lit.clone() },
        ASTNode::Real { lit } => Core::Float { float: lit.clone() },
        ASTNode::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() }
        },
        ASTNode::Str { lit } => Core::Str { _str: lit.clone() },

        ASTNode::AddOp => Core::AddOp,
        ASTNode::SubOp => Core::SubOp,
        ASTNode::SqrtOp => return Err(UnimplementedErr::new(node_pos, "square root")),
        ASTNode::MulOp => Core::MulOp,
        ASTNode::FDivOp => Core::FDivOp,
        ASTNode::DivOp => Core::DivOp,
        ASTNode::PowOp => Core::PowOp,
        ASTNode::ModOp => Core::ModOp,
        ASTNode::EqOp => Core::EqOp,
        ASTNode::LeOp => Core::LeOp,
        ASTNode::GeOp => Core::GeOp,

        ASTNode::IdType { id, .. } => desugar_node(id, imp, state)?,
        ASTNode::Id { lit } => Core::Id { lit: lit.clone() },
        ASTNode::_Self => Core::Id { lit: String::from("self") },
        ASTNode::Init => Core::Id { lit: String::from("init") },
        ASTNode::Bool { lit } => Core::Bool { _bool: *lit },

        ASTNode::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements, imp, state)? },
        ASTNode::List { elements } => Core::List { elements: desugar_vec(elements, imp, state)? },
        ASTNode::Set { elements } => Core::Set { elements: desugar_vec(elements, imp, state)? },

        ASTNode::ListBuilder { .. } => return Err(UnimplementedErr::new(node_pos, "list builder")),
        ASTNode::SetBuilder { .. } => return Err(UnimplementedErr::new(node_pos, "set builder")),

        ASTNode::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        ASTNode::Return { expr } =>
            Core::Return { expr: Box::from(desugar_node(expr, imp, state)?) },
        ASTNode::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr, imp, state)?) },

        ASTNode::IfElse { .. }
        | ASTNode::Match { .. }
        | ASTNode::While { .. }
        | ASTNode::For { .. }
        | ASTNode::Break
        | ASTNode::Continue => desugar_control_flow(node_pos, imp, state)?,
        ASTNode::Case { .. } => panic!("Case cannot be top-level"),

        ASTNode::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr, imp, state)?) },
        ASTNode::And { left, right } => Core::And {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Or { left, right } => Core::Or {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Is { left, right } => Core::Is {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::IsN { left, right } => Core::IsN {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Eq { left, right } => Core::Eq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Neq { left, right } => Core::Neq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::IsA { left, right } => Core::IsA {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left:  Box::from(desugar_node(left, imp, state)?),
                right: Box::from(desugar_node(right, imp, state)?)
            })
        },

        ASTNode::Add { left, right } => Core::Add {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Sub { left, right } => Core::Sub {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Mul { left, right } => Core::Mul {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Div { left, right } => Core::Div {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::FDiv { left, right } => Core::FDiv {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Mod { left, right } => Core::Mod {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Pow { left, right } => Core::Pow {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        ASTNode::BAnd { left, right } => Core::BAnd {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::BOr { left, right } => Core::BOr {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::BXOr { left, right } => Core::BXOr {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::BOneCmpl { expr } =>
            Core::BOneCmpl { expr: Box::from(desugar_node(expr, imp, state)?) },
        ASTNode::BLShift { left, right } => Core::BLShift {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::BRShift { left, right } => Core::BRShift {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        ASTNode::AddU { expr } => Core::AddU { expr: Box::from(desugar_node(expr, imp, state)?) },
        ASTNode::SubU { expr } => Core::SubU { expr: Box::from(desugar_node(expr, imp, state)?) },
        ASTNode::Sqrt { expr } => {
            imp.add_import("math");
            Core::Sqrt { expr: Box::from(desugar_node(expr, imp, state)?) }
        }

        ASTNode::Le { left, right } => Core::Le {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Leq { left, right } => Core::Leq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Ge { left, right } => Core::Ge {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Geq { left, right } => Core::Geq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        ASTNode::FunArg { vararg, id_maybe_type, default } => Core::FunArg {
            vararg:  *vararg,
            id:      Box::from(desugar_node(id_maybe_type, imp, state)?),
            default: match default {
                Some(default) => Box::from(desugar_node(default, imp, state)?),
                None => Box::from(Core::Empty)
            }
        },

        ASTNode::FunctionCall { .. } | ASTNode::PropertyCall { .. } =>
            desugar_call(node_pos, imp, state)?,

        ASTNode::AnonFun { args, body } => Core::AnonFun {
            args: desugar_vec(args, imp, state)?,
            body: Box::from(desugar_node(body, imp, state)?)
        },

        ASTNode::In { left, right } => Core::In {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Range { from, to, inclusive, step } => Core::Range {
            from: Box::from(desugar_node(from, imp, state)?),
            to:   Box::from(if *inclusive {
                Core::Add {
                    left:  Box::from(desugar_node(to, imp, state)?),
                    right: Box::from(Core::Int { int: String::from("1") })
                }
            } else {
                desugar_node(to, imp, state)?
            }),
            step: Box::from(if let Some(step) = step {
                desugar_node(step, imp, state)?
            } else {
                Core::Int { int: String::from("1") }
            })
        },
        ASTNode::Underscore => Core::UnderScore,
        ASTNode::QuestOr { left, right } => Core::Or {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        ASTNode::Script { statements } =>
            Core::Block { statements: desugar_vec(statements, imp, state)? },
        ASTNode::File { modules, type_defs, imports, .. } => {
            let mut imports = desugar_vec(imports, imp, state)?;
            let mut type_definitions = desugar_vec(type_defs, imp, state)?;
            let mut modules = desugar_vec(modules, imp, state)?;
            imports.append(&mut imp.imports);

            let mut statements = imports;
            statements.append(&mut type_definitions);
            statements.append(&mut modules);
            Core::Block { statements }
        }

        ASTNode::TypeAlias { .. }
        | ASTNode::TypeTup { .. }
        | ASTNode::Type { .. }
        | ASTNode::TypeFun { .. } => Core::Empty,

        ASTNode::TypeDef { .. } => desugar_class(node_pos, imp, state)?,
        ASTNode::Class { .. } => desugar_class(node_pos, imp, state)?,
        ASTNode::Generic { .. } => Core::Empty,
        ASTNode::Parent { .. } => panic!("Parent cannot be top-level"),

        ASTNode::Condition { .. } => return Err(UnimplementedErr::new(node_pos, "condition")),

        ASTNode::Comment { comment } => Core::Comment { comment: comment.clone() },
        ASTNode::Pass => Core::Pass,

        ASTNode::With { resource, _as, expr } => match _as {
            Some(_as) => Core::WithAs {
                resource: Box::from(desugar_node(resource, imp, state)?),
                _as:      Box::from(desugar_node(_as, imp, state)?),
                expr:     Box::from(desugar_node(expr, imp, state)?)
            },
            None => Core::With {
                resource: Box::from(desugar_node(resource, imp, state)?),
                expr:     Box::from(desugar_node(expr, imp, state)?)
            }
        },

        ASTNode::Step { .. } => panic!("Step cannot be top level."),
        ASTNode::Raises { expr_or_stmt, .. } => desugar_node(expr_or_stmt, imp, state)?,
        ASTNode::Raise { error } =>
            Core::Raise { error: Box::from(desugar_node(error, imp, state)?) },
        ASTNode::Retry { .. } => return Err(UnimplementedErr::new(node_pos, "retry")),

        ASTNode::Handle { expr_or_stmt, cases } => {
            let mut statements = vec![];
            if let ASTNode::Def { definition, .. } = &expr_or_stmt.node {
                if let ASTNode::VariableDef { id_maybe_type, .. } = &definition.node {
                    statements.push(Core::Assign {
                        left:  Box::from(desugar_node(id_maybe_type.as_ref(), imp, state)?),
                        right: Box::from(Core::None)
                    });
                }
            };

            statements.push(Core::TryExcept {
                _try:   Box::from(desugar_node(&expr_or_stmt.clone(), imp, state)?),
                except: {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            ASTNode::Case { cond, body } => (cond, body),
                            other => panic!("Expected case but was {:?}", other)
                        };

                        match &cond.node {
                            ASTNode::IdType { id, _type: Some(ty), .. } => match &ty.node {
                                ASTNode::Type { id: ty, .. } => except.push(Core::Except {
                                    id:    Box::from(desugar_node(id, imp, state)?),
                                    class: Box::from(desugar_node(ty, imp, state)?),
                                    body:  Box::from(desugar_node(body, imp, state)?)
                                }),
                                other => panic!("Expected type but was {:?}", other)
                            },
                            ASTNode::IdType { id, _type: None, .. } =>
                                except.push(Core::ExceptNoClass {
                                    id:   Box::from(desugar_node(id, imp, state)?),
                                    body: Box::from(desugar_node(body, imp, state)?)
                                }),
                            other => panic!("Expected id type but was {:?}", other)
                        };
                    }
                    except
                }
            });

            Core::Block { statements }
        }

        ASTNode::VariableDef { .. } => panic!("Variable definition cannot be top level."),
        ASTNode::FunDef { .. } => panic!("Function definition cannot be top level.")
    })
}
