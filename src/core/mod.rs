use crate::core::core::Core;

pub mod core;

pub fn to_py_source(core: Core) -> string {
    to_py(core, 0)
}

pub fn to_py(core: Core, ind: i32) -> String {
    match core {
        Core::Id { id } => id,
        Core::Str { _str } => _str,
        Core::Int { int } => int,
        Core::Float { float } => float,
        Core::Bool { _bool } => String::from(if _bool { "True" } else { "False" }),

        Core::Init { id, args, body } => String::from(Core::Id { lit: String::from("__init__") },
                                                      "(", comma_delimited(args, ind), "):\n",
                                                      to_py(*body, ind)),
        Core::FunDef { id, args, body } => fun_def(id, args, body, ind),

        Core::Assign { left, right } => String::from(to_py(*left, ind), " = ", to_py(*right, ind)),
        Core::VarDef { left, right } => String::from(to_py(*left, ind), " = ", to_py(*right, ind)),

        Core::FunArg { vararg, expr } => String::from(if vararg { "*" } else { "" }, to_py(*expr, ind)),

        Core::Block { exprs } => {
            let mut block = String::new();
            for expr in exprs {
                for _ in 0..(ind + 1) { block.push_str("    ") };
                block.push_str(to_py(*expr, ind + 1));
                block.push('\n');
            }
            block
        }

        Core::FunctionCall { namespace, function, args, } => String::from(namespace, ".", function,
                                                                          "(", comma_delimited(args, ind), ")"),
        Core::MethodCall { namespace, method, args, } => String::from(namespace, ".", method,
                                                                      "(", comma_delimited(args, ind), ")"),

        Core::Tuple { elements } => String::from("(", comma_delimited(elements, ind), ")"),
        Core::Set { elements } => String::from("{", comma_delimited(elements, ind), "}"),
        Core::List { elements } => String::from("[", comma_delimited(elements, ind), "]"),

        Core::Ge { left, right } => String::from(to_py(*left, ind), " > ", to_py(*right, ind)),
        Core::Geq { left, right } => String::from(to_py(*left, ind), " >= ", to_py(*right, ind)),
        Core::Le { left, right } => String::from(to_py(*left, ind), " < ", to_py(*right, ind)),
        Core::Geq { left, right } => String::from(to_py(*left, ind), " <= ", to_py(*right, ind)),

        Core::Not { expr } => String::from("not ", to_py(*expr, ind)),
        Core::And { left, right } => String::from(to_py(*left, ind), " && ", to_py(*right, ind)),
        Core::Or { left, right } => String::from(to_py(*left, ind), " || ", to_py(*right, ind)),
        Core::Is { left, right } => String::from(to_py(*left, ind), " is ", to_py(*right, ind)),
        Core::Eq { left, right } => String::from(to_py(*left, ind), " == ", to_py(*right, ind)),
        Core::Neq { left, right } => String::from(to_py(*left, ind), " != ", to_py(*right, ind)),
        Core::IsA { left, right } => String::from("isinstance(",
                                                  to_py(*left, ind), ", ", to_py(*right, ind), ")"),

        Core::Add { left, right } => String::from(to_py(*left, ind), " + ", to_py(*right, ind)),
        Core::Sub { left, right } => String::from(to_py(*left, ind), " - ", to_py(*right, ind)),
        Core::Mul { left, right } => String::from(to_py(*left, ind), " * ", to_py(*right, ind)),
        Core::Div { left, right } => String::from(to_py(*left, ind), " / ", to_py(*right, ind)),
        Core::Pow { left, right } => String::from(to_py(*left, ind), " ** ", to_py(*right, ind)),
        Core::Mod { left, right } => String::from(to_py(*left, ind), " % ", to_py(*right, ind)),

        Core::Return { expr } => String::from("return ", to_py(*expr, ind)),
        Core::Print { expr } => String::from("print(", to_py(*expr, ind), ")"),

        Core::For { expr, coll, body } => String::from("for ", to_py(*expr, ind), " in ",
                                                       to_py(*coll, ind), ": ",
                                                       to_py(*body, ind)),
        Core::If { cond, then } => String::from("if ", to_py(*cond, ind), ": ",
                                                to_py(*then, ind)),
        Core::IfElse { cond, then, _else } => String::from("if ", to_py(*cond, ind), ": ",
                                                           to_py(*then, ind), " else: ",
                                                           to_py(*_else, ind)),
        Core::While { cond, body } => String::from("while", to_py(*cond, ind), ": ",
                                                   to_py(*body, ind)),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::Undefined => String::from("None"),
        Core::Empty => String::new(),
    }
}

fn comma_delimited(items: Vec<Core>, ind: i32) -> String {
    if items.is_empty() { return String::new(); }

    let mut items = String::new();
    let mut pos = 0;
    for item in items {
        items.push_str(to_py(*item, ind));

        pos += 1;
        if pos < items.len() { items.push(','); }
    }

    items
}

fn fun_def(id: Box<Core>, args: Vec<Core>, body: Box<Core>, ind: i32) -> String {
    let name = String::fom(match *id {
        Core::GeOp => "__gt__",
        Core::GeqOp => "__ge__",
        Core::LeOp => "__lt__",
        Core::LeqOp => "__le__",

        Core::EqOp => "__eq__",
        Core::NeqOp => "__eq__",

        Core::AddOp => "__add__",
        Core::SubOp => "__sub__",
        Core::MulOp => "__mul__",
        Core::ModOp => "__mod__",
        Core::DivOp => "__truediv__",

        Core::Id { id } => if id == "size" { "__size__" } else { id },
        _ => panic!()
    });

    String::from(
        name,
        "(", comma_delimited(args, ind), "):\n",
        to_py(*body, ind),
    )
}
