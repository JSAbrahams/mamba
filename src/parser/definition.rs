use crate::lexer::token::Token;
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
    it.eat_token(Token::Def)?;
    let private = it.eat_if_token(Token::Private);
    let pure = it.eat_if_token(Token::Pure);

    macro_rules! op {
        ($it:expr, $token:ident, $node:ident) => {{
            let (en_line, en_pos) = $it.end_pos()?;
            let node_pos = ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::$node };
            $it.eat_token(Token::$token)?;
            parse_fun_def(node_pos, pure, $it)
        }};
    };

    let definition = if pure {
        let id = it.parse(&parse_id_maybe_type, "definition id")?;
        parse_fun_def(*id, pure, it)
    } else {
        it.peek_or_err(
            &|it, token_pos| match token_pos.token {
                Token::LRBrack | Token::LCBrack | Token::LSBrack => parse_variable_def(it),

                Token::Add => op!(it, Add, AddOp),
                Token::Sub => op!(it, Sub, SubOp),
                Token::Sqrt => op!(it, Sqrt, SqrtOp),
                Token::Mul => op!(it, Mul, MulOp),
                Token::FDiv => op!(it, FDiv, FDivOp),
                Token::Div => op!(it, Div, DivOp),
                Token::Pow => op!(it, Pow, PowOp),
                Token::Mod => op!(it, Mod, ModOp),
                Token::Eq => op!(it, Eq, EqOp),
                Token::Ge => op!(it, Ge, GeOp),
                Token::Le => op!(it, Le, LeOp),
                _ => parse_var_or_fun_def(it)
            },
            CustomEOFErr { expected: String::from("definition cannot be empty") }
        )
    }?;

    let (en_line, en_pos) = (definition.en_line, definition.en_pos);
    let node = ASTNode::Def { private, definition: Box::from(definition) };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_var_or_fun_def(it: &mut TPIterator) -> ParseResult {
    let id = *it.parse(&parse_id_maybe_type, "definition id")?;

    match id {
        ASTNodePos { node: ASTNode::IdType { _type: Some(_), .. }, .. }
        | ASTNodePos { node: ASTNode::TypeTup { .. }, .. } => parse_variable_def_id(id, it),
        ASTNodePos { node: ASTNode::IdType { _type: None, mutable, .. }, .. } => it.peek(
            &|it, token_pos| match token_pos.token {
                Token::LRBrack => {
                    if mutable {
                        return Err(InternalErr {
                            message: String::from("Function definition cannot be mutable.")
                        });
                    }
                    parse_fun_def(id.clone(), false, it)
                }
                _ => parse_variable_def_id(id.clone(), it)
            },
            {
                let (st_line, st_pos) = (id.st_line, id.st_pos);
                let (en_line, en_pos) = (id.en_line, id.en_pos);
                let node = ASTNode::VariableDef {
                    ofmut:         false,
                    id_maybe_type: Box::from(id.clone()),
                    expression:    None,
                    forward:       vec![]
                };
                Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
            }
        ),
        ASTNodePos { node, .. } =>
            Err(InternalErr { message: format!("def didn't start with id type: {:?}", node) }),
    }
}

fn parse_fun_def(id_type: ASTNodePos, pure: bool, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let fun_args = it.parse_vec(&parse_fun_args, "function arguments")?;

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

    let ret_ty = it.parse_if_token(Token::DoublePoint, &parse_type, "function return type")?;
    let raises = it.parse_vec_if_token(Token::Raises, &parse_generics, "raises")?;
    let body = it.parse_if_token(Token::BTo, &parse_expr_or_stmt, "function body")?;

    let (en_line, en_pos) = match (&ret_ty, &raises.last(), &body) {
        (_, _, Some(b)) => (b.en_line, b.en_pos),
        (_, Some(b), _) => (b.en_line, b.en_pos),
        (Some(b), ..) => (b.en_line, b.en_pos),
        _ => (id_type.en_line, id_type.en_pos)
    };

    let node = ASTNode::FunDef { id, pure, fun_args, ret_ty, raises, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat_token(Token::LRBrack)?;

    let mut args = Vec::new();
    it.while_not_token(Token::RRBrack, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "function arg")?);
        Ok(())
    })?;

    it.eat_token(Token::RRBrack)?;
    Ok(args)
}

pub fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos()?;
    let vararg = it.eat_if_token(Token::Vararg);

    let id_maybe_type = it.parse(&parse_id_maybe_type, "argument")?;
    let default = it.parse_if_token(Token::Assign, &parse_expression, "argument default")?;

    let (en_line, en_pos) = it.end_pos()?;
    let node = ASTNode::FunArg { vararg, id_maybe_type, default };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat_token(Token::Forward)?;

    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    it.while_not_token(Token::NL, &mut |it, _| {
        forwarded.push(*it.parse(&parse_id, "forward")?);
        it.eat_if_token(Token::Comma);
        Ok(())
    })?;

    Ok(forwarded)
}

fn parse_variable_def_id(id: ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let ofmut = it.eat_if_token(Token::OfMut);

    let expression = it.parse_if_token(Token::Assign, &parse_expression, "definition body")?;
    let forward = it.parse_vec_if_token(Token::Forward, &parse_forward, "definition raises")?;

    let (en_line, en_pos) = match (&expression, &forward.last()) {
        (_, Some(expr)) => (expr.en_line, expr.en_pos),
        (Some(expr), _) => (expr.en_line, expr.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node = ASTNode::VariableDef { ofmut, id_maybe_type: Box::from(id), expression, forward };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_variable_def(it: &mut TPIterator) -> ParseResult {
    let id = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::LRBrack | Token::LCBrack | Token::LSBrack =>
                it.parse(&parse_collection, "collection"),
            _ => it.parse(&parse_id_maybe_type, "variable id")
        },
        CustomEOFErr { expected: String::from("variable definition") }
    )?;

    parse_variable_def_id(*id, it)
}
