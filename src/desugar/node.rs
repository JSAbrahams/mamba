use crate::core::construct::Core;
use crate::desugar::call::desugar_call;
use crate::desugar::class::desugar_class;
use crate::desugar::common::{desugar_stmts, desugar_vec};
use crate::desugar::control_flow::desugar_control_flow;
use crate::desugar::definition::desugar_definition;
use crate::desugar::desugar_result::DesugarResult;
use crate::desugar::desugar_result::UnimplementedErr;
use crate::desugar::state::Imports;
use crate::desugar::state::State;
use crate::desugar::ty::desugar_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;

// TODO return imports instead of modifying mutable reference
pub fn desugar_node(ast: &AST, imp: &mut Imports, state: &State) -> DesugarResult {
    // If we expect a return, handle here and do not recursively pass
    // Unless we have a block, in which case we make use of desugar_stmts
    // Once our type checker can augment the ast, we can omit this less elegant
    // solution
    let expect_return = state.expect_ret;
    let state = &state.expect_return(false);

    let core = match &ast.node {
        Node::Import { import, _as } =>
            if _as.is_empty() {
                Core::Import { imports: desugar_vec(import, imp, state)? }
            } else {
                Core::ImportAs {
                    imports: desugar_vec(import, imp, state)?,
                    _as:     desugar_vec(_as, imp, state)?
                }
            },
        Node::FromImport { id, import } => Core::FromImport {
            from:   Box::from(desugar_node(id, imp, state)?),
            import: Box::from(desugar_node(import, imp, state)?)
        },

        Node::VariableDef { .. } | Node::FunDef { .. } => desugar_definition(ast, imp, state)?,
        Node::Reassign { left, right } => Core::Assign {
            left:  Box::from(desugar_node(left, imp, state)?),
            right: Box::from(desugar_node(right, imp, &state.expect_expr(true))?)
        },

        Node::Block { statements } => Core::Block {
            // Preserve expect_return boolean
            statements: desugar_stmts(statements, imp, &state.expect_return(expect_return))?
        },

        Node::Int { lit } => Core::Int { int: lit.clone() },
        Node::Real { lit } => Core::Float { float: lit.clone() },
        Node::ENum { num, exp } => Core::ENum {
            num: num.clone(),
            exp: if exp.is_empty() { String::from("0") } else { exp.clone() }
        },
        Node::Str { lit } => Core::Str { _str: lit.clone() },

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
        Node::IdType { .. } => desugar_type(ast, imp, state)?,
        Node::Id { lit } => Core::Id { lit: lit.clone() },
        Node::_Self => Core::Id { lit: String::from("self") },
        Node::Init => Core::Id { lit: String::from("init") },
        Node::Bool { lit } => Core::Bool { _bool: *lit },

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

        Node::FunArg { vararg, id_maybe_type, default } => Core::FunArg {
            vararg:  *vararg,
            id:      Box::from(desugar_node(id_maybe_type, imp, state)?),
            default: match default {
                Some(default) => Box::from(desugar_node(default, imp, state)?),
                None => Box::from(Core::Empty)
            }
        },

        Node::FunctionCall { .. } | Node::PropertyCall { .. } => desugar_call(ast, imp, state)?,

        Node::AnonFun { args, body } => Core::AnonFun {
            args: desugar_vec(args, imp, state)?,
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
        Node::File { modules, imports, .. } => {
            let mut imports = desugar_vec(imports, imp, state)?;
            let mut modules = desugar_vec(modules, imp, state)?;
            imports.append(&mut imp.imports.clone().into_iter().collect());

            let mut statements = imports;
            statements.append(&mut modules);
            Core::Block { statements }
        }

        Node::TypeAlias { .. }
        | Node::TypeTup { .. }
        | Node::Type { .. }
        | Node::TypeFun { .. }
        | Node::TypeUnion { .. } => desugar_type(ast, imp, state)?,

        Node::TypeDef { .. } => desugar_class(ast, imp, state)?,
        Node::Class { .. } => desugar_class(ast, imp, state)?,
        Node::Generic { .. } => Core::Empty,
        Node::Parent { .. } => panic!("Parent cannot be top-level"),

        Node::Condition { .. } => return Err(UnimplementedErr::new(ast, "condition")),

        Node::Comment { comment } => Core::Comment { comment: comment.clone() },
        Node::Pass => Core::Pass,

        Node::With { resource, _as, expr } => match _as {
            Some(_as) => Core::WithAs {
                resource: Box::from(desugar_node(resource, imp, state)?),
                _as:      Box::from(desugar_node(_as, imp, &state.expand_ty(false))?),
                expr:     Box::from(desugar_node(expr, imp, state)?)
            },
            None => Core::With {
                resource: Box::from(desugar_node(resource, imp, state)?),
                expr:     Box::from(desugar_node(expr, imp, state)?)
            }
        },

        Node::Step { .. } => panic!("Step cannot be top level."),
        Node::Raises { expr_or_stmt, .. } => desugar_node(expr_or_stmt, imp, state)?,
        Node::Raise { error } => Core::Raise { error: Box::from(desugar_node(error, imp, state)?) },
        Node::Retry { .. } => return Err(UnimplementedErr::new(ast, "retry")),

        Node::Handle { expr_or_stmt, cases } => {
            let mut statements = vec![];
            if let Node::VariableDef { id_maybe_type, .. } = &expr_or_stmt.node {
                statements.push(Core::Assign {
                    left:  Box::from(desugar_node(id_maybe_type.as_ref(), imp, state)?),
                    right: Box::from(Core::None)
                });
            };

            statements.push(Core::TryExcept {
                _try:   Box::from(desugar_node(&expr_or_stmt.clone(), imp, state)?),
                except: {
                    let mut except = Vec::new();
                    for case in cases {
                        let (cond, body) = match &case.node {
                            Node::Case { cond, body } => (cond, body),
                            other => panic!("Expected case but was {:?}", other)
                        };

                        match &cond.node {
                            Node::IdType { id, _type: Some(ty), .. } => match &ty.node {
                                Node::Type { id: ty, .. } => except.push(Core::Except {
                                    id:    Box::from(desugar_node(id, imp, state)?),
                                    class: Box::from(desugar_node(ty, imp, state)?),
                                    body:  Box::from(desugar_node(body, imp, state)?)
                                }),
                                other => panic!("Expected type but was {:?}", other)
                            },
                            Node::IdType { id, _type: None, .. } =>
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
    };

    if expect_return {
        match core {
            // If block, last statement has already been made a return using the desugar_stmts
            // function
            Core::Block { .. } | Core::Return { .. } => Ok(core),
            expr => Ok(Core::Return { expr: Box::from(expr.clone()) })
        }
    } else {
        Ok(core)
    }
}
