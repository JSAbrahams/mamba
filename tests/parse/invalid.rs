use std::path::PathBuf;

use mamba::parse::ast::AST;
use test_case::test_case;
use tests_util::resource_content;

use mamba::common::result::WithSource;
use mamba::parse::result::ParseResult;

#[test_case("assign_and_while"=> matches Err(_))]
#[test_case("top_lvl_class_access"=> matches Err(_))]
#[test_case("type_annotation_in_tuple"=> matches Err(_))]
#[test_case("pure_variable_def"=> matches Err(_))]
#[test_case("fin_without_def"=> matches Err(_))]
#[test_case("class_parent_bad_token"=> matches Err(_))]
#[test_case("class_parent_arg_bad_token"=> matches Err(_))]
#[test_case("unrecognized_character"=> matches Err(_))]
fn syntax(file_name: &str) -> ParseResult<()> {
    let file_name = format!("{file_name}.mamba");
    let source = resource_content(false, &["syntax"], &file_name).unwrap();

    // expect a parse error; print it, exercising Display/with_source
    source.parse::<AST>().map(|_| ()).map_err(|e| {
        let path = PathBuf::new().join("syntax").join(&file_name);
        let e = e.with_source(&Some(source), &Some(path));
        println!("{e}");
        Box::from(e)
    })
}
