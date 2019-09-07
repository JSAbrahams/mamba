use crate::lexer::token::Token;
use crate::parser::_type::parse_id;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_call;
use crate::parser::expression::is_start_expression_exclude_unary;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseResult;

macro_rules! inner_bin_op {
    ($it:expr, $start:expr, $fun:path, $ast:ident, $left:expr, $msg:expr) => {{
        $it.eat(&Token::$ast, "operation")?;
        let right = $it.parse(&$fun, $msg, $start)?;
        let node = ASTNode::$ast { left: $left, right };
        Ok(Box::from(ASTNodePos::new($start, right.position.end, node)))
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
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_7, "operation", start)?;
    if it.peak_if_fn(&is_start_expression_exclude_unary) {
        parse_call(&arithmetic, it)
    } else {
        Ok(arithmetic)
    }
}

fn parse_level_7(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_6, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::And => bin_op!(it, parse_level_7, And, arithmetic.clone(), "and"),
            Token::Or => bin_op!(it, parse_level_7, Or, arithmetic.clone(), "or"),
            Token::Question => bin_op!(it, parse_level_7, Question, arithmetic.clone(), "question"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_6(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_5, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
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
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_4, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
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
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_3, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
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
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_2, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, token_pos| match token_pos.token {
            Token::Mul => bin_op!(it, parse_level_3, Mul, arithmetic.clone(), "mul"),
            Token::Div => bin_op!(it, parse_level_3, Div, arithmetic.clone(), "div"),
            Token::FDiv => bin_op!(it, parse_level_3, FDiv, arithmetic.clone(), "floor div"),
            Token::Mod => bin_op!(it, parse_level_3, Mod, arithmetic.clone(), "mod"),
            Token::Range => {
                it.eat(&Token::Range, "operation")?;
                let to = it.parse(&parse_operation, "operation", start)?;
                let step = it.parse_if(&Token::Step, &parse_expression, "step", start)?;
                let node = ASTNode::Range { from: arithmetic.clone(), to, inclusive: false, step };
                Ok(Box::from(ASTNodePos::new(start, to.position.end, node)))
            }
            Token::RangeIncl => {
                it.eat(&Token::RangeIncl, "operation")?;
                let to = it.parse(&parse_operation, "operation", start)?;
                let step = it.parse_if(&Token::Step, &parse_expression, "step", start)?;
                let node = ASTNode::Range { from: arithmetic.clone(), to, inclusive: true, step };
                Ok(Box::from(ASTNodePos::new(start, to.position.end, node)))
            }
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone())
    )
}

fn parse_level_2(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    macro_rules! un_op {
        ($it:expr, $fun:path, $tok:ident, $ast:ident, $msg:expr) => {{
            let factor = $it.parse(&$fun, $msg, start)?;
            let node = ASTNode::$ast { expr: factor };
            Ok(Box::from(ASTNodePos::new(start, factor.position.end, node)))
        }};
    }

    if it.eat_if(&Token::Add).is_some() {
        un_op!(it, parse_level_2, Add, AddU, "plus")
    } else if it.eat_if(&Token::Sub).is_some() {
        un_op!(it, parse_level_2, Sub, SubU, "subtract")
    } else if it.eat_if(&Token::Sqrt).is_some() {
        un_op!(it, parse_operation, Sqrt, Sqrt, "square root")
    } else if it.eat_if(&Token::Not).is_some() {
        un_op!(it, parse_operation, Not, Not, "not")
    } else if it.eat_if(&Token::BOneCmpl).is_some() {
        un_op!(it, parse_operation, BOneCmpl, BOneCmpl, "bitwise ones compliment")
    } else {
        parse_level_1(it)
    }
}

fn parse_level_1(it: &mut TPIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_factor, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
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
    let start = it.start_pos("operation")?;
    macro_rules! literal {
        ($it:expr, $factor:expr, $ast:ident) => {{
            let end = $it.eat(&Token::$ast($factor.clone()), "factor")?;
            let node = ASTNode::$ast { lit: $factor };
            Ok(Box::from(ASTNodePos::new(start, end, node)))
        }};
    }

    it.peek_or_err(
        &|it, token_pos| match &token_pos.token {
            Token::Id(_) => parse_id(it),
            Token::_Self => parse_id(it),
            Token::Real(real) => literal!(it, real.to_string(), Real),
            Token::Int(int) => literal!(it, int.to_string(), Int),
            Token::Bool(b) => literal!(it, *b, Bool),
            Token::Str(str) => literal!(it, str.to_string(), Str),
            Token::ENum(num, exp) => {
                let end = it.eat(&Token::ENum(num.clone(), exp.clone()), "factor")?;
                let node = ASTNode::ENum { num: num.to_string(), exp: exp.to_string() };
                Ok(Box::from(ASTNodePos::new(start, end, node)))
            }
            _ => it.parse(&parse_expression, "operation", start)
        },
        // TODO add system to also allow us to say something should for instance be an expression
        &[
            Token::Id(String::new()),
            Token::_Self,
            Token::Real(String::new()),
            Token::Int(String::new()),
            Token::Bool(true),
            Token::Bool(false),
            Token::Str(String::new()),
            Token::ENum(String::new(), String::new())
        ],
        "factor"
    )
}
