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
    let start = it.start_pos("definition")?;
    it.eat(&Token::Def, "definition")?;
    let private = it.eat_if(&Token::Private).is_some();
    let pure = it.eat_if(&Token::Pure).is_some();

    macro_rules! op {
        ($it:expr, $token:ident, $node:ident) => {{
            let end = $it.eat(&Token::$token, "definition")?;
            let node_pos = ASTNodePos::new(&start, &end, ASTNode::$node);
            parse_fun_def(&node_pos, pure, private, $it)
        }};
    };

    if pure {
        let id = it.parse(&parse_id_maybe_type, "definition", &start)?;
        parse_fun_def(&id, pure, private, it)
    } else {
        it.peek_or_err(
            &|it, token_pos| match token_pos.token {
                Token::LRBrack | Token::LCBrack | Token::LSBrack => parse_variable_def(private, it),

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
                _ => parse_var_or_fun_def(it, private)
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
    }
}

fn parse_var_or_fun_def(it: &mut TPIterator, private: bool) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let id = *it.parse(&parse_id_maybe_type, "variable or function definition", &start)?;

    match id.node {
        ASTNode::IdType { _type: Some(_), .. } | ASTNode::TypeTup { .. } =>
            parse_variable_def_id(&id, private, it),
        ASTNode::IdType { _type: None, mutable, .. } => it.peek(
            &|it, token_pos| match token_pos.token {
                Token::LRBrack => {
                    if mutable {
                        return Err(custom("Function definition cannot be mutable.", &id.position));
                    }
                    parse_fun_def(&id, false, private, it)
                }
                _ => parse_variable_def_id(&id, private, it)
            },
            {
                let node = ASTNode::VariableDef {
                    ofmut: false,
                    private,
                    id_maybe_type: Box::from(id.clone()),
                    expression: None,
                    forward: vec![]
                };
                Ok(Box::from(ASTNodePos::new(&id.position.start, &id.position.end, node)))
            }
        ),
        _ => Err(custom("definition must start with id type", &id.position))
    }
}

fn parse_fun_def(
    id_type: &ASTNodePos,
    pure: bool,
    private: bool,
    it: &mut TPIterator
) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let fun_args = it.parse_vec(&parse_fun_args, "function definition", &start)?;

    let id = match &id_type.node {
        ASTNode::IdType { id, mutable, _type } => match (mutable, _type) {
            (false, None) => id.clone(),
            (true, _) => return Err(custom("Function definition cannot be mutable", &id.position)),
            (_, Some(_)) =>
                return Err(custom("Function identifier cannot have type", &id.position)),
        },

        ASTNode::AddOp
        | ASTNode::SubOp
        | ASTNode::SqrtOp
        | ASTNode::MulOp
        | ASTNode::DivOp
        | ASTNode::FDivOp
        | ASTNode::PowOp
        | ASTNode::ModOp
        | ASTNode::EqOp
        | ASTNode::GeOp
        | ASTNode::LeOp => Box::from(id_type.clone()),

        _ => return Err(custom("Function definition not given id or operator", &id_type.position))
    };

    let ret_ty = it.parse_if(&Token::DoublePoint, &parse_type, "function return type", &start)?;
    let raises = it.parse_vec_if(&Token::Raises, &parse_raises, "raises", &start)?;
    let body = it.parse_if(&Token::BTo, &parse_expr_or_stmt, "function body", &start)?;

    let end = match (&ret_ty, &raises.last(), &body) {
        (_, _, Some(b)) => b.position.end.clone(),
        (_, Some(b), _) => b.position.end.clone(),
        (Some(b), ..) => b.position.end.clone(),
        _ => id_type.position.end.clone()
    };

    let node = ASTNode::FunDef { id, pure, private, fun_args, ret_ty, raises, body };
    Ok(Box::from(ASTNodePos::new(&start, &end, node)))
}

pub fn parse_raises(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let start = it.eat(&Token::LSBrack, "raises")?;
    let args = it.parse_vec(&parse_generics, "raises", &start)?;
    it.eat(&Token::RSBrack, "raises")?;
    Ok(args)
}

pub fn parse_fun_args(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let start = it.eat(&Token::LRBrack, "function arguments")?;
    let mut args = vec![];
    it.peek_while_not_token(&Token::RRBrack, &mut |it, _| {
        args.push(*it.parse(&parse_fun_arg, "function arguments", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    it.eat(&Token::RRBrack, "function arguments")?;
    Ok(args)
}

pub fn parse_fun_arg(it: &mut TPIterator) -> ParseResult {
    let start = &it.start_pos("function argument")?;
    let vararg = it.eat_if(&Token::Vararg).is_some();

    let id_maybe_type = it.parse(&parse_id_maybe_type, "function argument", start)?;
    let default =
        it.parse_if(&Token::Assign, &parse_expression, "function argument default", start)?;

    let end = default.clone().map_or(id_maybe_type.position.end.clone(), |def| def.position.end);
    let node =
        ASTNode::FunArg { vararg, id_maybe_type: id_maybe_type.clone(), default: default.clone() };
    Ok(Box::from(ASTNodePos::new(start, &end, node)))
}

pub fn parse_forward(it: &mut TPIterator) -> ParseResult<Vec<ASTNodePos>> {
    let start = it.start_pos("forward")?;
    let mut forwarded: Vec<ASTNodePos> = vec![];
    it.peek_while_not_token(&Token::NL, &mut |it, _| {
        forwarded.push(*it.parse(&parse_id, "forward", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(forwarded)
}

fn parse_variable_def_id(id: &ASTNodePos, private: bool, it: &mut TPIterator) -> ParseResult {
    let start = &id.position.start;
    let ofmut = it.eat_if(&Token::OfMut).is_some();

    let expression = it.parse_if(&Token::Assign, &parse_expression, "definition body", start)?;
    let forward = it.parse_vec_if(&Token::Forward, &parse_forward, "definition raises", start)?;

    let end = &match (&expression, &forward.last()) {
        (_, Some(expr)) => expr.position.end.clone(),
        (Some(expr), _) => expr.position.end.clone(),
        _ => id.position.end.clone()
    };
    let node = ASTNode::VariableDef {
        ofmut,
        private,
        id_maybe_type: Box::from(id.clone()),
        expression,
        forward
    };
    Ok(Box::from(ASTNodePos::new(start, end, node)))
}

fn parse_variable_def(private: bool, it: &mut TPIterator) -> ParseResult {
    let id = it.peek_or_err(
        &|it, token_pos| match token_pos.token {
            Token::LRBrack | Token::LCBrack | Token::LSBrack =>
                it.parse(&parse_collection, "variable definition", &token_pos.start),
            _ => it.parse(&parse_id_maybe_type, "variable definition", &token_pos.start)
        },
        &[Token::LRBrack, Token::LCBrack, Token::LSBrack],
        "variable definition"
    )?;
    parse_variable_def_id(&id, private, it)
}
