use crate::parse::ast::{Node, AST};
use crate::parse::expression::parse_inner_expression;
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::result::ParseResult;

/// The left operand is already parsed; eat the operator and parse the right at `fun`.
macro_rules! bin_op {
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
/// Precedence is as follows, from tightest binding to loosest, each level parsed by the
/// `parse_level_` function it is numbered after:
/// 1. exponent, and the null-coalescing `?`
/// 2. unary plus, unary minus, not
/// 3. multiplication, division, floor division, modulus, range, range inclusive
/// 4. addition, subtraction
/// 5. greater, greater or equal, less, less or equal, equal, not equal, in
/// 6. and, or
///
/// A call, a property access and an index bind tighter than any of them.
/// Newlines in front of expressions are ignored.
pub fn parse_expression(it: &mut LexIterator) -> ParseResult {
    it.eat_while(&Token::NL);

    parse_level_6(it)
}

fn parse_level_6(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (6)")?;
    let left = it.parse(&parse_level_5, "operation", start)?;
    it.peek(
        &|it, lex| match lex.token {
            Token::And => bin_op!(it, start, parse_level_6, And, left.clone(), "and"),
            Token::Or => bin_op!(it, start, parse_level_6, Or, left.clone(), "or"),
            Token::Question => {
                bin_op!(it, start, parse_level_6, Question, left.clone(), "question")
            }
            _ => Ok(left.clone()),
        },
        Ok(left.clone()),
    )
}

fn parse_level_5(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (5)")?;
    let left = it.parse(&parse_level_4, "operation", start)?;
    it.peek(
        &|it, lex| match lex.token {
            Token::Ge => bin_op!(it, start, parse_level_5, Ge, left.clone(), "greater"),
            Token::Geq => bin_op!(
                it,
                start,
                parse_level_5,
                Geq,
                left.clone(),
                "greater, equal"
            ),
            Token::Le => bin_op!(it, start, parse_level_5, Le, left.clone(), "less"),
            Token::Leq => bin_op!(it, start, parse_level_5, Leq, left.clone(), "less, equal"),
            Token::Eq => bin_op!(it, start, parse_level_5, Eq, left.clone(), "equal"),
            Token::Neq => bin_op!(it, start, parse_level_5, Neq, left.clone(), "not equal"),
            Token::In => bin_op!(it, start, parse_level_5, In, left.clone(), "in"),
            _ => Ok(left.clone()),
        },
        Ok(left.clone()),
    )
}

fn parse_level_4(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (4)")?;
    let left = it.parse(&parse_level_3, "operation", start)?;
    it.peek(
        &|it, lex| match lex.token {
            Token::Add => bin_op!(it, start, parse_level_4, Add, left.clone(), "add"),
            Token::Sub => bin_op!(it, start, parse_level_4, Sub, left.clone(), "sub"),
            _ => Ok(left.clone()),
        },
        Ok(left.clone()),
    )
}

fn parse_level_3(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (3)")?;
    let left = it.parse(&parse_level_2, "operation", start)?;
    /// A second `..` after the bound is the step, as in `0 .. 10 .. 2`.
    macro_rules! range {
        ($it:expr, $token:ident, $incl:expr) => {{
            $it.eat(&Token::$token, "range")?;
            let to = $it.parse(&parse_expression, "range", start)?;
            let (to, step, end) = match to.node {
                Node::Range { from, to, .. } => (from.clone(), Some(to.clone()), to.pos),
                _ => {
                    let step = $it.parse_if(&Token::Range, &parse_expression, "range", start)?;
                    (to.clone(), step.clone(), step.map_or(to.pos, |ast| ast.pos))
                }
            };

            let node = Node::Range {
                from: left.clone(),
                to,
                inclusive: $incl,
                step,
            };
            Ok(Box::from(AST::new(start.union(end), node)))
        }};
    }

    it.peek(
        &|it, lex| match lex.token {
            Token::Mul => bin_op!(it, start, parse_level_3, Mul, left.clone(), "mul"),
            Token::Div => bin_op!(it, start, parse_level_3, Div, left.clone(), "div"),
            Token::FDiv => bin_op!(it, start, parse_level_3, FDiv, left.clone(), "floor div"),
            Token::Mod => bin_op!(it, start, parse_level_3, Mod, left.clone(), "mod"),
            Token::Range => range!(it, Range, false),
            Token::RangeIncl => range!(it, RangeIncl, true),
            _ => Ok(left.clone()),
        },
        Ok(left.clone()),
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
    } else if it.eat_if(&Token::Not).is_some() {
        un_op!(it, parse_expression, Not, Not, "not")
    } else {
        parse_level_1(it)
    }
}

fn parse_level_1(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("operation (1)")?;
    let left = it.parse(&parse_inner_expression, "operation", start)?;
    it.peek(
        &|it, lex| match lex.token {
            Token::Pow => bin_op!(it, start, parse_level_1, Pow, left.clone(), "exponent"),
            Token::Question => {
                it.eat(&Token::Question, "optional expression")?;
                let right = it.parse(&parse_expression, "optional expression", lex.pos)?;
                let node = Node::Question {
                    left: left.clone(),
                    right: right.clone(),
                };
                Ok(Box::from(AST::new(lex.pos.union(right.pos), node)))
            }
            _ => Ok(left.clone()),
        },
        Ok(left.clone()),
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
        assert_eq!(
            right.node,
            Node::Id {
                lit: "False".to_string()
            }
        );
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
        assert_eq!(
            left.node,
            Node::Id {
                lit: "True".to_string()
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

    /// `sqrt` is an ordinary identifier, so it stands alone and it can be called.
    ///
    /// As an operator it was neither: bare `sqrt` was a parse error, and `x.sqrt()` could not
    /// parse at all, because the name never reached the parser as an identifier.
    #[test]
    fn sqrt_is_an_identifier() {
        let ast = parse_direct(&String::from("sqrt")).unwrap();
        assert_eq!(
            ast.first().expect("script empty.").node,
            Node::Id {
                lit: String::from("sqrt")
            }
        );

        let ast = parse_direct(&String::from("x.sqrt()")).unwrap();
        let Node::PropertyCall { property, .. } = &ast.first().expect("script empty.").node else {
            panic!("was {:?}", ast.first().map(|a| a.node.clone()))
        };
        let Node::FunctionCall { name, .. } = &property.node else {
            panic!("was {:?}", property.node)
        };
        assert_eq!(
            name.node,
            Node::Id {
                lit: String::from("sqrt")
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
}
