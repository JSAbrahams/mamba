use std::fmt::{Display, Error, Formatter};

use crate::check::context::arg;
use crate::common::delimit::{comma_delm, custom_delimited};
use crate::parse::ast::{AST, Node};
use crate::parse::lex::token::Token;

fn equal_optional(this: &Option<Box<AST>>, that: &Option<Box<AST>>) -> bool {
    if let (Some(this), Some(that)) = (this, that) {
        this.same_value(that)
    } else {
        true
    }
}

fn equal_vec(this: &[AST], other: &[AST]) -> bool {
    if this.len() != other.len() {
        false
    } else {
        for (left, right) in this.iter().zip(other) {
            if !left.same_value(right) {
                return false;
            }
        }
        true
    }
}

impl Display for Node {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        let name = match &self {
            Node::File { statements, .. } => {
                format!("file: {}", custom_delimited(statements.iter().map(|ast| ast.node.clone()), "\\n", ""))
            }
            Node::Import { .. } => String::from("import"),
            Node::FromImport { .. } => String::from("from import"),
            Node::Class { .. } => String::from("class"),
            Node::Generic { .. } => String::from("generic"),
            Node::Parent { .. } => String::from("parent"),
            Node::Init => String::from("constructor (init)"),
            Node::Reassign { .. } => String::from("reassign"),
            Node::VariableDef { .. } => String::from("variable definition"),
            Node::FunDef { .. } => String::from("function definition"),
            Node::AnonFun { .. } => String::from("anonymous function"),
            Node::Raises { .. } => String::from("raises"),
            Node::Raise { .. } => String::from("raise"),
            Node::Handle { .. } => String::from("handle"),
            Node::With { .. } => String::from("with"),
            Node::FunctionCall { name, args } =>
                format!("{}({})", name.node, comma_delm(args.iter().map(|a| a.node.clone()))),
            Node::PropertyCall { instance, property } =>
                format!("{}.{}", instance.node, property.node),
            Node::Id { lit } => lit.clone(),
            Node::ExpressionType { .. } => String::from("expression type"),
            Node::TypeDef { .. } => String::from("type definition"),
            Node::TypeAlias { .. } => String::from("type alias"),
            Node::TypeTup { .. } => String::from("type tuple"),
            Node::TypeUnion { .. } => String::from("type union"),
            Node::Type { .. } => String::from("type"),
            Node::TypeFun { .. } => String::from("type function"),
            Node::Condition { .. } => String::from("condition"),
            Node::FunArg { .. } => String::from("function argument"),
            Node::_Self => String::from(arg::SELF),
            Node::AddOp => format!("{}", Token::Add),
            Node::SubOp => format!("{}", Token::Sub),
            Node::SqrtOp => format!("{}", Token::Sqrt),
            Node::MulOp => format!("{}", Token::Mul),
            Node::FDivOp => format!("{}", Token::FDiv),
            Node::DivOp => format!("{}", Token::Div),
            Node::PowOp => format!("{}", Token::Pow),
            Node::ModOp => format!("{}", Token::Mod),
            Node::EqOp => format!("{}", Token::Eq),
            Node::LeOp => format!("{}", Token::Le),
            Node::GeOp => format!("{}", Token::Ge),
            Node::Set { elements } =>
                format!("{{{}}}", comma_delm(elements.iter().map(|e| e.node.clone()))),
            Node::SetBuilder { .. } => String::from("set builder"),
            Node::List { elements } =>
                format!("[{}]", comma_delm(elements.iter().map(|e| e.node.clone()))),
            Node::ListBuilder { .. } => String::from("list builder"),
            Node::Tuple { elements } =>
                format!("({})", comma_delm(elements.iter().map(|e| e.node.clone()))),
            Node::Range { .. } => String::from("range"),
            Node::Block { .. } => String::from("Code block"),
            Node::Real { lit } => lit.clone(),
            Node::Int { lit } => lit.clone(),
            Node::ENum { num, exp } => format!("{}E{}", num, exp),
            Node::Str { lit, .. } => format!("\"{}\"", lit),
            Node::DocStr { .. } => String::from("doc string"),
            Node::Bool { .. } => String::from("boolean"),
            Node::Add { left, right } => format!("{} {} {}", left.node, Token::Add, right.node),
            Node::AddU { .. } => String::from("addition unary"),
            Node::Sub { left, right } => format!("{} {} {}", left.node, Token::Sub, right.node),
            Node::SubU { .. } => String::from("subtract unary"),
            Node::Mul { left, right } => format!("{} {} {}", left.node, Token::Mul, right.node),
            Node::Div { left, right } => format!("{} {} {}", left.node, Token::Div, right.node),
            Node::FDiv { left, right } => format!("{} {} {}", left.node, Token::FDiv, right.node),
            Node::Mod { left, right } => format!("{} {} {}", left.node, Token::Mod, right.node),
            Node::Pow { left, right } => format!("{} {} {}", left.node, Token::Pow, right.node),
            Node::Sqrt { expr } => format!("{} {}", Token::Sqrt, expr.node),
            Node::BAnd { left, right } => format!("{} {} {}", left.node, Token::BAnd, right.node),
            Node::BOr { left, right } => format!("{} {} {}", left.node, Token::BOr, right.node),
            Node::BXOr { left, right } => format!("{} {} {}", left.node, Token::BXOr, right.node),
            Node::BOneCmpl { .. } => String::from("binary ones compliment"),
            Node::BLShift { left, right } => format!("{} {} {}", left.node, Token::BLShift, right.node),
            Node::BRShift { left, right } => format!("{} {} {}", left.node, Token::BRShift, right.node),
            Node::Le { left, right } => format!("{} {} {}", left.node, Token::Le, right.node),
            Node::Ge { left, right } => format!("{} {} {}", left.node, Token::Ge, right.node),
            Node::Leq { left, right } => format!("{} {} {}", left.node, Token::Leq, right.node),
            Node::Geq { left, right } => format!("{} {} {}", left.node, Token::Geq, right.node),
            Node::Is { left, right } => format!("{} {} {}", left.node, Token::Is, right.node),
            Node::IsN { left, right } => format!("{} {} {}", left.node, Token::IsN, right.node),
            Node::Eq { left, right } => format!("{} {} {}", left.node, Token::Eq, right.node),
            Node::Neq { left, right } => format!("{} {} {}", left.node, Token::Neq, right.node),
            Node::IsA { left, right } => format!("{} {} {}", left.node, Token::IsA, right.node),
            Node::IsNA { left, right } => format!("{} {} {}", left.node, Token::IsNA, right.node),
            Node::Not { .. } => String::from("not"),
            Node::And { left, right } => format!("{} {} {}", left.node, Token::And, right.node),
            Node::Or { left, right } => format!("{} {} {}", left.node, Token::Or, right.node),
            Node::IfElse { el, .. } => String::from(if el.is_some() { "if" } else { "if else" }),
            Node::Match { .. } => String::from("match"),
            Node::Case { .. } => String::from("case"),
            Node::For { .. } => String::from("for loop"),
            Node::In { left, right } => format!("{} {} {}", left.node, Token::In, right.node),
            Node::Step { .. } => String::from("step"),
            Node::While { .. } => String::from("while loop"),
            Node::Break => format!("{}", Token::Break),
            Node::Continue => format!("{}", Token::Continue),
            Node::Return { .. } | Node::ReturnEmpty => String::from("return"),
            Node::Underscore => format!("{}", Token::Underscore),
            Node::Undefined => format!("{}", Token::Undefined),
            Node::Pass => format!("{}", Token::Pass),
            Node::Question { .. } => String::from("ternary operator"),
            Node::QuestionOp { .. } => String::from("unsafe operator"),
            Node::Print { .. } => format!("{}", Token::Print),
            Node::Comment { .. } => String::from("comment")
        };

