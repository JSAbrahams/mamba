use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::end_pos;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_operation(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let relation: Box<ASTNodePos> = get_or_err!(it, parse_relation, "operation");

    return match it.peek() {
        Some(TokenPos { token: Token::Eq, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "equals");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: right.en_line,
                en_pos: right.en_pos,
                node: ASTNode::Eq { left: relation, right },
            })
        }
        Some(TokenPos { token: Token::Is, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "is");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Is { left: relation, right } })
        }
        Some(TokenPos { token: Token::IsN, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "isnot");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::IsN { left: relation, right } })
        }
        Some(TokenPos { token: Token::Neq, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "notequals");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Neq { left: relation, right } })
        }
        Some(TokenPos { token: Token::And, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "and");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::And { left: relation, right } })
        }
        Some(TokenPos { token: Token::Or, .. }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "or");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Or { left: relation, right } })
        }
        Some(TokenPos { token: Token::IsA, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "isa");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::IsA { left: relation, right } })
        }
        _ => Ok(*relation)
    };
}

fn parse_relation(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_arithmetic, "comparison");

    return match it.peek() {
        Some(TokenPos { token: Token::Ge, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, ">");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Ge { left: arithmetic, right } })
        }
        Some(TokenPos { token: Token::Geq, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, ">=");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Geq { left: arithmetic, right } })
        }
        Some(TokenPos { token: Token::Le, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "<");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Le { left: arithmetic, right } })
        }
        Some(TokenPos { token: Token::Leq, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "<=");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Leq { left: arithmetic, right } })
        }
        _ => Ok(*arithmetic)
    };
}

fn parse_arithmetic(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let term: Box<ASTNodePos> = get_or_err!(it, parse_term, "arithmetic");

    match it.peek() {
        Some(TokenPos { token: Token::Add, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "+");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Add { left: term, right } })
        }
        Some(TokenPos { token: Token::Sub, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "-");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Sub { left: term, right } })
        }
        _ => Ok(*term)
    }
}

fn parse_term(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let inner_term: Box<ASTNodePos> = get_or_err!(it, parse_inner_term, "term");

    return match it.peek() {
        Some(TokenPos { token: Token::Mul, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "*");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Mul { left: inner_term, right } })
        }
        Some(TokenPos { token: Token::Div, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "/");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Div { left: inner_term, right } })
        }
        _ => Ok(*inner_term)
    };
}

fn parse_inner_term(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let factor: Box<ASTNodePos> = get_or_err!(it, parse_factor, "inner term");

    return match it.peek() {
        Some(TokenPos { token: Token::Pow, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "^");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Pow { left: factor, right } })
        }
        Some(TokenPos { token: Token::Mod, .. }) => {
            it.next();
            let right: Box<ASTNodePos> = get_or_err!(it, parse_operation, "mod");
            Ok(ASTNodePos { st_line, st_pos, en_line: right.en_line, en_pos: right.en_pos, node: ASTNode::Mod { left: factor, right } })
        }
        _ => Ok(*factor)
    };
}

fn parse_factor(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match it.peek() {
        Some(TokenPos { token: Token::Not, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_operation, "not");
            Ok(ASTNodePos { st_line, st_pos, en_line: expr.en_line, en_pos: expr.en_pos, node: ASTNode::Not { expr } })
        }
        Some(TokenPos { token: Token::Add, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_operation, "+");
            Ok(ASTNodePos { st_line, st_pos, en_line: expr.en_line, en_pos: expr.en_pos, node: ASTNode::AddU { expr } })
        }
        Some(TokenPos { token: Token::Sub, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_operation, "-");
            Ok(ASTNodePos { st_line, st_pos, en_line: expr.en_line, en_pos: expr.en_pos, node: ASTNode::SubU { expr } })
        }
        Some(TokenPos { token: Token::Sqrt, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_operation, "sqrt");
            Ok(ASTNodePos { st_line, st_pos, en_line: expr.en_line, en_pos: expr.en_pos, node: ASTNode::Sqrt { expr } })
        }

        _ => {
            let (en_line, en_pos) = end_pos(it);
            return match it.next() {
                Some(TokenPos { token: Token::Id(id), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Id { id: id.to_string() } }),
                Some(TokenPos { token: Token::Str(string), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Str { string: string.to_string() } }),
                Some(TokenPos { token: Token::Real(real), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Real { real: real.to_string() } }),
                Some(TokenPos { token: Token::Int(int), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Int { int: int.to_string() } }),
                Some(TokenPos { token: Token::ENum(num, exp), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::ENum { int_digits: num.to_string(), frac_digits: exp.to_string() } }),
                Some(TokenPos { token: Token::Bool(ref _bool), .. }) =>
                    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Bool { _bool: *_bool } }),

                Some(_) => parse_expression(it),
                None => Err(CustomEOFErr { expected: "factor".to_string() })
            };
        }
    };
}
