use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_call;
use crate::parser::common::end_pos;
use crate::parser::common::start_pos;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::TPIterator;

macro_rules! inner_bin_op {
    ($it:expr, $st_line:expr, $st_pos:expr, $fun:path, $ast:ident, $left:expr, $msg:expr) => {{
        $it.next();
        let right: Box<ASTNodePos> = get_or_err!($it, $fun, $msg);
        Ok(ASTNodePos {
            st_line: $st_line,
            st_pos:  $st_pos,
            en_line: right.en_line,
            en_pos:  right.en_pos,
            node:    ASTNode::$ast { left: $left, right }
        })
    }};
}

/// Parse an operation.
///
/// Precedence is as follows, from top to bottom:
/// 1. exponent
/// 2. unary and, unary or, bitwise ones complement
/// 3. multiplication, division, floor division, modulus, range, range inclusive
/// 4. addition, subtraction
/// 5. binary left shift, binary right shift, binary and, binary or, binary xor
/// 6. greater, greater or equal, less, less or equal, equal, not equal, is, is,
/// in not, is a, is not a
/// 7. and, or, question or
/// 8. postfix calls
pub fn parse_operation(it: &mut TPIterator) -> ParseResult { parse_level_8(it) }

fn parse_level_8(it: &mut TPIterator) -> ParseResult {
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_7, "comparison");
    match it.peek() {
        Some(&tp) if is_start_expression_exclude_unary(tp) => parse_call(*arithmetic, it),
        _ => Ok(*arithmetic)
    }
}

fn parse_level_7(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_6, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::And, .. }) => bin_op!(parse_level_7, And, "and"),
        Some(TokenPos { token: Token::Or, .. }) => bin_op!(parse_level_7, Or, "or"),
        Some(TokenPos { token: Token::QuestOr, .. }) => bin_op!(parse_level_7, QuestOr, "?or"),
        _ => Ok(*arithmetic)
    }
}

fn parse_level_6(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_5, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Ge, .. }) => bin_op!(parse_level_6, Ge, "greater"),
        Some(TokenPos { token: Token::Geq, .. }) => bin_op!(parse_level_6, Geq, "greater, equal"),
        Some(TokenPos { token: Token::Le, .. }) => bin_op!(parse_level_6, Le, "less"),
        Some(TokenPos { token: Token::Leq, .. }) => bin_op!(parse_level_6, Leq, "less, equal"),
        Some(TokenPos { token: Token::Eq, .. }) => bin_op!(parse_level_6, Eq, "equal"),
        Some(TokenPos { token: Token::Neq, .. }) => bin_op!(parse_level_6, Neq, "not equal"),
        Some(TokenPos { token: Token::Is, .. }) => bin_op!(parse_level_6, Is, "is"),
        Some(TokenPos { token: Token::IsN, .. }) => bin_op!(parse_level_6, IsN, "is not"),
        Some(TokenPos { token: Token::IsA, .. }) => bin_op!(parse_level_6, IsA, "is a"),
        Some(TokenPos { token: Token::IsNA, .. }) => bin_op!(parse_level_6, IsNA, "is not a"),
        Some(TokenPos { token: Token::In, .. }) => bin_op!(parse_level_6, In, "in"),
        _ => Ok(*arithmetic)
    }
}

fn parse_level_5(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_4, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::BLShift, .. }) =>
            bin_op!(parse_level_5, BLShift, "bitwise left shift"),
        Some(TokenPos { token: Token::BRShift, .. }) =>
            bin_op!(parse_level_5, BRShift, "bitwise right shift"),
        Some(TokenPos { token: Token::BAnd, .. }) => bin_op!(parse_level_5, BAnd, "bitwise and"),
        Some(TokenPos { token: Token::BOr, .. }) => bin_op!(parse_level_5, BOr, "bitwise or"),
        Some(TokenPos { token: Token::BXOr, .. }) => bin_op!(parse_level_5, BXOr, "bitwise xor"),
        _ => Ok(*arithmetic)
    }
}

fn parse_level_4(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_3, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Add, .. }) => bin_op!(parse_level_4, Add, "add"),
        Some(TokenPos { token: Token::Sub, .. }) => bin_op!(parse_level_4, Sub, "sub"),
        _ => Ok(*arithmetic)
    }
}

