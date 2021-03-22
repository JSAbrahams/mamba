use std::fmt::Debug;

use crate::common::position::Position;

pub mod node;

/// Wrapper of Node, and its start end end position in the source code.
/// The start and end positions can be used to generate useful error messages.
#[derive(PartialEq, Eq, Hash, Clone, Debug)]
pub struct AST {
    pub pos: Position,
    pub node: Node,
}

impl AST {
    pub fn new(pos: &Position, node: Node) -> AST { AST { pos: pos.clone(), node } }

    pub fn equal_structure(&self, other: &AST) -> bool { self.node.equal_structure(&other.node) }

    pub fn map(&self, mapping: &dyn Fn(&Node) -> Node) -> AST {
        AST {
            pos: self.pos.clone(),
            node: self.node.map(mapping),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum Node {
    File {
        pure: bool,
        modules: Vec<AST>,
    },
    Import {
        import: Vec<AST>,
        aliases: Vec<AST>,
    },
    FromImport {
        id: Box<AST>,
        import: Box<AST>,
    },
    Class {
        ty: Box<AST>,
        args: Vec<AST>,
        parents: Vec<AST>,
        body: Option<Box<AST>>,
    },
    Generic {
        id: Box<AST>,
        isa: Option<Box<AST>>,
    },
    Parent {
        ty: Box<AST>,
        args: Vec<AST>,
    },
    Script {
        statements: Vec<AST>
    },
    Init,
    Reassign {
        left: Box<AST>,
        right: Box<AST>,
    },
    VariableDef {
        private: bool,
        mutable: bool,
        var: Box<AST>,
        ty: Option<Box<AST>>,
        expression: Option<Box<AST>>,
        forward: Vec<AST>,
    },
    FunDef {
        pure: bool,
        private: bool,
        id: Box<AST>,
        fun_args: Vec<AST>,
        ret_ty: Option<Box<AST>>,
        raises: Vec<AST>,
        body: Option<Box<AST>>,
    },
    AnonFun {
        args: Vec<AST>,
        body: Box<AST>,
    },
    Raises {
        expr_or_stmt: Box<AST>,
        errors: Vec<AST>,
    },
    Raise {
        error: Box<AST>
    },
    Handle {
        expr_or_stmt: Box<AST>,
        cases: Vec<AST>,
    },
    With {
        resource: Box<AST>,
        alias: Option<(Box<AST>, bool, Option<Box<AST>>)>,
        expr: Box<AST>,
    },
    FunctionCall {
        name: Box<AST>,
        args: Vec<AST>,
    },
    PropertyCall {
        instance: Box<AST>,
        property: Box<AST>,
    },
    Id {
        lit: String
    },
    ExpressionType {
        expr: Box<AST>,
        mutable: bool,
        ty: Option<Box<AST>>,
    },
    TypeDef {
        ty: Box<AST>,
        isa: Option<Box<AST>>,
        body: Option<Box<AST>>,
    },
    TypeAlias {
        ty: Box<AST>,
        isa: Box<AST>,
        conditions: Vec<AST>,
    },
    TypeTup {
        types: Vec<AST>
    },
    TypeUnion {
        types: Vec<AST>
    },
    Type {
        id: Box<AST>,
        generics: Vec<AST>,
    },
    TypeFun {
        args: Vec<AST>,
        ret_ty: Box<AST>,
    },
    Condition {
        cond: Box<AST>,
        el: Option<Box<AST>>,
    },
    FunArg {
        vararg: bool,
        mutable: bool,
        var: Box<AST>,
        ty: Option<Box<AST>>,
        default: Option<Box<AST>>,
    },
    _Self,
    AddOp,
    SubOp,
    SqrtOp,
    MulOp,
    FDivOp,
    DivOp,
    PowOp,
    ModOp,
    EqOp,
    LeOp,
    GeOp,
    Set {
        elements: Vec<AST>
    },
    SetBuilder {
        item: Box<AST>,
        conditions: Vec<AST>,
    },
    List {
        elements: Vec<AST>
    },
    ListBuilder {
        item: Box<AST>,
        conditions: Vec<AST>,
    },
    Tuple {
        elements: Vec<AST>
    },
    Range {
        from: Box<AST>,
        to: Box<AST>,
        inclusive: bool,
        step: Option<Box<AST>>,
    },
    Block {
        statements: Vec<AST>
    },
    Real {
        lit: String
    },
    Int {
        lit: String
    },
    ENum {
        num: String,
        exp: String,
    },
    Str {
        lit: String,
        expressions: Vec<AST>,
    },
    DocStr {
        lit: String
    },
    Bool {
        lit: bool
    },
    Add {
        left: Box<AST>,
        right: Box<AST>,
    },
    AddU {
        expr: Box<AST>
    },
    Sub {
        left: Box<AST>,
        right: Box<AST>,
    },
    SubU {
        expr: Box<AST>
    },
    Mul {
        left: Box<AST>,
        right: Box<AST>,
    },
    Div {
        left: Box<AST>,
        right: Box<AST>,
    },
    FDiv {
        left: Box<AST>,
        right: Box<AST>,
    },
    Mod {
        left: Box<AST>,
        right: Box<AST>,
    },
    Pow {
        left: Box<AST>,
        right: Box<AST>,
    },
    Sqrt {
        expr: Box<AST>
    },
    BAnd {
        left: Box<AST>,
        right: Box<AST>,
    },
    BOr {
        left: Box<AST>,
        right: Box<AST>,
    },
    BXOr {
        left: Box<AST>,
        right: Box<AST>,
    },
    BOneCmpl {
        expr: Box<AST>
    },
    BLShift {
        left: Box<AST>,
        right: Box<AST>,
    },
    BRShift {
        left: Box<AST>,
        right: Box<AST>,
    },
    Le {
        left: Box<AST>,
        right: Box<AST>,
    },
    Ge {
        left: Box<AST>,
        right: Box<AST>,
    },
    Leq {
        left: Box<AST>,
        right: Box<AST>,
    },
    Geq {
        left: Box<AST>,
        right: Box<AST>,
    },
    Is {
        left: Box<AST>,
        right: Box<AST>,
    },
    IsN {
        left: Box<AST>,
        right: Box<AST>,
    },
    Eq {
        left: Box<AST>,
        right: Box<AST>,
    },
    Neq {
        left: Box<AST>,
        right: Box<AST>,
    },
    IsA {
        left: Box<AST>,
        right: Box<AST>,
    },
    IsNA {
        left: Box<AST>,
        right: Box<AST>,
    },
    Not {
        expr: Box<AST>
    },
    And {
        left: Box<AST>,
        right: Box<AST>,
    },
    Or {
        left: Box<AST>,
        right: Box<AST>,
    },
    IfElse {
        cond: Box<AST>,
        then: Box<AST>,
        el: Option<Box<AST>>,
    },
    Match {
        cond: Box<AST>,
        cases: Vec<AST>,
    },
    Case {
        cond: Box<AST>,
        body: Box<AST>,
    },
    For {
        expr: Box<AST>,
        col: Box<AST>,
        body: Box<AST>,
    },
    In {
        left: Box<AST>,
        right: Box<AST>,
    },
    Step {
        amount: Box<AST>
    },
    While {
        cond: Box<AST>,
        body: Box<AST>,
    },
    Break,
    Continue,
    Return {
        expr: Box<AST>
    },
    ReturnEmpty,
    Underscore,
    Undefined,
    Pass,
    Question {
        left: Box<AST>,
        right: Box<AST>,
    },
    QuestionOp {
        expr: Box<AST>
    },
    Print {
        expr: Box<AST>
    },
    Comment {
        comment: String
    },
}

impl Node {
    /// Apply mapping to node, before recursively applying mapping to result
    pub fn map(&self, mapping: &dyn Fn(&Node) -> Node) -> Node {
        match mapping(self) {
            Node::File { pure, modules } =>Node::File{
                pure,
                modules: modules.iter().map(|m| m.map(mapping)).collect()
            },
            Node::Import { import, aliases: _as } => Node::Import {
                import: import.iter().map(|i| i.map(mapping)).collect(),
                aliases: _as.iter().map(|a| a.map(mapping)).collect()
            },
            Node::FromImport { id, import } => Node::FromImport {
                id: Box::from(id.map(mapping)),
                import: Box::from(import.map(mapping))
            },
            Node::Class { ty, args, parents, body } => Node::Class{
                ty: Box::from(ty.map(mapping)),
                args: args.iter().map(|a| a.map(mapping)).collect(),
                parents: parents.iter().map(|p| p.map(mapping)).collect(),
                body: body.map(|b| Box::from(b.map(mapping)))
            },
            Node::Generic { id, isa } => Node::Generic{
                id: Box::from(id.map(mapping)),
                isa: isa.map(|isa| Box::from(isa.map(mapping)))
            },
            Node::Parent { ty, args } => Node::Parent {
              ty: Box::from(ty.map(mapping)),
                args: args.iter().map(|a| a.map(mapping)).collect()
            },
            Node::Script { statements } => Node::Script {
                statements: statements.iter().map(|s| s.map(mapping)).collect() },
            Node::Reassign { left, right } => Node::Reassign {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::VariableDef { private, mutable, var, ty, expression, forward } => Node::VariableDef {
                private,
                mutable,
                var: Box::from(var.map(mapping)),
                ty: ty.map(|t| Box::from(t.map(mapping))),
                expression: expression.map(|e| Box::from(e.map(mapping))),
                forward: forward.iter().map(|f| f.map(mapping)).collect(),
            },
            Node::FunDef { pure, private, id, fun_args, ret_ty, raises, body } => Node::FunDef {
                pure,
                private,
                id: Box::from(id.map(mapping)),
                fun_args: fun_args.iter().map(|a| a.map(mapping)).collect(),
                ret_ty: ret_ty.map(|r| Box::from(r.map(mapping))),
                raises: raises.iter().map(|r| r.map(mapping)).collect(),
                body: body.map(|b| Box::from(b.map(mapping))),
            },
            Node::AnonFun { args, body } => Node::AnonFun {
                args: args.iter().map(|a| a.map(mapping)).collect(),
                body: Box::from(body.map(mapping)),
            },
            Node::Raises { expr_or_stmt, errors } => Node::Raises {
                expr_or_stmt: Box::from(expr_or_stmt.map(mapping)),
                errors: errors.iter().map(|e| e.map(mapping)).collect(),
            },
            Node::Raise { error } => Node::Raise { error: Box::from(error.map(mapping)) },
            Node::Handle { expr_or_stmt, cases } => Node::Handle {
                expr_or_stmt: Box::from(expr_or_stmt.map(mapping)),
                cases: cases.iter().map(|c| c.map(mapping)).collect(),
            },
            Node::With { resource, alias, expr } => Node::With {
                resource: Box::from(resource.map(mapping)),
                alias: alias.map(|(resource, alias, expr)| (
                    Box::from(resource.map(mapping)),
                    alias,
                    expr.map(|expr| Box::from(expr.map(mapping)))
                )),
                expr: Box::from(expr.map(mapping)),
            },
            Node::FunctionCall { name, args } => Node::FunctionCall {
                name: Box::from(name.map(mapping)),
                args: args.iter().map(|a| a.map(mapping)).collect(),
            },
            Node::PropertyCall { instance, property } => Node::PropertyCall {
                instance: Box::from(instance.map(mapping)),
                property: Box::from(property.map(mapping)),
            },
            Node::ExpressionType { expr, mutable, ty } => Node::ExpressionType {
                expr: Box::from(expr.map(mapping)),
                mutable,
                ty: ty.map(|ty| Box::from(ty.map(mapping))),
            },
            Node::TypeDef { ty, isa, body } => Node::TypeDef {
                ty: Box::from(ty.map(mapping)),
                isa: isa.map(|isa| Box::from(isa.map(mapping))),
                body: body.map(|body| Box::from(body.map(mapping))),
            },
            Node::TypeAlias { ty, isa, conditions } => Node::TypeAlias {
                ty: Box::from(ty.map(mapping)),
                isa: Box::from(isa.map(mapping)),
                conditions: conditions.iter().map(|c| c.map(mapping)).collect(),
            },
            Node::TypeTup { types } => Node::TypeTup {
                types: types.iter().map(|ty| ty.map(mapping)).collect()
            },
            Node::TypeUnion { types } => Node::TypeUnion {
                types: types.iter().map(|ty| ty.map(mapping)).collect()
            },
            Node::Type { id, generics } => Node::Type {
                id: Box::from(id.map(mapping)),
                generics: generics.iter().map(|gen| gen.map(mapping)).collect(),
            },
            Node::TypeFun { args, ret_ty } => Node::TypeFun {
                args: args.iter().map(|arg| arg.map(mapping)).collect(),
                ret_ty: Box::from(ret_ty.map(mapping)),
            },
            Node::Condition { cond, el } => Node::Condition {
                cond: Box::from(cond.map(mapping)),
                el: el.map(|el| Box::from(el.map(mapping))),
            },
            Node::FunArg { vararg, mutable, var, ty, default } => Node::FunArg {
                vararg,
                mutable,
                var: Box::from(var.map(mapping)),
                ty: ty.map(|ty| Box::from(ty.map(mapping))),
                default: default.map(|d| Box::from(d.map(mapping))),
            },
            Node::Set { elements } => Node::Set {
                elements: elements.iter().map(|e| e.map(mapping)).collect()
            },
            Node::SetBuilder { item, conditions } => Node::SetBuilder {
                item: Box::from(item.map(mapping)),
                conditions: conditions.iter().map(|cond| cond.map(mapping)).collect(),
            },
            Node::List { elements } => Node::List {
                elements: elements.iter().map(|e| e.map(mapping)).collect()
            },
            Node::ListBuilder { item, conditions } => Node::ListBuilder {
                item: Box::from(item.map(mapping)),
                conditions: conditions.iter().map(|cond| cond.map(mapping)).collect(),
            },
            Node::Tuple { elements } => Node::Tuple {
                elements: elements.iter().map(|e| e.map(mapping)).collect()
            },
            Node::Range { from, to, inclusive, step } => Node::Range {
                from: Box::from(from.map(mapping)),
                to: Box::from(to.map(mapping)),
                inclusive,
                step: step.map(|ast| Box::from(ast.map(mapping))),
            },
            Node::Block { statements } => Node::Block {
                statements: statements.iter().map(|stmt| stmt.map(mapping)).collect()
            },
            Node::Add { left, right } => Node::Add { left: Box::from(left.map(mapping)), right: Box::from(right.map(mapping)) },
            Node::AddU { expr } => Node::AddU {
                expr: Box::from(expr.map(mapping))
            },
            Node::Sub { left, right } => Node::Sub {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::SubU { expr } => Node::SubU {
                expr: Box::from(expr.map(mapping))
            },
            Node::Mul { left, right } => Node::Mul {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Div { left, right } => Node::Div {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::FDiv { left, right } => Node::FDiv {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Mod { left, right } => Node::Mod {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Pow { left, right } => Node::Pow {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Sqrt { expr } => Node::Sqrt { expr: Box::from(expr.map(mapping)) },
            Node::BAnd { left, right } => Node::BAnd {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::BOr { left, right } => Node::BOr {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::BXOr { left, right } => Node::BXOr {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::BOneCmpl { expr } => Node::BOneCmpl { expr: Box::from(expr.map(mapping)) },
            Node::BLShift { left, right } => Node::BLShift {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::BRShift { left, right } => Node::BRShift {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Le { left, right } => Node::Le {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Ge { left, right } => Node::Ge {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Leq { left, right } => Node::Leq {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Geq { left, right } => Node::Geq {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Is { left, right } => Node::Is {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::IsN { left, right } => Node::IsN {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Eq { left, right } => Node::Eq {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Neq { left, right } => Node::Neq {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::IsA { left, right } => Node::IsA {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::IsNA { left, right } => Node::IsNA {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Not { expr } => Node::Not { expr: Box::from(expr.map(mapping)) },
            Node::And { left, right } => Node::And {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Or { left, right } => Node::Or {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::IfElse { cond, then, el } => Node::IfElse {
                cond: Box::from(cond.map(mapping)),
                then: Box::from(then.map(mapping)),
                el: el.map(|el| Box::from(el.map(mapping))),
            },
            Node::Match { cond, cases } => Node::Match {
                cond: Box::from(cond.map(mapping)),
                cases: cases.iter().map(|c| c.map(mapping)).collect(),
            },
            Node::Case { cond, body } => Node::Case {
                cond: Box::from(cond.map(mapping)),
                body: Box::from(body.map(mapping)),
            },
            Node::For { expr, col, body } => Node::For {
                expr: Box::from(expr.map(mapping)),
                col: Box::from(col.map(mapping)),
                body: Box::from(body.map(mapping)),
            },
            Node::In { left, right } => Node::In {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::Step { amount } => Node::Step { amount: Box::from(amount.map(mapping)) },
            Node::While { cond, body } => Node::While {
                cond: Box::from(cond.map(mapping)),
                body: Box::from(body.map(mapping)),
            },
            Node::Return { expr } => Node::Return {
                expr: Box::from(expr.map(mapping))
            },
            Node::Question { left, right } => Node::Question {
                left: Box::from(left.map(mapping)),
                right: Box::from(right.map(mapping)),
            },
            Node::QuestionOp { expr } => Node::QuestionOp { expr: Box::from(expr.map(mapping)) },
            Node::Print { expr } => Node::Print { expr: Box::from(expr.map(mapping)) },

            other => mapping(&other)
        }
    }
}
