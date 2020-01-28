use crate::parser::ast::{Node, AST};

fn equal_optional(this: &Option<Box<AST>>, that: &Option<Box<AST>>) -> bool {
    if let (Some(this), Some(that)) = (this, that) {
        this.equal_structure(that)
    } else {
        false
    }
}

fn equal_vec(this: &Vec<AST>, other: &Vec<AST>) -> bool {
    if this.len() != other.len() {
        false
    } else {
        for (left, right) in this.iter().zip(other) {
            if !left.equal_structure(right) {
                return false;
            }
        }
        true
    }
}

impl Node {
    pub fn equal_structure(&self, other: &Node) -> bool {
        match (&self, &other) {
            (Node::File { pure: lp, modules: lm }, Node::File { pure: rp, modules: rm }) =>
                lp == rp && equal_vec(lm, rm),
            (Node::Import { import: li, _as: la }, Node::Import { import: ri, _as: ra }) =>
                equal_vec(li, ri) && equal_vec(la, ra),
            (
                Node::FromImport { id: lid, import: li },
                Node::FromImport { id: rid, import: ri }
            ) => lid.equal_structure(rid) && li.equal_structure(ri),
            (
                Node::Class { ty: lt, args: la, parents: lp, body: lb },
                Node::Class { ty: rt, args: ra, parents: rp, body: rb }
            ) =>
                lt.equal_structure(rt)
                    && equal_vec(la, ra)
                    && equal_vec(lp, rp)
                    && equal_optional(lb, rb),
            (Node::Generic { id: li, isa: lisa }, Node::Generic { id: ri, isa: risa }) =>
                li.equal_structure(ri) && equal_optional(lisa, risa),
            (
                Node::Parent { id: li, generics: lg, args: la },
                Node::Parent { id: ri, generics: rg, args: ra }
            ) => li.equal_structure(ri) && equal_vec(lg, rg) && equal_vec(la, ra),
            (Node::Script { statements: l }, Node::Script { statements: r }) => equal_vec(l, r),
            (Node::Init, Node::Init) => true,
            (Node::Reassign { left: ll, right: lr }, Node::Reassign { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (
                Node::VariableDef {
                    private: lp,
                    mutable: lm,
                    var: lv,
                    ty: lt,
                    expression: le,
                    forward: lf
                },
                Node::VariableDef {
                    private: rp,
                    mutable: rm,
                    var: rv,
                    ty: rt,
                    expression: re,
                    forward: rf
                }
            ) =>
                lp == rp
                    && lm == rm
                    && lv.equal_structure(rv)
                    && equal_optional(lt, rt)
                    && equal_optional(le, re)
                    && equal_vec(lf, rf),
            (
                Node::FunDef {
                    pure: lpu,
                    private: lp,
                    id: li,
                    fun_args: la,
                    ret_ty: lret,
                    raises: lraise,
                    body: lb
                },
                Node::FunDef {
                    pure: rpu,
                    private: rp,
                    id: ri,
                    fun_args: ra,
                    ret_ty: rret,
                    raises: rraise,
                    body: rb
                }
            ) =>
                lpu == rpu
                    && lp == rp
                    && li.equal_structure(ri)
                    && equal_vec(la, ra)
                    && equal_optional(lret, rret)
                    && equal_vec(lraise, rraise)
                    && equal_optional(lb, rb),
            (Node::AnonFun { args: la, body: lb }, Node::AnonFun { args: ra, body: rb }) =>
                equal_vec(la, ra) && lb.equal_structure(rb),
            (
                Node::Raises { expr_or_stmt: les, errors: le },
                Node::Raises { expr_or_stmt: res, errors: re }
            ) => les.equal_structure(res) && equal_vec(le, re),
            (Node::Raise { error: le }, Node::Raise { error: re }) => le.equal_structure(re),
            (
                Node::Handle { expr_or_stmt: les, cases: lc },
                Node::Handle { expr_or_stmt: res, cases: rc }
            ) => les.equal_structure(res) && equal_vec(lc, rc),
            (
                Node::With { resource: lr, alias: Some((la, lmut, lty)), expr: le },
                Node::With { resource: rr, alias: Some((ra, rmut, rty)), expr: re }
            ) =>
                lr.equal_structure(rr)
                    && la.equal_structure(ra)
                    && lmut == rmut
                    && equal_optional(lty, rty)
                    && le.equal_structure(re),
            (
                Node::With { resource: lr, alias: None, expr: le },
                Node::With { resource: rr, alias: None, expr: re }
            ) => lr.equal_structure(rr) && le.equal_structure(re),
            (
                Node::ConstructorCall { name: ln, args: la },
                Node::ConstructorCall { name: rn, args: ra }
            ) => ln.equal_structure(rn) && equal_vec(la, ra),
            (
                Node::FunctionCall { name: ln, args: la },
                Node::FunctionCall { name: rn, args: ra }
            ) => ln.equal_structure(rn) && equal_vec(la, ra),
            (
                Node::PropertyCall { instance: li, property: lp },
                Node::PropertyCall { instance: ri, property: rp }
            ) => li.equal_structure(ri) && lp.equal_structure(rp),
            (Node::Id { lit: l }, Node::Id { lit: r }) => l == r,
            (
                Node::ExpressionType { expr: le, mutable: lm, ty: lt },
                Node::ExpressionType { expr: re, mutable: rm, ty: rt }
            ) => le.equal_structure(re) && lm == rm && equal_optional(lt, rt),
            (
                Node::TypeDef { ty: lt, isa: li, body: lb },
                Node::TypeDef { ty: rt, isa: ri, body: rb }
            ) => lt.equal_structure(rt) && equal_optional(li, ri) && equal_optional(lb, rb),
            (
                Node::TypeAlias { ty: lt, isa: li, conditions: lc },
                Node::TypeAlias { ty: rt, isa: ri, conditions: rc }
            ) => lt.equal_structure(rt) && li.equal_structure(ri) && equal_vec(lc, rc),
            (Node::TypeTup { types: l }, Node::TypeTup { types: r }) => equal_vec(l, r),
            (Node::TypeUnion { types: l }, Node::TypeUnion { types: r }) => equal_vec(l, r),
            (Node::Type { id: li, generics: lg }, Node::Type { id: ri, generics: rg }) =>
                li.equal_structure(ri) && equal_vec(lg, rg),
            (Node::TypeFun { args: la, ret_ty: lr }, Node::TypeFun { args: ra, ret_ty: rr }) =>
                equal_vec(la, ra) && lr.equal_structure(rr),
            (Node::Condition { cond: lc, el: le }, Node::Condition { cond: rc, el: re }) =>
                lc.equal_structure(rc) && equal_optional(le, re),
            (
                Node::FunArg { vararg: lv, mutable: lm, var: lvar, ty: lt, default: ld },
                Node::FunArg { vararg: rv, mutable: rm, var: rvar, ty: rt, default: rd }
            ) =>
                lv == rv
                    && lm == rm
                    && lvar.equal_structure(rvar)
                    && equal_optional(lt, rt)
                    && equal_optional(ld, rd),
            (Node::_Self, Node::_Self) => true,
            (Node::AddOp, Node::AddOp) => true,
            (Node::SubOp, Node::SubOp) => true,
            (Node::SqrtOp, Node::SqrtOp) => true,
            (Node::MulOp, Node::MulOp) => true,
            (Node::FDivOp, Node::FDivOp) => true,
            (Node::DivOp, Node::DivOp) => true,
            (Node::PowOp, Node::PowOp) => true,
            (Node::ModOp, Node::ModOp) => true,
            (Node::EqOp, Node::EqOp) => true,
            (Node::LeOp, Node::LeOp) => true,
            (Node::GeOp, Node::GeOp) => true,
            (
                Node::SetBuilder { item: li, conditions: lc },
                Node::SetBuilder { item: ri, conditions: rc }
            ) => li.equal_structure(ri) && equal_vec(lc, rc),
            (
                Node::ListBuilder { item: li, conditions: lc },
                Node::ListBuilder { item: ri, conditions: rc }
            ) => li.equal_structure(ri) && equal_vec(lc, rc),
            (Node::Set { elements: l }, Node::Set { elements: r }) => equal_vec(l, r),
            (Node::List { elements: l }, Node::List { elements: r }) => equal_vec(l, r),
            (Node::Tuple { elements: l }, Node::Tuple { elements: r }) => equal_vec(l, r),
            (
                Node::Range { from: lf, to: lt, inclusive: li, step: ls },
                Node::Range { from: rf, to: rt, inclusive: ri, step: rs }
            ) =>
                lf.equal_structure(rf)
                    && lt.equal_structure(rt)
                    && li == ri
                    && equal_optional(ls, rs),
            (Node::Block { statements: l }, Node::Block { statements: r }) => equal_vec(l, r),
            (Node::Real { lit: l }, Node::Real { lit: r }) => l == r,
            (Node::Int { lit: l }, Node::Int { lit: r }) => l == r,
            (Node::ENum { num: ln, exp: le }, Node::ENum { num: rn, exp: re }) =>
                ln == rn && le == re,
            (Node::Str { lit: l, expressions: le }, Node::Str { lit: r, expressions: re }) =>
                l == r && equal_vec(le, re),
            (Node::DocStr { lit: l }, Node::DocStr { lit: r }) => l == r,
            (Node::Bool { lit: l }, Node::Bool { lit: r }) => l == r,
            (Node::AddU { expr: l }, Node::AddU { expr: r }) => l.equal_structure(r),
            (Node::SubU { expr: l }, Node::SubU { expr: r }) => l.equal_structure(r),
            (Node::Add { left: ll, right: lr }, Node::Add { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Sub { left: ll, right: lr }, Node::Sub { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Mul { left: ll, right: lr }, Node::Mul { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Div { left: ll, right: lr }, Node::Div { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::BOr { left: ll, right: lr }, Node::BOr { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Mod { left: ll, right: lr }, Node::Mod { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Pow { left: ll, right: lr }, Node::Pow { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Sqrt { expr: l }, Node::Sqrt { expr: r }) => l.equal_structure(r),
            (Node::FDiv { left: ll, right: lr }, Node::FDiv { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::BAnd { left: ll, right: lr }, Node::BAnd { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::BXOr { left: ll, right: lr }, Node::BXOr { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::BOneCmpl { expr: l }, Node::BOneCmpl { expr: r }) => l.equal_structure(r),
            (Node::BLShift { left: ll, right: lr }, Node::BLShift { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::BRShift { left: ll, right: lr }, Node::BRShift { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Leq { left: ll, right: lr }, Node::Leq { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Geq { left: ll, right: lr }, Node::Geq { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::IsN { left: ll, right: lr }, Node::IsN { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Neq { left: ll, right: lr }, Node::Neq { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::IsA { left: ll, right: lr }, Node::IsA { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Le { left: ll, right: lr }, Node::Le { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Ge { left: ll, right: lr }, Node::Ge { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Is { left: ll, right: lr }, Node::Is { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Eq { left: ll, right: lr }, Node::Eq { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::IsNA { left: ll, right: lr }, Node::IsNA { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Not { expr: l }, Node::Not { expr: r }) => l.equal_structure(r),
            (Node::And { left: ll, right: lr }, Node::And { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Or { left: ll, right: lr }, Node::Or { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (
                Node::IfElse { cond: lc, then: lt, el: le },
                Node::IfElse { cond: rc, then: rt, el: re }
            ) => lc.equal_structure(rc) && lt.equal_structure(rt) && equal_optional(le, re),
            (Node::Match { cond: lco, cases: lc }, Node::Match { cond: rco, cases: rc }) =>
                lco.equal_structure(rco) && equal_vec(lc, rc),
            (Node::Case { cond: lc, body: lb }, Node::Case { cond: rc, body: rb }) =>
                lc.equal_structure(rc) && lb.equal_structure(rb),
            (
                Node::For { expr: le, col: lc, body: lb },
                Node::For { expr: re, col: rc, body: rb }
            ) => le.equal_structure(re) && lc.equal_structure(rc) && lb.equal_structure(rb),
            (Node::In { left: ll, right: lr }, Node::In { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::Step { amount: la }, Node::Step { amount: ra }) => la.equal_structure(ra),
            (Node::While { cond: lc, body: lb }, Node::While { cond: rc, body: rb }) =>
                lc.equal_structure(rc) && lb.equal_structure(rb),
            (Node::Break, Node::Break) => true,
            (Node::Continue, Node::Continue) => true,
            (Node::Return { expr: left }, Node::Return { expr: right }) =>
                left.equal_structure(right),
            (Node::ReturnEmpty, Node::ReturnEmpty) => true,
            (Node::Underscore, Node::Underscore) => true,
            (Node::Undefined, Node::Undefined) => true,
            (Node::Pass, Node::Pass) => true,
            (Node::Question { left: ll, right: lr }, Node::Question { left: rl, right: rr }) =>
                ll.equal_structure(rl) && lr.equal_structure(rr),
            (Node::QuestionOp { expr: left }, Node::QuestionOp { expr: right }) =>
                left.equal_structure(right),
            (Node::Print { expr: left }, Node::Print { expr: right }) =>
                left.equal_structure(right),
            (Node::Comment { .. }, Node::Comment { .. }) => true,
            _ => false
        }
    }
}