use crate::lexer::token::Token;
use crate::lexer::token::TokenPos;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ast::ASTNode;
use crate::parser::ast::ASTNodePos;
use crate::parser::collection::parse_collection;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::expression::parse_expression;
use crate::parser::iterator::TPIterator;
use crate::parser::parse_result::ParseErr::*;
use crate::parser::parse_result::ParseResult;

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    it.eat(Token::Def);

    let private = it.eat_if(Token::Private);
    let pure = it.eat_if(Token::Pure);

    macro_rules! op {
        ($node:ident) => {{
            let (en_line, en_pos) = it.end_pos()?;
            let node_pos = ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::$node };
            it.next();
            parse_fun_def(node_pos, pure, it)
        }};
    };

    let definition = if pure {
        let id = it.parse(&parse_id_maybe_type, "definition id")?;
        parse_fun_def(*id, pure, it)
    } else {
        it.peek(
            &|token_pos| match token_pos.token {
                Token::Mut | Token::LRBrack | Token::LCBrack | Token::LSBrack =>
                    parse_variable_def(it),

                Token::Add => op!(AddOp),
                Token::Sub => op!(SubOp),
                Token::Sqrt => op!(SqrtOp),
                Token::Mul => op!(MulOp),
                Token::FDiv => op!(FDivOp),
                Token::Div => op!(DivOp),
                Token::Pow => op!(PowOp),
                Token::Mod => op!(ModOp),
                Token::Eq => op!(EqOp),
                Token::Ge => op!(GeOp),
                Token::Le => op!(LeOp),

                _ => unimplemented!() /*            _ => match it.parse(&parse_id_maybe_type,
                                       * "definition id")? {
                                       *                id @ ASTNodePos { node: ASTNode::IdType {
                                       * _type: Some(_), .. }, .. }
                                       *                | id @ ASTNodePos { node:
                                       * ASTNode::TypeTup { .. }, .. } =>
                                       *                    parse_variable_def_id(id, it),
                                       *                id @ ASTNodePos {
                                       *                    node: ASTNode::IdType { _type: None,
                                       * mutable: false, .. },
                                       *                    ..
                                       *                } => match it.peek() {
                                       *                    Some(TokenPos { token:
                                       * Token::LRBrack, .. }) => parse_fun_def(id, pure, it),
                                       *                    None | Some(_) =>
                                       * parse_variable_def_id(id, it)
                                       *                },
                                       *                _ => return Err(InternalErr { message:
                                       * String::from("couldn't parse def") })
                                       *            } */
            },
            CustomEOFErr { expected: String::from("definition cannot be empty") }
        )?;
    };

    match definition {
        Ok(definition) => Ok(ASTNodePos {
            st_line,
            st_pos,
            en_line: definition.en_line,
            en_pos: definition.en_pos,
            node: ASTNode::Def { private, definition: Box::from(definition) }
        }),
        err => err
    }
}

fn parse_fun_def(id_type: ASTNodePos, pure: bool, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let fun_args = it.parse(parse_fun_args, "function arguments");

    let id = match &id_type {
        ASTNodePos { node: ASTNode::IdType { id, mutable, _type }, .. } => match (mutable, _type) {
            (false, None) => id.clone(),
            (true, _) =>
                return Err(InternalErr {
                    message: String::from("Function definition cannot be mutable")
                }),
            (_, Some(_)) =>
                return Err(InternalErr {
                    message: String::from("Function definition given id type with some type.")
                }),
        },

        op @ ASTNodePos { node: ASTNode::AddOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::SubOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::SqrtOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::MulOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::DivOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::FDivOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::PowOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::ModOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::EqOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::GeOp, .. } => Box::from(op.clone()),
        op @ ASTNodePos { node: ASTNode::LeOp, .. } => Box::from(op.clone()),

        other =>
            return Err(InternalErr {
                message: format!("Function definition not given id or operator: {:?}", other)
            }),
    };

    let ret_ty: Option<Box<ASTNodePos>> = match it.peek() {
        Some(TokenPos { token: Token::DoublePoint, .. }) => {
            it.next();
            Some(it.parse(parse_type, "function return type"))
        }
        _ => None
    };

    let raises: Option<Vec<ASTNodePos>>;
    if let Some(TokenPos { token: Token::Raises, .. }) = it.peek() {
        it.next();
        raises = Some(it.parse(parse_generics, "raises"));
    } else {
        raises = None
    }

    let body: Option<Box<ASTNodePos>>;
    if let Some(TokenPos { token: Token::BTo, .. }) = it.peek() {
        it.next();
        body = Some(it.parse(parse_expr_or_stmt, "function body"));
    } else {
        body = None
    }

    let (en_line, en_pos) = match (&ret_ty, &raises, &body) {
        (_, _, Some(b)) => (b.en_line, b.en_pos),
        (_, Some(b), _) if b.last().is_some() =>
            (b.last().unwrap().en_line, b.last().unwrap().en_pos),
        (Some(b), ..) => (b.en_line, b.en_pos),
        _ => (id_type.en_line, id_type.en_pos)
    };

    let node = ASTNode::FunDef { id, pure, fun_args, ret_ty, raises, body };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::LRBrack);

    let mut args = Vec::new();
    it.while_some_and_not(Token::RRBrack, &|token_pos| {
        args.push(*it.parse(&parse_fun_arg, "function arg")?)
    });

    it.eat(Token::RRBrack);
    Ok(args)
}

pub fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let vararg = it.eat_if(Token::Vararg);

    let id_maybe_type = it.parse(parse_id_maybe_type, "argument")?;
    let default = it.peek_if_or_none(Token::Assign, &|token_pos| {
        it.parse(&parse_expression, "argument default")?
    });

    let (en_line, en_pos) = it.end_pos()?;
    let node = ASTNode::FunArg { vararg, id_maybe_type, default };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(Token::Forward);

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    it.while_some_and_not(Token::NL, &|token_pos| {
        forwarded.push(*it.parse(parse_id, "forward")?);
        it.eat_if(Token::Comma);
        Ok(())
    });

    Ok(forwarded)
}

fn parse_variable_def_id(id: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let ofmut = it.eat_if(Token::OfMut);

    let expression = if it.eat_if(Token::Assign) {
        Some(it.parse(parse_expression, "definition expression"))
    } else {
        None
    };

    let forward: Vec<ASTNodePos> = if it.peek_is(Token::Forward) {
        it.parse(&parse_forward, "definition raises")
    } else {
        vec![]
    };

    let (en_line, en_pos) = match (&expression, forward.last()) {
        (_, Some(expr)) => (expr.en_line, expr.en_pos),
        (Some(expr), _) => (expr.en_line, expr.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::VariableDef { ofmut, id_maybe_type: Box::from(id), expression, forward };
    Ok(ASTNodePos { st_line, st_pos, en_line, en_pos, node })
}

fn parse_variable_def(it: &mut TPIterator) -> ParseResult {
    let id = it.peek(
        &|token_pos| match token_pos.token {
            Token::LRBrack | Token::LCBrack | Token::LSBrack =>
                it.parse(&parse_collection, "collection"),
            _ => it.parse(&parse_id_maybe_type, "variable id")
        },
        CustomEOFErr { expected: String::from("variable definition") }
    )?;

    parse_variable_def_id(id, it)
}
