use std::fmt::Write;

use crate::generate::ast::node::Core;

pub mod node;

pub const IND_SPACES: usize = 4;

impl Core {
    /// Convert [Core](mamba::generate.ast::construct::Core) to a String which represent
    /// python source code.
    ///
    /// Takes [Core](mamba::generate.ast::construct::Core) nodes as-is, meaning that this
    /// should never panic, unless a certain generate.ast construct can still not be
    /// converted.
    ///
    /// # Examples
    ///
    /// Writing a Return statement:
    ///
    /// ```
    /// # use mamba::generate::ast::node::Core;
    /// let core_node = Core::Return { expr: Box::from(Core::None) };
    /// let py_source = core_node.to_source();
    ///
    /// assert_eq!(py_source, "return None\n");
    /// ```
    ///
    /// Writing an If statement:
    ///
    /// ```
    /// # use mamba::generate::ast::node::Core;
    /// let core_node = Core::IfElse {
    ///  cond:  Box::from(Core::Id { lit: String::from("a") }),
    ///  then:  Box::from(Core::Str { string: String::from("b") }),
    ///  el: Box::from(Core::Str { string: String::from("c") })
    /// };
    ///
    /// assert_eq!(core_node.to_source(), "if a: \n    \"b\"\nelse: \n    \"c\"\n");
    /// ```
    pub fn to_source(&self) -> String {
        format!("{}\n", to_py(self, 0))
    }
}

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::Import { from, import, alias } => format!(
            "{}import {}{}",
            if let Some(from) = from {
                format!("from {} ", to_py(from, ind))
            } else {
                String::from("")
            },
            comma_delimited(import, ind),
            if !alias.is_empty() {
                format!(" as {}", comma_delimited(alias, ind))
            } else {
                String::from("")
            }
        ),
        Core::Id { lit } => lit.clone(),
        Core::Type { lit, generics } => {
            if generics.is_empty() {
                lit.clone()
            } else {
                format!("{}[{}]", lit, comma_delimited(generics, ind))
            }
        }
        Core::ExpressionType { expr, ty } => format!("{}: {}", to_py(expr, ind), to_py(ty, ind)),
        Core::DocStr { string } => format!("\"\"\"{}\"\"\"", string),
        Core::Str { string } => format!("\"{}\"", string),
        Core::FStr { string } => format!("f\"{}\"", string),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } => format!("({} * 10 ** {})", num, exp),
        Core::Float { float } => float.clone(),
        Core::Bool { boolean } => String::from(if *boolean { "True" } else { "False" }),

        Core::FunDefOp { op, arg, ty, body } => {
            let id = format!("{}", op);
            let dec = vec![];
            to_py(
                &Core::FunDef { dec, id, arg: arg.clone(), ty: ty.clone(), body: body.clone() },
                ind,
            )
        }
        Core::FunDef { dec, id, arg, ty, body } => {
            let dec: Vec<Core> = dec.iter().map(|d| Core::Id { lit: format!("@{}", d) }).collect();
            format!(
                "{}{}def {}({}){}: {}\n",
                if dec.is_empty() { String::from("") } else { newline_delimited(&dec, ind - 1) },
                if dec.is_empty() { String::from("") } else { indent(ind) },
                id,
                comma_delimited(arg, ind),
                if let Some(ret_ty) = ty {
                    format!(" -> {}", to_py(ret_ty.as_ref(), ind))
                } else {
                    String::new()
                },
                newline_if_body(body, ind)
            )
        }

        Core::Assign { left, right, op } => {
            format!("{} {} {}", to_py(left, ind), op, to_py(right, ind))
        }
        Core::VarDef { var, expr, ty } => format!(
            "{}{} = {}",
            to_py(var, ind),
            if let Some(ty) = ty { format!(": {}", to_py(ty, ind)) } else { String::new() },
            if let Some(expr) = expr { to_py(expr, ind) } else { String::from("None") }
        ),

        Core::FunArg { vararg, var, ty, default } => format!(
            "{}{}{}{}",
            if *vararg { "*" } else { "" },
            to_py(var, ind),
            if let Some(ty) = ty { format!(": {}", to_py(ty, ind)) } else { String::new() },
            if let Some(default) = default {
                format!(" = {}", to_py(default, ind))
            } else {
                String::new()
            }
        ),

        Core::AnonFun { args, body } => format!(
            "lambda{}: {}",
            if args.is_empty() {
                String::new()
            } else {
                format!(" {}", comma_delimited(args, ind))
            },
            to_py(body, ind)
        ),

        Core::Block { statements } => newline_delimited(statements, ind),

        Core::PropertyCall { object, property } => {
            format!("{}.{}", to_py(object, ind), to_py(property, ind))
        }
        Core::FunctionCall { function, args } => {
            format!("{}({})", to_py(function, ind), comma_delimited(args, ind))
        }

        Core::Tuple { elements } => format!("({})", comma_delimited(elements, ind)),
        Core::TupleLiteral { elements } => comma_delimited(elements, ind),
        Core::Set { elements } => format!("{{{}}}", comma_delimited(elements, ind)),
        Core::List { elements } => format!("[{}]", comma_delimited(elements, ind)),

        Core::Match { expr, cases } => {
            format!("match {}:\n{}", to_py(expr, ind), newline_delimited(cases, ind + 1))
        }
        Core::Case { expr, body } => {
            format!("case {}: {}", to_py(expr, ind), newline_if_body(body, ind))
        }
        Core::KeyValue { key, value } => format!("{}: {}", to_py(key, ind), to_py(value, ind)),

        Core::UnderScore => String::from("_"),

        Core::Ge { left, right } => {
            format!("{} > {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Geq { left, right } => {
            format!("{} >= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Le { left, right } => {
            format!("{} < {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Leq { left, right } => {
            format!("{} <= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }

        Core::Not { expr } => format!("not {}", to_py(expr.as_ref(), ind)),
        Core::And { left, right } => {
            format!("{} and {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Or { left, right } => {
            format!("{} or {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Is { left, right } => {
            format!("{} is {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::IsN { left, right } => {
            format!("{} is not {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Eq { left, right } => {
            format!("{} == {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Neq { left, right } => {
            format!("{} != {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::IsA { left, right } => {
            format!("isinstance({},{})", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }

        Core::AddU { expr } => format!("+{}", to_py(expr, ind)),
        Core::Add { left, right } => {
            format!("{} + {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::SubU { expr } => format!("-{}", to_py(expr, ind)),
        Core::Sub { left, right } => {
            format!("{} - {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Mul { left, right } => {
            format!("{} * {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Div { left, right } => {
            format!("{} / {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::FDiv { left, right } => {
            format!("{} // {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Pow { left, right } => {
            format!("{} ** {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Mod { left, right } => {
            format!("{} % {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::Sqrt { expr } => format!("math.sqrt({})", to_py(expr.as_ref(), ind)),

        Core::BAnd { left, right } => {
            format!("{} & {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::BOr { left, right } => {
            format!("{} | {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::BXOr { left, right } => {
            format!("{} ^ {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::BOneCmpl { expr } => format!("~{}", to_py(expr, ind)),
        Core::BLShift { left, right } => {
            format!("{} << {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }
        Core::BRShift { left, right } => {
            format!("{} >> {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind))
        }

        Core::Return { expr } => format!("return {}", to_py(expr.as_ref(), ind)),

        Core::For { expr, col, body } => format!(
            "for {} in {}:{}",
            to_py(expr.as_ref(), ind),
            to_py(col.as_ref(), ind),
            newline_if_body(body, ind)
        ),
        Core::In { left, right } => format! {"{} in {}", to_py(left, ind), to_py(right, ind)},
        Core::Index { item, range } => format!("{}[{}]", to_py(item, ind), to_py(range, ind)),
        Core::If { cond, then } => {
            format!("if {}:{}", to_py(cond.as_ref(), ind), newline_if_body(then, ind))
        }
        Core::IfElse { cond, then, el } => format!(
            "if {}: {}\n{}else: {}",
            to_py(cond.as_ref(), ind),
            newline_if_body(then, ind),
            indent(ind),
            newline_if_body(el, ind)
        ),
        Core::Ternary { cond, then, el } => format!(
            "{} if {} else {}",
            to_py(then.as_ref(), ind),
            to_py(cond.as_ref(), ind + 1),
            to_py(el.as_ref(), ind + 1)
        ),
        Core::While { cond, body } => {
            format!("while {}:{}", to_py(cond.as_ref(), ind), newline_if_body(body, ind))
        }
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::ClassDef { name, parent_names, body } => format!(
            "class {}{}: {}\n",
            to_py(name, ind),
            if parent_names.is_empty() {
                String::new()
            } else {
                format!("({})", comma_delimited(parent_names, ind))
            },
            newline_if_body(body, ind)
        ),

        Core::Pass => String::from("pass"),
        Core::None => String::from("None"),
        Core::Empty => String::new(),

        Core::With { resource, expr } => {
            format!("with {}: {}", to_py(resource, ind), newline_if_body(expr, ind))
        }
        Core::WithAs { resource, alias, expr } => format!(
            "with {} as {}: {}",
            to_py(resource, ind),
            to_py(alias, ind),
            newline_if_body(expr, ind)
        ),

        Core::TryExcept { setup, attempt, except } => format!(
            "{}try: {}\n{}",
            if let Some(setup) = setup {
                format!("{}\n{}", to_py(setup, ind), indent(ind))
            } else {
                String::from("")
            },
            newline_if_body(attempt, ind),
            newline_delimited(except, ind)
        ),
        Core::Except { id, class, body } => format!(
            "except {} as {}: {}",
            if let Some(class) = class { to_py(class, ind) } else { String::from("Exception") },
            to_py(id, ind),
            newline_if_body(body, ind)
        ),

        Core::Raise { error } => format!("raise {}", to_py(error, ind)),
    }
}

fn indent(amount: usize) -> String {
    " ".repeat(IND_SPACES * amount)
}

fn newline_if_body(core: &Core, ind: usize) -> String {
    match core {
        Core::Block { .. } => format!("\n{}", to_py(core, ind + 1)),
        _ => format!("\n{}{}", indent(ind + 1), to_py(core, ind + 1)),
    }
}

fn newline_delimited(items: &[Core], ind: usize) -> String {
    let mut s = String::new();
    items.iter().for_each(|item| writeln!(s, "{}{}", indent(ind), to_py(item, ind)).unwrap());
    s
}

fn comma_delimited(items: &[Core], ind: usize) -> String {
    let mut s = String::new();
    items.iter().for_each(|item| write!(s, "{}, ", to_py(item, ind)).unwrap());

    if s.len() > 2 {
        s.remove(s.len() - 2);
    }
    String::from(s.trim_end())
}
