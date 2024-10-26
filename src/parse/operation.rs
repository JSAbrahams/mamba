use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::expression::parse_inner_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::ParseResult;

macro_rules! inner_bin_op {
    ($it:expr, $start:expr, $fun:path, $ast:ident, $left:expr, $msg:expr) => {{
        $it.eat(&Token::$ast, "operation")?;
        let right = $it.parse(&$fun, $msg, $start)?;
        let node = Node::$ast {
            left: $left,
            right: right.clone(),
        };
        Ok(Box::from(AST::new($start.union(right.pos), node)))
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
///    in not, is a, is not a
/// 7. and, or, question or
/// 8. postfix calls
pub fn parse_expression(it: &mut LexIterator) -> ParseResult {
    parse_level_7(it)
}

fn parse_level_7(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (7)")?;
    let arithmetic = it.parse(&parse_level_6, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::And => bin_op!(it, parse_level_7, And, arithmetic.clone(), "and"),
            Token::Or => bin_op!(it, parse_level_7, Or, arithmetic.clone(), "or"),
            Token::Question => bin_op!(it, parse_level_7, Question, arithmetic.clone(), "question"),
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_6(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (6)")?;
    let arithmetic = it.parse(&parse_level_5, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
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
            Token::IsA => bin_op!(it, parse_level_6, IsA, arithmetic.clone(), "is a"),
            Token::In => bin_op!(it, parse_level_6, In, arithmetic.clone(), "in"),
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_5(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (5)")?;
    let arithmetic = it.parse(&parse_level_4, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::BLShift => bin_op!(
                it,
                parse_level_5,
                BLShift,
                arithmetic.clone(),
                "bitwise left shift"
            ),
            Token::BRShift => bin_op!(
                it,
                parse_level_5,
                BRShift,
                arithmetic.clone(),
                "bitwise right shift"
            ),
            Token::BAnd => bin_op!(it, parse_level_5, BAnd, arithmetic.clone(), "bitwise and"),
            Token::BOr => bin_op!(it, parse_level_5, BOr, arithmetic.clone(), "bitwise or"),
            Token::BXOr => bin_op!(it, parse_level_5, BXOr, arithmetic.clone(), "bitwise xor"),
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_4(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (4)")?;
    let arithmetic = it.parse(&parse_level_3, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Add => bin_op!(it, parse_level_4, Add, arithmetic.clone(), "add"),
            Token::Sub => bin_op!(it, parse_level_4, Sub, arithmetic.clone(), "sub"),
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_3(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (3)")?;
    let arithmetic = it.parse(&parse_level_2, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    macro_rules! match_range_slice {
        ($it:expr, $token:ident, $incl:expr, $node:ident, $msg:expr) => {{
            $it.eat(&Token::$token, $msg)?;
            let to = $it.parse(&parse_expression, $msg, start)?;
            let (to, step, end) = match to.node {
                Node::$node { from, to, .. } => (from.clone(), Some(to.clone()), to.pos),
                _ => {
                    let step = $it.parse_if(&Token::$node, &parse_expression, $msg, start)?;
                    (to.clone(), step.clone(), step.map_or(to.pos, |ast| ast.pos))
                }
            };

            let node = Node::$node {
                from: arithmetic.clone(),
                to,
                inclusive: $incl,
                step,
            };
            Ok(Box::from(AST::new(start.union(end), node)))
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Mul => bin_op!(it, parse_level_3, Mul, arithmetic.clone(), "mul"),
            Token::Div => bin_op!(it, parse_level_3, Div, arithmetic.clone(), "div"),
            Token::FDiv => bin_op!(it, parse_level_3, FDiv, arithmetic.clone(), "floor div"),
            Token::Mod => bin_op!(it, parse_level_3, Mod, arithmetic.clone(), "mod"),
            Token::Range => match_range_slice!(it, Range, false, Range, "range"),
            Token::RangeIncl => match_range_slice!(it, RangeIncl, true, Range, "range"),
            Token::Slice => match_range_slice!(it, Slice, false, Slice, "range"),
            Token::SliceIncl => match_range_slice!(it, SliceIncl, true, Slice, "range"),
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

fn parse_level_2(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (2)")?;
    macro_rules! un_op {
        ($it:expr, $fun:path, $tok:ident, $ast:ident, $msg:expr) => {{
            let factor = $it.parse(&$fun, $msg, start)?;
            let node = Node::$ast {
                expr: factor.clone(),
            };
            Ok(Box::from(AST::new(start.union(factor.pos), node)))
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
        un_op!(
            it,
            parse_expression,
            BOneCmpl,
            BOneCmpl,
            "bitwise ones compliment"
        )
    } else {
        parse_level_1(it)
    }
}

fn parse_level_1(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (1)")?;
    let arithmetic = it.parse(&parse_inner_expression, "operation", start)?;
    macro_rules! bin_op {
        ($it:expr, $fun:path, $ast:ident, $arithmetic:expr, $msg:expr) => {{
            inner_bin_op!($it, start, $fun, $ast, $arithmetic, $msg)
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Pow => bin_op!(it, parse_level_1, Pow, arithmetic.clone(), "exponent"),
            Token::Question => {
                it.eat(&Token::Question, "optional expression")?;
                let right = it.parse(&parse_expression, "optional expression", lex.pos)?;
                let node = Node::Question {
                    left: arithmetic.clone(),
                    right: right.clone(),
                };
                Ok(Box::from(AST::new(lex.pos.union(right.pos), node)))
            }
            _ => Ok(arithmetic.clone()),
        },
        Ok(arithmetic.clone()),
    )
}

#[cfg(test)]
mod test {
    use std::convert::From;

    use crate::parse::ast::{Node, AST};
    use crate::parse::lex::token::Token::*;
    use crate::parse::parse_direct;

    macro_rules! verify_is_operation {
        ($op:ident, $ast:expr) => {{
            match &$ast.first().expect("script empty.").node {
                Node::$op { left, right } => (left.clone(), right.clone()),
                other => panic!(
                    "first element script was not op: {}, but was: {:?}",
                    $op, other
                ),
            }
        }};
    }

    macro_rules! verify_is_un_operation {
        ($op:ident, $ast:expr) => {{
            match &$ast.first().expect("script empty.").node {
                Node::$op { expr } => expr.clone(),
                _ => panic!("first element script was not tuple."),
            }
        }};
    }

    #[test]
    fn addition_verify() {
        let source = String::from("a + b");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Add, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("b")
            }
        );
    }

    #[test]
    fn addition_unary_verify() {
        let source = String::from("+ b");
        let ast = parse_direct(&source).unwrap();

        let expr = verify_is_un_operation!(AddU, ast);
        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("b")
            }
        );
    }

    #[test]
    fn subtraction_verify() {
        let source = String::from("a - False");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Sub, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("a")
            }
        );
        assert_eq!(right.node, Node::Bool { lit: false });
    }

    #[test]
    fn subtraction_unary_verify() {
        let source = String::from("- c");
        let ast = parse_direct(&source).unwrap();

        let expr = verify_is_un_operation!(SubU, ast);
        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("c")
            }
        );
    }

    #[test]
    fn multiplication_verify() {
        let source = String::from("True * b");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Mul, ast);
        assert_eq!(left.node, Node::Bool { lit: true });
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("b")
            }
        );
    }

    #[test]
    fn division_verify() {
        let source = String::from("10.0 / fgh");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Div, ast);
        assert_eq!(
            left.node,
            Node::Real {
                lit: String::from("10.0")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("fgh")
            }
        );
    }

    #[test]
    fn floor_division_verify() {
        let source = String::from("10.0 // fgh");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(FDiv, ast);
        assert_eq!(
            left.node,
            Node::Real {
                lit: String::from("10.0")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("fgh")
            }
        );
    }

    #[test]
    fn power_verify() {
        let source = String::from("chopin ^ liszt");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Pow, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("chopin")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("liszt")
            }
        );
    }

    #[test]
    fn mod_verify() {
        let source = String::from("chopin mod 3E10");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Mod, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("chopin")
            }
        );
        assert_eq!(
            right.node,
            Node::ENum {
                num: String::from("3"),
                exp: String::from("10")
            }
        );
    }

    #[test]
    fn is_verify() {
        let source = String::from("p is q");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Is, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("p")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("q")
            }
        );
    }

    #[test]
    fn isa_verify() {
        let source = String::from("lizard isa animal");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(IsA, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("lizard")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("animal")
            }
        );
    }

    #[test]
    fn equality_verify() {
        let source = String::from("i = s");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Eq, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("i")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("s")
            }
        );
    }

    #[test]
    fn le_verify() {
        let source = String::from("one < two");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Le, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("two")
            }
        );
    }

    #[test]
    fn leq_verify() {
        let source = String::from("two_hundred <= three");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Leq, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("two_hundred")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("three")
            }
        );
    }

    #[test]
    fn ge_verify() {
        let source = String::from("r > 10");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Ge, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("r")
            }
        );
        assert_eq!(
            right.node,
            Node::Int {
                lit: String::from("10")
            }
        );
    }

    #[test]
    fn geq_verify() {
        let source = String::from("4 >= 10");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Geq, ast);
        assert_eq!(
            left.node,
            Node::Int {
                lit: String::from("4")
            }
        );
        assert_eq!(
            right.node,
            Node::Int {
                lit: String::from("10")
            }
        );
    }

    #[test]
    fn in_verify() {
        let source = String::from("one in my_set");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(In, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("my_set")
            }
        );
    }

    #[test]
    fn and_verify() {
        let source = String::from("one and three");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(And, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("three")
            }
        );
    }

    #[test]
    fn or_verify() {
        let source = String::from("one or \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(Or, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn not_verify() {
        let source = String::from("not some_cond");
        let ast = parse_direct(&source).unwrap();

        let expr = verify_is_un_operation!(Not, ast);
        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("some_cond")
            }
        );
    }

    #[test]
    fn sqrt_verify() {
        let source = String::from("sqrt some_num");
        let ast = parse_direct(&source).unwrap();

        let expr = verify_is_un_operation!(Sqrt, ast);
        assert_eq!(
            expr.node,
            Node::Id {
                lit: String::from("some_num")
            }
        );
    }

    #[test]
    fn b_and_verify() {
        let source = String::from("one _and_ three");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(BAnd, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Id {
                lit: String::from("three")
            }
        );
    }

    #[test]
    fn b_or_verify() {
        let source = String::from("one _or_ \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(BOr, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn b_xor_verify() {
        let source = String::from("one _xor_ \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(BXOr, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn b_ones_complement_verify() {
        let source = String::from("_not_ \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let expr = verify_is_un_operation!(BOneCmpl, ast);
        assert_eq!(
            expr.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn b_lshift_verify() {
        let source = String::from("one << \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(BLShift, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn brshift_verify() {
        let source = String::from("one >> \"asdf\"");
        let ast = parse_direct(&source).unwrap();

        let (left, right) = verify_is_operation!(BRShift, ast);
        assert_eq!(
            left.node,
            Node::Id {
                lit: String::from("one")
            }
        );
        assert_eq!(
            right.node,
            Node::Str {
                lit: String::from("asdf"),
                expressions: vec![]
            }
        );
    }

    #[test]
    fn addition_missing_factor() {
        let source = String::from("a +");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn subtraction_missing_factor() {
        let source = String::from("b -");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn multiplication_missing_factor() {
        let source = String::from("b *");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn division_missing_factor() {
        let source = String::from("b /");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn power_missing_factor() {
        let source = String::from("a ^");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn mod_missing_factor() {
        let source = String::from("y mod");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn is_missing_value_left() {
        let source = String::from("is a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn is_missing_value_right() {
        let source = String::from("kotlin is");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isnt_missing_value_left() {
        let source = String::from("isnt a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isnt_missing_value_right() {
        let source = String::from("kotlin isnt");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isa_missing_value_left() {
        let source = String::from("isa a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isa_missing_value_right() {
        let source = String::from("kotlin isa");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isnta_missing_value_left() {
        let source = String::from("isnta a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn isnta_missing_value_right() {
        let source = String::from("kotlin isnta");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn equality_missing_value_left() {
        let source = String::from("= a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn equality_missing_value_right() {
        let source = String::from("kotlin =");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn le_missing_value_left() {
        let source = String::from("< a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn le_missing_value_right() {
        let source = String::from("kotlin <");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn leq_missing_value_left() {
        let source = String::from("<= a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn leq_missing_value_right() {
        let source = String::from("kotlin <=");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn ge_missing_value_left() {
        let source = String::from("> a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn ge_missing_value_right() {
        let source = String::from("kotlin >");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn geq_missing_value_left() {
        let source = String::from(">= a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn geq_missing_value_right() {
        let source = String::from("kotlin >=");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn and_missing_value_left() {
        let source = String::from("and a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn and_missing_value_right() {
        let source = String::from("kotlin and");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn or_missing_value_left() {
        let source = String::from("or a");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn or_missing_value_right() {
        let source = String::from("kotlin or");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn not_missing_value() {
        let source = String::from("not");
        source.parse::<AST>().unwrap_err();
    }

    #[test]
    fn sqrt_missing_value() {
        let source = String::from("sqrt");
        source.parse::<AST>().unwrap_err();
    }
}
