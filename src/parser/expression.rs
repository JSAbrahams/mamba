use crate::lexer::token::Lex;
use crate::lexer::token::Token;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::call::parse_anon_fun;
use crate::parser::call::parse_call;
use crate::parser::call::parse_reassignment;
use crate::parser::collection::parse_collection;
use crate::parser::control_flow_expr::parse_cntrl_flow_expr;
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_operation;
use crate::parser::parse_result::expected_one_of;
use crate::parser::parse_result::ParseResult;

pub fn parse_expression(it: &mut LexIterator) -> ParseResult {
    let result = it.peek_or_err(
        &|it, lex| match lex.token {
            Token::If | Token::Match => parse_cntrl_flow_expr(it),
            Token::LRBrack | Token::LSBrack | Token::LCBrack => parse_collection(it),
            Token::Ret => parse_return(it),
            Token::Underscore => parse_underscore(it),

            Token::_Self
            | Token::Real(_)
            | Token::Int(_)
            | Token::ENum(..)
            | Token::Str(_)
            | Token::Bool(_)
            | Token::Not
            | Token::Sqrt
            | Token::Add
            | Token::Id(_)
            | Token::Sub
            | Token::Undefined
            | Token::BOneCmpl => parse_operation(it),

            Token::BSlash => parse_anon_fun(it),

            _ => Err(expected_one_of(
                &[
                    Token::If,
                    Token::Match,
                    Token::LRBrack,
                    Token::LSBrack,
                    Token::LCBrack,
                    Token::Ret,
                    Token::Underscore,
                    Token::_Self,
                    Token::Real(String::new()),
                    Token::Int(String::new()),
                    Token::ENum(String::new(), String::new()),
                    Token::Bool(true),
                    Token::Bool(false),
                    Token::Not,
                    Token::Sqrt,
                    Token::Add,
                    Token::Id(String::new()),
                    Token::Sub,
                    Token::Undefined,
                    Token::BOneCmpl,
                    Token::BSlash
                ],
                lex,
                "expression"
            ))
        },
        &[
            Token::If,
            Token::Match,
            Token::LRBrack,
            Token::LSBrack,
            Token::LCBrack,
            Token::Ret,
            Token::Underscore,
            Token::_Self,
            Token::Real(String::new()),
            Token::Int(String::new()),
            Token::ENum(String::new(), String::new()),
            Token::Bool(true),
            Token::Bool(false),
            Token::Not,
            Token::Sqrt,
            Token::Add,
            Token::Id(String::new()),
            Token::Sub,
            Token::Undefined,
            Token::BOneCmpl,
            Token::BSlash
        ],
        "expression"
    );

    match result {
        Ok(res) => parse_post_expr(&res, it),
        err => err
    }
}

fn parse_underscore(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("underscore")?;
    let end = it.eat(&Token::Underscore, "underscore")?;
    Ok(Box::from(AST::new(&start.union(&end), Node::Underscore)))
}

fn parse_post_expr(pre: &AST, it: &mut LexIterator) -> ParseResult {
    it.peek(
        &|it, lex| match lex.token {
            Token::Question => {
                it.eat(&Token::Question, "postfix expression")?;
                let right = it.parse(&parse_expression, "postfix expression", &lex.pos)?;
                let node = Node::Question { left: Box::new(pre.clone()), right: right.clone() };
                let res = AST::new(&lex.pos.union(&right.pos), node);
                parse_post_expr(&res, it)
            }
            Token::Assign => {
                let res = parse_reassignment(pre, it)?;
                parse_post_expr(&res, it)
            }
            Token::LRBrack | Token::Point => {
                let res = parse_call(pre, it)?;
                parse_post_expr(&res, it)
            }
            _ =>
                if is_start_expression_exclude_unary(lex) {
                    let res = parse_call(pre, it)?;
                    parse_post_expr(&res, it)
                } else {
                    Ok(Box::from(pre.clone()))
                },
        },
        Ok(Box::from(pre.clone()))
    )
}

fn parse_return(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("return")?;
    it.eat(&Token::Ret, "return")?;

    if let Some(end) = it.eat_if(&Token::NL) {
        let node = Node::ReturnEmpty;
        return Ok(Box::from(AST::new(&start.union(&end), node)));
    }

    let expr = it.parse(&parse_expression, "return", &start)?;
    Ok(Box::from(AST::new(&start.union(&expr.pos), Node::Return { expr })))
}

/// Excluding unary addition and subtraction
pub fn is_start_expression_exclude_unary(tp: &Lex) -> bool {
    match tp.token {
        Token::If
        | Token::Match
        | Token::LRBrack
        | Token::LSBrack
        | Token::LCBrack
        | Token::Underscore
        | Token::BSlash
        | Token::_Self
        | Token::Real(_)
        | Token::Int(_)
        | Token::ENum(..)
        | Token::Str(_)
        | Token::Bool(_)
        | Token::Not
        | Token::Undefined
        | Token::Id(_) => true,
        _ => false
    }
}

pub fn is_start_expression(tp: &Lex) -> bool {
    let start_expr = is_start_expression_exclude_unary(tp);
    start_expr || tp.token == Token::Add || tp.token == Token::Sub
}
