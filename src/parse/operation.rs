use crate::lex::token::Token;
use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::expression::parse_inner_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::result::ParseResult;

macro_rules! inner_bin_op {
    ($it:expr, $start:expr, $fun:path, $ast:ident, $left:expr, $msg:expr) => {{
        $it.eat(&Token::$ast, "operation")?;
        let right = $it.parse(&$fun, $msg, $start)?;
        let node = Node::$ast { left: $left, right: right.clone() };
        Ok(Box::from(AST::new(&$start.union(&right.pos), node)))
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
pub fn parse_expression(it: &mut LexIterator) -> ParseResult { parse_level_7(it) }

fn parse_level_7(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_6, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::And => bin_op!(it, parse_level_7, And, arithmetic.clone(), "and"),
            Token::Or => bin_op!(it, parse_level_7, Or, arithmetic.clone(), "or"),
            Token::Question => bin_op!(it, parse_level_7, Question, arithmetic.clone(), "question"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_6(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_5, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
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
        Ok(arithmetic.clone()),
    )
}

fn parse_level_5(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_4, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::BLShift =>
                bin_op!(it, parse_level_5, BLShift, arithmetic.clone(), "bitwise left shift"),
            Token::BRShift =>
                bin_op!(it, parse_level_5, BRShift, arithmetic.clone(), "bitwise right shift"),
            Token::BAnd => bin_op!(it, parse_level_5, BAnd, arithmetic.clone(), "bitwise and"),
            Token::BOr => bin_op!(it, parse_level_5, BOr, arithmetic.clone(), "bitwise or"),
            Token::BXOr => bin_op!(it, parse_level_5, BXOr, arithmetic.clone(), "bitwise xor"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_4(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_3, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Add => bin_op!(it, parse_level_4, Add, arithmetic.clone(), "add"),
            Token::Sub => bin_op!(it, parse_level_4, Sub, arithmetic.clone(), "sub"),
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_3(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_level_2, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Mul => bin_op!(it, parse_level_3, Mul, arithmetic.clone(), "mul"),
            Token::Div => bin_op!(it, parse_level_3, Div, arithmetic.clone(), "div"),
            Token::FDiv => bin_op!(it, parse_level_3, FDiv, arithmetic.clone(), "floor div"),
            Token::Mod => bin_op!(it, parse_level_3, Mod, arithmetic.clone(), "mod"),
            Token::Range => {
                it.eat(&Token::Range, "operation")?;
                let to = it.parse(&parse_expression, "operation", &start)?;
                let step = it.parse_if(&Token::Step, &parse_expression, "step", &start)?;
                let node = Node::Range {
                    from: arithmetic.clone(),
                    to: to.clone(),
                    inclusive: false,
                    step,
                };
                Ok(Box::from(AST::new(&start.union(&to.pos), node)))
            }
            Token::RangeIncl => {
                it.eat(&Token::RangeIncl, "operation")?;
                let to = it.parse(&parse_expression, "operation", &start)?;
                let step = it.parse_if(&Token::Step, &parse_expression, "step", &start)?;
                let node =
                    Node::Range { from: arithmetic.clone(), to: to.clone(), inclusive: true, step };
                Ok(Box::from(AST::new(&start.union(&to.pos), node)))
            }
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_2(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    macro_rules! un_op {
        ($it:expr, $fun:path, $tok:ident, $ast:ident, $msg:expr) => {{
            let factor = $it.parse(&$fun, $msg, &start)?;
            let node = Node::$ast { expr: factor.clone() };
            Ok(Box::from(AST::new(&start.union(&factor.pos), node)))
        }};
    }

    if it.eat_if(&Token::Add).is_some() {
        un_op!(it, parse_level_2, Add, AddU, "plus")
    } else if it.eat_if(&Token::Sub).is_some() {
        un_op!(it, parse_level_2, Sub, SubU, "subtract")
    } else if it.eat_if(&Token::Sqrt).is_some() {
        un_op!(it, parse_expression, Sqrt, Sqrt, "square root")
    } else if it.eat_if(&Token::Not).is_some() {
        un_op!(it, parse_expression, Not, Not, "not")
    } else if it.eat_if(&Token::BOneCmpl).is_some() {
        un_op!(it, parse_expression, BOneCmpl, BOneCmpl, "bitwise ones compliment")
    } else {
        parse_level_1(it)
    }
}

fn parse_level_1(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation")?;
    let arithmetic = it.parse(&parse_inner_expression, "operation", &start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, &start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Pow => bin_op!(it, parse_level_1, Pow, arithmetic.clone(), "exponent"),
            Token::Question => {
                it.eat(&Token::Question, "optional expression")?;
                let right = it.parse(&parse_expression, "optional expression", &lex.pos)?;
                let node = Node::Question { left: arithmetic.clone(), right: right.clone() };
                Ok(Box::from(AST::new(&lex.pos.union(&right.pos), node)))
            }
            _ => Ok(arithmetic.clone())
        },
        Ok(arithmetic.clone()),
    )
}
