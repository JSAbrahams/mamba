use crate::lexer::token::Token;
use crate::parser::_type::parse_generics;
use crate::parser::_type::parse_id;
use crate::parser::_type::parse_id_maybe_type;
use crate::parser::_type::parse_type;
use crate::parser::ast::Node;
use crate::parser::ast::AST;
use crate::parser::expr_or_stmt::parse_expr_or_stmt;
use crate::parser::iterator::LexIterator;
use crate::parser::operation::parse_expression;
use crate::parser::parse_result::custom;
use crate::parser::parse_result::ParseResult;

pub fn parse_definition(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("definition")?;
    it.eat(&Token::Def, "definition")?;
    let private = it.eat_if(&Token::Private).is_some();
    let pure = it.eat_if(&Token::Pure).is_some();

    macro_rules! op {
        ($it:expr, $token:ident, $node:ident) => {{
            let end = $it.eat(&Token::$token, "definition")?;
            let node_pos = AST::new(&start.union(&end), Node::$node);
            parse_fun_def(&node_pos, pure, private, $it)
        }};
    };

    if pure {
        let id = it.parse(&parse_id_maybe_type, "definition", &start)?;
        parse_fun_def(&id, pure, private, it)
    } else {
        it.peek_or_err(
            &|it, lex| match lex.token {
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

fn parse_var_or_fun_def(it: &mut LexIterator, private: bool) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let id = *it.parse(&parse_id_maybe_type, "variable or function definition", &start)?;

    match id.node {
        Node::IdType { _type: Some(_), .. } | Node::TypeTup { .. } =>
            parse_variable_def_id(&id, private, it),
        Node::IdType { _type: None, mutable, .. } => it.peek(
            &|it, lex| match lex.token {
                Token::LRBrack => {
                    if mutable {
                        return Err(custom("Function definition cannot be mutable.", &id.pos));
                    }
                    parse_fun_def(&id, false, private, it)
                }
                _ => parse_variable_def_id(&id, private, it)
            },
            {
                let node = Node::VariableDef {
                    private,
                    id_maybe_type: Box::from(id.clone()),
                    expression: None,
                    forward: vec![]
                };
                Ok(Box::from(AST::new(&id.pos.union(&id.pos), node)))
            }
        ),
        _ => Err(custom("definition must start with id type", &id.pos))
    }
}

fn parse_fun_def(id_type: &AST, pure: bool, private: bool, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("function definition")?;
    let fun_args = it.parse_vec(&parse_fun_args, "function definition", &start)?;

    let id = match &id_type.node {
        Node::IdType { id, mutable, _type } => match (mutable, _type) {
            (false, None) => id.clone(),
            (true, _) => return Err(custom("Function definition cannot be mutable", &id.pos)),
            (_, Some(_)) => return Err(custom("Function identifier cannot have type", &id.pos))
        },

        Node::AddOp
        | Node::SubOp
        | Node::SqrtOp
        | Node::MulOp
        | Node::DivOp
        | Node::FDivOp
        | Node::PowOp
        | Node::ModOp
        | Node::EqOp
        | Node::GeOp
        | Node::LeOp => Box::from(id_type.clone()),

        _ => return Err(custom("Function definition not given id or operator", &id_type.pos))
    };

    let ret_ty = it.parse_if(&Token::To, &parse_type, "function return type", &start)?;
    let raises = it.parse_vec_if(&Token::Raises, &parse_raises, "raises", &start)?;
    let body = it.parse_if(&Token::BTo, &parse_expr_or_stmt, "function body", &start)?;

    let end = match (&ret_ty, &raises.last(), &body) {
        (_, _, Some(b)) => b.pos.clone(),
        (_, Some(b), _) => b.pos.clone(),
        (Some(b), ..) => b.pos.clone(),
        _ => id_type.pos.clone()
    };

    let node = Node::FunDef { id, pure, private, fun_args, ret_ty, raises, body };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_raises(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.eat(&Token::LSBrack, "raises")?;
    let args = it.parse_vec(&parse_generics, "raises", &start)?;
    it.eat(&Token::RSBrack, "raises")?;
    Ok(args)
}

pub fn parse_fun_args(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
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

pub fn parse_fun_arg(it: &mut LexIterator) -> ParseResult {
    let start = &it.start_pos("function argument")?;
    let vararg = it.eat_if(&Token::Vararg).is_some();

    let id_maybe_type = it.parse(&parse_id_maybe_type, "function argument", start)?;
    let default =
        it.parse_if(&Token::Assign, &parse_expression, "function argument default", start)?;

    let end = default.clone().map_or(id_maybe_type.pos.clone(), |def| def.pos);
    let node = Node::FunArg { vararg, id_maybe_type, default };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

pub fn parse_forward(it: &mut LexIterator) -> ParseResult<Vec<AST>> {
    let start = it.start_pos("forward")?;
    let mut forwarded: Vec<AST> = vec![];
    it.peek_while_not_token(&Token::NL, &mut |it, _| {
        forwarded.push(*it.parse(&parse_id, "forward", &start)?);
        it.eat_if(&Token::Comma);
        Ok(())
    })?;

    Ok(forwarded)
}

fn parse_variable_def_id(id: &AST, private: bool, it: &mut LexIterator) -> ParseResult {
    let start = &id.pos;
    let expression = it.parse_if(&Token::Assign, &parse_expression, "definition body", start)?;
    let forward = it.parse_vec_if(&Token::Forward, &parse_forward, "definition raises", start)?;

    let end = &match (&expression, &forward.last()) {
        (_, Some(expr)) => expr.pos.clone(),
        (Some(expr), _) => expr.pos.clone(),
        _ => id.pos.clone()
    };
    let node =
        Node::VariableDef { private, id_maybe_type: Box::from(id.clone()), expression, forward };
    Ok(Box::from(AST::new(&start.union(&end), node)))
}

fn parse_variable_def(private: bool, it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("variable definition")?;
    let id = it.parse(&parse_id_maybe_type, "variable definition", &start)?;
    parse_variable_def_id(&id, private, it)
}
