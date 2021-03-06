use crate::lex::token::Lex;
use crate::lex::token::Token;
use crate::parse::ast::Node;
use crate::parse::ast::AST;
use crate::parse::call::parse_anon_fun;
use crate::parse::call::parse_call;
use crate::parse::call::parse_reassignment;
use crate::parse::collection::parse_collection;
use crate::parse::control_flow_expr::parse_cntrl_flow_expr;
use crate::parse::iterator::LexIterator;
use crate::parse::operation::parse_expression;
use crate::parse::parse_direct;
use crate::parse::result::expected_one_of;
use crate::parse::result::ParseResult;
use crate::parse::ty::parse_id;
use std::ops::Deref;

pub fn parse_inner_expression(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("literal")?;
    macro_rules! literal {
        ($it:expr, $factor:expr, $ast:ident) => {{
            let end = $it.eat(&Token::$ast($factor.clone()), "factor")?;
            let node = Node::$ast { lit: $factor };
            Ok(Box::from(AST::new(&start.union(&end), node)))
        }};
    }

    let result = it.peek_or_err(
        &|it, lex| match &lex.token {
            Token::If | Token::Match => parse_cntrl_flow_expr(it),
            Token::LRBrack | Token::LSBrack | Token::LCBrack => parse_collection(it),
            Token::Ret => parse_return(it),
            Token::Underscore => parse_underscore(it),

            Token::Id(_) => parse_id(it),
            Token::_Self => parse_id(it),
            Token::Real(real) => literal!(it, real.to_string(), Real),
            Token::Int(int) => literal!(it, int.to_string(), Int),
            Token::Bool(b) => literal!(it, *b, Bool),
            Token::Str(string, tokens) => {
                let end = it.eat(&Token::Str(string.clone(), tokens.clone()), "factor")?;

                let expressions: Vec<Box<AST>> =
                    tokens.iter().map(|tokens| parse_direct(tokens)).collect::<Result<_, _>>()?;
                let node = Node::Str {
                    lit:         string.clone(),
                    expressions: expressions.iter().map(|expr| expr.deref().clone()).collect()
                };
                Ok(Box::from(AST::new(&start.union(&end), node)))
            }
            Token::ENum(num, exp) => {
                let end = it.eat(&Token::ENum(num.clone(), exp.clone()), "factor")?;
                let node = Node::ENum { num: num.to_string(), exp: exp.to_string() };
                Ok(Box::from(AST::new(&start.union(&end), node)))
            }
            Token::Undefined => {
                let end = it.eat(&Token::Undefined, "factor")?;
                Ok(Box::from(AST::new(&start.union(&end), Node::Undefined)))
            }

            Token::Not | Token::Sqrt | Token::Add | Token::Sub | Token::BOneCmpl =>
                parse_expression(it),

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
        | Token::Str(..)
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
