use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_id;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_call;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

macro_rules! inner_bin_op {
    ($it:expr, $st_line:expr, $st_pos:expr, $fun:path, $ast:ident, $left:expr, $msg:expr) => {{
        $it.next();
        let right = $it.parse($fun, $msg);
        let node = ASTNode::$ast { left: $left, right };
        Ok(Box::from(ASTNodePos {
            st_line: $st_line,
            st_pos: $st_pos,
            en_line: right.en_line,
            en_pos: right.en_pos,
            node
        }))
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
    let arithmetic = it.parse(&parse_level_7, "comparison");
    if it.if_next(&is_start_expression_exclude_unary) {
        it.parse(&parse_call, "arithmetic")
    } else {
        arithmetic
    }
}

fn parse_level_7(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_6, "comparison");

    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::And => bin_op!(parse_level_7, And, "and"),
            Token::Or => bin_op!(parse_level_7, Or, "or"),
            Token::QuestOr => bin_op!(parse_level_7, QuestOr, "question or"),
            _ => arithmetic
        },
        arithmetic
    )
}

fn parse_level_6(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_5, "comparison");
    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::Ge => bin_op!(parse_level_6, Ge, "greater"),
            Token::Geq => bin_op!(parse_level_6, Geq, "greater, equal"),
            Token::Le => bin_op!(parse_level_6, Le, "less"),
            Token::Leq => bin_op!(parse_level_6, Leq, "less, equal"),
            Token::Eq => bin_op!(parse_level_6, Eq, "equal"),
            Token::Neq => bin_op!(parse_level_6, Neq, "not equal"),
            Token::Is => bin_op!(parse_level_6, Is, "is"),
            Token::IsN => bin_op!(parse_level_6, IsN, "is not"),
            Token::IsA => bin_op!(parse_level_6, IsA, "is a"),
            Token::IsNA => bin_op!(parse_level_6, IsNA, "is not a"),
            Token::In => bin_op!(parse_level_6, In, "in"),
            _ => arithmetic
        },
        arithmetic
    )
}

fn parse_level_5(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_4, "comparison");
    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::BLShift => bin_op!(parse_level_5, BLShift, "bitwise left shift"),
            Token::BRShift => bin_op!(parse_level_5, BRShift, "bitwise right shift"),
            Token::BAnd => bin_op!(parse_level_5, BAnd, "bitwise and"),
            Token::BOr => bin_op!(parse_level_5, BOr, "bitwise or"),
            Token::BXOr => bin_op!(parse_level_5, BXOr, "bitwise xor"),
            _ => arithmetic
        },
        arithmetic
    )
}

fn parse_level_4(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_3, "comparison");
    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek(
        &|token_pos| match token_pos.token {
            Token::Add => bin_op!(parse_level_4, Add, "add"),
            Token::Sub => bin_op!(parse_level_4, Sub, "sub"),
            _ => arithmetic
        },
        arithmetic
    )
}

fn parse_level_3(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_2, "comparison");
    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::Mul => bin_op!(parse_level_3, Mul, "mul"),
            Token::Div => bin_op!(parse_level_3, Div, "div"),
            Token::FDiv => bin_op!(parse_level_3, FDiv, "floor div"),
            Token::Mod => bin_op!(parse_level_3, Mod, "mod"),
            Token::Range => {
                it.eat(Token::Range);
                let to = it.parse(&parse_operation, "range")?;
                let step = it.parse_if(Token::Step, &parse_expression, "step")?;
                let (en_line, en_pos) = (to.en_line, to.en_pos);
                let node = ASTNode::Range { from: arithmetic, to, inclusive: false, step };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Token::RangeIncl => {
                it.eat(Token::RangeIncl);
                let to = it.parse(parse_operation, "range inclusive")?;
                let step = it.parse_if(Token::Step, &parse_expression, "step")?;

                let (en_line, en_pos) = (to.en_line, to.en_pos);
                let node = ASTNode::Range { from: arithmetic, to, inclusive: true, step };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => arithmetic
        },
        arithmetic
    )
}

fn parse_level_2(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    macro_rules! un_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            it.next();
            let factor: Box<ASTNodePos> = it.parse($fun, $msg);
            let (en_line, en_pos) = (factor.en_line, factor.en_pos);
            let node = ASTNode::$ast { expr: factor };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }};
    }

    it.peek(
        &|token_pos| match token_pos.token {
            Token::Add => un_op!(parse_level_2, AddU, "plus"),
            Token::Sub => un_op!(parse_level_2, SubU, "subtract"),
            Token::Sqrt => un_op!(parse_operation, Sqrt, "square root"),
            Token::Not => un_op!(parse_operation, Not, "not"),
            Token::BOneCmpl => un_op!(parse_operation, BOneCmpl, "bitwise ones compliment"),
            _ => parse_level_1(it)
        },
        CustomEOFErr { expected: String::from("expression after unary operator") }
    )
}

fn parse_level_1(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_factor, "comparison");
    macro_rules! bin_op {
        ($fun:path, $ast:ident, $msg:expr) => {{
            inner_bin_op!(it, st_line, st_pos, $fun, $ast, arithmetic, $msg)
        }};
    }

    it.peek_or(
        &|token_pos| match token_pos.token {
            Token::Pow => bin_op!(parse_level_1, Pow, "exponent"),
            _ => Ok(*arithmetic)
        },
        arithmetic
    )
}

fn parse_factor(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let (en_line, en_pos) = it.end_pos()?;
    macro_rules! literal {
        ($factor:expr, $ast:ident) => {{
            it.next();
            let node = ASTNode::$ast { lit: $factor };
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }};
    }

    it.peek(
        &|token_pos| match token_pos {
            TokenPos { token: Token::Id(_), .. } => parse_id(it),
            TokenPos { token: Token::_Self, .. } => parse_id(it),
            TokenPos { token: Token::Real(real), .. } => literal!(real.to_string(), Real),
            TokenPos { token: Token::Int(int), .. } => literal!(int.to_string(), Int),
            TokenPos { token: Token::Bool(b), .. } => literal!(b.clone(), Bool),
            TokenPos { token: Token::Str(str), .. } => literal!(str.to_string(), Str),
            TokenPos { token: Token::ENum(num, exp), .. } => {
                it.eat(Token::ENum(num.clone(), exp.clone()));
                let (en_line, en_pos) = it.end_pos()?;
                let node = ASTNode::ENum { num: num.to_string(), exp: exp.to_string() };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => it.parse(&parse_expression, "factor")
        },
        CustomEOFErr { expected: "factor".to_string() }
    )
}