fn parse_level_3(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_level_2, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Mul, .. }) => bin_op!(parse_level_3, Mul, "mul"),
        Some(TokenPos { token: Token::Div, .. }) => bin_op!(parse_level_3, Div, "div"),
        Some(TokenPos { token: Token::FDiv, .. }) => bin_op!(parse_level_3, FDiv, "floor div"),
        Some(TokenPos { token: Token::Mod, .. }) => bin_op!(parse_level_3, Mod, "mod"),
        Some(TokenPos { token: Token::Range, .. }) => {
            it.next();
            let to: Box<ASTNodePos> = get_or_err!(it, parse_operation, "range");
            let step = if let Some(&TokenPos { token: Token::Step, .. }) = it.peek() {
                it.next();
                Some(get_or_err!(it, parse_expression, "step"))
            } else {
                None
            };

            let (en_line, en_pos) = (to.en_line, to.en_pos);
            let node = ASTNode::Range { from: arithmetic, to, inclusive: false, step };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(TokenPos { token: Token::RangeIncl, .. }) => {
            it.next();
            let to: Box<ASTNodePos> = get_or_err!(it, parse_operation, "range inclusive");
            let step = if let Some(&TokenPos { token: Token::Step, .. }) = it.peek() {
                it.next();
                Some(get_or_err!(it, parse_expression, "step"))
            } else {
                None
            };

            let (en_line, en_pos) = (to.en_line, to.en_pos);
            let node = ASTNode::Range { from: arithmetic, to, inclusive: true, step };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        _ => Ok(*arithmetic)
    }
}

fn parse_level_2(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    macro_rules! un_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            it.next();
            let factor: Box<ASTNodePos> = get_or_err!(it, $fun, $msg);
            let (en_line, en_pos) = (factor.en_line, factor.en_pos);
            let node = ASTNode::$ast { expr: factor };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Add, .. }) => un_op!(parse_level_2, AddU, "plus"),
        Some(TokenPos { token: Token::Sub, .. }) => un_op!(parse_level_2, SubU, "subtract"),
        Some(TokenPos { token: Token::Sqrt, .. }) => un_op!(parse_operation, Sqrt, "square root"),
        Some(TokenPos { token: Token::Not, .. }) => un_op!(parse_operation, Not, "not"),
        Some(TokenPos { token: Token::BOneCmpl, .. }) =>
            un_op!(parse_operation, BOneCmpl, "bitwise ones compliment"),
        _ => parse_level_1(it)
    }
}

fn parse_level_1(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let arithmetic: Box<ASTNodePos> = get_or_err!(it, parse_factor, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Pow, .. }) => bin_op!(parse_level_1, Pow, "exponent"),
        _ => Ok(*arithmetic)
    }
}

fn parse_factor(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let (en_line, en_pos) = end_pos(it);
    macro_rules! literal {
        ($factor:expr, $ast:ident) => {{
            it.next();
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line,
                en_pos,
                node: ASTNode::$ast { lit: $factor }
            })
        }};
    }

    match it.peek() {
        Some(TokenPos { token: Token::Id(_), .. }) => parse_id(it),
        Some(TokenPos { token: Token::_Self, .. }) => parse_id(it),
        Some(TokenPos { token: Token::Real(real), .. }) => literal!(real.to_string(), Real),
        Some(TokenPos { token: Token::Int(int), .. }) => literal!(int.to_string(), Int),
        Some(TokenPos { token: Token::Bool(ref _bool), .. }) => literal!(*_bool, Bool),
        Some(TokenPos { token: Token::Str(str), .. }) => literal!(str.to_string(), Str),
        Some(TokenPos { token: Token::ENum(num, exp), .. }) => {
            it.next();
            let (en_line, en_pos) = end_pos(it);
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line,
                en_pos,
                node: ASTNode::ENum { num: num.to_string(), exp: exp.to_string() }
            })
        }
        Some(_) => parse_expression(it),
        None => Err(CustomEOFErr { expected: "factor".to_string() })
    }
}
