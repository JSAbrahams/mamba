use std::path::PathBuf;

use test_case::test_case;
use tests_util::resource_content;

use mamba::check::check_all;
use mamba::check::result::TypeResult;
use mamba::common::result::WithSource;
use mamba::parse::ast::AST;

#[test_case("access", "access_list_with_string" => matches Err(_))]
#[test_case("access", "access_set" => matches Err(_))]
#[test_case("access", "access_string_dict_with_int" => matches Err(_))]
#[test_case("access", "slice_begin_wrong_type" => matches Err(_))]
#[test_case("access", "empty_define_f" => matches Err(_))]
#[test_case("access", "empty_define_field" => matches Err(_))]
#[test_case("access", "empty_define_str" => matches Err(_))]
#[test_case("access", "slice_end_wrong_type" => matches Err(_))]
#[test_case("access", "slice_step_wrong_type" => matches Err(_))]
#[test_case("access", "access_int" => matches Err(_))]
#[test_case("call", "call_with_parent" => matches Err(_))]
#[test_case("call", "calls_wrong_primitive" => matches Err(_))]
#[test_case("class", "reassign_non_existent" => matches Err(_))]
#[test_case("class", "assign_to_non_existent_self" => matches Err(_))]
#[test_case("class", "reassign_wrong_type" => matches Err(_))]
#[test_case("class", "reassign_function" => matches Err(_))]
#[test_case("class", "access_field_wrong_type" => matches Err(_))]
#[test_case("class", "access_function_wrong_type" => matches Err(_))]
#[test_case("class", "assign_to_inner_inner_not_allowed" => matches Err(_))]
#[test_case("class", "assign_to_inner_not_allowed" => matches Err(_))]
#[test_case("class", "generic_unknown_type" => matches Err(_))]
#[test_case("class", "incompat_parent_generics" => matches Err(_))]
#[test_case("class", "no_generic_arg" => matches Err(_))]
#[test_case("class", "object_has_no_attribute_self" => matches Err(_))]
#[test_case("class", "parent_function_type" => matches Err(_))]
#[test_case("class", "parent_tuple_type" => matches Err(_))]
#[test_case("class", "bare_statement_in_body" => matches Err(_))]
#[test_case("class", "associated_function_undefined" => matches Err(_))]
#[test_case("class", "construct_outside_class" => matches Err(_))]
#[test_case("class", "construct_other_class" => matches Err(_))]
#[test_case("class", "field_without_value" => matches Err(_))]
#[test_case("class", "derived_field_passed" => matches Err(_))]
#[test_case("class", "init_not_a_thing" => matches Err(_))]
#[test_case("class", "pure_new_impure_field" => matches Err(_))]
// The marker's argument list stands for the class arguments, so it has to say how many there
// are: `()` none, `(_)` exactly one, `(..)` one or more. These are the mismatches.
#[test_case("class", "new_empty_list_with_class_arguments" => matches Err(_))]
#[test_case("class", "new_underscore_without_class_arguments" => matches Err(_))]
#[test_case("class", "new_underscore_with_two_class_arguments" => matches Err(_))]
#[test_case("class", "new_rest_without_class_arguments" => matches Err(_))]
#[test_case("class", "new_as_field" => matches Err(_))]
#[test_case("class", "same_parent_twice" => matches Err(_))]
#[test_case("class", "wrong_generic_type" => matches Err(_))]
#[test_case("collection", "dictionary_assume_not_optional" => ignore["type checker incorrectly assumes access always non-optional"])]
#[test_case("collection", "dictionary_in_fun_wrong_ret_ty" => matches Err(_))]
#[test_case("collection", "dictionary_not_sliceable" => matches Err(_))]
#[test_case("collection", "dictionary_use_value_as_other_type" => matches Err(_))]
#[test_case("collection", "dictionary_wrong_key_type" => matches Err(_))]
#[test_case("collection", "list_assign_wrong_type" => matches Err(_))]
#[test_case("collection", "list_assign_non_mut" => matches Err(_))]
#[test_case("collection", "tuple_assign_not_supported" => matches Err(_))]
#[test_case("collection", "round_bracket_index_wrong_type" => matches Err(_))]
#[test_case("collection", "round_bracket_index_write_wrong_type" => matches Err(_))]
#[test_case("collection", "round_bracket_index_wrong_arg_count" => matches Err(_))]
#[test_case("collection", "list_builder_illegal_op_cond" => matches Err(_))]
#[test_case("collection", "list_builder_illegal_op_not_bool" => matches Err(_))]
#[test_case("collection", "list_builder_nested_undefined_var" => matches Err(_))]
#[test_case("collection", "list_builder_with_cond_not_boolean" => matches Err(_))]
#[test_case("collection", "list_conflicting_collection_types" => matches Err(_))]
#[test_case("collection", "list_builder_with_undefined_var" => matches Err(_))]
#[test_case("collection", "list_builder_with_no_expr" => matches Err(_))]
#[test_case("collection", "set_behind_function_not_subscriptable" => matches Err(_))]
#[test_case("collection", "set_builder_illegal_op_cond" => matches Err(_))]
#[test_case("collection", "set_builder_illegal_op_not_bool" => matches Err(_))]
#[test_case("collection", "set_builder_nested_undefined_var" => matches Err(_))]
#[test_case("collection", "set_builder_with_cond_not_boolean" => matches Err(_))]
#[test_case("collection", "set_conflicting_collection_types" => matches Err(_))]
#[test_case("collection", "set_builder_with_undefined_var" => matches Err(_))]
#[test_case("collection", "set_builder_with_no_expr" => matches Err(_))]
#[test_case("collection", "set_no_get" => matches Err(_))]
#[test_case("collection", "set_not_subscriptable" => matches Err(_))]
#[test_case("control_flow", "access_match_arms_variable" => matches Err(_))]
#[test_case("control_flow", "float_and" => matches Err(_))]
#[test_case("control_flow", "for_non_iterable" => matches Err(_))]
#[test_case("control_flow", "for_over_union_one_not_minus" => matches Err(_))]
#[test_case("control_flow", "if_not_bool_union" => matches Err(_))]
#[test_case("control_flow", "if_not_boolean" => matches Err(_))]
#[test_case("control_flow", "not_integer" => matches Err(_))]
#[test_case("control_flow", "or_float" => matches Err(_))]
#[test_case("control_flow", "undefined_var_in_match_arm" => matches Err(_))]
#[test_case("control_flow", "variable_defined_in_then" => matches Err(_))]
#[test_case("control_flow", "variable_defined_in_else" => matches Err(_))]
#[test_case("definition", "argument_after_argument_with_default" => matches Err(_))]
#[test_case("definition", "assign_wrong_type" => matches Err(_))]
#[test_case("definition", "body_is_stmt" => matches Err(_))]
#[test_case("definition", "function_ret_in_class_not_super" => matches Err(_))]
#[test_case("definition", "function_raise_not_super" => matches Err(_))]
#[test_case("definition", "function_ret_not_super" => matches Err(_))]
#[test_case("definition", "if_else_assign_non_nullable" => matches Err(_))]
#[test_case("definition", "assign_to_function_call" => matches Err(_))]
#[test_case("definition", "assign_to_inner_non_mut" => matches Err(_))]
#[test_case("definition", "assign_to_inner_non_mut2" => matches Err(_))]
#[test_case("definition", "assign_to_inner_non_mut3" => matches Err(_))]
#[test_case("definition", "undefined_variable" => matches Err(_))]
#[test_case("definition", "nested_non_mut_field" => ignore["checker incorrectly allows assigning to fields of non-mutable types"])]
#[test_case("definition", "reassign_non_mut" => matches Err(_))]
#[test_case("definition", "non_mutable_in_call_chain" => matches Err(_))]
#[test_case("definition", "non_existent_type_annotation" => matches Err(_))]
#[test_case("definition", "raises_unmentioned_exception" => matches Err(_))]
#[test_case("definition", "raises_non_exception" => matches Err(_))]
#[test_case("definition", "reassign_non_mut_field" => ignore["checker incorrectly allows reassign to non-mutable fields"])]
#[test_case("definition", "tuple_modify_inner_mut" => matches Err(_))]
#[test_case("definition", "tuple_modify_inner_mut_annotated" => matches Err(_))]
#[test_case("definition", "tuple_nested_wrong_size" => matches Err(_))]
#[test_case("definition", "tuple_fun_arg" => matches Err(_))]
#[test_case("definition", "tuple_modify_mut" => matches Err(_))]
#[test_case("definition", "tuple_modify_mut_entire" => matches Err(_))]
#[test_case("definition", "tuple_assign_itself" => matches Err(_))]
#[test_case("definition", "wrong_size_tuple" => matches Err(_))]
#[test_case("definition", "wrong_size_tuple_2" => matches Err(_))]
#[test_case("definition", "list_not_a_tuple" => matches Err(_))]
#[test_case("error", "handle_only_id" => matches Err(_))]
#[test_case("error", "unhandled_exception" => matches Err(_))]
#[test_case("error", "using_old_resource_in_with" => matches Err(_))]
#[test_case("error", "with_wrong_type" => matches Err(_))]
#[test_case("error", "with_not_expression" => matches Err(_))]
#[test_case("function", "outside_class_with_self" => matches Err(_))]
#[test_case("function", "pure_calls_impure" => matches Err(_))]
#[test_case("function", "pure_method_calls_impure_method" => matches Err(_))]
#[test_case("function", "pure_method_calls_impure_function" => matches Err(_))]
#[test_case("function", "pure_calls_impure_method_on_arg" => matches Err(_))]
#[test_case("function", "pure_mut_self" => matches Err(_))]
#[test_case("function", "pure_prints" => matches Err(_))]
#[test_case("function", "pure_assigns_outer_mut" => matches Err(_))]
#[test_case("function", "pure_assigns_arg_field" => matches Err(_))]
#[test_case("function", "pure_reads_mut_field" => matches Err(_))]
#[test_case("function", "pure_reads_outer_mut" => matches Err(_))]
#[test_case("function", "self_type_outside_class" => matches Err(_))]
#[test_case("function", "incompatible_types" => matches Err(_))]
#[test_case("function", "no_enough_arg" => matches Err(_))]
#[test_case("function", "empty_return_outside_function" => matches Err(_))]
#[test_case("function", "not_enough_arg_with_default" => matches Err(_))]
#[test_case("function", "arg_no_type" => matches Err(_))]
#[test_case("function", "arg_default_not_literal" => matches Err(_))]
#[test_case("function", "as_statement" => matches Err(_))]
#[test_case("function", "return_illegal" => matches Err(_))]
#[test_case("function", "statement_as_param" => matches Err(_))]
#[test_case("function", "too_many_arg" => matches Err(_))]
#[test_case("function", "unexpected_pass" => matches Err(_))]
#[test_case("function", "unmentioned_exception" => matches Err(_))]
#[test_case("function", "wrong_exception" => matches Err(_))]
#[test_case("function", "function_with_stmt_body" => matches Err(_))]
#[test_case("function", "function_with_stmt_body_ret" => matches Err(_))]
#[test_case("function", "return_exp_expr" => matches Err(_))]
#[test_case("function", "wrong_return_type" => matches Err(_))]
#[test_case("function", "return_if_else_none_el" => matches Err(_))]
#[test_case("function", "return_if_else_none_then" => matches Err(_))]
#[test_case("function", "return_if_else_undefined_explicit" => matches Err(_))]
#[test_case("function", "call_mut_function" => matches Err(_))]
#[test_case("function", "call_mut_function_on_non_mut" => ignore["checker shouldn't allow mut methods on non-mut"])]
#[test_case("operation", "in_dict_wrong_ty" => matches Err(_))]
#[test_case("operation", "in_list_wrong_ty" => matches Err(_))]
#[test_case("operation", "in_set_wrong_ty" => matches Err(_))]
#[test_case("operation", "reassign_to_nullable" => matches Err(_))]
#[test_case("operation", "reassign_to_undefined" => matches Err(_))]
#[test_case("operation", "string_minus" => matches Err(_))]
#[test_case("operation", "undefined_field_fstring" => matches Err(_))]
// `..` is allowed by the grammar wherever an expression or an argument may appear, so that a
// misplaced one is explained rather than reported as a syntax error. These are the positions
// it reaches, and the only one it may survive is the argument list of a bodiless `new`.
#[test_case("rest", "in_function_args" => matches Err(_))]
#[test_case("rest", "in_default_argument" => matches Err(_))]
#[test_case("rest", "as_class_argument" => matches Err(_))]
#[test_case("rest", "in_tuple" => matches Err(_))]
#[test_case("rest", "in_list" => matches Err(_))]
#[test_case("rest", "in_set" => matches Err(_))]
#[test_case("rest", "in_call_args" => matches Err(_))]
#[test_case("rest", "on_its_own" => matches Err(_))]
#[test_case("rest", "as_condition" => matches Err(_))]
#[test_case("rest", "as_return_value" => matches Err(_))]
#[test_case("rest", "in_match_case_tuple" => matches Err(_))]
#[test_case("rest", "in_nested_match_case_tuple" => matches Err(_))]
#[test_case("rest", "as_whole_match_case" => matches Err(_))]
#[test_case("rest", "new_with_body" => matches Err(_))]
#[test_case("rest", "method_args" => matches Err(_))]
#[test_case("rest", "bodiless_function_other_than_new" => matches Err(_))]
#[test_case("rest", "underscore_in_function_args" => matches Err(_))]
#[test_case("rest", "underscore_in_method_args" => matches Err(_))]
#[test_case("rest", "underscore_as_class_argument" => matches Err(_))]
fn fail_check(input_dir: &str, file_name: &str) -> TypeResult<()> {
    let file_name = format!("{file_name}.mamba");
    let source = resource_content(false, &["type", input_dir], &file_name).unwrap();
    let path = PathBuf::new().join("type").join(input_dir).join(file_name);

    // except no parse error, but if we got one, print it.
    let ast = source
        .parse::<AST>()
        .map_err(|mut e| {
            e.source = Some(source.clone());
            e.path = Some(path.clone());

            println!("{e}");
            e
        })
        .unwrap();

    // expect error when type checking; print it, exercising Display/with_source
    check_all(&[ast]).map(|_| ()).map_err(|errs| {
        errs.into_iter()
            .map(|e| {
                let e = e.with_source(&Some(source.clone()), &Some(path.clone()));
                println!("{e}");
                e
            })
            .collect()
    })
}
