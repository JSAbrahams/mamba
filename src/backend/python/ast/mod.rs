use std::fmt::{Display, Formatter, Write};

use crate::backend::python::ast::node::PythonCore;
use crate::common::delimit::{comma_delm, custom_delimited};

pub mod node;

pub const IND_SPACES: usize = 4;

impl Display for PythonCore {
    /// Convert [PythonCore](mamba::backend::python::ast::node::PythonCore) to a String which represent
    /// python source code.
    ///
    /// Takes [PythonCore](mamba::backend::python::ast::node::PythonCore) nodes as-is, meaning that this
    /// should never panic, unless a certain backend::python::ast construct can still not be
    /// converted.
    ///
    /// # Examples
    ///
    /// Writing a Return statement:
    ///
    /// ```
    /// # use mamba::backend::python::ast::node::PythonCore;
    /// let core_node = PythonCore::Return { expr: Box::from(PythonCore::None) };
    /// let py_source = format!("{core_node}");
    ///
    /// assert_eq!(py_source, "return None\n");
    /// ```
    ///
    /// Writing an If statement:
    ///
    /// ```
    /// # use mamba::backend::python::ast::node::PythonCore;
    /// let core_node = PythonCore::IfElse {
    ///  cond:  Box::from(PythonCore::Id { lit: String::from("a") }),
    ///  then:  Box::from(PythonCore::Str { string: String::from("b") }),
    ///  el: Box::from(PythonCore::Str { string: String::from("c") })
    /// };
    ///
    /// assert_eq!(format!("{core_node}"), "if a: \n    \"b\"\nelse: \n    \"c\"\n");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let source = to_py(self, 0) + "\n";
        write!(f, "{}", at_most_one_empty_line(&source))
    }
}

/// `source` with every run of empty lines cut to one, where a line of only white space counts as empty.
fn at_most_one_empty_line(source: &str) -> String {
    let mut out = String::with_capacity(source.len());
    let mut previous_empty = false;
    for line in source.split_inclusive('\n') {
        let empty = line.trim().is_empty();
        if empty && previous_empty {
            continue;
        }

        out.push_str(if !empty {
            line
        } else if line.ends_with('\n') {
            "\n"
        } else {
            ""
        });
        previous_empty = empty;
    }
    out
}

