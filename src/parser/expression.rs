use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ASTNode;
use crate::parser::ASTNodePos;
use crate::parser::collection::parse_collection;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::definition::parse_reassignment;
use crate::parser::end_pos;
use crate::parser::function::parse_function_anonymous;
use crate::parser::function::parse_function_call;
use crate::parser::function::parse_function_call_direct;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::start_pos;
use crate::parser::TPIterator;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let mut tuple = false;

    return match match it.peek() {
        Some(TokenPos { token: Token::If, .. }) | Some(TokenPos { token: Token::When, .. }) =>
            parse_cntrl_flow_expr(it),

        Some(TokenPos { line: _, pos: _, token: Token::LRBrack }) => {
            tuple = true;
            parse_collection(it)
        }
        Some(TokenPos { token: Token::LSBrack, .. }) |
        Some(TokenPos { token: Token::LCBrack, .. }) => parse_collection(it),

        Some(TokenPos { token: Token::_Self, .. }) => {
            it.next();
            let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "self");
            Ok(ASTNodePos {
                st_line,
                st_pos,
                en_line: expr.en_line,
                en_pos: expr.en_pos,
                node: ASTNode::_Self { expr },
            })
        }
        Some(TokenPos { token: Token::Ret, .. }) => parse_return(it),

        Some(TokenPos { token: Token::Real(_), .. }) |
        Some(TokenPos { token: Token::Int(_), .. }) |
        Some(TokenPos { token: Token::ENum(_, _), .. }) |
        Some(TokenPos { token: Token::Id(_), .. }) |
        Some(TokenPos { token: Token::Str(_), .. }) |
        Some(TokenPos { token: Token::Bool(_), .. }) |
        Some(TokenPos { token: Token::Not, .. }) |
        Some(TokenPos { token: Token::Add, .. }) |
        Some(TokenPos { token: Token::Ver, .. }) |
        Some(TokenPos { token: Token::Sub, .. }) => parse_operation(it),

        Some(&next) => Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    } {
        Ok(pre) => match it.peek() {
            Some(TokenPos { token: Token::To, .. }) if tuple => {
                it.next();
                let right: Box<ASTNodePos> = get_or_err!(it, parse_function_anonymous,
                                                         "anonymous function");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: right.en_line,
                    en_pos: right.en_pos,
                    node: ASTNode::ReAssign { left: Box::new(pre), right },
                })
            }
            Some(TokenPos { token: Token::QuestOr, .. }) => {
                it.next();
                let _default: Box<ASTNodePos> = get_or_err!(it, parse_expression, "?or");
                Ok(ASTNodePos {
                    st_line,
                    st_pos,
                    en_line: expr.en_line,
                    en_pos: expr.en_pos,
                    node: ASTNode::QuestOr { _do: Box::new(pre), _default },
                })
            }
            Some(TokenPos { token: Token::LRBrack, .. }) => parse_function_call_direct(pre, it),
            Some(TokenPos { token: Token::Point, .. }) => parse_function_call(pre, it),
            Some(TokenPos { token: Token::Assign, .. }) => parse_reassignment(pre, it),
            Some(_) | None => Ok(pre)
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
