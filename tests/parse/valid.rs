use std::path::PathBuf;

use mamba::parse::ast::AST;
use test_case::test_case;
use tests_util::resource_content;

use mamba::parse::result::ParseResult;

#[test_case("error", "with")]
#[test_case("error", "handle")]
#[test_case("error", "raise")]
#[test_case("function", "definition")]
#[test_case("function", "calls")]
#[test_case("control_flow", "while")]
#[test_case("control_flow", "for_statements")]
#[test_case("control_flow", "if")]
#[test_case("control_flow", "match_stmt")]
#[test_case("collection", "list")]
#[test_case("collection", "dictionary")]
#[test_case("collection", "set")]
#[test_case("collection", "tuple")]
#[test_case("class", "types")]
#[test_case("class", "import")]
fn syntax(input_dir: &str, file_name: &str) -> ParseResult<()> {
    let file_name = format!("{file_name}.mamba");
    let source = resource_content(true, &[input_dir], &file_name).unwrap();

    // include path and source in error for faster debugging
    source.parse::<AST>().map(|_| ()).map_err(|mut e| {
        e.source = Some(source);
        e.path = Some(PathBuf::new().join(input_dir).join(file_name));
        e
    })
}
