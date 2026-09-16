use std::path::PathBuf;

use mamba::parse::ast::AST;
use test_case::test_case;
use tests_util::resource_content;

use mamba::parse::result::ParseResult;

#[test_case("error", "with")]
#[test_case("error", "handle")]
#[test_case("error", "raise")]
#[test_case("function", "definition")]
#[test_case("definition", "forward")]
#[test_case("function", "calls")]
#[test_case("control_flow", "while")]
#[test_case("control_flow", "for_statements")]
#[test_case("control_flow", "if")]
#[test_case("control_flow", "match_guard")]
#[test_case("control_flow", "match_first_arm_wins")]
#[test_case("control_flow", "match_guard_falls_through")]
#[test_case("control_flow", "match_guard_calls_function")]
#[test_case("control_flow", "match_guard_on_literal_pattern")]
#[test_case("control_flow", "match_guard_on_wildcard")]
#[test_case("control_flow", "match_guard_reads_outer")]
#[test_case("control_flow", "match_compound_guard")]
#[test_case("control_flow", "match_literal_str")]
#[test_case("control_flow", "match_negative_int_literal")]
#[test_case("control_flow", "match_nested_tuple_literals")]
#[test_case("control_flow", "match_in_for_body")]
#[test_case("control_flow", "match_wildcard_only")]
#[test_case("control_flow", "match_single_irrefutable_arm")]
#[test_case("control_flow", "match_subject_is_call")]
#[test_case("control_flow", "match_arm_body_block")]
#[test_case("control_flow", "match_shadow_restores_outer")]
#[test_case("control_flow", "match_binding_read_in_arm")]
#[test_case("control_flow", "match_guard_not_evaluated_when_pattern_fails")]
#[test_case("control_flow", "match_tuple_pattern")]
#[test_case("control_flow", "match_tuple_underscore")]
#[test_case("control_flow", "match_exhaustive_bool")]
#[test_case("control_flow", "match_stmt")]
#[test_case("collection", "list")]
#[test_case("collection", "list_assign")]
#[test_case("collection", "list_round_bracket_index")]
#[test_case("collection", "dictionary")]
#[test_case("collection", "dictionary_assign")]
#[test_case("collection", "dictionary_round_bracket_index")]
#[test_case("collection", "set")]
#[test_case("collection", "tuple")]
#[test_case("class", "types")]
#[test_case("class", "import")]
#[test_case("class", "trait_and_type")]
#[test_case("readme_example", "ackermann")]
#[test_case("readme_example", "builtin_trait" => ignore["@ decorator syntax and meta keyword not implemented"])]
#[test_case("readme_example", "class_associated_function")]
#[test_case("readme_example", "class_construction")]
#[test_case("readme_example", "class_derived_field")]
#[test_case("readme_example", "class_new_enforces")]
#[test_case("readme_example", "pure_construction")]
#[test_case("readme_example", "pure_construction_methods")]
#[test_case("readme_example", "class_with_constants")]
#[test_case("readme_example", "class")]
#[test_case("readme_example", "error_handling_as_expression")]
#[test_case("readme_example", "error_handling_desyntax_sugared" => ignore["a case cannot bind an error's constructor arguments yet (err: Type(args))"])]
#[test_case("readme_example", "error_handling_early_exit")]
#[test_case("readme_example", "error_handling_result_type" => ignore["explicit Result[...] type not implemented, and a nested generic argument such as Union[...] does not parse"])]
#[test_case("readme_example", "error_handling_handle_subset")]
#[test_case("readme_example", "error_handling" => ignore["match case cannot bind an error's constructor arguments yet (err: Type(args))"])]
#[test_case("readme_example", "factorial_dynamic")]
#[test_case("readme_example", "factorial")]
#[test_case("readme_example", "impl_trait" => ignore["`def <Trait> for <Class> where ...` external-implementation syntax and meta modifier not implemented"])]
#[test_case("readme_example", "list_shorthand")]
#[test_case("readme_example", "lists")]
#[test_case("readme_example", "mutability")]
#[test_case("readme_example", "pure_functions")]
#[test_case("readme_example", "sets_maps")]
#[test_case("readme_example", "total_functions" => ignore["`total` keyword not implemented"])]
#[test_case("readme_example", "trait_meta" => ignore["meta modifier on trait methods not implemented"])]
#[test_case("readme_example", "trait_inheritance" => ignore["composing multiple parent traits not implemented"])]
#[test_case("readme_example", "traits" => ignore["generics on traits and external-implementation syntax not implemented"])]
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