        write!(f, "{}", name)
    }
}

impl Node {
    pub fn same_value(&self, other: &Node) -> bool {
        match (&self, &other) {
            (Node::Import { import: li, aliases: la }, Node::Import { import: ri, aliases: ra }) =>
                equal_vec(li, ri) && equal_vec(la, ra),
            (
                Node::FromImport { id: lid, import: li },
                Node::FromImport { id: rid, import: ri }
            ) => lid.same_value(rid) && li.same_value(ri),
            (
                Node::Class { ty: lt, args: la, parents: lp, body: lb },
                Node::Class { ty: rt, args: ra, parents: rp, body: rb }
            ) =>
                lt.same_value(rt)
                    && equal_vec(la, ra)
                    && equal_vec(lp, rp)
                    && equal_optional(lb, rb),
            (Node::Generic { id: li, isa: lisa }, Node::Generic { id: ri, isa: risa }) =>
                li.same_value(ri) && equal_optional(lisa, risa),
            (Node::Parent { ty: l_ty, args: la }, Node::Parent { ty: r_ty, args: ra }) =>
                l_ty.same_value(r_ty) && equal_vec(la, ra),
            (Node::Reassign { left: ll, right: lr }, Node::Reassign { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (
                Node::VariableDef {
                    mutable: lm,
                    var: lv,
                    ty: lt,
                    expr: le,
                    forward: lf
                },
                Node::VariableDef {
                    mutable: rm,
                    var: rv,
                    ty: rt,
                    expr: re,
                    forward: rf
                }
            ) => lm == rm
                && lv.same_value(rv)
                && equal_optional(lt, rt)
                && equal_optional(le, re)
                && equal_vec(lf, rf),
            (
                Node::FunDef {
                    pure: lpu,
                    id: li,
                    args: la,
                    ret: lret,
                    raises: lraise,
                    body: lb
                },
                Node::FunDef {
                    pure: rpu,
                    id: ri,
                    args: ra,
                    ret: rret,
                    raises: rraise,
                    body: rb
                }
            ) =>
                lpu == rpu
                    && li.same_value(ri)
                    && equal_vec(la, ra)
                    && equal_optional(lret, rret)
                    && equal_vec(lraise, rraise)
                    && equal_optional(lb, rb),
            (Node::AnonFun { args: la, body: lb }, Node::AnonFun { args: ra, body: rb }) =>
                equal_vec(la, ra) && lb.same_value(rb),
            (
                Node::Raises { expr_or_stmt: les, errors: le },
                Node::Raises { expr_or_stmt: res, errors: re }
            ) => les.same_value(res) && equal_vec(le, re),
            (Node::Raise { error: le }, Node::Raise { error: re }) => le.same_value(re),
            (
                Node::Handle { expr_or_stmt: les, cases: lc },
                Node::Handle { expr_or_stmt: res, cases: rc }
            ) => les.same_value(res) && equal_vec(lc, rc),
            (
                Node::With { resource: lr, alias: Some((la, lmut, lty)), expr: le },
                Node::With { resource: rr, alias: Some((ra, rmut, rty)), expr: re }
            ) =>
                lr.same_value(rr)
                    && la.same_value(ra)
                    && lmut == rmut
                    && equal_optional(lty, rty)
                    && le.same_value(re),
            (
                Node::With { resource: lr, alias: None, expr: le },
                Node::With { resource: rr, alias: None, expr: re }
            ) => lr.same_value(rr) && le.same_value(re),
            (
                Node::FunctionCall { name: ln, args: la },
                Node::FunctionCall { name: rn, args: ra }
            ) => ln.same_value(rn) && equal_vec(la, ra),
            (
                Node::PropertyCall { instance: li, property: lp },
                Node::PropertyCall { instance: ri, property: rp }
            ) => li.same_value(ri) && lp.same_value(rp),
            (Node::Id { lit: l }, Node::Id { lit: r }) => l == r,
            (
                Node::ExpressionType { expr: le, mutable: lm, ty: lt },
                Node::ExpressionType { expr: re, mutable: rm, ty: rt }
            ) => le.same_value(re) && lm == rm && equal_optional(lt, rt),
            (
                Node::TypeDef { ty: lt, isa: li, body: lb },
                Node::TypeDef { ty: rt, isa: ri, body: rb }
            ) => lt.same_value(rt) && equal_optional(li, ri) && equal_optional(lb, rb),
            (
                Node::TypeAlias { ty: lt, isa: li, conditions: lc },
                Node::TypeAlias { ty: rt, isa: ri, conditions: rc }
            ) => lt.same_value(rt) && li.same_value(ri) && equal_vec(lc, rc),
            (Node::TypeTup { types: l }, Node::TypeTup { types: r }) => equal_vec(l, r),
            (Node::TypeUnion { types: l }, Node::TypeUnion { types: r }) => equal_vec(l, r),
            (Node::Type { id: li, generics: lg }, Node::Type { id: ri, generics: rg }) =>
                li.same_value(ri) && equal_vec(lg, rg),
            (Node::TypeFun { args: la, ret_ty: lr }, Node::TypeFun { args: ra, ret_ty: rr }) =>
                equal_vec(la, ra) && lr.same_value(rr),
            (Node::Condition { cond: lc, el: le }, Node::Condition { cond: rc, el: re }) =>
                lc.same_value(rc) && equal_optional(le, re),
            (
                Node::FunArg { vararg: lv, mutable: lm, var: lvar, ty: lt, default: ld },
                Node::FunArg { vararg: rv, mutable: rm, var: rvar, ty: rt, default: rd }
            ) =>
                lv == rv
                    && lm == rm
                    && lvar.same_value(rvar)
                    && equal_optional(lt, rt)
                    && equal_optional(ld, rd),
            (
                Node::SetBuilder { item: li, conditions: lc },
                Node::SetBuilder { item: ri, conditions: rc }
            ) => li.same_value(ri) && equal_vec(lc, rc),
            (
                Node::ListBuilder { item: li, conditions: lc },
                Node::ListBuilder { item: ri, conditions: rc }
            ) => li.same_value(ri) && equal_vec(lc, rc),
            (Node::Set { elements: l }, Node::Set { elements: r }) => equal_vec(l, r),
            (Node::List { elements: l }, Node::List { elements: r }) => equal_vec(l, r),
            (Node::Tuple { elements: l }, Node::Tuple { elements: r }) => equal_vec(l, r),
            (
                Node::Range { from: lf, to: lt, inclusive: li, step: ls },
                Node::Range { from: rf, to: rt, inclusive: ri, step: rs }
            ) =>
                lf.same_value(rf)
                    && lt.same_value(rt)
                    && li == ri
                    && equal_optional(ls, rs),
            (Node::Block { statements: l }, Node::Block { statements: r }) => equal_vec(l, r),
            (Node::Real { lit: l }, Node::Real { lit: r }) => l == r,
            (Node::Int { lit: l }, Node::Int { lit: r }) => l == r,
            (Node::ENum { num: ln, exp: le }, Node::ENum { num: rn, exp: re }) =>
                ln == rn && le == re,
            (Node::Str { lit: l, expressions: le }, Node::Str { lit: r, expressions: re }) =>
                l == r && equal_vec(le, re),
            (Node::DocStr { .. }, Node::DocStr { .. }) => true,
            (Node::Bool { lit: l }, Node::Bool { lit: r }) => l == r,
            (Node::AddU { expr: l }, Node::AddU { expr: r }) => l.same_value(r),
            (Node::SubU { expr: l }, Node::SubU { expr: r }) => l.same_value(r),
            (Node::Add { left: ll, right: lr }, Node::Add { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Sub { left: ll, right: lr }, Node::Sub { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Mul { left: ll, right: lr }, Node::Mul { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Div { left: ll, right: lr }, Node::Div { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::BOr { left: ll, right: lr }, Node::BOr { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Mod { left: ll, right: lr }, Node::Mod { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Pow { left: ll, right: lr }, Node::Pow { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Sqrt { expr: l }, Node::Sqrt { expr: r }) => l.same_value(r),
            (Node::FDiv { left: ll, right: lr }, Node::FDiv { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::BAnd { left: ll, right: lr }, Node::BAnd { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::BXOr { left: ll, right: lr }, Node::BXOr { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::BOneCmpl { expr: l }, Node::BOneCmpl { expr: r }) => l.same_value(r),
            (Node::BLShift { left: ll, right: lr }, Node::BLShift { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::BRShift { left: ll, right: lr }, Node::BRShift { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Leq { left: ll, right: lr }, Node::Leq { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Geq { left: ll, right: lr }, Node::Geq { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::IsN { left: ll, right: lr }, Node::IsN { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Neq { left: ll, right: lr }, Node::Neq { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::IsA { left: ll, right: lr }, Node::IsA { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Le { left: ll, right: lr }, Node::Le { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Ge { left: ll, right: lr }, Node::Ge { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Is { left: ll, right: lr }, Node::Is { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Eq { left: ll, right: lr }, Node::Eq { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::IsNA { left: ll, right: lr }, Node::IsNA { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Not { expr: l }, Node::Not { expr: r }) => l.same_value(r),
            (Node::And { left: ll, right: lr }, Node::And { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Or { left: ll, right: lr }, Node::Or { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (
                Node::IfElse { cond: lc, then: lt, el: le },
                Node::IfElse { cond: rc, then: rt, el: re }
            ) => lc.same_value(rc) && lt.same_value(rt) && equal_optional(le, re),
            (Node::Match { cond: lco, cases: lc }, Node::Match { cond: rco, cases: rc }) =>
                lco.same_value(rco) && equal_vec(lc, rc),
            (Node::Case { cond: lc, body: lb }, Node::Case { cond: rc, body: rb }) =>
                lc.same_value(rc) && lb.same_value(rb),
            (
                Node::For { expr: le, col: lc, body: lb },
                Node::For { expr: re, col: rc, body: rb }
            ) => le.same_value(re) && lc.same_value(rc) && lb.same_value(rb),
            (Node::In { left: ll, right: lr }, Node::In { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::Step { amount: la }, Node::Step { amount: ra }) => la.same_value(ra),
            (Node::While { cond: lc, body: lb }, Node::While { cond: rc, body: rb }) =>
                lc.same_value(rc) && lb.same_value(rb),
            (Node::Return { expr: left }, Node::Return { expr: right }) =>
                left.same_value(right),
            (Node::Question { left: ll, right: lr }, Node::Question { left: rl, right: rr }) =>
                ll.same_value(rl) && lr.same_value(rr),
            (Node::QuestionOp { expr: left }, Node::QuestionOp { expr: right }) =>
                left.same_value(right),
            (Node::Print { expr: left }, Node::Print { expr: right }) =>
                left.same_value(right),
            (Node::Comment { .. }, Node::Comment { .. }) => true,

            (left, right) if **left == **right => true,
            _ => false
        }
    }

    /// True if node is an expression with certainty.
    pub fn trivially_expression(&self) -> bool {
        match &self {
            Node::AnonFun { .. }
            | Node::PropertyCall { .. }
            | Node::Id { .. }
            | Node::Set { .. }
            | Node::SetBuilder { .. }
            | Node::List { .. }
            | Node::ListBuilder { .. }
            | Node::Tuple { .. }
            | Node::Range { .. }
            | Node::Real { .. }
            | Node::Int { .. }
            | Node::ENum { .. }
            | Node::Str { .. }
            | Node::Bool { .. }
            | Node::Match { .. }
            | Node::Underscore
            | Node::Undefined
            | Node::Pass
            | Node::_Self
            | Node::Question { .. }
            | Node::QuestionOp { .. } => true,

            Node::IfElse { el, .. } => el.is_some(),

            Node::Block { statements } => if let Some(stmt) = statements.last() {
                stmt.node.trivially_expression()
            } else {
                false
            },

            _ => self.is_operator()
        }
    }

    fn is_operator(&self) -> bool {
        matches!(&self, Node::Add { .. }
            | Node::AddU { .. }
            | Node::Sub { .. }
            | Node::SubU { .. }
            | Node::Mul { .. }
            | Node::Div { .. }
            | Node::FDiv { .. }
            | Node::Mod { .. }
            | Node::Pow { .. }
            | Node::Sqrt { .. }
            | Node::BAnd { .. }
            | Node::BOr { .. }
            | Node::BXOr { .. }
            | Node::BOneCmpl { .. }
            | Node::BLShift { .. }
            | Node::BRShift { .. }
            | Node::Le { .. }
            | Node::Ge { .. }
            | Node::Leq { .. }
            | Node::Geq { .. }
            | Node::Is { .. }
            | Node::IsN { .. }
            | Node::Eq { .. }
            | Node::Neq { .. }
            | Node::IsA { .. }
            | Node::IsNA { .. }
            | Node::Not { .. }
            | Node::And { .. }
            | Node::Or { .. }
            | Node::In { .. })
    }
}

#[cfg(test)]
mod test {
    use crate::common::position::{CaretPos, Position};
    use crate::parse::ast::{AST, Node};

    macro_rules! two_ast_ne {
        ($left:expr, $right: expr) => {{
            let pos = Position::new(&CaretPos::new(3, 403), &CaretPos::new(324, 673));
            let pos2 = Position::new(&CaretPos::new(32, 4032), &CaretPos::new(3242, 6732));
            let (ast, ast2) = (AST::new(&pos, $left), AST::new(&pos2, $right));
            assert!(!ast.same_value(&ast2))
        }};
    }

    macro_rules! two_ast {
        ($left:expr) => {{
            let pos = Position::new(&CaretPos::new(3, 403), &CaretPos::new(324, 673));
            let pos2 = Position::new(&CaretPos::new(32, 4032), &CaretPos::new(3242, 6732));

            let right = $left.clone();
            let (ast, ast2) = (AST::new(&pos, $left), AST::new(&pos2, right));
            assert!(ast.same_value(&ast2))
        }};
        ($left:expr, $right: expr) => {{
            let pos = Position::new(&CaretPos::new(3, 403), &CaretPos::new(324, 673));
            let pos2 = Position::new(&CaretPos::new(32, 4032), &CaretPos::new(3242, 6732));
            let (ast, ast2) = (AST::new(&pos, $left), AST::new(&pos2, $right));
            assert!(ast.same_value(&ast2))
        }};
    }

    #[test]
    fn simple_ast() {
        let pos = Position::new(&CaretPos::new(3, 403), &CaretPos::new(324, 673));
        let node = Node::Id { lit: String::from("fd") };

        let ast = AST::new(&pos, node.clone());

        assert_eq!(ast.pos, pos);
        assert_eq!(ast.node, node);
    }

    #[test]
    fn id_equal_structure() {
        two_ast!(Node::Id { lit: String::from("fd") }, Node::Id { lit: String::from("fd") });
    }

    #[test]
    fn tuple_equal_structure() {
        let node = Node::Tuple {
            elements: vec![
                AST::new(&Position::default(), Node::Id { lit: String::from("aa") }),
                AST::new(&Position::default(), Node::Id { lit: String::from("ba") })]
        };

        two_ast!(node);
    }

    #[test]
    fn tuple_not_equal_structure() {
        let pos = Position::default();
        let node1 = Node::Tuple {
            elements: vec![
                AST::new(&pos, Node::Id { lit: String::from("aa") }),
                AST::new(&pos, Node::Id { lit: String::from("ba") }),
                AST::new(&pos, Node::Id { lit: String::from("ca") })]
        };
        let node2 = Node::Tuple {
            elements: vec![
                AST::new(&pos, Node::Id { lit: String::from("aa") }),
                AST::new(&pos, Node::Id { lit: String::from("ba") }),
                AST::new(&pos, Node::Id { lit: String::from("ca") }),
                AST::new(&pos, Node::Id { lit: String::from("ca") })]
        };

        two_ast_ne!(node1, node2);
    }

    #[test]
    fn break_equal_structure() {
        two_ast!(Node::Break, Node::Break);
    }

    #[test]
    fn break_continue_not_equal_structure() {
        two_ast_ne!(Node::Break, Node::Continue);
    }

    #[test]
    fn file_equal_value() {
        let node = Node::File {
            pure: true,
            statements: vec![AST::new(&Position::default(), Node::Continue)],
        };

        two_ast!(node);
    }

    #[test]
    fn import_equal_value() {
        let node = Node::Import {
            import: vec![AST::new(&Position::default(), Node::AddOp)],
            aliases: vec![AST::new(&Position::default(), Node::Pass)],
        };

        two_ast!(node);
    }

    #[test]
    fn from_import_equal_value() {
        let node = Node::Import {
            import: vec![AST::new(&Position::default(), Node::AddOp)],
            aliases: vec![AST::new(&Position::default(), Node::Pass)],
        };

        two_ast!(node);
    }

    #[test]
    fn class_equal_value() {
        let node = Node::Class {
            ty: Box::new(AST::new(&Position::default(), Node::SubOp)),
            args: vec![AST::new(&Position::default(), Node::AddOp)],
            parents: vec![AST::new(&Position::default(), Node::Pass)],
            body: Some(Box::from(AST::new(&Position::default(), Node::AddOp))),
        };

        two_ast!(node);
    }

    #[test]
    fn generic_equal_value() {
        let node = Node::Generic {
            id: Box::new(AST::new(&Position::default(), Node::SubOp)),
            isa: Some(Box::from(AST::new(&Position::default(), Node::AddOp))),
        };

        two_ast!(node);
    }

    #[test]
    fn parent_equal_value() {
        let node = Node::Parent {
            ty: Box::new(AST::new(&Position::default(), Node::SubOp)),
            args: vec![AST::new(&Position::default(), Node::DivOp)],
        };

        two_ast!(node);
    }

    #[test]
    fn init_equal_value() {
        two_ast!(Node::Init);
    }

    #[test]
    fn reassign_equal_value() {
        let node = Node::Reassign {
            left: Box::new(AST::new(&Position::default(), Node::SubOp)),
            right: Box::new(AST::new(&Position::default(), Node::LeOp)),
        };

        two_ast!(node);
    }

    #[test]
    fn def_equal_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));
        let third = Box::from(AST::new(&Position::default(), Node::Pass));

        two_ast!(Node::VariableDef {
            mutable: false,
            var:first.clone(),
            ty: Some(second.clone()),
            expr: Some(third.clone()),
            forward: vec![*first.clone()]
        });
        two_ast!(
        Node::FunDef {
            pure: false,
            id: first.clone(),
            args: vec![*second.clone()],
            ret: Some(third.clone()),
            raises: vec![*first.clone(), *second.clone()],
            body: Some(Box::from(AST::new(&Position::default(), Node::Raise {error:third.clone()})))
        });
    }

    #[test]
    fn anon_fun_same_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));

        two_ast!(Node::AnonFun {args: vec![*first.clone()], body: second.clone()});
    }

    #[test]
    fn anon_raise_same_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));

        two_ast!(Node::Raises {errors: vec![*first.clone()], expr_or_stmt: second.clone()});
        two_ast!(Node::Raise {error: second.clone()});
    }

    #[test]
    fn handle_same_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));
        let third = Box::from(AST::new(&Position::default(), Node::Pass));

        two_ast!(Node::Handle {cases: vec![*first.clone()], expr_or_stmt: second.clone()});
        two_ast!(Node::With {
            resource: first.clone(),
            alias: Some((second.clone(), false, Some(third.clone()))),
            expr: Box::from(AST::new(&Position::default(), Node::GeOp))
        });
    }

    #[test]
    fn call_same_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));

        two_ast!(Node::FunctionCall {name: first.clone(), args: vec![*second.clone()]});
        two_ast!(Node::PropertyCall {instance: first.clone(), property: second.clone()});
    }

    #[test]
    fn id_equal_value() {
        two_ast!(Node::Id{ lit:String::from("id") });
    }

    #[test]
    fn id_differnt_str_not_equal_value() {
        two_ast_ne!(Node::Id { lit:String::from("id") }, Node::Id { lit:String::from("id2") });
    }

    #[test]
    fn expression_type_equal_value() {
        let expr = Box::from(AST::new(&Position::default(), Node::Continue));
        let expr2 = Box::from(AST::new(&Position::default(), Node::Pass));
        two_ast!(Node::ExpressionType {expr: expr.clone(), mutable: false, ty: Some(expr2.clone())});
    }

    #[test]
    fn type_equal_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Continue));
        let second = Box::from(AST::new(&Position::default(), Node::Break));
        let third = Box::from(AST::new(&Position::default(), Node::Pass));

        two_ast!(Node::TypeDef {ty: first.clone(), isa: Some(second.clone()), body: Some(third.clone())});
        two_ast!(Node::TypeAlias {ty: first.clone(), isa: second.clone(), conditions: vec![*third.clone()]});
        two_ast!(Node::TypeTup {types: vec![*third.clone(), *second.clone()]});
        two_ast!(Node::TypeUnion {types: vec![*third.clone(), *second.clone()]});
        two_ast!(Node::Type {id: first.clone(), generics:vec![*second.clone(), *third.clone()]});
        two_ast!(Node::TypeFun {args: vec![*first.clone(), *second.clone()], ret_ty: third.clone()});

        two_ast!(Node::Condition {cond: first.clone(), el: Some(second.clone())});
    }

    #[test]
    fn self_equal_value() {
        two_ast!(Node::_Self);
    }

    #[test]
    fn op_equal_value() {
        two_ast!(Node::AddOp);
        two_ast!(Node::SubOp);
        two_ast!(Node::SqrtOp);
        two_ast!(Node::MulOp);
        two_ast!(Node::FDivOp);
        two_ast!(Node::DivOp);
        two_ast!(Node::PowOp);
        two_ast!(Node::ModOp);
        two_ast!(Node::EqOp);
        two_ast!(Node::LeOp);
        two_ast!(Node::GeOp);
    }

    #[test]
    fn literal_value() {
        two_ast!(Node::Real { lit:String::from("dgfdh") });
        two_ast!(Node::Int { lit:String::from("sdfdf") });
        two_ast!(Node::Bool { lit: true });
        two_ast!(Node::ENum { num:String::from("werw"), exp:String::from("reter") });
        two_ast!(Node::Str {
            lit:String::from("yuk"),
            expressions: vec![AST::new(&Position::default(), Node::LeOp)] });
    }

    #[test]
    fn string_different_expression_not_same_value() {
        two_ast_ne!(Node::Str {
                    lit:String::from("yuk"),
                    expressions: vec![AST::new(&Position::default(), Node::LeOp)] },
                Node::Str {
                    lit:String::from("yuk"),
                    expressions: vec![AST::new(&Position::default(), Node::GeOp)] });
    }

    #[test]
    fn collection_same_value() {
        let item = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("asdf") }));

        two_ast!(Node::Set {elements: vec![*item.clone()]});
        two_ast!(Node::List {elements: vec![*item.clone()]});
        two_ast!(Node::Tuple {elements: vec![*item.clone()]});

        two_ast!(Node::SetBuilder {item: item.clone(), conditions: vec![*item.clone()]});
        two_ast!(Node::ListBuilder {item: item.clone(), conditions: vec![*item.clone()]});
    }

    #[test]
    fn block_same_value() {
        let first = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("asdf") }));
        let second = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("lkjh") }));

        two_ast!(Node::Block {statements: vec![*first.clone(), *second.clone()]});
    }

    #[test]
    fn docstr_different_str_same_value() {
        two_ast!(Node::DocStr { lit: String::from("asdf") }, Node::DocStr { lit: String::from("lkjh") });
    }

    #[test]
    fn binary_op_same_value() {
        let left = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("asdf") }));
        let right = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("lkjh") }));

        two_ast!(Node::Add {left: left.clone(), right: right.clone()});
        two_ast!(Node::Sub {left: left.clone(), right: right.clone()});
        two_ast!(Node::Mul {left: left.clone(), right: right.clone()});
        two_ast!(Node::Div {left: left.clone(), right: right.clone()});
        two_ast!(Node::FDiv {left: left.clone(), right: right.clone()});
        two_ast!(Node::Mod {left: left.clone(), right: right.clone()});
        two_ast!(Node::Pow {left: left.clone(), right: right.clone()});
        two_ast!(Node::BAnd {left: left.clone(), right: right.clone()});
        two_ast!(Node::BOr {left: left.clone(), right: right.clone()});
        two_ast!(Node::BXOr {left: left.clone(), right: right.clone()});

        two_ast!(Node::BAnd {left: left.clone(), right: right.clone()});
        two_ast!(Node::BOr {left: left.clone(), right: right.clone()});
        two_ast!(Node::BXOr {left: left.clone(), right: right.clone()});
        two_ast!(Node::BLShift {left: left.clone(), right: right.clone()});
        two_ast!(Node::BRShift {left: left.clone(), right: right.clone()});

        two_ast!(Node::Le {left: left.clone(), right: right.clone()});
        two_ast!(Node::Ge {left: left.clone(), right: right.clone()});
        two_ast!(Node::Leq {left: left.clone(), right: right.clone()});
        two_ast!(Node::Geq {left: left.clone(), right: right.clone()});
        two_ast!(Node::Is {left: left.clone(), right: right.clone()});
        two_ast!(Node::IsN {left: left.clone(), right: right.clone()});
        two_ast!(Node::Eq {left: left.clone(), right: right.clone()});
        two_ast!(Node::Neq {left: left.clone(), right: right.clone()});
        two_ast!(Node::IsA {left: left.clone(), right: right.clone()});
        two_ast!(Node::IsNA {left: left.clone(), right: right.clone()});

        two_ast!(Node::And {left: left.clone(), right: right.clone()});
        two_ast!(Node::Or {left: left.clone(), right: right.clone()});
    }

    #[test]
    fn unary_op_same_value() {
        let expr = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("qwerty") }));

        two_ast!(Node::AddU {expr: expr.clone()});
        two_ast!(Node::SubU {expr: expr.clone()});
        two_ast!(Node::Sqrt {expr: expr.clone()});
        two_ast!(Node::BOneCmpl {expr: expr.clone()});
        two_ast!(Node::Not {expr: expr.clone()});
    }

    #[test]
    fn contrl_flow_same_value() {
        let cond = Box::from(AST::new(&Position::default(), Node::Id { lit: String::from("qwerty") }));
        let body = Box::from(AST::new(&Position::default(), Node::GeOp));
        let third = Box::from(AST::new(&Position::default(), Node::LeOp));

        two_ast!(Node::IfElse {cond: cond.clone(), then: body.clone(), el: Some(third.clone())});
        two_ast!(Node::Match {cond: cond.clone(), cases: vec![*body.clone(), *third.clone()]});
        two_ast!(Node::Case {cond: cond.clone(), body: body.clone()});
        two_ast!(Node::Range { from: cond.clone(), to: body.clone(), inclusive: true, step: Some(third.clone()) });
        two_ast!(Node::For {expr: cond.clone(), col: body.clone(), body: third.clone()});
        two_ast!(Node::In {left:cond.clone(), right: body.clone()});
        two_ast!(Node::Step {amount: body.clone()});

        two_ast!(Node::While {cond: cond.clone(), body: body.clone()});
    }

    #[test]
    fn cntrl_flow_op_equal_value() {
        two_ast!(Node::Break);
        two_ast!(Node::Continue);
        two_ast!(Node::ReturnEmpty);
        two_ast!(Node::Underscore);
        two_ast!(Node::Undefined);
        two_ast!(Node::Pass);
    }

    #[test]
    fn return_equal_value() {
        two_ast!(Node::Return { expr: Box::from(AST::new(&Position::default(), Node::Continue)) });
    }

    #[test]
    fn print_equal_value() {
        two_ast!(Node::Print { expr: Box::from(AST::new(&Position::default(), Node::Continue)) });
    }

    #[test]
    fn question_equal_value() {
        two_ast!(Node::QuestionOp { expr: Box::from(AST::new(&Position::default(), Node::Continue)) });
        two_ast!(Node::Question { left: Box::from(AST::new(&Position::default(), Node::Continue)), right: Box::from(AST::new(&Position::default(), Node::Break)) });
    }

    #[test]
    fn comment_op_equal_value() {
        two_ast!(Node::Comment { comment: String::from("cca") });
    }

    #[test]
    fn comment_op_equal_value_different_string() {
        two_ast!(Node::Comment { comment: String::from("cca") }, Node::Comment { comment: String::from("aaa") });
    }
}
