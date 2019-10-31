use crate::core::construct::Core;
use std::ops::Deref;

pub mod construct;

/// Convert [Core](crate::core::construct::Core) to a String which represent
/// python source code.
///
/// Takes [Core](crate::core::construct::Core) nodes as-is, meaning that this
/// should never panic, unless a certain core construct can still not be
/// converted.
///
/// # Examples
///
/// Writing a Return statement:
///
/// ```
/// # use mamba::core::construct::Core;
/// # use mamba::core::to_source;
/// let core_node = Core::Return { expr: Box::from(Core::None) };
/// let py_source = to_source(&core_node);
///
/// assert_eq!(py_source, "return None\n");
/// ```
///
/// Writing an If statement:
///
/// ```
/// # use mamba::core::construct::Core;
/// # use mamba::core::to_source;
/// let core_node = Core::IfElse {
///     cond:  Box::from(Core::Id { lit: String::from("a") }),
///     then:  Box::from(Core::Str { _str: String::from("b") }),
///     _else: Box::from(Core::Str { _str: String::from("c") })
/// };
///
/// assert_eq!(to_source(&core_node), "if a:\n    \"b\"\nelse:\n    \"c\"\n");
/// ```
pub fn to_source(core: &Core) -> String { format!("{}\n", to_py(&core, 0)) }

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::FromImport { from, import } =>
            format!("from {} {}", to_py(from, ind), to_py(import, ind)),
        Core::Import { imports } => format!("import {}", comma_delimited(imports, ind)),
        Core::ImportAs { imports, _as } =>
            format!("import {} as {}", comma_delimited(imports, ind), comma_delimited(_as, ind)),

        Core::Id { lit } => format!("{}", lit),
        Core::Type { lit, generics } =>
            if generics.is_empty() {
                format!("{}", lit)
            } else {
                format!("{}[{}]", lit, comma_delimited(generics, ind))
            },
        Core::IdType { lit, ty } => format!("{}: {}", lit, to_py(ty, ind)),
        Core::Str { _str } => format!("\"{}\"", _str),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } => format!("({} * 10 ** {})", num, exp),
        Core::Float { float } => float.clone(),
        Core::Bool { _bool } => String::from(if *_bool { "True" } else { "False" }),

        Core::FunDef { private, id, args, ret_ty, body } => {
            let name = match id.as_ref() {
                Core::GeOp => String::from("__gt__"),
                Core::GeqOp => String::from("__ge__"),
                Core::LeOp => String::from("__lt__"),
                Core::LeqOp => String::from("__le__"),

                Core::EqOp => String::from("__eq__"),
                Core::NeqOp => String::from("__ne__"),

                Core::AddOp => String::from("__add__"),
                Core::SubOp => String::from("__sub__"),
                Core::PowOp => String::from("__pow__"),
                Core::MulOp => String::from("__mul__"),
                Core::ModOp => String::from("__mod__"),
                Core::DivOp => String::from("__truediv__"),
                Core::FDivOp => String::from("__floordiv__"),

                Core::Id { ref lit, .. } => match lit.as_str() {
                    "size" => String::from("__size__"),
                    "init" => String::from("__init__"),
                    other =>
                        if *private {
                            format!("_{}", other)
                        } else {
                            String::from(other)
                        },
                },
                other => panic!("Not a valid identifier for a function: {:?}", other)
            };

            format!(
                "def {}({}){}:{}\n",
                name,
                comma_delimited(args, ind),
                if let Some(ret_ty) = ret_ty {
                    format!(" -> {}", to_py(ret_ty.as_ref(), ind))
                } else {
                    String::new()
                },
                match body.deref() {
                    Core::Block { .. } => format!("\n{}", to_py(body.as_ref(), ind + 1)),
                    _ => format!(" {}", to_py(body.as_ref(), ind + 1))
                }
            )
        }

        Core::Assign { left, right } => format!("{} = {}", to_py(left.as_ref(), ind), {
            let right = to_py(right.as_ref(), ind);
            if right.is_empty() {
                String::from("None")
            } else {
                right
            }
        }),
        Core::VarDef { private, id, right } =>
            format!("{}{} = {}", if *private { "_" } else { "" }, to_py(id.as_ref(), ind), {
                let right = to_py(right.as_ref(), ind);
                if right.is_empty() {
                    String::from("None")
                } else {
                    right
                }
            }),

        Core::FunArg { vararg, id, default } => format!(
            "{}{}{}",
            if *vararg { "*" } else { "" },
            to_py(id.as_ref(), ind),
            if **default == Core::Empty {
                String::new()
            } else {
                format!(" = {}", to_py(default.as_ref(), ind))
            }
        ),

        Core::AnonFun { args, body } =>
            format!("lambda {}: {}", comma_delimited(args, ind), to_py(body, ind)),

        Core::Block { statements } => format!("{}", newline_delimited(statements, ind)),

        Core::PropertyCall { object, property } =>
            format!("{}.{}", to_py(object, ind), to_py(property, ind)),
        Core::FunctionCall { function, args } =>
            format!("{}({})", to_py(function, ind), comma_delimited(args, ind)),

        Core::Tuple { elements } => format!("({})", comma_delimited(elements, ind)),
        Core::Set { elements } => format!("{{{}}}", comma_delimited(elements, ind)),
        Core::List { elements } => format!("[{}]", comma_delimited(elements, ind)),
        Core::KeyValue { key, value } => format!("{} : {}", to_py(key, ind), to_py(value, ind)),
        Core::Dictionary { expr, cases } => format!(
            "{{\n{}{}\n{}}}[{}]",
            indent(ind + 1),
            comma_delimited(cases, ind + 1),
            indent(ind),
            to_py(expr, ind)
        ),
        Core::DefaultDictionary { expr, cases, default } => format!(
            "defaultdict({}, {{\n{}\n{}}})[{}]",
            to_py(default, ind),
            comma_delimited(cases, ind + 1),
            indent(ind),
            to_py(expr, ind)
        ),

        Core::GeOp => String::from(">"),
        Core::GeqOp => String::from(">="),
        Core::LeOp => String::from("<"),
        Core::LeqOp => String::from("<="),
        Core::EqOp => String::from("="),
        Core::NeqOp => String::from("/="),
        Core::AddOp => String::from("+"),
        Core::SubOp => String::from("-"),
        Core::MulOp => String::from("*"),
        Core::DivOp => String::from("/"),
        Core::ModOp => String::from("%"),
        Core::PowOp => String::from("**"),
        Core::FDivOp => String::from("//"),
        Core::UnderScore => String::from("_"),

        Core::Ge { left, right } =>
            format!("{} > {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Geq { left, right } =>
            format!("{} >= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Le { left, right } =>
            format!("{} < {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Leq { left, right } =>
            format!("{} <= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Not { expr } => format!("not {}", to_py(expr.as_ref(), ind)),
        Core::And { left, right } =>
            format!("{} and {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Or { left, right } =>
            format!("{} or {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Is { left, right } =>
            format!("{} is {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::IsN { left, right } =>
            format!("{} is not {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Eq { left, right } =>
            format!("{} == {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Neq { left, right } =>
            format!("{} != {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::IsA { left, right } =>
            format!("isinstance({},{})", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::AddU { expr } => format!("+{}", to_py(expr, ind)),
        Core::Add { left, right } =>
            format!("{} + {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::SubU { expr } => format!("-{}", to_py(expr, ind)),
        Core::Sub { left, right } =>
            format!("{} - {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Mul { left, right } =>
            format!("{} * {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Div { left, right } =>
            format!("{} / {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::FDiv { left, right } =>
            format!("{} // {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Pow { left, right } =>
            format!("{} ** {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Mod { left, right } =>
            format!("{} % {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Sqrt { expr } => format!("math.sqrt({})", to_py(expr.as_ref(), ind)),

        Core::BAnd { left, right } =>
            format!("{} & {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::BOr { left, right } =>
            format!("{} | {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::BXOr { left, right } =>
            format!("{} ^ {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::BOneCmpl { expr } => format!("~{}", to_py(expr, ind)),
        Core::BLShift { left, right } =>
            format!("{} << {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::BRShift { left, right } =>
            format!("{} >> {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Return { expr } => format!("return {}", to_py(expr.as_ref(), ind)),
        Core::Print { expr } => format!("print({})", to_py(expr.as_ref(), ind)),

        Core::For { expr, col, body } => format!(
            "for {} in {}:\n{}{}",
            to_py(expr.as_ref(), ind),
            to_py(col.as_ref(), ind),
            indent(ind + 1),
            to_py(body.as_ref(), ind + 1)
        ),
        Core::In { left, right } => format! {"{} in {}", to_py(left, ind), to_py(right, ind)},
        Core::Range { from, to, step } => format!(
            "range({}, {}, {})",
            to_py(from.as_ref(), ind),
            to_py(to.as_ref(), ind),
            to_py(step.as_ref(), ind),
        ),
        Core::If { cond, then } => format!(
            "if {}:\n{}{}",
            to_py(cond.as_ref(), ind),
            indent(ind + 1),
            to_py(then.as_ref(), ind + 1)
        ),
        Core::IfElse { cond, then, _else } => format!(
            "if {}:\n{}{}\n{}else:\n{}{}",
            to_py(cond.as_ref(), ind),
            indent(ind + 1),
            to_py(then.as_ref(), ind + 1),
            indent(ind),
            indent(ind + 1),
            to_py(_else.as_ref(), ind + 1)
        ),
        Core::Ternary { cond, then, _else } => format!(
            "{} if {} else {}",
            to_py(then.as_ref(), ind),
            to_py(cond.as_ref(), ind + 1),
            to_py(_else.as_ref(), ind + 1)
        ),
        Core::While { cond, body } => format!(
            "while {}:\n{}{}",
            to_py(cond.as_ref(), ind),
            indent(ind + 1),
            to_py(body.as_ref(), ind + 1)
        ),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::ClassDef { name, parents, definitions } => format!(
            "class {}{}:\n{}\n",
            to_py(name, ind),
            if parents.is_empty() {
                String::new()
            } else {
                format!("({})", comma_delimited(parents, ind))
            },
            newline_delimited(definitions, ind + 1)
        ),

        Core::Pass => String::from("pass"),
        Core::None => String::from("None"),
        Core::Empty => String::new(),
        Core::Comment { comment } => format!("#{}", comment),

        Core::With { resource, expr } =>
            format!("with {}:\n{}{}", to_py(resource, ind), indent(ind + 1), to_py(expr, ind + 1)),
        Core::WithAs { resource, _as, expr } => format!(
            "with {} as {}:\n{}{}",
            to_py(resource, ind),
            to_py(_as, ind),
            indent(ind + 1),
            to_py(expr, ind + 1)
        ),

        Core::TryExcept { _try, except } => format!(
            "try:\n{}{}\n{}",
            indent(ind + 1),
            to_py(_try, ind + 1),
            newline_delimited(except, ind)
        ),
        Core::Except { id, class, body } => format!(
            "except {} as {}:\n{}{}\n",
            to_py(class, ind),
            to_py(id, ind),
            indent(ind + 1),
            to_py(body, ind + 1)
        ),
        Core::ExceptNoClass { id, body } => format!(
            "except Exception as {}:\n{}{}\n",
            to_py(id, ind),
            indent(ind + 1),
            to_py(body, ind + 1)
        ),

        Core::Raise { error } => format!("raise {}", to_py(error, ind))
    }
}

fn indent(amount: usize) -> String { " ".repeat(4 * amount) }

fn newline_delimited(items: &[Core], ind: usize) -> String {
    let mut result = String::new();

    for (pos, item) in items.iter().enumerate() {
        result.push_str(indent(ind).as_ref());
        result.push_str(to_py(item, ind).as_ref());

        if pos < items.len() - 1 {
            result.push('\n');
        }
    }

    result
}

fn comma_delimited(items: &[Core], ind: usize) -> String {
    if items.is_empty() {
        return String::new();
    }

    let mut result = String::new();
    for (pos, item) in items.iter().enumerate() {
        result.push_str(to_py(item, ind).as_ref());
        if pos < items.len() - 1 {
            result.push(',');
            result.push(' ');
        }
    }

    result
}