fn to_py(core: &PythonCore, ind: usize) -> String {
    match core {
        PythonCore::Import {
            from,
            import,
            alias,
        } => format!(
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
        PythonCore::Id { lit } => lit.clone(),
        PythonCore::Type { lit, generics } => {
            if generics.is_empty() {
                lit.clone()
            } else {
                format!("{}[{}]", lit, comma_delimited(generics, ind))
            }
        }
        PythonCore::ExpressionType { expr, ty } => {
            format!("{}: {}", to_py(expr, ind), to_py(ty, ind))
        }
        PythonCore::DocStr { string } => format!("\"\"\"{string}\"\"\""),
        PythonCore::Str { string } => format!("\"{string}\""),
        PythonCore::FStr { string } => format!("f\"{string}\""),
        PythonCore::Int { int } => int.clone(),
        PythonCore::ENum { num, exp } => format!("({num} * 10 ** {exp})"),
        PythonCore::Float { float } => float.clone(),
        PythonCore::Bool { boolean } => String::from(if *boolean { "True" } else { "False" }),

        PythonCore::FunDefOp { op, arg, ty, body } => {
            let id = format!("{op}");
            let dec = vec![];
            to_py(
                &PythonCore::FunDef {
                    dec,
                    id,
                    arg: arg.clone(),
                    ty: ty.clone(),
                    body: body.clone(),
                },
                ind,
            )
        }
        PythonCore::FunDef {
            dec,
            id,
            arg,
            ty,
            body,
        } => {
            let dec: Vec<PythonCore> = dec
                .iter()
                .map(|d| PythonCore::Id {
                    lit: format!("@{d}"),
                })
                .collect();
            format!(
                "{}{}def {id}({}){}: {}\n",
                if dec.is_empty() {
                    String::from("")
                } else {
                    newline_delimited(&dec, ind - 1)
                },
                if dec.is_empty() {
                    String::from("")
                } else {
                    indent(ind)
                },
                comma_delimited(arg, ind),
                if let Some(ret_ty) = ty {
                    format!(" -> {}", to_py(ret_ty.as_ref(), ind))
                } else {
                    String::new()
                },
                newline_if_body(body, ind)
            )
        }

        PythonCore::Assign { left, right, op } => {
            format!("{} {op} {}", to_py(left, ind), to_py(right, ind))
        }
        PythonCore::VarDef { var, expr, ty } => format!(
            "{}{} = {}",
            to_py(var, ind),
            if let Some(ty) = ty {
                format!(": {}", to_py(ty, ind))
            } else {
                String::new()
            },
            if let Some(expr) = expr {
                to_py(expr, ind)
            } else {
                String::from("None")
            }
        ),

        PythonCore::FunArg {
            vararg,
            var,
            ty,
            default,
        } => format!(
            "{}{}{}{}",
            if *vararg { "*" } else { "" },
            to_py(var, ind),
            if let Some(ty) = ty {
                format!(": {}", to_py(ty, ind))
            } else {
                String::new()
            },
            if let Some(default) = default {
                format!(" = {}", to_py(default, ind))
            } else {
                String::new()
            }
        ),

        PythonCore::AnonFun { args, body } => format!(
            "lambda{}: {}",
            if args.is_empty() {
                String::new()
            } else {
                format!(" {}", comma_delimited(args, ind))
            },
            to_py(body, ind)
        ),

        PythonCore::Block { statements } => newline_delimited(statements, ind),

        PythonCore::PropertyCall { object, property } => {
            format!("{}.{}", to_py(object, ind), to_py(property, ind))
        }
        PythonCore::FunctionCall { function, args } => {
            format!("{}({})", to_py(function, ind), comma_delimited(args, ind))
        }

        PythonCore::DictComprehension {
            from,
            to,
            col,
            conds,
        } if conds.is_empty() => {
            format!(
                "{{{}: {} for {}}}",
                to_py(from, ind),
                to_py(to, ind),
                to_py(col, ind)
            )
        }
        PythonCore::DictComprehension {
            from,
            to,
            col,
            conds,
        } => {
            let conds: Vec<String> = conds.iter().map(|cond| to_py(cond, ind)).collect();
            format!(
                "{{{}: {} for {} if {}}}",
                to_py(from, ind),
                to_py(to, ind),
                to_py(col, ind),
                custom_delimited(conds, " and ", "")
            )
        }
        PythonCore::Comprehension { expr, col, conds } if conds.is_empty() => {
            format!("{} for {}", to_py(expr, ind), to_py(col, ind))
        }
        PythonCore::Comprehension { expr, col, conds } => {
            let conds: Vec<String> = conds.iter().map(|cond| to_py(cond, ind)).collect();
            format!(
                "{} for {} if {}",
                to_py(expr, ind),
                to_py(col, ind),
                custom_delimited(conds, " and ", "")
            )
        }

        PythonCore::Tuple { elements } => format!("({})", comma_delimited(elements, ind)),
        PythonCore::TupleLiteral { elements } => comma_delimited(elements, ind),
        PythonCore::Dictionary { elements } => {
            let elements: Vec<String> = elements
                .iter()
                .map(|(from, to)| format!("{}: {}", to_py(from, ind), to_py(to, ind)))
                .collect();
            format!("{{{}}}", comma_delm(elements))
        }
        PythonCore::Set { elements } => format!("{{{}}}", comma_delimited(elements, ind)),
        PythonCore::List { elements } => format!("[{}]", comma_delimited(elements, ind)),

        PythonCore::Match { expr, cases } => {
            format!(
                "match {}:\n{}",
                to_py(expr, ind),
                newline_delimited(cases, ind + 1)
            )
        }
        PythonCore::Case { expr, guard, body } => {
            let guard = guard
                .as_ref()
                .map_or_else(String::new, |guard| format!(" if {}", to_py(guard, ind)));
            format!(
                "case {}{}: {}",
                to_py(expr, ind),
                guard,
                newline_if_body(body, ind)
            )
        }
        PythonCore::KeyValue { key, value } => {
            format!("{}: {}", to_py(key, ind), to_py(value, ind))
        }

        PythonCore::UnderScore => String::from("_"),

        PythonCore::Ge { left, right } => binary(core, left, ">", right, ind),
        PythonCore::Geq { left, right } => binary(core, left, ">=", right, ind),
        PythonCore::Le { left, right } => binary(core, left, "<", right, ind),
        PythonCore::Leq { left, right } => binary(core, left, "<=", right, ind),

        PythonCore::Not { expr } => unary(core, "not ", expr, ind),
        PythonCore::And { left, right } => binary(core, left, "and", right, ind),
        PythonCore::Or { left, right } => binary(core, left, "or", right, ind),
        PythonCore::Eq { left, right } => binary(core, left, "==", right, ind),
        PythonCore::Neq { left, right } => binary(core, left, "!=", right, ind),
        PythonCore::AddU { expr } => unary(core, "+", expr, ind),
        PythonCore::Add { left, right } => binary(core, left, "+", right, ind),
        PythonCore::SubU { expr } => unary(core, "-", expr, ind),
        PythonCore::Sub { left, right } => binary(core, left, "-", right, ind),
        PythonCore::Mul { left, right } => binary(core, left, "*", right, ind),
        PythonCore::Div { left, right } => binary(core, left, "/", right, ind),
        PythonCore::FDiv { left, right } => binary(core, left, "//", right, ind),
        PythonCore::Pow { left, right } => binary(core, left, "**", right, ind),
        PythonCore::Mod { left, right } => binary(core, left, "%", right, ind),

        PythonCore::Return { expr } => format!("return {}", to_py(expr.as_ref(), ind)),

        PythonCore::For { expr, col, body } => format!(
            "for {} in {}:{}",
            to_py(expr.as_ref(), ind),
            to_py(col.as_ref(), ind),
            newline_if_body(body, ind)
        ),
        PythonCore::In { left, right } => binary(core, left, "in", right, ind),
        PythonCore::Index { item, range } => format!("{}[{}]", to_py(item, ind), to_py(range, ind)),
        PythonCore::If { cond, then } => {
            format!(
                "if {}:{}",
                to_py(cond.as_ref(), ind),
                newline_if_body(then, ind)
            )
        }
        PythonCore::IfElse { cond, then, el } => format!(
            "if {}: {}\n{}else: {}",
            to_py(cond.as_ref(), ind),
            newline_if_body(then, ind),
            indent(ind),
            newline_if_body(el, ind)
        ),
        PythonCore::Ternary { cond, then, el } => format!(
            "{} if {} else {}",
            operand(then, ind, precedence(then) <= TERNARY),
            operand(cond, ind + 1, precedence(cond) <= TERNARY),
            operand(el, ind + 1, precedence(el) < TERNARY)
        ),
        PythonCore::While { cond, body } => {
            format!(
                "while {}:{}",
                to_py(cond.as_ref(), ind),
                newline_if_body(body, ind)
            )
        }
        PythonCore::Continue => String::from("continue"),
        PythonCore::Break => String::from("break"),
        PythonCore::Del { name } => format!("del {name}"),

        PythonCore::ClassDef {
            name,
            parent_names,
            body,
        } => format!(
            "class {}{}: {}\n",
            to_py(name, ind),
            if parent_names.is_empty() {
                String::new()
            } else {
                format!("({})", comma_delimited(parent_names, ind))
            },
            newline_if_body(body, ind)
        ),

        PythonCore::Pass => String::from("pass"),
        PythonCore::None => String::from("None"),
        PythonCore::Empty => String::new(),

        PythonCore::With { resource, expr } => {
            format!(
                "with {}: {}",
                to_py(resource, ind),
                newline_if_body(expr, ind)
            )
        }
        PythonCore::WithAs {
            resource,
            alias,
            expr,
        } => format!(
            "with {} as {}: {}",
            to_py(resource, ind),
            to_py(alias, ind),
            newline_if_body(expr, ind)
        ),

        PythonCore::TryExcept {
            setup,
            attempt,
            except,
        } => format!(
            "{}try: {}\n{}",
            if let Some(setup) = setup {
                format!("{}\n{}", to_py(setup, ind), indent(ind))
            } else {
                String::from("")
            },
            newline_if_body(attempt, ind),
            newline_delimited(except, ind)
        ),
        PythonCore::ExceptId { id, class, body } => {
            let (id, class) = (to_py(id, ind), to_py(class, ind));
            let body = newline_if_body(body, ind);
            format!("except {class} as {id}: {body}")
        }
        PythonCore::Except { class, body } => {
            let class = to_py(class, ind);
            let body = newline_if_body(body, ind);
            format!("except {class}: {body}")
        }

        PythonCore::Raise { error } => format!("raise {}", to_py(error, ind)),
    }
}

/// How tightly a Python expression binds, lowest first, following the Python grammar.
///
/// Mamba's grouping lives in the tree rather than in brackets, so brackets are put back wherever Python would otherwise regroup.
fn precedence(core: &PythonCore) -> u8 {
    match core {
        PythonCore::AnonFun { .. } => LAMBDA,
        PythonCore::Ternary { .. } => TERNARY,
        PythonCore::Or { .. } => 3,
        PythonCore::And { .. } => 4,
        PythonCore::Not { .. } => 5,
        PythonCore::Ge { .. }
        | PythonCore::Geq { .. }
        | PythonCore::Le { .. }
        | PythonCore::Leq { .. }
        | PythonCore::Eq { .. }
        | PythonCore::Neq { .. }
        | PythonCore::In { .. } => COMPARISON,
        PythonCore::Add { .. } | PythonCore::Sub { .. } => 7,
        PythonCore::Mul { .. }
        | PythonCore::Div { .. }
        | PythonCore::FDiv { .. }
        | PythonCore::Mod { .. } => 8,
        PythonCore::AddU { .. } | PythonCore::SubU { .. } => 9,
        PythonCore::Pow { .. } => POWER,
        _ => ATOM,
    }
}

const LAMBDA: u8 = 1;
const TERNARY: u8 = 2;
const COMPARISON: u8 = 6;
const POWER: u8 = 10;
const ATOM: u8 = 11;

/// A binary operation, bracketing an operand where Python would otherwise group it differently.
///
/// `**` groups to the right, and comparisons chain, so `(a < b) < c` keeps its brackets.
fn binary(
    core: &PythonCore,
    left: &PythonCore,
    op: &str,
    right: &PythonCore,
    ind: usize,
) -> String {
    let own = precedence(core);
    let (wrap_left, wrap_right) = match own {
        POWER => (precedence(left) <= own, precedence(right) < own),
        COMPARISON => (precedence(left) <= own, precedence(right) <= own),
        _ => (precedence(left) < own, precedence(right) <= own),
    };
    format!(
        "{} {op} {}",
        operand(left, ind, wrap_left),
        operand(right, ind, wrap_right)
    )
}

/// A unary operation, bracketing an operand that binds less tightly, as in `-(a + b)`.
fn unary(core: &PythonCore, op: &str, expr: &PythonCore, ind: usize) -> String {
    format!(
        "{op}{}",
        operand(expr, ind, precedence(expr) < precedence(core))
    )
}

fn operand(core: &PythonCore, ind: usize, wrap: bool) -> String {
    if wrap {
        format!("({})", to_py(core, ind))
    } else {
        to_py(core, ind)
    }
}

fn indent(amount: usize) -> String {
    " ".repeat(IND_SPACES * amount)
}

fn newline_if_body(core: &PythonCore, ind: usize) -> String {
    match core {
        PythonCore::Block { .. } => format!("\n{}", to_py(core, ind + 1)),
        _ => format!("\n{}{}", indent(ind + 1), to_py(core, ind + 1)),
    }
}

fn newline_delimited(items: &[PythonCore], ind: usize) -> String {
    let mut s = String::new();
    write_items(&mut s, items, ind);
    s
}

/// Render each of `items` at `ind`, flattening any item that's itself a [`PythonCore::Block`]
/// (recursively, in case of several layers) rather than indenting it as a single opaque item.
///
/// A `Block` already indents each of its own statements uniformly at `ind` (this is the same function, called again one level down).
fn write_items(s: &mut String, items: &[PythonCore], ind: usize) {
    for item in items {
        match item {
            PythonCore::Block { statements } => write_items(s, statements, ind),
            other => writeln!(s, "{}{}", indent(ind), to_py(other, ind)).unwrap(),
        }
    }
}

fn comma_delimited(items: &[PythonCore], ind: usize) -> String {
    let mut s = String::new();
    items
        .iter()
        .for_each(|item| write!(s, "{}, ", to_py(item, ind)).unwrap());

    if s.len() > 2 {
        s.remove(s.len() - 2);
    }
    String::from(s.trim_end())
}
