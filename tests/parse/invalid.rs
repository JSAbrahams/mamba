use mamba::parse::ast::AST;
use test_case::test_case;
use tests_util::resource_content;

use mamba::parse::result::ParseResult;

#[test_case("assign_and_while"=> matches Err(_))]
#[test_case("top_lvl_class_access"=> matches Err(_))]
#[test_case("type_annotation_in_tuple"=> matches Err(_))]
#[test_case("pure_variable_def"=> matches Err(_))]
#[test_case("class_parent_bad_token"=> matches Err(_))]
#[test_case("class_parent_arg_bad_token"=> matches Err(_))]
fn syntax(file_name: &str) -> ParseResult<()> {
    let file_name = format!("{file_name}.mamba");
    let source = resource_content(false, &["syntax"], &file_name).unwrap();

    source.parse::<AST>().map(|_| ())
}
