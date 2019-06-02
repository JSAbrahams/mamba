use crate::core::construct::Core;

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
/// # use mamba::core::to_py_source;
/// let core_node = Core::Return { expr: Box::from(Core::None) };
/// let py_source = to_py_source(&core_node);
///
/// assert_eq!(py_source, "return None\n");
/// ```
///
/// Writing an If statement:
///
/// ```
/// # use mamba::core::construct::Core;
/// # use mamba::core::to_py_source;
/// let core_node = Core::IfElse {
///     cond:  Box::from(Core::Id { lit: String::from("a") }),
///     then:  Box::from(Core::Str { _str: String::from("b") }),
///     _else: Box::from(Core::Str { _str: String::from("c") })
/// };
///
/// assert_eq!(to_py_source(&core_node), "if a:\n    \"b\"\nelse:\n    \"c\"\n");
/// ```
pub fn to_py_source(core: &Core) -> String { format!("{}\n", to_py(&core, 0)) }

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::FromImport { from, import } =>
            format!("from {} {}", to_py(from, ind), to_py(import, ind)),
        Core::Import { import } => format!("import {}", comma_delimited(import, ind)),
        Core::ImportAs { import, _as } =>
            format!("import {} as {}", comma_delimited(import, ind), comma_delimited(_as, ind)),

        Core::File { doc, statements } => format!(
            "{}{}",
            doc.clone().map_or(String::from(""), |doc| format!("\"\"\"{}\"\"\"\n", doc)),
            newline_delimited(statements, ind)
        ),

        Core::Id { lit } => lit.clone(),
        Core::Str { _str } => format!("\"{}\"", _str),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } => format!("({} * 10 ** {})", num, exp),
        Core::Float { float } => float.clone(),
        Core::Bool { _bool } => String::from(if *_bool { "True" } else { "False" }),

        Core::FunDef { private, id, doc, args, body } => {
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

                Core::Id { ref lit } => match lit.as_str() {
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
                "def {}({}):\n{}{}{}",
                name,
                comma_delimited(args, ind),
                doc.clone().map_or(String::from(""), |doc| format!("\"\"\"{}\"\"\"\n", doc)),
                indent(ind + 1),
                to_py(body.as_ref(), ind + 1)
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

        Core::Block { statements } => format!("\n{}", newline_delimited(statements, ind)),

        Core::PropertyCall { object, property } =>
            format!("{}.{}", to_py(object, ind), to_py(property, ind)),
        Core::FunctionCall { function, args } =>
            format!("{}({})", to_py(function, ind), comma_delimited(args, ind)),

        Core::Tuple { elements } => format!("({})", comma_delimited(elements, ind)),
        Core::Set { elements } => format!("{{{}}}", comma_delimited(elements, ind)),
        Core::List { elements } => format!("[{}]", comma_delimited(elements, ind)),

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

        Core::For { expr, body } => format!(
            "for {}:\n{}{}",
            to_py(expr.as_ref(), ind),
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
        Core::While { cond, body } => format!(
            "while {}:\n{}{}",
            to_py(cond.as_ref(), ind),
            indent(ind + 1),
            to_py(body.as_ref(), ind + 1)
        ),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::ClassDef { name, doc, parents, definitions } => format!(
            "class {}({}):\n{}{}\n",
            to_py(name, ind),
            comma_delimited(parents, ind),
            doc.clone().map_or(String::from(""), |doc| format!("\"\"\"{}\"\"\"\n", doc)),
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
            except_unwrap(except, ind)
        ),
        Core::Raise { error } => format!("raise {}", to_py(error, ind)),

        other => panic!("To python not implemented yet for: {:?}", other)
    }
}

fn indent(amount: usize) -> String { " ".repeat(4 * amount) }

fn except_unwrap(items: &[Core], ind: usize) -> String {
    let mut result = String::new();

    for item in items {
        match item {
            Core::Except { id, class, body } => result.push_str(
                format!(
                    "{}except {} as {}:\n{}{}\n",
                    indent(ind),
                    to_py(class, ind),
                    to_py(id, ind),
                    indent(ind + 1),
                    to_py(body, ind + 1)
                )
                .as_ref()
            ),
            other => panic!("Expected two id's but was: {:?}", other)
        }
    }

    result
}

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
