use crate::parser::ast::{Node, AST};
use crate::type_checker::context::Context;
use crate::type_checker::type_result::TypeResult;

pub mod constructor;

pub trait Modification {
    fn modify(&self, ast: &AST, ctx: &Context) -> TypeResult<(AST, bool)>;

    fn recursion(&self, ast: &AST, ctx: &Context) -> TypeResult<(AST, bool)> {
        macro_rules! modify {
            ($ast:expr) => {{
                let (expr, m_expr) = self.modify($ast, ctx)?;
                (Box::from(expr), m_expr)
            }};
        }

        macro_rules! inner {
            ($node:ident, $left:expr, $right:expr) => {{
                let (left, m_left) = modify!($left);
                let (right, m_right) = modify!($right);
                Ok((AST { node: Node::$node { left, right }, ..ast.clone() }, m_left || m_right))
            }};
            ($node:ident, $expr:expr) => {{
                let (expr, m_expr) = modify!($expr);
                Ok((AST { node: Node::$node { expr }, ..ast.clone() }, m_expr))
            }};
        }

        macro_rules! vec_recursion {
            ($vec:expr) => {{
                let scanned: Vec<(AST, bool)> =
                    $vec.iter().map(|com| self.modify(com, ctx)).collect::<Result<_, _>>()?;
                let (asts, modified): (Vec<AST>, Vec<bool>) = scanned.into_iter().unzip();
                let modified = modified.iter().any(|b| b.clone());
                (asts, modified)
            }};
        }

        macro_rules! optional {
            ($expr:expr) => {{
                if let Some(expr) = $expr {
                    let (expr, m_expr): (Box<AST>, bool) = modify!(expr);
                    (Some(expr), m_expr)
                } else {
                    (None, false)
                }
            }};
        }

        match &ast.node {
            Node::File { pure, comments, imports, modules } => {
                let (comments, m_comments) = vec_recursion!(comments);
                let (imports, m_imports) = vec_recursion!(imports);
                let (modules, m_modules) = vec_recursion!(modules);
                Ok((
                    AST {
                        node: Node::File { comments, imports, modules, pure: *pure },
                        ..ast.clone()
                    },
                    m_comments || m_imports || m_modules
                ))
            }
            Node::Import { import, _as } => {
                let (import, m_import) = vec_recursion!(import);
                let (_as, m_as) = vec_recursion!(_as);
                Ok((AST { node: Node::Import { import, _as }, ..ast.clone() }, m_import || m_as))
            }
            Node::FromImport { id, import } => {
                let (id, m_id) = modify!(id);
                let (import, m_import) = modify!(import);
                Ok((AST { node: Node::FromImport { id, import }, ..ast.clone() }, m_id || m_import))
            }
            Node::Class { _type, args, parents, body } => {
                let (_type, m_type) = modify!(_type);
                let (args, m_args) = vec_recursion!(args);
                let (parents, m_parents) = vec_recursion!(parents);
                let (body, m_body) = optional!(body);
                Ok((
                    AST { node: Node::Class { _type, args, parents, body }, ..ast.clone() },
                    m_type || m_args || m_parents || m_body
                ))
            }
            Node::Generic { id, isa } => {
                let (id, m_id) = modify!(id);
                let (isa, m_isa) = optional!(isa);
                Ok((AST { node: Node::Generic { id, isa }, ..ast.clone() }, m_id || m_isa))
            }
            Node::Parent { id, generics, args } => {
                let (id, m_id) = modify!(id);
                let (generics, m_generics) = vec_recursion!(generics);
                let (args, m_args) = vec_recursion!(args);
                Ok((
                    AST { node: Node::Parent { id, generics, args }, ..ast.clone() },
                    m_id || m_generics || m_args
                ))
            }
            Node::Script { statements } => {
                let (statements, m_statements) = vec_recursion!(statements);
                Ok((AST { node: Node::Script { statements }, ..ast.clone() }, m_statements))
            }
            Node::Init => Ok((ast.clone(), false)),
            Node::Reassign { left, right } => {
                let (left, m_left) = modify!(left);
                let (right, m_right) = modify!(right);
                Ok((AST { node: Node::Reassign { left, right }, ..ast.clone() }, m_left || m_right))
            }
            Node::VariableDef { private, id_maybe_type, expression, forward } => {
                let (id_maybe_type, m_id_maybe_type) = modify!(id_maybe_type);
                let (expression, m_expression) = optional!(expression);
                let (forward, m_forward) = vec_recursion!(forward);
                Ok((
                    AST {
                        node: Node::VariableDef {
                            private: *private,
                            id_maybe_type,
                            expression,
                            forward
                        },
                        ..ast.clone()
                    },
                    m_id_maybe_type || m_expression || m_forward
                ))
            }
            Node::FunDef { pure, private, id, fun_args, ret_ty, raises, body } => {
                let (id, m_id) = modify!(id);
                let (fun_args, m_fun_args) = vec_recursion!(fun_args);
                let (ret_ty, m_ret_ty) = optional!(ret_ty);
                let (raises, m_raises) = vec_recursion!(raises);
                let (body, m_body) = optional!(body);
                Ok((
                    AST {
                        node: Node::FunDef {
                            pure: *pure,
                            private: *private,
                            id,
                            fun_args,
                            ret_ty,
                            raises,
                            body
                        },
                        ..ast.clone()
                    },
                    m_id || m_fun_args || m_ret_ty || m_raises || m_body
                ))
            }
            Node::AnonFun { args, body } => {
                let (args, m_args) = vec_recursion!(args);
                let (body, m_body) = modify!(body);
                Ok((AST { node: Node::AnonFun { args, body }, ..ast.clone() }, m_args || m_body))
            }
            Node::Raises { expr_or_stmt, errors } => {
                let (expr_or_stmt, m_expr_or_stmt) = modify!(expr_or_stmt);
                let (errors, m_errors) = vec_recursion!(errors);
                Ok((
                    AST { node: Node::Raises { expr_or_stmt, errors }, ..ast.clone() },
                    m_expr_or_stmt || m_errors
                ))
            }
            Node::Raise { error } => {
                let (error, m_error) = modify!(error);
                Ok((AST { node: Node::Raise { error }, ..ast.clone() }, m_error))
            }
            Node::Handle { expr_or_stmt, cases } => {
                let (expr_or_stmt, m_expr_or_stmt) = modify!(expr_or_stmt);
                let (cases, m_cases) = vec_recursion!(cases);
                Ok((
                    AST { node: Node::Handle { expr_or_stmt, cases }, ..ast.clone() },
                    m_expr_or_stmt || m_cases
                ))
            }
            Node::With { resource, _as, expr } => {
                let (resource, m_resource) = modify!(resource);
                let (_as, m_as) = optional!(_as);
                let (expr, m_expr) = modify!(expr);
                Ok((
                    AST { node: Node::With { resource, _as, expr }, ..ast.clone() },
                    m_resource || m_as || m_expr
                ))
            }
            Node::ConstructorCall { name, args } => {
                let (name, m_name) = modify!(name);
                let (args, m_args) = vec_recursion!(args);
                Ok((
                    AST { node: Node::ConstructorCall { name, args }, ..ast.clone() },
                    m_name || m_args
                ))
            }
            Node::FunctionCall { name, args } => {
                let (name, m_name) = modify!(name);
                let (args, m_args) = vec_recursion!(args);
                Ok((
                    AST { node: Node::FunctionCall { name, args }, ..ast.clone() },
                    m_name || m_args
                ))
            }
            Node::PropertyCall { instance, property } => {
                let (instance, m_instance) = modify!(instance);
                let (property, m_property) = modify!(property);
                Ok((
                    AST { node: Node::PropertyCall { instance, property }, ..ast.clone() },
                    m_instance || m_property
                ))
            }
            Node::Id { .. } => Ok((ast.clone(), false)),
            Node::IdType { id, mutable, _type } => {
                let (id, m_id) = modify!(id);
                let (_type, m_type) = optional!(_type);
                Ok((
                    AST { node: Node::IdType { mutable: *mutable, id, _type }, ..ast.clone() },
                    m_id || m_type
                ))
            }
            Node::TypeDef { _type, isa, body } => {
                let (_type, m_type) = modify!(_type);
                let (isa, m_isa) = optional!(isa);
                let (body, m_body) = optional!(body);
                Ok((
                    AST { node: Node::TypeDef { _type, isa, body }, ..ast.clone() },
                    m_type || m_isa || m_body
                ))
            }
            Node::TypeAlias { _type, isa, conditions } => {
                let (_type, m_type) = modify!(_type);
                let (isa, m_isa) = modify!(isa);
                let (conditions, m_conditions) = vec_recursion!(conditions);
                Ok((
                    AST { node: Node::TypeAlias { _type, isa, conditions }, ..ast.clone() },
                    m_type || m_isa || m_conditions
                ))
            }
            Node::TypeTup { types } => {
                let (types, m_types) = vec_recursion!(types);
                Ok((AST { node: Node::TypeTup { types }, ..ast.clone() }, m_types))
            }
            Node::TypeUnion { types } => {
                let (types, m_types) = vec_recursion!(types);
                Ok((AST { node: Node::TypeUnion { types }, ..ast.clone() }, m_types))
            }
            Node::Type { id, generics } => {
                let (id, m_id) = modify!(id);
                let (generics, m_generics) = vec_recursion!(generics);
                Ok((AST { node: Node::Type { id, generics }, ..ast.clone() }, m_id || m_generics))
            }
            Node::TypeFun { args, ret_ty } => {
                let (args, m_args) = vec_recursion!(args);
                let (ret_ty, m_ret_ty) = modify!(ret_ty);
                Ok((
                    AST { node: Node::TypeFun { args, ret_ty }, ..ast.clone() },
                    m_args || m_ret_ty
                ))
            }
            Node::Condition { cond, _else } => {
                let (cond, m_cond) = modify!(cond);
                let (_else, m_else) = optional!(_else);
                Ok((AST { node: Node::Condition { cond, _else }, ..ast.clone() }, m_cond || m_else))
            }
            Node::FunArg { vararg, id_maybe_type, default } => {
                let (id_maybe_type, m_id_maybe_type) = modify!(id_maybe_type);
                let (default, m_default) = optional!(default);
                Ok((
                    AST {
                        node: Node::FunArg { vararg: *vararg, id_maybe_type, default },
                        ..ast.clone()
                    },
                    m_id_maybe_type || m_default
                ))
            }
            Node::_Self => Ok((ast.clone(), false)),
            Node::AddOp => Ok((ast.clone(), false)),
            Node::SubOp => Ok((ast.clone(), false)),
            Node::SqrtOp => Ok((ast.clone(), false)),
            Node::MulOp => Ok((ast.clone(), false)),
            Node::FDivOp => Ok((ast.clone(), false)),
            Node::DivOp => Ok((ast.clone(), false)),
            Node::PowOp => Ok((ast.clone(), false)),
            Node::ModOp => Ok((ast.clone(), false)),
            Node::EqOp => Ok((ast.clone(), false)),
            Node::LeOp => Ok((ast.clone(), false)),
            Node::GeOp => Ok((ast.clone(), false)),
            Node::SetBuilder { item, conditions } => {
                let (item, m_item) = modify!(item);
                let (conditions, m_conditions) = vec_recursion!(conditions);
                Ok((
                    AST { node: Node::SetBuilder { item, conditions }, ..ast.clone() },
                    m_item || m_conditions
                ))
            }
            Node::ListBuilder { item, conditions } => {
                let (item, m_item) = modify!(item);
                let (conditions, m_conditions) = vec_recursion!(conditions);
                Ok((
                    AST { node: Node::ListBuilder { item, conditions }, ..ast.clone() },
                    m_item || m_conditions
                ))
            }
            Node::Set { elements } => {
                let (elements, m_elements) = vec_recursion!(elements);
                Ok((AST { node: Node::Set { elements }, ..ast.clone() }, m_elements))
            }
            Node::List { elements } => {
                let (elements, m_elements) = vec_recursion!(elements);
                Ok((AST { node: Node::List { elements }, ..ast.clone() }, m_elements))
            }
            Node::Tuple { elements } => {
                let (elements, m_elements) = vec_recursion!(elements);
                Ok((AST { node: Node::Tuple { elements }, ..ast.clone() }, m_elements))
            }
            Node::Range { from, to, inclusive, step } => {
                let (from, m_from) = modify!(from);
                let (to, m_to) = modify!(to);
                let (step, m_step) = optional!(step);
                Ok((
                    AST {
                        node: Node::Range { from, to, inclusive: *inclusive, step },
                        ..ast.clone()
                    },
                    m_from || m_to || m_step
                ))
            }
            Node::Block { statements } => {
                let (statements, m_statements) = vec_recursion!(statements);
                Ok((AST { node: Node::Block { statements }, ..ast.clone() }, m_statements))
            }
            Node::Real { .. } => Ok((ast.clone(), false)),
            Node::Int { .. } => Ok((ast.clone(), false)),
            Node::ENum { .. } => Ok((ast.clone(), false)),
            Node::Str { lit, expressions } => {
                let (expressions, m_expressions) = vec_recursion!(expressions);
                Ok((
                    AST { node: Node::Str { lit: lit.clone(), expressions }, ..ast.clone() },
                    m_expressions
                ))
            }
            Node::Bool { .. } => Ok((ast.clone(), false)),
            Node::Add { left, right } => inner!(Add, left, right),
            Node::Sub { left, right } => inner!(Sub, left, right),
            Node::Mul { left, right } => inner!(Mul, left, right),
            Node::Div { left, right } => inner!(Div, left, right),
            Node::FDiv { left, right } => inner!(FDiv, left, right),
            Node::Mod { left, right } => inner!(Mod, left, right),
            Node::Pow { left, right } => inner!(Pow, left, right),
            Node::BAnd { left, right } => inner!(BAnd, left, right),
            Node::BOr { left, right } => inner!(BOr, left, right),
            Node::BXOr { left, right } => inner!(BXOr, left, right),
            Node::BLShift { left, right } => inner!(BLShift, left, right),
            Node::BRShift { left, right } => inner!(BRShift, left, right),
            Node::Le { left, right } => inner!(Le, left, right),
            Node::Ge { left, right } => inner!(Ge, left, right),
            Node::Leq { left, right } => inner!(Leq, left, right),
            Node::Geq { left, right } => inner!(Geq, left, right),
            Node::Is { left, right } => inner!(Is, left, right),
            Node::IsN { left, right } => inner!(IsN, left, right),
            Node::Eq { left, right } => inner!(Eq, left, right),
            Node::Neq { left, right } => inner!(Neq, left, right),
            Node::IsA { left, right } => inner!(IsA, left, right),
            Node::IsNA { left, right } => inner!(IsNA, left, right),
            Node::And { left, right } => inner!(And, left, right),
            Node::Or { left, right } => inner!(Or, left, right),
            Node::BOneCmpl { expr } => inner!(BOneCmpl, expr),
            Node::Sqrt { expr } => inner!(Sqrt, expr),
            Node::SubU { expr } => inner!(SubU, expr),
            Node::AddU { expr } => inner!(AddU, expr),
            Node::Not { expr } => inner!(Not, expr),
            Node::IfElse { cond, then, _else } => {
                let (cond, m_cond) = modify!(cond);
                let (then, m_then) = modify!(then);
                let (_else, m_else) = optional!(_else);
                Ok((
                    AST { node: Node::IfElse { cond, then, _else }, ..ast.clone() },
                    m_cond || m_then || m_else
                ))
            }
            Node::Match { cond, cases } => {
                let (cond, m_cond) = modify!(cond);
                let (cases, m_cases) = vec_recursion!(cases);
                Ok((AST { node: Node::Match { cond, cases }, ..ast.clone() }, m_cond || m_cases))
            }
            Node::Case { cond, body } => {
                let (cond, m_cond) = modify!(cond);
                let (body, m_body) = modify!(body);
                Ok((AST { node: Node::Case { cond, body }, ..ast.clone() }, m_cond || m_body))
            }
            Node::For { expr, col, body } => {
                let (expr, m_expr) = modify!(expr);
                let (col, m_col) = modify!(col);
                let (body, m_body) = modify!(body);
                Ok((
                    AST { node: Node::For { expr, col, body }, ..ast.clone() },
                    m_expr || m_col || m_body
                ))
            }
            Node::In { left, right } => inner!(In, left, right),
            Node::Step { amount } => {
                let (amount, modified) = modify!(amount);
                Ok((AST { node: Node::Step { amount }, ..ast.clone() }, modified))
            }
            Node::While { cond, body } => {
                let (cond, m_cond) = modify!(cond);
                let (body, m_body) = modify!(body);
                Ok((AST { node: Node::While { cond, body }, ..ast.clone() }, m_cond || m_body))
            }
            Node::Break => Ok((ast.clone(), false)),
            Node::Continue => Ok((ast.clone(), false)),
            Node::Return { expr } => inner!(Return, expr),
            Node::ReturnEmpty => Ok((ast.clone(), false)),
            Node::Underscore => Ok((ast.clone(), false)),
            Node::Undefined => Ok((ast.clone(), false)),
            Node::Pass => Ok((ast.clone(), false)),
            Node::Question { left, right } => inner!(Question, left, right),
            Node::QuestionOp { expr } => inner!(QuestionOp, expr),
            Node::Print { expr } => inner!(Print, expr),
            Node::Comment { .. } => Ok((ast.clone(), false))
        }
    }
}
