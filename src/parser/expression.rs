use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::call::parse_anon_fun;
use crate::parser::call::parse_function_call;
use crate::parser::call::parse_reassignment;
use crate::parser::collection::parse_collection;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::end_pos;
use crate::parser::expr_or_stmt::parse_handle;
use crate::parser::expr_or_stmt::parse_raise;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);

    return match match it.peek() {
        Some(TokenPos { token: Token::If, .. }) |
        Some(TokenPos { token: Token::When, .. }) => parse_cntrl_flow_expr(it),

        Some(TokenPos { line: _, pos: _, token: Token::LRBrack }) => parse_collection(it),
        Some(TokenPos { token: Token::LSBrack, .. }) |
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_collection(it),

        Some(TokenPos { token: Token::Ret, .. }) => parse_return(it),

        Some(TokenPos { token: Token::Underscore, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::UnderScore })
        }

        Some(TokenPos { token: Token::_Self, .. }) |
        Some(TokenPos { token: Token::Real(_), .. }) |
        Some(TokenPos { token: Token::Int(_), .. }) |
        Some(TokenPos { token: Token::ENum(_, _), .. }) |
        Some(TokenPos { token: Token::Id(_), .. }) |
        Some(TokenPos { token: Token::Str(_), .. }) |
        Some(TokenPos { token: Token::Bool(_), .. }) |
        Some(TokenPos { token: Token::Not, .. }) |
        Some(TokenPos { token: Token::Add, .. }) |
        Some(TokenPos { token: Token::Sub, .. }) => parse_operation(it),

        Some(&next) => Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    } {
        Ok(pre) => match it.peek() {
            Some(TokenPos { token: Token::QuestOr, .. }) => {
                it.next();
                let _default: Box<ASTNodePos> = get_or_err!(it, parse_expression, "?or");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: _default.en_line,
                    en_pos: _default.en_pos,
                    node: ASTNode::QuestOr { _do: Box::new(pre), _default },
                })
            }
            Some(TokenPos { token: Token::Range, .. }) => {
                it.next();
                let to: Box<ASTNodePos> = get_or_err!(it, parse_expression, "?or");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: to.en_line,
                    en_pos: to.en_pos,
                    node: ASTNode::Range { from: Box::from(pre), to },
                })
            }
            Some(TokenPos { token: Token::RangeIncl, .. }) => {
                it.next();
                let to: Box<ASTNodePos> = get_or_err!(it, parse_expression, "?or");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: to.en_line,
                    en_pos: to.en_pos,
                    node: ASTNode::RangeIncl { from: Box::from(pre), to },
                })
            }
            Some(TokenPos { token: Token::To, .. }) => parse_anon_fun(pre, it),
            Some(TokenPos { token: Token::Raises, .. }) => parse_raise(pre, it),
            Some(TokenPos { token: Token::Handle, .. }) => parse_handle(pre, it),

            Some(TokenPos { token: Token::Assign, .. }) => parse_reassignment(pre, it),

            // normal method or function call
            Some(TokenPos { token: Token::LRBrack, .. }) |
            Some(TokenPos { token: Token::DDoublePoint, .. }) |
            Some(TokenPos { token: Token::Point, .. }) |

            // postfix method or function call
            Some(TokenPos { token: Token::If, .. }) | Some(TokenPos { token: Token::When, .. }) |
            Some(TokenPos { token: Token::LSBrack, .. }) |
            Some(TokenPos { token: Token::LCBrack, .. }) |
            Some(TokenPos { token: Token::_Self, .. }) |
            Some(TokenPos { token: Token::Real(_), .. }) |
            Some(TokenPos { token: Token::Int(_), .. }) |
            Some(TokenPos { token: Token::ENum(_, _), .. }) |
            Some(TokenPos { token: Token::Id(_), .. }) |
            Some(TokenPos { token: Token::Str(_), .. }) |
            Some(TokenPos { token: Token::Bool(_), .. }) |
            Some(TokenPos { token: Token::Not, .. }) |
            Some(TokenPos { token: Token::Add, .. }) | Some(TokenPos { token: Token::Sub, .. }) |
            Some(TokenPos { token: Token::Not, .. }) => parse_function_call(pre, it),

            _ => Ok(pre)
        }

        err => err
    };
}

fn parse_return(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Ret);

    if let Some(&&TokenPos { token: Token::NL, .. }) = it.peek() {
        let (en_line, en_pos) = end_pos(it);
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::ReturnEmpty });
    }

    let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "return");
    return Ok(ASTNodePos {
        st_line,
        st_pos,
        en_line: expr.en_line,
        en_pos: expr.en_pos,
        node: ASTNode::Return { expr },
    });
}
