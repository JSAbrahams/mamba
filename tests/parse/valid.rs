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
#[test_case("reamde_example", "builtin_trait" => ignore["rewrite parser"])]
#[test_case("reamde_example", "class_with_constants" => ignore["rewrite parser"])]
#[test_case("reamde_example", "class" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling_as_expression" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling_desyntax_sugared" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling_early_exit" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling_handle_subset" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling_recover" => ignore["rewrite parser"])]
#[test_case("reamde_example", "error_handling" => ignore["rewrite parser"])]
#[test_case("reamde_example", "factorial_dynamic" => ignore["rewrite parser"])]
#[test_case("reamde_example", "factorial" => ignore["rewrite parser"])]
#[test_case("reamde_example", "impl_trait" => ignore["rewrite parser"])]
#[test_case("reamde_example", "list_shorthand" => ignore["rewrite parser"])]
#[test_case("reamde_example", "lists" => ignore["rewrite parser"])]
#[test_case("reamde_example", "mutability" => ignore["rewrite parser"])]
#[test_case("reamde_example", "pure_functions" => ignore["rewrite parser"])]
#[test_case("reamde_example", "sets_maps" => ignore["rewrite parser"])]
#[test_case("reamde_example", "total_functions" => ignore["rewrite parser"])]
#[test_case("reamde_example", "trait_fin_meta" => ignore["rewrite parser"])]
#[test_case("reamde_example", "trait_inheritance" => ignore["rewrite parser"])]
#[test_case("reamde_example", "traits" => ignore["rewrite parser"])]
#[test_case("reamde_example", "type_refinement_call_site" => ignore["rewrite parser"])]
#[test_case("reamde_example", "type_refinement_in_fun" => ignore["rewrite parser"])]
#[test_case("reamde_example", "type_refinement_matrix" => ignore["rewrite parser"])]
#[test_case("reamde_example", "type_refinement_on_matrix" => ignore["rewrite parser"])]
#[test_case("reamde_example", "type_refinement_set" => ignore["rewrite parser"])]
fn syntax(input_dir: &str, file_name: &str) -> ParseResult<AST> {
    let file_name = format!("{file_name}.mamba");
    let source = resource_content(true, &[input_dir], &file_name).unwrap();

    // include path and source in error for faster debugging
    source.parse::<AST>().map_err(|mut e| {
        e.source = Some(source);
        e.path = Some(PathBuf::new().join(input_dir).join(file_name));
        eprintln!("{e}");
        e
    })
}
