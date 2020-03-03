use crate::check::context::clss::concrete_to_python;
use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::class::desugar_class;
use crate::desugar::common::{desugar_stmts, desugar_vec};
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::desugar::result::DesugarResult;
use crate::desugar::result::UnimplementedErr;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::desugar::ty::desugar_type;
use crate::parse::ast::Node;
use crate::parse::ast::AST;

// TODO return imports instead of modifying mutable reference
pub fn desugar_node(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    let assign_to = state.assign_to.clone();
    let state = &state.assign_to(None);

    let core = match &ast.node {
        Node::Import { import, _as } =>
            if _as.is_empty() {
                Core::Import { imports: desugar_vec(import, imp, state)? }
            } else {
                Core::ImportAs {
                    imports: desugar_vec(import, imp, state)?,
                    alias:   desugar_vec(_as, imp, state)?
                }
            },
        Node::FromImport { id, import } => Core::FromImport {
            from:   Box::from(desugar_node(id, imp, state)?),
            import: Box::from(desugar_node(import, imp, state)?)
        },

        Node::VariableDef { .. } | Node::FunDef { .. } => desugar_definition(ast, imp, state)?,
        Node::Reassign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        Node::Block { statements } => Core::Block {
            statements: desugar_stmts(statements, imp, &state.assign_to(assign_to.as_ref()))?
        },

        Node::Int { lit } => Core::Int { int: lit.clone() },
        Node::Real { lit } => Core::Float { float: lit.clone() },
        Node::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() }
        },
        Node::DocStr { lit } => Core::DocStr { string: lit.clone() },
        Node::Str { lit, expressions } =>
            if expressions.is_empty() {
                Core::Str { string: lit.clone() }
            } else {
                Core::FStr { string: lit.clone() }
            },

        Node::AddOp => Core::AddOp,
        Node::SubOp => Core::SubOp,
        Node::SqrtOp => Core::Id { lit: String::from("sqrt") },
        Node::MulOp => Core::MulOp,
        Node::FDivOp => Core::FDivOp,
        Node::DivOp => Core::DivOp,
        Node::PowOp => Core::PowOp,
        Node::ModOp => Core::ModOp,
        Node::EqOp => Core::EqOp,
        Node::LeOp => Core::LeOp,
        Node::GeOp => Core::GeOp,
        Node::QuestionOp { .. } => desugar_type(ast, imp, state)?,

        Node::Undefined => Core::None,
        Node::ExpressionType { .. } => desugar_type(ast, imp, state)?,
        Node::Id { lit } => Core::Id { lit: concrete_to_python(lit) },
        Node::_Self => Core::Id { lit: String::from("self") },
        Node::Init => Core::Id { lit: String::from("init") },
        Node::Bool { lit } => Core::Bool { boolean: *lit },

        Node::Tuple { elements } => Core::Tuple { elements: desugar_vec(elements, imp, state)? },
        Node::List { elements } => Core::List { elements: desugar_vec(elements, imp, state)? },
        Node::Set { elements } => Core::Set { elements: desugar_vec(elements, imp, state)? },

        Node::ListBuilder { .. } => return Err(UnimplementedErr::new(ast, "list builder")),
        Node::SetBuilder { .. } => return Err(UnimplementedErr::new(ast, "set builder")),

        Node::ReturnEmpty => Core::Return { expr: Box::from(Core::None) },
        Node::Return { expr } => Core::Return { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::Print { expr } => Core::Print { expr: Box::from(desugar_node(expr, imp, state)?) },

        Node::IfElse { .. }
        | Node::While { .. }
        | Node::For { .. }
        | Node::Break
        | Node::Continue => desugar_control_flow(ast, imp, state)?,
        Node::Match { .. } => desugar_control_flow(ast, imp, &state.expand_ty(false))?,
        Node::Case { .. } => panic!("Case cannot be top-level"),

        Node::Not { expr } => Core::Not { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::And { left, right } => Core::And {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Or { left, right } => Core::Or {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Is { left, right } => Core::Is {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::IsN { left, right } => Core::IsN {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Eq { left, right } => Core::Eq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Neq { left, right } => Core::Neq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::IsA { left, right } => Core::IsA {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::IsNA { left, right } => Core::Not {
            expr: Box::from(Core::IsA {
                left:  Box::from(desugar_node(left, imp, state)?),
                right: Box::from(desugar_node(right, imp, state)?)
            })
        },

        Node::Add { left, right } => Core::Add {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Sub { left, right } => Core::Sub {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Mul { left, right } => Core::Mul {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Div { left, right } => Core::Div {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::FDiv { left, right } => Core::FDiv {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Mod { left, right } => Core::Mod {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Pow { left, right } => Core::Pow {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        Node::BAnd { left, right } => Core::BAnd {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::BOr { left, right } => Core::BOr {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::BXOr { left, right } => Core::BXOr {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::BOneCmpl { expr } =>
            Core::BOneCmpl { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::BLShift { left, right } => Core::BLShift {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::BRShift { left, right } => Core::BRShift {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        Node::AddU { expr } => Core::AddU { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::SubU { expr } => Core::SubU { expr: Box::from(desugar_node(expr, imp, state)?) },
        Node::Sqrt { expr } => {
            imp.add_import("math");
            Core::Sqrt { expr: Box::from(desugar_node(expr, imp, state)?) }
        }

        Node::Le { left, right } => Core::Le {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Leq { left, right } => Core::Leq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Ge { left, right } => Core::Ge {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Geq { left, right } => Core::Geq {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },

        Node::FunArg { vararg, var, ty, default, .. } => Core::FunArg {
            vararg:  *vararg,
            var:     Box::from(desugar_node(var, imp, state)?),
            ty:      if state.expand_ty {
                match ty {
                    Some(ty) => match &var.node {
                        Node::_Self => None,
                        _ => Some(Box::from(desugar_node(ty, imp, state)?))
                    },
                    None => None
                }
            } else {
                None
            },
            default: match default {
                Some(default) => Some(Box::from(desugar_node(default, imp, state)?)),
                None => None
            }
        },

        Node::FunctionCall { .. } | Node::PropertyCall { .. } => desugar_call(ast, imp, state)?,
        Node::AnonFun { args, body } => Core::AnonFun {
            args: desugar_vec(args, imp, &state.expand_ty(false))?,
            body: Box::from(desugar_node(body, imp, state)?)
        },

        Node::In { left, right } => Core::In {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Range { from, to, inclusive, step } => Core::Range {
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
        Node::Underscore => Core::UnderScore,
        Node::Question { left, right } => Core::Or {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, state)?)
        },
        Node::Script { statements } =>
            Core::Block { statements: desugar_vec(statements, imp, state)? },
        Node::File { modules, .. } => {
            let mut modules = desugar_vec(modules, imp, state)?;
            let mut statements = imp.imports.clone();
            statements.append(&mut modules);
            Core::Block { statements }
        }

        Node::TypeTup { .. }
        | Node::Type { .. }
        | Node::TypeFun { .. }
        | Node::TypeUnion { .. } => desugar_type(ast, imp, state)?,

        Node::TypeDef { .. } | Node::TypeAlias { .. } => desugar_class(ast, imp, state)?,
        Node::Class { .. } => desugar_class(ast, imp, state)?,
        Node::Generic { .. } => Core::Empty,
        Node::Parent { .. } => panic!("Parent cannot be top-level: {:?}", ast),

        Node::Condition { .. } => return Err(UnimplementedErr::new(ast, "condition")),

        Node::Comment { comment } => Core::Comment { comment: comment.clone() },
        Node::Pass => Core::Pass,

        Node::With { resource, alias: Some((alias, ..)), expr } => Core::WithAs {
            resource: Box::from(desugar_node(resource, imp, state)?),
            alias:    Box::from(desugar_node(alias, imp, &state.expand_ty(false))?),
            expr:     Box::from(desugar_node(expr, imp, state)?)
        },
        Node::With { resource, expr, .. } => Core::With {
            resource: Box::from(desugar_node(resource, imp, state)?),
            expr:     Box::from(desugar_node(expr, imp, state)?)
        },

        Node::Step { .. } => panic!("Step cannot be top level."),
        Node::Raises { expr_or_stmt, .. } => desugar_node(expr_or_stmt, imp, state)?,
        Node::Raise { error } => Core::Raise { error: Box::from(desugar_node(error, imp, state)?) },

        Node::Handle { expr_or_stmt, cases } => {
            let (var, private, ty) =
                if let Node::VariableDef { var, private, ty, .. } = &expr_or_stmt.node {
                    (
                        Some(Box::from(desugar_node(var, imp, state)?)),
                        *private,
                        if let Some(ty) = ty {
                            Some(Box::from(desugar_node(ty, imp, state)?))
                        } else {
                            None
                        }
                    )
                } else {
                    (None, false, None)
                };
            let assign_state = state.assign_to(var.as_deref());

            Core::TryExcept {
                setup:   if let Some(var) = var {
                    Some(Box::from(Core::VarDef { private, var, ty, expr: None }))
                } else {
                    None
                },
                attempt: Box::from(desugar_node(&expr_or_stmt.clone(), imp, state)?),
                except:  {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            Node::Case { cond, body } => (cond, body),
                            other => panic!("Expected case but was {:?}", other)
                        };

                        match &cond.node {
                            Node::ExpressionType { expr, ty, .. } => except.push(Core::Except {
                                id:    Box::from(desugar_node(expr, imp, state)?),
                                class: if let Some(ty) = ty {
                                    Some(Box::from(desugar_node(ty, imp, state)?))
                                } else {
                                    None
                                },
                                body:  Box::from(desugar_node(body, imp, &assign_state)?)
                            }),
                            other => panic!("Expected id type but was {:?}", other)
                        };
                    }
                    except
                }
            }
        }
    };

    let core = if let Some(assign_to) = assign_to {
        match core {
            Core::Block { .. } | Core::Return { .. } => core,
            expr => Core::Assign { left: Box::from(assign_to), right: Box::from(expr) }
        }
    } else {
        core
    };

    Ok(core)
}
