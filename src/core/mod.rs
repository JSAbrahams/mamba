use crate::core::construct::Core;

pub mod construct;

/// Convert [Core](mamba::core::construct::Core) to a String which represent
/// python source code.
///
/// Takes [Core](mamba::core::construct::Core) nodes as-is, meaning that this
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
///     then:  Box::from(Core::Str { string: String::from("b") }),
///     el: Box::from(Core::Str { string: String::from("c") })
/// };
///
/// assert_eq!(to_source(&core_node), "if a:\n    \"b\"\nelse:\n    \"c\"\n");
/// ```
pub fn to_source(core: &Core) -> String { format!("{}\n", to_py(core, 0)) }

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::FromImport { from, import } =>
            format!("from {} {}", to_py(from, ind), to_py(import, ind)),
        Core::Import { imports } => format!("import {}", comma_delimited(imports, ind)),
        Core::ImportAs { imports, alias } =>
            format!("import {} as {}", comma_delimited(imports, ind), comma_delimited(alias, ind)),

        Core::Id { lit } => lit.clone(),
        Core::Type { lit, generics } =>
            if generics.is_empty() {
                lit.clone()
            } else {
                format!("{}[{}]", lit, comma_delimited(generics, ind))
            },
        Core::ExpressionType { expr, ty } => format!("{}: {}", to_py(expr, ind), to_py(ty, ind)),
        Core::DocStr { string } => format!("\"\"\"{}\"\"\"", string),
        Core::Str { string } => format!("\"{}\"", string),
        Core::FStr { string } => format!("f\"{}\"", string),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } => format!("({} * 10 ** {})", num, exp),
        Core::Float { float } => float.clone(),
        Core::Bool { boolean } => String::from(if *boolean { "True" } else { "False" }),

        Core::FunDef { id, arg, ty, body, .. } => {
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
                    other => String::from(other)
                },
                other => panic!("Not a valid identifier for a function: {:?}", other)
            };

            format!(
                "def {}({}){}:{}\n",
                name,
                comma_delimited(arg, ind),
                if let Some(ret_ty) = ty {
                    format!(" -> {}", to_py(ret_ty.as_ref(), ind))
                } else {
                    String::new()
                },
                newline_if_body(body, ind)
            )
        }

        Core::Assign { left, right } => format!("{} = {}", to_py(left, ind), to_py(right, ind)),
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

        Core::PropertyCall { object, property } =>
            format!("{}.{}", to_py(object, ind), to_py(property, ind)),
        Core::FunctionCall { function, args } =>
            format!("{}({})", to_py(function, ind), comma_delimited(args, ind)),

        Core::Tuple { elements } => format!("({})", comma_delimited(elements, ind)),
        Core::Set { elements } => format!("{{{}}}", comma_delimited(elements, ind)),
        Core::List { elements } => format!("[{}]", comma_delimited(elements, ind)),
        Core::KeyValue { key, value } => format!("{}: {}", to_py(key, ind), to_py(value, ind)),
        Core::Dictionary { expr, cases } => format!(
            "{{\n{}\n{}}}[{}]",
            newline_comma_delimited(cases, ind + 1),
            indent(ind),
            to_py(expr, ind)
        ),
        Core::DefaultDictionary { expr, cases, default } => format!(
            "defaultdict({}, {{\n{}\n{}}})[{}]",
            to_py(default, ind),
            newline_comma_delimited(cases, ind + 1),
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
            "for {} in {}:{}",
            to_py(expr.as_ref(), ind),
            to_py(col.as_ref(), ind),
            newline_if_body(body, ind)
        ),
        Core::In { left, right } => format! {"{} in {}", to_py(left, ind), to_py(right, ind)},
        Core::Range { from, to, step } => format!(
            "range({}, {}, {})",
            to_py(from.as_ref(), ind),
            to_py(to.as_ref(), ind),
            to_py(step.as_ref(), ind),
        ),
        Core::If { cond, then } =>
            format!("if {}:{}", to_py(cond.as_ref(), ind), newline_if_body(then, ind)),
        Core::IfElse { cond, then, el } => format!(
            "if {}:{}\n{}else:{}",
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
        Core::While { cond, body } =>
            format!("while {}:{}", to_py(cond.as_ref(), ind), newline_if_body(body, ind)),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::ClassDef { name, parent_names, body } => format!(
            "class {}{}:\n{}\n",
            to_py(name, ind),
            if parent_names.is_empty() {
                String::new()
            } else {
                format!("({})", comma_delimited(parent_names, ind))
            },
            to_py(body, ind + 1)
        ),

        Core::Pass => String::from("pass"),
        Core::None => String::from("None"),
        Core::Empty => String::new(),
        Core::Comment { comment } => format!("#{}", comment),

        Core::With { resource, expr } =>
            format!("with {}:{}", to_py(resource, ind), newline_if_body(expr, ind)),
        Core::WithAs { resource, alias, expr } => format!(
            "with {} as {}:{}",
            to_py(resource, ind),
            to_py(alias, ind),
            newline_if_body(expr, ind)
        ),

        Core::TryExcept { setup, attempt, except } => format!(
            "{}try:{}\n{}",
            if let Some(setup) = setup {
                format!("{}\n{}", to_py(setup, ind), indent(ind))
            } else {
                String::from("")
            },
            newline_if_body(attempt, ind + 1),
            newline_delimited(except, ind)
        ),
        Core::Except { id, class, body } => format!(
            "except {} as {}:{}",
            if let Some(class) = class { to_py(class, ind) } else { String::from("Exception") },
            to_py(id, ind),
            newline_if_body(body, ind)
        ),

        Core::Raise { error } => format!("raise {}", to_py(error, ind))
    }
}

fn indent(amount: usize) -> String { " ".repeat(4 * amount) }

fn newline_if_body(core: &Core, ind: usize) -> String {
    match core {
        Core::Block { .. } => format!("\n{}", to_py(core, ind + 1)),
        _ => format!("\n{}{}", indent(ind + 1), to_py(core, ind + 1))
    }
}

fn newline_delimited(items: &[Core], ind: usize) -> String {
    let mut string = String::new();
    items
        .iter()
        .for_each(|item| string.push_str(&format!("{}{}\n", indent(ind), to_py(item, ind))));
    String::from(string.trim_end())
}

fn newline_comma_delimited(items: &[Core], ind: usize) -> String {
    let mut string = String::new();
    items
        .iter()
        .for_each(|item| string.push_str(&format!("{}{},\n", indent(ind), to_py(item, ind))));
    String::from(string.trim_end())
}

fn comma_delimited(items: &[Core], ind: usize) -> String {
    let mut string = String::new();
    items.iter().for_each(|item| string.push_str(&format!("{}, ", to_py(item, ind))));
    if string.len() > 2 {
        string.remove(string.len() - 2);
    }
    String::from(string.trim_end())
}
