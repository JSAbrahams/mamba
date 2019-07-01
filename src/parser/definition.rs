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
use crate::parser::parse_result::custom;
use crate::parser::parse_result::ParseResult;

pub fn parse_definition(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("definition")?;
    it.eat(&Token::Def, "definition")?;
    let private = it.eat_if(&Token::Private).is_some();
    let pure = it.eat_if(&Token::Pure).is_some();

    macro_rules! op {
        ($it:expr, $token:ident, $node:ident) => {{
            let (en_line, en_pos) = $it.eat(&Token::$token, "definition")?;
            let node_pos = ASTNodePos { st_line, st_pos, en_line, en_pos, node: ASTNode::$node };
            parse_fun_def(&node_pos, pure, $it)
        }};
    };

    let definition = if pure {
        let id = it.parse(&parse_id_maybe_type, "definition")?;
        parse_fun_def(&id, pure, it)
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
            &[
                Token::Id(String::new()),
                Token::LRBrack,
                Token::LCBrack,
                Token::LSBrack,
                Token::Add,
                Token::Sub,
                Token::Sqrt,
                Token::Mul,
                Token::FDiv,
                Token::Div,
                Token::Pow,
                Token::Mod,
                Token::Eq,
                Token::Ge,
                Token::Le
            ],
            "definition"
        )
    }?;

    let (en_line, en_pos) = (definition.en_line, definition.en_pos);
    let node = ASTNode::Def { private, definition };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_var_or_fun_def(it: &mut TPIterator) -> ParseResult {
    let id = *it.parse(&parse_id_maybe_type, "variable or function definition")?;

    match id {
        ASTNodePos { node: ASTNode::IdType { _type: Some(_), .. }, .. }
        | ASTNodePos { node: ASTNode::TypeTup { .. }, .. } => parse_variable_def_id(&id, it),
        ASTNodePos {
            node: ASTNode::IdType { _type: None, mutable, .. }, st_line, st_pos, ..
        } => it.peek(
            &|it, token_pos| match token_pos.token {
                Token::LRBrack => {
                    if mutable {
                        return Err(custom(
                            "Function definition cannot be mutable.",
                            st_line,
                            st_pos
                        ));
                    }
                    parse_fun_def(&id, false, it)
                }
                _ => parse_variable_def_id(&id, it)
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
        ASTNodePos { st_line, st_pos, .. } =>
            Err(custom("definition must start with id type.", st_line, st_pos)),
    }
}

fn parse_fun_def(id_type: &ASTNodePos, pure: bool, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("function definition")?;
    let fun_args = it.parse_vec(&parse_fun_args, "function definition")?;

    let id = match id_type {
        ASTNodePos { node: ASTNode::IdType { id, mutable, _type }, st_line, st_pos, .. } =>
            match (mutable, _type) {
                (false, None) => id.clone(),
                (true, _) => {
                    return Err(custom("Function definition cannot be mutable", *st_line, *st_pos));
                }
                (_, Some(_)) => {
                    return Err(custom(
                        "Function definition given id type with some type.",
                        *st_line,
                        *st_pos
                    ));
                }
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

        ASTNodePos { st_line, st_pos, .. } => {
            return Err(custom(
                "Function definition not given id or operator: {:?}",
                *st_line,
                *st_pos
            ));
        }
    };

    let ret_ty = it.parse_if(&Token::DoublePoint, &parse_type, "function return type")?;
    let raises = it.parse_vec_if(&Token::Raises, &parse_raises, "raises")?;
    let body = it.parse_if(&Token::BTo, &parse_expr_or_stmt, "function body")?;

    let (en_line, en_pos) = match (&ret_ty, &raises.last(), &body) {
        (_, _, Some(b)) => (b.en_line, b.en_pos),
        (_, Some(b), _) => (b.en_line, b.en_pos),
        (Some(b), ..) => (b.en_line, b.en_pos),
        _ => (id_type.en_line, id_type.en_pos)
    };

    let node = ASTNode::FunDef { id, pure, fun_args, ret_ty, raises, body };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_raises(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(&Token::LSBrack, "raises")?;
    let args = it.parse_vec(&parse_generics, "raises")?;
    it.eat(&Token::RSBrack, "raises")?;
    Ok(args)
}

pub fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    it.eat(&Token::LRBrack, "function arguments")?;

    let mut args = Vec::new();
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "function arguments")?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::RRBrack, "function arguments")?;
    Ok(args)
}

pub fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = it.start_pos("function argument")?;
    let vararg = it.eat_if(&Token::Vararg).is_some();

    let id_maybe_type = it.parse(&parse_id_maybe_type, "function argument")?;
    let default = it.parse_if(&Token::Assign, &parse_expression, "function argument default")?;

    let (en_line, en_pos) = match &default {
        Some(ast_node_pos) => (ast_node_pos.en_line, ast_node_pos.en_pos),
        _ => (id_maybe_type.en_line, id_maybe_type.en_pos)
    };
    let node = ASTNode::FunArg { vararg, id_maybe_type, default };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let mut forwarded: Vec<ASTNodePos> = Vec::new();
    it.peek_while_not_token(&Token::NL, &mut |it, _| {
        forwarded.push(*it.parse(&parse_id, "forward")?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(forwarded)
}

fn parse_variable_def_id(id: &ASTNodePos, it: &mut TPIterator) -> ParseResult {
    let (st_line, st_pos) = (id.st_line, id.st_pos);
    let ofmut = it.eat_if(&Token::OfMut).is_some();

    let expression = it.parse_if(&Token::Assign, &parse_expression, "definition body")?;
    let forward = it.parse_vec_if(&Token::Forward, &parse_forward, "definition raises")?;

    let (en_line, en_pos) = match (&expression, &forward.last()) {
        (_, Some(expr)) => (expr.en_line, expr.en_pos),
        (Some(expr), _) => (expr.en_line, expr.en_pos),
        _ => (id.en_line, id.en_pos)
    };
    let node =
        ASTNode::VariableDef { ofmut, id_maybe_type: Box::from(id.clone()), expression, forward };
    Ok(Box::from(ASTNodePos { st_line, st_pos, en_line, en_pos, node }))
}

fn parse_variable_def(it: &mut TPIterator) -> ParseResult {
    let id = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::LRBrack | Token::LCBrack | Token::LSBrack =>
                it.parse(&parse_collection, "variable definition"),
            _ => it.parse(&parse_id_maybe_type, "variable definition")
        },
        &[Token::LRBrack, Token::LCBrack, Token::LSBrack],
        "variable definition"
    )?;
    parse_variable_def_id(&id, it)
}
