use crate::core::core::Core;

pub mod core;

pub fn to_py_source(core: Core) -> String {
    match core {
        Core::Id { id } => id,
        Core::Str { _str } => _str,
        Core::Int { int } => int,
        Core::Float { float } => float,
        Core::Bool { _bool } => String::from(if _bool { "True" } else { "False" }),

        Core::Init { id, args, body } => String::from(Core::Id { lit: String::from("__init__") },
                                                      "(", comma_delimited(args), "):\n",
                                                      to_py_source(*body)),
        Core::FunDef { id, args, body } => fun_def(id, args, body),

        Core::Assign { left, right } => String::from(to_py_source(*left), " = ", to_py_source(*right)),
        Core::VarDef { left, right } => String::from(to_py_source(*left), " = ", to_py_source(*right)),

        Core::FunArg { vararg, expr } => String::from(if vararg { "*" } else { "" }, to_py_source(*expr)),

        Core::Block { exprs } => {
            let mut block = String::new();
            for expr in exprs {
                block.push_str(to_py_source(*expr));
                block.push('\n');
            }
            block
        }

        Core::FunctionCall { namespace, function, args, } => String::from(namespace, ".", function,
                                                                          "(", comma_delimited(args), ")"),
        Core::MethodCall { namespace, method, args, } => String::from(namespace, ".", method,
                                                                      "(", comma_delimited(args), ")"),

        Core::Tuple { elements } => String::from("(", comma_delimited(elements), ")"),
        Core::Set { elements } => String::from("{", comma_delimited(elements), "}"),
        Core::List { elements } => String::from("[", comma_delimited(elements), "]"),

        Core::Ge { left, right } => String::from(to_py_source(*left), " > ", to_py_source(*right)),
        Core::Geq { left, right } => String::from(to_py_source(*left), " >= ", to_py_source(*right)),
        Core::Le { left, right } => String::from(to_py_source(*left), " < ", to_py_source(*right)),
        Core::Geq { left, right } => String::from(to_py_source(*left), " <= ", to_py_source(*right)),

        Core::Not { expr } => String::from("not", to_py_source(*expr)),
        Core::And { left, right } => String::from(to_py_source(*left), " && ", to_py_source(*right)),
        Core::Or { left, right } => String::from(to_py_source(*left), " || ", to_py_source(*right)),
        Core::Is { left, right } => String::from(to_py_source(*left), " is ", to_py_source(*right)),
        Core::Eq { left, right } => String::from(to_py_source(*left), " == ", to_py_source(*right)),
        Core::Neq { left, right } => String::from(to_py_source(*left), " != ", to_py_source(*right)),
        Core::IsA { left, right } => String::from("isinstance(", to_py_source(*left), ", ", to_py_source(*right), ")"),

        Core::Add { left, right } => String::from(to_py_source(*left), " + ", to_py_source(*right)),
        Core::Sub { left, right } => String::from(to_py_source(*left), " - ", to_py_source(*right)),
        Core::Mul { left, right } => String::from(to_py_source(*left), " * ", to_py_source(*right)),
        Core::Div { left, right } => String::from(to_py_source(*left), " / ", to_py_source(*right)),
        Core::Pow { left, right } => String::from(to_py_source(*left), " ** ", to_py_source(*right)),
        Core::Mod { left, right } => String::from(to_py_source(*left), " % ", to_py_source(*right)),

        Core::Return { expr } => String::from("return ", to_py_source(*expr)),
        Core::Print { expr } => String::from("print(", to_py_source(*expr), ")"),

        Core::For { expr, coll, body } => String::from("for ", to_py_source(*expr), " in ",
                                                       to_py_source(*coll), ":\n",
                                                       to_py_source(*body)),
        Core::If { cond, then } => String::from("if ", to_py_source(*cond), ":\n",
                                                to_py_source(*then)),
        Core::IfElse { cond, then, _else } => String::from("if ", to_py_source(*cond), ":\n",
                                                           to_py_source(*then), "\nelse:\n",
                                                           to_py_source(*_else)),
        Core::While { cond, body } => String::from("while", to_py_source(*cond), ":",
                                                   to_py_source(*body)),
        Core::Continue => String::from("continue"),
        Core::Break => String::from("break"),

        Core::Undefined => String::from("None"),
        Core::Empty => String::new(),
    }
}

fn comma_delimited(items: Vec<Core>) -> String {
    if items.is_empty() { return String::new(); }

    let mut items = String::new();
    let mut pos = 0;
    for item in items {
        items.push_str(to_py_source(*item));

        pos += 1;
        if pos < items.len() { items.push(','); }
    }

    items
}

fn fun_def(id: Box<Core>, args: Vec<Core>, body: Box<Core>) -> String {
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
        "(", comma_delimited(args), "):\n",
        to_py_source(*body),
    )
}
