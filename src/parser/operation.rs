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
        $it.eat_token(Token::$ast)?;
        let right = $it.parse(&$fun, $msg)?;
        let (en_line, en_pos) = (right.en_line, right.en_pos);
        let node = ASTNode::$ast { left: $left, right };
        Ok(Box::from(ASTNodePos { st_line: $st_line, st_pos: $st_pos, en_line, en_pos, node }))
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
    let arithmetic = it.parse(&parse_level_7, "comparison")?;
    if it.peak_if_fn(&is_start_expression_exclude_unary) {
        parse_call(&arithmetic, it)
    } else {
        Ok(arithmetic)
    }
}

fn parse_level_7(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_6, "comparison")?;

    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::And => bin_op!(it, parse_level_7, And, arithmetic.clone(), "and"),
            Token::Or => bin_op!(it, parse_level_7, Or, arithmetic.clone(), "or"),
            Token::QuestOr =>
                bin_op!(it, parse_level_7, QuestOr, arithmetic.clone(), "question or"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_6(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_5, "comparison")?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Ge => bin_op!(it, parse_level_6, Ge, arithmetic.clone(), "greater"),
            Token::Geq => bin_op!(it, parse_level_6, Geq, arithmetic.clone(), "greater, equal"),
            Token::Le => bin_op!(it, parse_level_6, Le, arithmetic.clone(), "less"),
            Token::Leq => bin_op!(it, parse_level_6, Leq, arithmetic.clone(), "less, equal"),
            Token::Eq => bin_op!(it, parse_level_6, Eq, arithmetic.clone(), "equal"),
            Token::Neq => bin_op!(it, parse_level_6, Neq, arithmetic.clone(), "not equal"),
            Token::Is => bin_op!(it, parse_level_6, Is, arithmetic.clone(), "is"),
            Token::IsN => bin_op!(it, parse_level_6, IsN, arithmetic.clone(), "is not"),
            Token::IsA => bin_op!(it, parse_level_6, IsA, arithmetic.clone(), "is a"),
            Token::IsNA => bin_op!(it, parse_level_6, IsNA, arithmetic.clone(), "is not a"),
            Token::In => bin_op!(it, parse_level_6, In, arithmetic.clone(), "in"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_5(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_4, "comparison")?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::BLShift =>
                bin_op!(it, parse_level_5, BLShift, arithmetic.clone(), "bitwise left shift"),
            Token::BRShift =>
                bin_op!(it, parse_level_5, BRShift, arithmetic.clone(), "bitwise right shift"),
            Token::BAnd => bin_op!(it, parse_level_5, BAnd, arithmetic.clone(), "bitwise and"),
            Token::BOr => bin_op!(it, parse_level_5, BOr, arithmetic.clone(), "bitwise or"),
            Token::BXOr => bin_op!(it, parse_level_5, BXOr, arithmetic.clone(), "bitwise xor"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_4(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_3, "comparison")?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Add => bin_op!(it, parse_level_4, Add, arithmetic.clone(), "add"),
            Token::Sub => bin_op!(it, parse_level_4, Sub, arithmetic.clone(), "sub"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_3(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_level_2, "comparison")?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Mul => bin_op!(it, parse_level_3, Mul, arithmetic.clone(), "mul"),
            Token::Div => bin_op!(it, parse_level_3, Div, arithmetic.clone(), "div"),
            Token::FDiv => bin_op!(it, parse_level_3, FDiv, arithmetic.clone(), "floor div"),
            Token::Mod => bin_op!(it, parse_level_3, Mod, arithmetic.clone(), "mod"),
            Token::Range => {
                it.eat_token(Token::Range)?;
                let to = it.parse(&parse_operation, "range")?;
                let step = it.parse_if_token(Token::Step, &parse_expression, "step")?;
                let (en_line, en_pos) = (to.en_line, to.en_pos);
                let node = ASTNode::Range { from: arithmetic.clone(), to, inclusive: false, step };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            Token::RangeIncl => {
                it.eat_token(Token::RangeIncl)?;
                let to = it.parse(&parse_operation, "range inclusive")?;
                let step = it.parse_if_token(Token::Step, &parse_expression, "step")?;

                let (en_line, en_pos) = (to.en_line, to.en_pos);
                let node = ASTNode::Range { from: arithmetic.clone(), to, inclusive: true, step };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_2(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    macro_rules! un_op {
        ($it:expr, $fun:path, $tok:ident, $ast:ident, $msg:expr) => {{
            $it.eat_token(Token::$tok)?;
            let factor: Box<ASTNodePos> = $it.parse(&$fun, $msg)?;
            let (en_line, en_pos) = (factor.en_line, factor.en_pos);
            let node = ASTNode::$ast { expr: factor };
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }};
    }

    it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::Add => un_op!(it, parse_level_2, Add, AddU, "plus"),
            Token::Sub => un_op!(it, parse_level_2, Sub, SubU, "subtract"),
            Token::Sqrt => un_op!(it, parse_operation, Sqrt, Sqrt, "square root"),
            Token::Not => un_op!(it, parse_operation, Not, Not, "not"),
            Token::BOneCmpl =>
                un_op!(it, parse_operation, BOneCmpl, BOneCmpl, "bitwise ones compliment"),
            _ => parse_level_1(it)
        },
        CustomEOFErr { expected: String::from("expression after unary operator") }
    )
}

fn parse_level_1(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let arithmetic = it.parse(&parse_factor, "comparison")?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, st_line, st_pos, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Pow => bin_op!(it, parse_level_1, Pow, arithmetic.clone(), "exponent"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_factor(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let (en_line, en_pos) = it.end_pos()?;
    macro_rules! literal {
        ($it:expr, $factor:expr, $ast:ident) => {{
            $it.eat_token(Token::$ast($factor.clone()))?;
            let node = ASTNode::$ast { lit: $factor };
            Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
        }};
    }

    it.peek_or_err(
        &|it, token_pos| match token_pos {
            TokenPos { token: Token::Id(_), .. } => parse_id(it),
            TokenPos { token: Token::_Self, .. } => parse_id(it),
            TokenPos { token: Token::Real(real), .. } => literal!(it, real.to_string(), Real),
            TokenPos { token: Token::Int(int), .. } => literal!(it, int.to_string(), Int),
            TokenPos { token: Token::Bool(b), .. } => literal!(it, b.clone(), Bool),
            TokenPos { token: Token::Str(str), .. } => literal!(it, str.to_string(), Str),
            TokenPos { token: Token::ENum(num, exp), .. } => {
                it.eat_token(Token::ENum(num.clone(), exp.clone()))?;
                let (en_line, en_pos) = it.end_pos()?;
                let node = ASTNode::ENum { num: num.to_string(), exp: exp.to_string() };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
            _ => it.parse(&parse_expression, "factor")
        },
        CustomEOFErr { expected: "factor".to_string() }
    )
}
