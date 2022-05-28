use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::ast::node_op::NodeOp;
use crate::parse::control_flow_stmt::parse_cntrl_flow_stmt;
use crate::parse::definition::parse_definition;
use crate::parse::expr_or_stmt::parse_expr_or_stmt;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::{Lex, Token};
use crate::parse::operation::parse_expression;
use crate::parse::result::{eof_expected_one_of, expected, ParseResult};
use crate::parse::result::{custom, expected_one_of};
use crate::parse::ty::{parse_expression_type, parse_id};

pub fn parse_statement(it: &mut LexIterator) -> ParseResult {
    it.peek_or_err(
        &|it, lex| match lex.token {
            Token::Pass => {
                let end = it.eat(&Token::Pass, "statement")?;
                Ok(Box::from(AST::new(end, Node::Pass)))
            }
            Token::Raise => {
                it.eat(&Token::Raise, "statement")?;
                let error = it.parse(&parse_expression, "statement", lex.pos)?;
                let node = Node::Raise { error: error.clone() };
                Ok(Box::from(AST::new(lex.pos.union(error.pos), node)))
            }
            Token::Def => parse_definition(it),
            Token::With => parse_with(it),
            Token::For | Token::While => parse_cntrl_flow_stmt(it),
            _ => Err(expected_one_of(
                &[
                    Token::Pass,
                    Token::Raise,
                    Token::Def,
                    Token::With,
                    Token::For,
                    Token::While,
                ],
                lex,
                "statement",
            )),
        },
        &[
            Token::Pass,
            Token::Raise,
            Token::Def,
            Token::With,
            Token::For,
            Token::While,
        ],
        "statement",
    )
}

pub fn parse_import(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("import")?;
    let from = if it.peek_if(&|lex| lex.token == Token::From) {
        it.parse_if(&Token::From, &parse_id, "import (from)", start)?
    } else {
        None
    };

    let end = it.eat(&Token::Import, "import")?;
    let mut import = vec![];
    it.peek_while_not_tokens(&[Token::As, Token::NL], &mut |it, _| {
        import.push(*it.parse(&parse_id, "import", start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    let alias = if it.eat_if(&Token::As).is_some() {
        let mut alias = vec![];
        it.peek_while_not_token(&Token::NL, &mut |it, lex| match lex.token {
            Token::Id(_) => {
                alias.push(*it.parse(&parse_id, "as", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(expected(&Token::Id(String::new()), lex, "as"))
        })?;
        alias
    } else {
        vec![]
    };

    let end = match (import.last(), alias.last()) {
        (_, Some(ast)) => ast.pos,
        (Some(ast), _) => ast.pos,
        (..) => end,
    };
    Ok(Box::from(AST::new(start.union(end), Node::Import { from, import, alias })))
}

pub fn parse_reassignment(pre: &AST, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("reassignment")?;
    let expect = [
        Token::Assign,
        Token::AddAssign,
        Token::SubAssign,
        Token::MulAssign,
        Token::DivAssign,
        Token::PowAssign,
        Token::BLShiftAssign,
        Token::BRShiftAssign
    ];

    let (token, op) = if let Some(token) = it.peek_next() {
        match &token {
            Lex { token: Token::Assign, .. } => (Token::Assign, NodeOp::Assign),
            Lex { token: Token::AddAssign, .. } => (Token::AddAssign, NodeOp::Add),
            Lex { token: Token::SubAssign, .. } => (Token::SubAssign, NodeOp::Sub),
            Lex { token: Token::MulAssign, .. } => (Token::MulAssign, NodeOp::Mul),
            Lex { token: Token::DivAssign, .. } => (Token::DivAssign, NodeOp::Div),
            Lex { token: Token::PowAssign, .. } => (Token::PowAssign, NodeOp::Pow),
            Lex { token: Token::BLShiftAssign, .. } => (Token::BLShiftAssign, NodeOp::BLShift),
            Lex { token: Token::BRShiftAssign, .. } => (Token::BRShiftAssign, NodeOp::BRShift),
            lex => { return Err(expected_one_of(&expect, lex, "reassignment")); }
        }
    } else {
        return Err(eof_expected_one_of(&expect, "reassignment"));
    };
    it.eat(&token, "reassignment")?;

    let right = it.parse(&parse_expression, "reassignment", start)?;

    let node = Node::Reassign { left: Box::new(pre.clone()), right: right.clone(), op };
    Ok(Box::from(AST::new(start.union(right.pos), node)))
}

pub fn parse_with(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("with")?;
    it.eat(&Token::With, "with")?;
    let resource = it.parse(&parse_expression, "with", start)?;

    let alias = it.parse_if(&Token::As, &parse_expression_type, "with id", start)?;
    let alias = if let Some(alias) = &alias {
        match alias.node.clone() {
            Node::ExpressionType { expr, mutable, ty } => Some((expr, mutable, ty)),
            _ => return Err(custom("Expected expression type", alias.pos)),
        }
    } else {
        None
    };

    it.eat(&Token::Do, "with")?;
    let expr = it.parse(&parse_expr_or_stmt, "with", start)?;

    let node = Node::With { resource, alias, expr: expr.clone() };
    Ok(Box::from(AST::new(start.union(expr.pos), node)))
}

pub fn is_start_statement(tp: &Token) -> bool {
    matches!(
        tp,
        Token::Def
            | Token::Fin
            | Token::For
            | Token::While
            | Token::Pass
            | Token::Raise
            | Token::With
    )
}

#[cfg(test)]
mod test {
    use crate::parse::ast::Node;
    use crate::parse::ast::node_op::NodeOp;
    use crate::parse::parse_direct;

    #[test]
    fn parse_reassignment() {
        let source = String::from("a := 1");
        let asts = parse_direct(&source).expect("valid AST");

        assert_eq!(asts.len(), 1);
        let reassignment = asts.first().expect("reassignment");
        let (left, right, op) = match &reassignment.node {
            Node::Reassign { left, right, op } => (left.clone(), right.clone(), op.clone()),
            other => panic!("Expected reassignment, was {:?}", other),
        };

        assert_eq!(left.node, Node::Id { lit: String::from("a") });
        assert_eq!(right.node, Node::Int { lit: String::from("1") });
        assert_eq!(op, NodeOp::Assign);
    }

    #[test]
    fn parse_reassignment_call() {
        let source = String::from("a.b := 1");
        let asts = parse_direct(&source).expect("valid AST");

        assert_eq!(asts.len(), 1);
        let reassignment = asts.first().expect("reassignment");
        let (left, right, op) = match &reassignment.node {
            Node::Reassign { left, right, op } => (left.clone(), right.clone(), op.clone()),
            other => panic!("Expected reassignment, was {:?}", other),
        };

        let (object, property) = match &left.node {
            Node::PropertyCall { instance, property } => (instance.clone(), property.clone()),
            other => panic!("Expected propertycall, was {:?}", other),
        };
        assert_eq!(object.node, Node::Id { lit: String::from("a") });
        assert_eq!(property.node, Node::Id { lit: String::from("b") });

        assert_eq!(right.node, Node::Int { lit: String::from("1") });
        assert_eq!(op, NodeOp::Assign);
    }
}
