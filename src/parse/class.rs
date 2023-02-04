use crate::parse::ast::AST;
use crate::parse::ast::Node;
use crate::parse::block::parse_block;
use crate::parse::definition::{parse_definition, parse_fun_arg};
use crate::parse::iterator::LexIterator;
use crate::parse::lex::token::Token;
use crate::parse::operation::parse_expression;
use crate::parse::result::{custom, expected, expected_one_of};
use crate::parse::result::ParseResult;
use crate::parse::ty::{parse_conditions, parse_id};
use crate::parse::ty::parse_type;

pub fn parse_class(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("class")?;
    it.eat(&Token::Class, "class")?;
    let ty = it.parse(&parse_type, "class", start)?;

    let mut args = vec![];
    if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, lex| match lex.token {
            Token::Def => {
                args.push(*it.parse(&parse_definition, "constructor argument", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => {
                args.push(*it.parse(&parse_fun_arg, "constructor argument", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
        })?;
        it.eat(&Token::RRBrack, "class arguments")?;
    }

    let mut parents = vec![];
    if it.eat_if(&Token::DoublePoint).is_some() {
        it.peek_while_not_token(&Token::NL, &mut |it, lex| match lex.token {
            Token::Id(_) | Token::LRBrack => {
                parents.push(*it.parse(&parse_parent, "parents", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(Box::from(expected(&Token::Id(String::new()), &lex.clone(), "parents")))
        })?;
    }

    let (body, pos) = if it.peek_if_followed_by(&Token::NL, &Token::Indent) {
        let body = it.parse(&parse_block, "class", start)?;
        (Some(body.clone()), start.union(body.pos))
    } else {
        (None, start)
    };

    let node = Node::Class { ty, args, parents, body };
    Ok(Box::from(AST::new(pos, node)))
}

pub fn parse_parent(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("parent")?;
    let ty = it.parse(&parse_type, "parent", start)?;

    let mut args = vec![];
    let end = if it.eat_if(&Token::LRBrack).is_some() {
        it.peek_while_not_token(&Token::RRBrack, &mut |it, lex| match &lex.token {
            Token::Id { .. } => {
                args.push(*it.parse(&parse_id, "parent arguments", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            Token::Str { .. } => {
                args.push(*it.parse(&parse_expression, "parent arguments", start)?);
                it.eat_if(&Token::Comma);
                Ok(())
            }
            _ => Err(Box::from(expected_one_of(
                &[
                    Token::Id(String::new()),
                    Token::Str(String::new(), vec![]),
                    Token::Int(String::new()),
                    Token::Real(String::new()),
                    Token::ENum(String::new(), String::new()),
                    Token::Bool(true),
                    Token::Bool(false)
                ],
                lex,
                "parent arguments",
            )))
        })?;
        it.eat(&Token::RRBrack, "parent arguments")?
    } else {
        ty.pos
    };

    let node = Node::Parent { ty, args };
    Ok(Box::from(AST::new(start.union(end), node)))
}

pub fn parse_type_def(it: &mut LexIterator) -> ParseResult {
    let start = it.start_pos("type definition")?;
    it.eat(&Token::Type, "type definition")?;
    let ty = it.parse(&parse_type, "type definition", start)?;
    let isa = it.parse_if(&Token::DoublePoint, &parse_parent, "type parent", start)?;

    it.peek(
        &|it, lex| match lex.token {
            Token::When => {
                it.eat(&Token::When, "conditional type")?;
                let isa = isa
                    .clone()
                    .ok_or_else(|| custom("conditional type must have parent type", lex.pos))?;

                let conditions = it.parse_vec(&parse_conditions, "conditional type", start)?;
                let end = conditions.last().map_or(ty.pos, |cond| cond.pos);

                let node = Node::TypeAlias { ty: ty.clone(), isa, conditions };
                Ok(Box::from(AST::new(start.union(end), node)))
            }
            _ if it.peek_if_followed_by(&Token::NL, &Token::Indent) => {
                it.eat_if(&Token::NL);
                let body = it.parse(&parse_block, "type definition", start)?;
                let isa = isa.clone();
                let node = Node::TypeDef { ty: ty.clone(), isa, body: Some(body.clone()) };
                Ok(Box::from(AST::new(start.union(body.pos), node)))
            }
            _ => {
                let node = Node::TypeDef { ty: ty.clone(), isa: isa.clone(), body: None };
                Ok(Box::from(AST::new(start.union(ty.pos), node)))
            }
        },
        {
            let isa = isa.clone();
            let node = Node::TypeDef { ty: ty.clone(), isa, body: None };
            Ok(Box::from(AST::new(start.union(ty.pos), node)))
        },
    )
}

#[cfg(test)]
mod test {
    use crate::common::result::WithSource;
    use crate::parse::ast::{AST, Node};
    use crate::parse::result::{ParseErr, ParseResult};
    use crate::test_util::resource_content;

    #[test]
    fn import_verify() {
        let source = String::from("import d");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block { statements: modules, .. } => match &modules.first().expect("script empty.").node {
                Node::Import { from, import, alias } => (from.clone(), import.clone(), alias.clone()),
                _ => panic!("first element script was not list.")
            },
            _ => panic!("ast was not script.")
        };

        assert_eq!(from, None);
        assert_eq!(import.len(), 1);
        assert!(alias.is_empty());
        assert_eq!(import[0].node, Node::Id { lit: String::from("d") });
    }

    #[test]
    fn import_as_verify() {
        let source = String::from("import d as e");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block { statements: modules, .. } => match &modules.first().expect("script empty.").node {
                Node::Import { from, import, alias } => (from.clone(), import.clone(), alias.clone()),
                other => panic!("first element script was not import: {:?}.", other)
            },
            other => panic!("ast was not script: {:?}", other)
        };

        assert_eq!(from, None);
        assert_eq!(import.len(), 1);
        assert_eq!(alias.len(), 1);
        assert_eq!(import[0].node, Node::Id { lit: String::from("d") });
        assert_eq!(alias[0].node, Node::Id { lit: String::from("e") });
    }

    #[test]
    fn from_import_as_verify() {
        let source = String::from("from c import d,f as e,g");
        let ast = source.parse::<AST>().unwrap();

        let (from, import, alias) = match ast.node {
            Node::Block { statements: modules, .. } => match &modules.first().expect("script empty.").node {
                Node::Import { from, import, alias } => (from.clone(), import.clone(), alias.clone()),
                other => panic!("first element script was not from: {:?}.", other)
            },
            other => panic!("ast was not script: {:?}", other)
        };

        assert_eq!(from.unwrap().node, Node::Id { lit: String::from("c") });
        assert_eq!(import.len(), 2);
        assert_eq!(alias.len(), 2);
        assert_eq!(import[0].node, Node::Id { lit: String::from("d") });
        assert_eq!(import[1].node, Node::Id { lit: String::from("f") });
        assert_eq!(alias[0].node, Node::Id { lit: String::from("e") });
        assert_eq!(alias[1].node, Node::Id { lit: String::from("g") });
    }

    #[test]
    fn parse_class_alias() {
        let source = String::from("class MyErr1: Exception(\"Something went wrong\")");
        let ast = source.parse::<AST>().unwrap();

        let (ty, args, parents, body) = match ast.node {
            Node::Block { statements: modules, .. } => match &modules.first().expect("script empty.").node {
                Node::Class { ty, args, parents, body } =>
                    (ty.clone(), args.clone(), parents.clone(), body.clone()),
                other => panic!("Was not class: {:?}.", other)
            },
            other => panic!("Ast was not script: {:?}", other)
        };

        match ty.node {
            Node::Type { id, generics } => {
                assert_eq!(id.node, Node::Id { lit: String::from("MyErr1") });
                assert_eq!(generics.len(), 0);
            }
            _ => panic!("Expected type: {:?}", ty.node)
        };

        assert_eq!(args.len(), 0);
        assert_eq!(body, None);

        assert_eq!(parents.len(), 1);
        let parent = parents.first().unwrap();
        match &parent.node {
            Node::Parent { ty, args } => {
                match &ty.node {
                    Node::Type { id, generics } => {
                        assert_eq!(id.node, Node::Id { lit: String::from("Exception") });
                        assert_eq!(generics.len(), 0);
                    }
                    _ => panic!("Expected type: {:?}", ty.node)
                }
                assert_eq!(args.len(), 1);
                let arg = args.first().unwrap();
                assert_eq!(arg.node, Node::Str { lit: String::from("Something went wrong"), expressions: vec![] })
            }
            _ => panic!("Expected parent: {:?}", parent.node)
        }
    }

    #[test]
    fn parse_class() -> ParseResult<()> {
        let source = resource_content(true, &["class"], "types.mamba");
        source.parse::<AST>().map(|_| ())
    }

    #[test]
    fn parse_imports_class() -> ParseResult<()> {
        let source = resource_content(true, &["class"], "import.mamba");
        source.parse::<AST>().map(|_| ())
    }

    #[test]
    fn single_line_class() {
        let source = String::from("class MyClass");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn two_classes_no_newline_after() {
        let source = String::from("class MyClass\nclass MyClass1");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn two_classes_newline_after() {
        let source = String::from("class MyClass\nclass MyClass1\n");
        source.parse::<AST>().unwrap();
    }

    #[test]
    fn class_with_single_line_body_no_newline() -> Result<(), ParseErr> {
        let source = "class MyClass\n    def var := 10";
        source.parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map(|_| ())
    }

    #[test]
    fn class_with_single_line_body_newline() -> Result<(), ParseErr> {
        let source = "class MyClass\n    def var := 10\n";
        source.parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map(|_| ())
    }

    #[test]
    fn class_with_body_class_right_after() -> Result<(), ParseErr> {
        let source = "class MyClass\n    def var := 10\nclass MyClass1\n";
        source.parse::<AST>()
            .map_err(|e| e.with_source(&Some(String::from(source)), &None))
            .map(|_| ())
    }

    #[test]
    fn top_lvl_class_access() {
        let source = resource_content(false, &["syntax"], "top_lvl_class_access.mamba");
        source.parse::<AST>().unwrap_err();
    }
}
