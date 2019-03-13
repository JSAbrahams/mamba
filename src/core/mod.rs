use crate::core::core::Core;

pub mod core;

pub fn to_py_source(core: Core) -> String {
    to_py(&core, 0)
}

// TODO add indentation when newlining inside format

fn to_py(core: &Core, ind: usize) -> String {
    match core {
        Core::Id { lit } => lit.clone(),
        Core::Str { _str } => _str.clone(),
        Core::Int { int } => int.clone(),
        Core::ENum { num, exp } => format!("Enum({},{})", num, exp),
        Core::Float { float } => float.clone(),
        Core::Bool { _bool } => String::from(if *_bool { "True" } else { "False" }),

        Core::Init { args, body } =>
            format!("__init__({}):{}", comma_delimited(args.as_ref(), ind), to_py(body.as_ref(), ind)),
        Core::FunDef { id, args, body } => {
            let name = String::from(match id.as_ref() {
                Core::GeOp => "__gt__",
                Core::GeqOp => "__ge__",
                Core::LeOp => "__lt__",
                Core::LeqOp => "__le__",

                Core::EqOp => "__eq__",
                Core::NeqOp => "__ne__",

                Core::AddOp => "__add__",
                Core::SubOp => "__sub__",
                Core::MulOp => "__mul__",
                Core::ModOp => "__mod__",
                Core::DivOp => "__truediv__",

                Core::Id { ref lit } => match lit.as_str() {
                    "size" => "__size__",
                    other => other
                }
                _ => panic!()
            });

            format!("{}({}):{}", name, comma_delimited(args.as_ref(), ind), to_py(body.as_ref(), ind))
        }

        Core::Assign { left, right } => format!("{} = {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::VarDef { id, right } => format!("{} = {}", to_py(id.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::FunArg { vararg, id } => format!("{}{}", if *vararg { "*" } else { "" }, to_py(id.as_ref(), ind)),

        Core::Block { statements } => {
            let mut block = String::from("\n");
            for statement in statements {
                block.push_str(indent(ind).as_ref());
                block.push_str(to_py(&statement, ind).as_ref());
                block.push('\n');
            }
            block
        }

        Core::FunctionCall { namespace, function, args, } =>
            format!("{}.{}({})", namespace, function, comma_delimited(args.as_ref(), ind)),
        Core::MethodCall { object, method, args, } =>
            format!("{}.{}({})", to_py(object.as_ref(), ind), method, comma_delimited(args.as_ref(), ind)),

        Core::Tuple { elements } => format!("({})", comma_delimited(elements.as_ref(), ind)),
        Core::Set { elements } => format!("{{{}}}", comma_delimited(elements.as_ref(), ind)),
        Core::List { elements } => format!("[{}]", comma_delimited(elements.as_ref(), ind)),

        Core::Ge { left, right } => format!("{} > {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Geq { left, right } => format!("{} >= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Le { left, right } => format!("{} < {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Leq { left, right } => format!("{} <= {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Not { expr } => format!("not {}", to_py(expr.as_ref(), ind)),
        Core::And { left, right } => format!("{} && {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Or { left, right } => format!("{} || {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Is { left, right } => format!("{} is {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Eq { left, right } => format!("{} == {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Neq { left, right } => format!("{} != {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::IsA { left, right } => format!("isintance({},{})", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Add { left, right } => format!("{} + {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Sub { left, right } => format!("{} - {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Mul { left, right } => format!("{} * {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Div { left, right } => format!("{} / {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Pow { left, right } => format!("{} ** {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),
        Core::Mod { left, right } => format!("{} % {}", to_py(left.as_ref(), ind), to_py(right.as_ref(), ind)),

        Core::Return { expr } => format!("return {}", to_py(expr.as_ref(), ind)),
        Core::Print { expr } => format!("print({})", to_py(expr.as_ref(), ind)),

        Core::For { expr, coll, body } =>
            format!("for {} in {}:{}", to_py(expr.as_ref(), ind), to_py(coll.as_ref(), ind),
                    to_py(body.as_ref(), ind)),
        Core::If { cond, then } =>
            format!("if {}:{}", to_py(cond.as_ref(), ind), to_py(then.as_ref(), ind)),
        Core::IfElse { cond, then, _else } =>
            format!("if {}:{}\n{}else:\n{}", to_py(cond.as_ref(), ind),
                    to_py(then.as_ref(), ind),
                    indent(ind), to_py(_else.as_ref(), ind)),
        Core::While { cond, body } =>
            format!("while {}: {}", to_py(cond.as_ref(), ind),
                    to_py(body.as_ref(), ind + 1)),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::Undefined => String::from("None"),
        Core::Empty => String::new(),

        other => panic!("Not implemented yet: {:?}", other)
    }
}

fn indent(amount: usize) -> String { " ".repeat(4 * amount) }

fn comma_delimited(items: &Vec<Core>, ind: usize) -> String {
    if items.is_empty() { return String::new(); }

    let mut result = String::new();
    let mut pos = 0;
    for item in items {
        result.push_str(to_py(item, ind).as_ref());

        pos += 1;
        if pos < items.len() { result.push(','); }
    }

    result
}
