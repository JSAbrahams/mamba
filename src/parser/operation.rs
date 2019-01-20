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

macro_rules! inner_bin_op { ($it:expr, $st_line:expr, $st_pos: expr, $fun:path, $ast:ident,
                             $left: expr, $msg:expr) => {{
    $it.next();
    let right: Box<ASTNodePos> = get_or_err!($it, $fun, $msg);
    Ok(ASTNodePos {
        st_line: $st_line, st_pos: $st_pos, en_line: right.en_line, en_pos: right.en_pos,
        node: ASTNode::$ast { left: $left, right },
    })
}}}

pub fn parse_operation(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let relation: Box<ASTNodePos> = get_or_err!(it, parse_relation, "operation");

    macro_rules! bin_op { ($fun: path, $ast: ident, $msg:expr) => {{
        inner_bin_op!(it, st_line, st_pos, $fun, $ast, relation, $msg)
    }}}

    return match it.peek() {
        Some(TokenPos { token: Token::Eq, .. }) => bin_op!(parse_operation, Eq, "equal"),
        Some(TokenPos { token: Token::Neq, .. }) => bin_op!(parse_operation, Neq, "not equal"),
        Some(TokenPos { token: Token::Is, .. }) => bin_op!(parse_operation, Is, "is"),
        Some(TokenPos { token: Token::IsN, .. }) => bin_op!(parse_operation, IsN, "is not"),
        Some(TokenPos { token: Token::And, .. }) => bin_op!(parse_operation, And, "and"),
        Some(TokenPos { token: Token::Or, .. }) => bin_op!(parse_operation, Or, "or"),
        Some(TokenPos { token: Token::IsA, .. }) => bin_op!(parse_operation, IsA, "is a"),
        _ => Ok(*relation)
    };
}

fn parse_relation(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_arithmetic, "comparison");

    macro_rules! bin_op { ($fun: path, $ast: ident, $msg:expr) => {{
        inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
    }}}

    return match it.peek() {
        Some(TokenPos { token: Token::Ge, .. }) => bin_op!(parse_relation, IsA, "greater than"),
        Some(TokenPos { token: Token::Geq, .. }) =>
            bin_op!(parse_relation, Geq, "greater or equal than"),
        Some(TokenPos { token: Token::Le, .. }) => bin_op!(parse_relation, Le, "less than"),
        Some(TokenPos { token: Token::Leq, .. }) =>
            bin_op!(parse_relation, Leq, "less or equal than"),
        _ => Ok(*arithmetic)
    };
}

fn parse_arithmetic(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let term: Box<ASTNodePos> = get_or_err!(it, parse_term, "arithmetic");

    macro_rules! bin_op { ($fun: path, $ast: ident, $msg:expr) => {{
        inner_bin_op!(it, st_line, st_pos, $fun, $ast, term, $msg)
    }}}

    match it.peek() {
        Some(TokenPos { token: Token::Add, .. }) => bin_op!(parse_arithmetic, Add, "add"),
        Some(TokenPos { token: Token::Sub, .. }) => bin_op!(parse_arithmetic, Sub, "subtract"),
        _ => Ok(*term)
    }
}

fn parse_term(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let inner_term: Box<ASTNodePos> = get_or_err!(it, parse_inner_term, "term");

    macro_rules! bin_op { ($fun: path, $ast: ident, $msg:expr) => {{
        inner_bin_op!(it, st_line, st_pos, $fun, $ast, inner_term, $msg)
    }}}

    return match it.peek() {
        Some(TokenPos { token: Token::Mul, .. }) => bin_op!(parse_term, Mul, "multiply"),
        Some(TokenPos { token: Token::Div, .. }) => bin_op!(parse_term, Div, "divide"),
        _ => Ok(*inner_term)
    };
}

fn parse_inner_term(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let factor: Box<ASTNodePos> = get_or_err!(it, parse_factor, "inner term");

    macro_rules! bin_op { ($fun: path, $ast: ident, $msg:expr) => {{
        inner_bin_op!(it, st_line, st_pos, $fun, $ast, factor, $msg)
    }}}

    return match it.peek() {
        Some(TokenPos { token: Token::Pow, .. }) => bin_op!(parse_inner_term, Pow, "power"),
        Some(TokenPos { token: Token::Mod, .. }) => bin_op!(parse_inner_term, Mod, "modulus"),
        _ => Ok(*factor)
    };
}

fn parse_factor(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    macro_rules! un_op { ($fun: path, $ast: ident, $msg:expr) => {{
        it.next();
        let factor: Box<ASTNodePos> = get_or_err!(it, $fun, $msg);
        Ok(ASTNodePos { st_line, st_pos, en_line: factor.en_line, en_pos: factor.en_pos,
                        node: ASTNode::$ast { expr: factor } })
    }}}

    return match it.peek() {
        Some(TokenPos { token: Token::Not, .. }) => un_op!(parse_operation, Not, "not"),
        Some(TokenPos { token: Token::Add, .. }) => un_op!(parse_operation, AddU, "plus"),
        Some(TokenPos { token: Token::Sub, .. }) => un_op!(parse_operation, SubU, "subtract"),
        Some(TokenPos { token: Token::Sqrt, .. }) => un_op!(parse_operation, Sqrt, "square root"),

        _ => {
            let (en_line, en_pos) = end_pos(it);
            macro_rules! literal { ($factor: expr, $ast: ident) => {{
                it.next();
                Ok(ASTNodePos { st_line, st_pos, en_line: en_line, en_pos: en_pos,
                                node: ASTNode::$ast { lit: $factor } })
            }}}

            return match it.next() {
                Some(TokenPos { token: Token::Id(id), .. }) => literal!(id.to_string(), Id),
                Some(TokenPos { token: Token::Real(real), .. }) => literal!(real.to_string(), Real),
                Some(TokenPos { token: Token::Int(int), .. }) => literal!(int.to_string(), Int),
                Some(TokenPos { token: Token::Bool(ref _bool), .. }) => literal!(*_bool, Bool),
                Some(TokenPos { token: Token::Str(str), .. }) => literal!(str.to_string(), Str),
                Some(TokenPos { token: Token::ENum(num, exp), .. }) =>
                    Ok(ASTNodePos {
                        st_line,
                        st_pos,
                        en_line,
                        en_pos,
                        node: ASTNode::ENum {
                            int_digits: num.to_string(),
                            frac_digits: exp.to_string(),
                        },
                    }),

                Some(_) => parse_expression(it),
                None => Err(CustomEOFErr { expected: "factor".to_string() })
            };
        }
    };
}
