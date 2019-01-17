use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use std::env;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

pub fn parse_operation(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    print_parse!(it, "operation");
    let relation = get_or_err!(it, parse_relation, "operation");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Eq }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "equals");
            Ok(ASTNode::Eq { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Is }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "is");
            Ok(ASTNode::Is { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::IsN }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "isnot");
            Ok(ASTNode::IsN { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Neq }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "notequals");
            Ok(ASTNode::Neq { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::And }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "and");
            Ok(ASTNode::And { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Or }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "or");
            Ok(ASTNode::Or { left: relation, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::IsA }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "isa");
            Ok(ASTNode::IsA { left: relation, right })
        }
        _ => Ok(*relation)
    };
}

fn parse_relation(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    let arithmetic = get_or_err!(it, parse_arithmetic, "comparison");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Ge }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, ">");
            Ok(ASTNode::Ge { left: arithmetic, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Geq }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, ">=");
            Ok(ASTNode::Geq { left: arithmetic, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Le }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "<");
            Ok(ASTNode::Le { left: arithmetic, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Leq }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "<=");
            Ok(ASTNode::Leq { left: arithmetic, right })
        }
        _ => Ok(*arithmetic)
    };
}

fn parse_arithmetic(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    let term = get_or_err!(it, parse_term, "arithmetic");

    match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Add }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "+");
            Ok(ASTNode::Add { left: term, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "-");
            Ok(ASTNode::Sub { left: term, right })
        }
        _ => Ok(*term)
    }
}

fn parse_term(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    let inner_term = get_or_err!(it, parse_inner_term, "term");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Mul }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "*");
            Ok(ASTNode::Mul { left: inner_term, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Div }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "/");
            Ok(ASTNode::Div { left: inner_term, right })
        }
        _ => Ok(*inner_term)
    };
}

fn parse_inner_term(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    let factor = get_or_err!(it, parse_factor, "inner term");

    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Pow }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "^");
            Ok(ASTNode::Pow { left: factor, right })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Mod }) => {
            it.next();
            let right = get_or_err!(it, parse_operation, "mod");
            Ok(ASTNode::Mod { left: factor, right })
        }
        _ => Ok(*factor)
    };
}

fn parse_factor(it: &mut Peekable<Iter<TokenPos>>) -> ParseResult {
    return match it.peek() {
        Some(TokenPos { line: _, pos: _, token: Token::Not }) => {
            it.next();
            let expr = get_or_err!(it, parse_operation, "not");
            Ok(ASTNode::Not { expr })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Add }) => {
            it.next();
            let expr = get_or_err!(it, parse_operation, "+");
            Ok(ASTNode::AddU { expr })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Sub }) => {
            it.next();
            let expr = get_or_err!(it, parse_operation, "-");
            Ok(ASTNode::SubU { expr })
        }
        Some(TokenPos { line: _, pos: _, token: Token::Sqrt }) => {
            it.next();
            let expr = get_or_err!(it, parse_operation, "sqrt");
            Ok(ASTNode::Sqrt { expr })
        }

        _ => {
            return match it.next() {
                Some(TokenPos { line: _, pos: _, token: Token::Id(id) }) =>
                    Ok(ASTNode::Id { id: id.to_string() }),
                Some(TokenPos { line: _, pos: _, token: Token::Str(string) }) =>
                    Ok(ASTNode::Str { string: string.to_string() }),
                Some(TokenPos { line: _, pos: _, token: Token::Real(real) }) =>
                    Ok(ASTNode::Real { real: real.to_string() }),
                Some(TokenPos { line: _, pos: _, token: Token::Int(int) }) =>
                    Ok(ASTNode::Int { int: int.to_string() }),
                Some(TokenPos { line: _, pos: _, token: Token::ENum(num, exp) }) =>
                    Ok(ASTNode::ENum { int_digits: num.to_string(), frac_digits: exp.to_string() }),
                Some(TokenPos { line: _, pos: _, token: Token::Bool(ref _bool) }) =>
                    Ok(ASTNode::Bool { _bool: *_bool }),

                Some(_) => parse_expression(it),
                None => Err(CustomEOFErr { expected: "factor".to_string() })
            };
        }
    };
}
