use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::call::parse_anon_fun;
use crate::parser::call::parse_call;
use crate::parser::call::parse_reassignment;
use crate::parser::collection::parse_collection;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::common::end_pos;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;
use crate::parser::common::start_pos;
use crate::parser::TPIterator;

pub fn parse_expression(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    let result = match it.peek() {
        Some(TokenPos { token: Token::If, .. }) | Some(TokenPos { token: Token::Match, .. }) =>
            parse_cntrl_flow_expr(it),

        Some(TokenPos { token: Token::LRBrack, .. }) => parse_collection(it),
        Some(TokenPos { token: Token::LSBrack, .. })
        | Some(TokenPos { token: Token::LCBrack, .. }) => parse_collection(it),

        Some(TokenPos { token: Token::Ret, .. }) => parse_return(it),

        Some(TokenPos { token: Token::Underscore, .. }) => {
            let (en_line, en_pos) = end_pos(it);
            it.next();
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::Underscore })
        }

        Some(TokenPos { token: Token::_Self, .. })
        | Some(TokenPos { token: Token::Real(_), .. })
        | Some(TokenPos { token: Token::Int(_), .. })
        | Some(TokenPos { token: Token::ENum(..), .. })
        | Some(TokenPos { token: Token::Str(_), .. })
        | Some(TokenPos { token: Token::Bool(_), .. })
        | Some(TokenPos { token: Token::Not, .. })
        | Some(TokenPos { token: Token::Sqrt, .. })
        | Some(TokenPos { token: Token::Add, .. })
        | Some(TokenPos { token: Token::Id(_), .. })
        | Some(TokenPos { token: Token::Sub, .. }) => parse_operation(it),

        Some(TokenPos { token: Token::BSlash, .. }) => parse_anon_fun(it),

        Some(&next) =>
            Err(CustomErr { expected: "expression".to_string(), actual: next.clone() }),
        None => Err(CustomEOFErr { expected: "expression".to_string() })
    };

    match result {
        Ok(res) => parse_post_expr(res, it),
        err => err
    }
}

fn parse_post_expr(pre: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (pre.st_line, pre.st_pos);
    let result = match it.peek() {
        Some(TokenPos { token: Token::QuestOr, .. }) => {
            it.next();
            let _def: Box<ASTNodePos> = get_or_err!(it, parse_expression, "?or");

            let (en_line, en_pos) = (_def.en_line, _def.en_pos);
            let node = ASTNode::QuestOr { _do: Box::new(pre), _default: _def };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(TokenPos { token: Token::Range, .. }) => {
            it.next();
            let to: Box<ASTNodePos> = get_or_err!(it, parse_expression, "..");

            let (en_line, en_pos) = (to.en_line, to.en_pos);
            let node = ASTNode::Range { from: Box::from(pre), to };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }
        Some(TokenPos { token: Token::RangeIncl, .. }) => {
            it.next();
            let to: Box<ASTNodePos> = get_or_err!(it, parse_expression, "..=");

            let (en_line, en_pos) = (to.en_line, to.en_pos);
            let node = ASTNode::RangeIncl { from: Box::from(pre), to };
            Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
        }

        Some(TokenPos { token: Token::Assign, .. }) => parse_reassignment(pre, it),

        // normal method or function call
        Some(TokenPos { token: Token::LRBrack, .. })
        | Some(TokenPos { token: Token::Point, .. }) => parse_call(pre, it),
        Some(&tp) if is_start_expression_exclude_unary(tp) => parse_call(pre, it),

        _ => return Ok(pre)
    };

    match result {
        Ok(res) => parse_post_expr(res, it),
        err => err
    }
}

fn parse_return(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = start_pos(it);
    check_next_is!(it, Token::Ret);

    if let Some(&&TokenPos { token: Token::NL, .. }) = it.peek() {
        let (en_line, en_pos) = end_pos(it);
        return Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::ReturnEmpty });
    }

    let expr: Box<ASTNodePos> = get_or_err!(it, parse_expression, "return");

    let (en_line, en_pos) = (expr.en_line, expr.en_pos);
    let node = ASTNode::Return { expr };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

/// Excluding unary addition and subtraction
pub fn is_start_expression_exclude_unary(next: &TokenPos) -> bool {
    match next {
        TokenPos { token: Token::If, .. }
        | TokenPos { token: Token::Match, .. }
        | TokenPos { token: Token::LRBrack, .. }
        | TokenPos { token: Token::LSBrack, .. }
        | TokenPos { token: Token::LCBrack, .. }
        | TokenPos { token: Token::Underscore, .. }
        | TokenPos { token: Token::_Self, .. }
        | TokenPos { token: Token::Real(_), .. }
        | TokenPos { token: Token::Int(_), .. }
        | TokenPos { token: Token::ENum(..), .. }
        | TokenPos { token: Token::Str(_), .. }
        | TokenPos { token: Token::Bool(_), .. }
        | TokenPos { token: Token::Not, .. }
        | TokenPos { token: Token::Id(_), .. } => true,
        _ => false
    }
}

pub fn is_start_expression(next: &TokenPos) -> bool {
    let start_expr = is_start_expression_exclude_unary(next);
    start_expr || next.token == Token::Add || next.token == Token::Sub
}
