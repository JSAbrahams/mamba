use crate::lexer::Token;
use crate::lexer::TokenPos;
use crate::parser::ASTNode;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::function::parse_function_anonymous;
use crate::parser::maybe_expr::parse_expression;
use crate::parser::parse_result::ParseError;
use crate::parser::parse_result::ParseResult;
use crate::parser::util;
use std::iter::Iterator;
use std::iter::Peekable;
use std::slice::Iter;

// control-flow-expr::= if | from | when
// if               ::= ( "if" | "unless" ) maybe-expr "then" expr-or-stmt [ "else" expr-or-stmt ]
// from             ::= "from" maybe-expr [ newline ] "where" maybe-expression
//                      [ "map" function-anon ]
// when             ::= "when" maybe-expr newline { { indent } when-case }
// when-case        ::= maybe-expr "then" expr-or-stmt

pub fn parse_cntrl_flow_expr(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                             -> (ParseResult<ASTNode>, i32) {
    return match it.peek() {
        Some(TokenPos { line, pos, token: Token::If }) |
        Some(TokenPos { line, pos, token: Token::Unless }) => parse_if(it, ind),
        Some(TokenPos { line, pos, token: Token::From }) => parse_from(it, ind),
        Some(TokenPos { line, pos, token: Token::When }) => parse_when(it, ind),

        Some(tp) => (Err(ParseError::TokenError(**tp, Token::If)), ind),
        None => (Err(ParseError::EOFError(Token::If)), ind)
    };
}

fn parse_if(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    let if_expr = match it.next() {
        Some(TokenPos { line, pos, token: TokenPos::If }) => true,
        Some(TokenPos { line, pos, token: TokenPos::Unless }) => false,
        Some(tp @ TokenPos { ref line, ref pos, token })

        if (*token != Token::If || *token != Token::Unless) =>
            return (Err(ParseError::TokenError(*tp, Token::If)), ind),
        None => return (Err(ParseError::EOFError(Token::If)), ind)
    };

    return match parse_expression(it, ind) {
        (Ok(cond), ind) => {
            match it.next() {
                Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::Then =>
                    return (Err(ParseError::TokenError(*tp, Token::Then)), ind),
                None => return (Err(ParseError::EOFError(Token::Then)), ind)
            }

            match parse_expr_or_stmt(it, ind) {
                (Ok(then), ind) => if Some(&&TokenPos::Else) != it.peek() {
                    if if_expr {
                        (Ok(ASTNode::If(wrap!(cond), wrap!(then))), ind)
                    } else {
                        (Ok(ASTNode::Unless(wrap!(cond), wrap!(then))), ind)
                    }
                } else {
                    it.next();
                    match parse_expr_or_stmt(it, ind) {
                        (Ok(otherwise), ind) => if if_expr {
                            (Ok(ASTNode::IfElse(wrap!(cond), wrap!(then), wrap!(otherwise))), ind)
                        } else {
                            (Ok(ASTNode::UnlessElse(wrap!(cond), wrap!(then), wrap!(otherwise))),
                             ind)
                        }
                        err => err
                    }
                }
                err => err
            }
        }
        err => err
    };
}

fn parse_from(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::From =>
            return (Err(ParseError::TokenError(*tp, Token::From)), ind),
        None => return (Err(ParseError::EOFError(Token::From)), ind)
    }

    return match parse_expression(it, ind) {
        (Ok(coll), ind) => {
            match it.next() {
                Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::When =>
                    return (Err(ParseError::TokenError(*tp, Token::When)), ind),
                None => return (Err(ParseError::EOFError(Token::When)), ind)
            }

            match parse_expression(it, ind) {
                (Ok(cond), ind) => if it.peek() == Some(&&TokenPos::Map) {
                    match (it.next(), parse_function_anonymous(it, ind)) {
                        (_, (Ok(mapping), ind)) => (Ok(
                            ASTNode::FromMap(wrap!(coll), wrap!(cond), wrap!(mapping))), ind),
                        (_, err) => err
                    }
                } else { (Ok(ASTNode::From(wrap!(coll), wrap!(cond))), ind) }
                err => err
            }
        }
        err => err
    };
}

fn parse_when(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match it.next() {
        Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::When =>
            return (Err(ParseError::TokenError(*tp, Token::When)), ind),
        None => return (Err(ParseError::EOFError(Token::When)), ind)
    }

    match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            match it.next() {
                Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::NL =>
                    return (Err(ParseError::TokenError(*tp, Token::NL)), ind),
                None => return (Err(ParseError::EOFError(Token::NL)), ind)
            }

            match parse_when_cases(it, ind + 1) {
                (Ok(cases), ind) => (Ok(ASTNode::When(wrap!(expr), cases)), ind),
                (Err(err), ind) => (Err(err), ind)
            }
        }
        err => err
    }
}

fn parse_when_cases(it: &mut Peekable<Iter<TokenPos>>, ind: i32)
                    -> (Result<Vec<ASTNode>, String>, i32) {
    let mut when_cases = Vec::new();

    while let Some(_) = it.peek() {
        let next_ind = util::ind_count(it);
        if next_ind < ind { break; }; /* Indentation decrease marks end of do block */
        if next_ind > ind && it.peek().is_some() {
            return (Err(format!("Expected indentation of {}.", ind)), next_ind);
        }

        match parse_when_case(it, ind) {
            (Err(err), ind) => return (Err(err), ind),
            (Ok(case), _) => when_cases.push(case),
        }

        if Some(&&TokenPos::NL) == it.peek() {
            it.next();
            if Some(&&TokenPos::NL) == it.peek() {
                it.next();
                break;
            }
        } else { break; }
    }

    return (Ok(when_cases), ind);
}

fn parse_when_case(it: &mut Peekable<Iter<TokenPos>>, ind: i32) -> (ParseResult<ASTNode>, i32) {
    match parse_expression(it, ind) {
        (Ok(expr), ind) => {
            match it.next() {
                Some(tp @ TokenPos { ref line, ref pos, token }) if *token != Token::Then =>
                    return (Err(ParseError::TokenError(*tp, Token::Then)), ind),
                None => return (Err(ParseError::EOFError(Token::Then)), ind)
            }

            match parse_expr_or_stmt(it, ind) {
                (Ok(expr_or_do), ind) => (Ok(ASTNode::If(wrap!(expr), wrap!(expr_or_do))), ind),
                err => err
            }
        }
        err => err
    }
}
