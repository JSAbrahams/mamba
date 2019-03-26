use crate::core::construct::Core;

pub mod construct;

pub fn to_py_source(core: &Core) -> String { format!("{}\n", to_py(&core, 0)) }

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::Id { lit } => lit.clone(),
        Core::Str { _str } => format!("\"{}\"", _str),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } =>
            format!("Enum({}, {})", num, if exp.is_empty() { "0" } else { exp }),
        Core::Float { float } => float.clone(),
        Core::Bool { _bool } => String::from(if *_bool { "True" } else { "False" }),

        Core::FunDef { private, id, args, body } => {
            let name = match id.as_ref() {
                Core::GeOp => String::from("__gt__"),
                Core::GeqOp => String::from("__ge__"),
                Core::LeOp => String::from("__lt__"),
                Core::LeqOp => String::from("__le__"),

                Core::EqOp => String::from("__eq__"),
                Core::NeqOp => String::from("__ne__"),

                Core::AddOp => String::from("__add__"),
                Core::SubOp => String::from("__sub__"),
                Core::MulOp => String::from("__mul__"),
                Core::ModOp => String::from("__mod__"),
                Core::DivOp => String::from("__truediv__"),

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
                _ => panic!()
            };

            format!(
                "\n{}{}({}): {}",
                indent(ind),
                name,
                comma_delimited(args, ind),
                to_py(body.as_ref(), ind + 1)
            )
        }

        Core::Assign { left, right } =>
            format!("{} = {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::VarDef { private, id, right } => format!(
            "{}{} = {}",
            if *private { "_" } else { "" },
            to_py(id.as_ref(), ind),
            to_py(right.as_ref(), ind)
        ),

        Core::FunArg { vararg, id } =>
            format!("{}{}", if *vararg { "*" } else { "" }, to_py(id.as_ref(), ind)),

        Core::AnonFun { args, body } =>
            format!("lambda {} : {}", comma_delimited(args, ind), to_py(body, ind)),

        Core::Block { statements } => format!("\n{}", newline_delimited(statements, ind)),

        Core::PropertyCall { object, property } => format!("{}.{}", to_py(object, ind), property),
        Core::MethodCall { object, method, args } => match object.as_ref() {
            Core::Empty => format!("{}({})", method, comma_delimited(args, ind)),
            other => format!("{}.{}({})", to_py(other, ind), method, comma_delimited(args, ind))
        },

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
            format!("{} && {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Or { left, right } =>
            format!("{} || {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Is { left, right } =>
            format!("{} is {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::IsN { left, right } =>
            format!("{} is not {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Eq { left, right } =>
            format!("{} == {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Neq { left, right } =>
            format!("{} != {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::IsA { left, right } =>
            format!("isintance({},{})", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

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
        Core::Pow { left, right } =>
            format!("{} ** {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Mod { left, right } =>
            format!("{} % {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Return { expr } => format!("return {}", to_py(expr.as_ref(), ind)),
        Core::Print { expr } => format!("print({})", to_py(expr.as_ref(), ind)),

        Core::For { exprs, collection, body } => format!(
            "for {} in {}: {}",
            comma_delimited(exprs.as_ref(), ind),
            to_py(collection.as_ref(), ind),
            to_py(body.as_ref(), ind + 1)
        ),
        Core::If { cond, then } =>
            format!("if {}: {}", comma_delimited(cond.as_ref(), ind), to_py(then.as_ref(), ind + 1)),
        Core::IfElse { cond, then, _else } => format!(
            "if {}: {}\n{}else: {}",
            comma_delimited(cond.as_ref(), ind),
            to_py(then.as_ref(), ind + 1),
            indent(ind),
            to_py(_else.as_ref(), ind + 1)
        ),
        Core::While { cond, body } => format!(
            "while {}: {}",
            comma_delimited(cond.as_ref(), ind),
            to_py(body.as_ref(), ind + 1)
        ),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::ClassDef { name, parents, definitions, .. } => format!(
            "class {}({}):\n{}\n",
            to_py(name, ind),
            comma_delimited(parents, ind),
            newline_delimited(definitions, ind + 1)
        ),

        Core::Pass => String::from("pass"),
        Core::Undefined => String::from("None"),
        Core::Empty => String::new(),

        other => panic!("To python not implemented yet for: {:?}", other)
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
