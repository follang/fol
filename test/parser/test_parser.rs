// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{
    AstNode, AstParser, CharEncoding, FloatSize, FolType, InquiryTarget, IntSize, Literal,
    Parameter, ParseError, TypeDefinition,
};
use fol_stream::FileStream;

fn inquiry_target_key(target: &InquiryTarget) -> String {
    target.duplicate_key()
}

fn inquiry_target_is(target: &InquiryTarget, expected: &str) -> bool {
    inquiry_target_key(target) == expected
}

#[cfg(test)]
#[path = "test_parser_parts/alternative_routine_headers.rs"]
mod alternative_routine_headers;
#[cfg(test)]
#[path = "test_parser_parts/assignments_and_logical_expressions.rs"]
mod assignments_and_logical_expressions;
#[cfg(test)]
#[path = "test_parser_parts/anonymous_function_expressions.rs"]
mod anonymous_function_expressions;
#[cfg(test)]
#[path = "test_parser_parts/anonymous_capture_separators.rs"]
mod anonymous_capture_separators;
#[cfg(test)]
#[path = "test_parser_parts/anonymous_capture_lists.rs"]
mod anonymous_capture_lists;
#[cfg(test)]
#[path = "test_parser_parts/basic_declarations.rs"]
mod basic_declarations;
#[cfg(test)]
#[path = "test_parser_parts/book_spec_examples.rs"]
mod book_spec_examples;
#[cfg(test)]
#[path = "test_parser_parts/book_routine_examples.rs"]
mod book_routine_examples;
#[cfg(test)]
#[path = "test_parser_parts/call_argument_keywords.rs"]
mod call_argument_keywords;
#[cfg(test)]
#[path = "test_parser_parts/call_argument_unpacking.rs"]
mod call_argument_unpacking;
#[cfg(test)]
#[path = "test_parser_parts/availability_access_expressions.rs"]
mod availability_access_expressions;
#[cfg(test)]
#[path = "test_parser_parts/binding_alternatives.rs"]
mod binding_alternatives;
#[cfg(test)]
#[path = "test_parser_parts/binding_multi.rs"]
mod binding_multi;
#[cfg(test)]
#[path = "test_parser_parts/binding_options.rs"]
mod binding_options;
#[cfg(test)]
#[path = "test_parser_parts/call_and_postfix_expressions.rs"]
mod call_and_postfix_expressions;
#[cfg(test)]
#[path = "test_parser_parts/call_argument_separators.rs"]
mod call_argument_separators;
#[cfg(test)]
#[path = "test_parser_parts/comparison_keyword_expressions.rs"]
mod comparison_keyword_expressions;
#[cfg(test)]
#[path = "test_parser_parts/container_and_unary_expressions.rs"]
mod container_and_unary_expressions;
#[cfg(test)]
#[path = "test_parser_parts/container_literal_expressions.rs"]
mod container_literal_expressions;
#[cfg(test)]
#[path = "test_parser_parts/custom_error_report_validation.rs"]
mod custom_error_report_validation;
#[cfg(test)]
#[path = "test_parser_parts/definition_declarations.rs"]
mod definition_declarations;
#[cfg(test)]
#[path = "test_parser_parts/definition_meta_declarations.rs"]
mod definition_meta_declarations;
#[cfg(test)]
#[path = "test_parser_parts/declaration_option_separators.rs"]
mod declaration_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/flow_bodies.rs"]
mod flow_bodies;
#[cfg(test)]
#[path = "test_parser_parts/quoted_declaration_targets.rs"]
mod quoted_declaration_targets;
#[cfg(test)]
#[path = "test_parser_parts/quoted_function_type_refs.rs"]
mod quoted_function_type_refs;
#[cfg(test)]
#[path = "test_parser_parts/quoted_iteration_binders.rs"]
mod quoted_iteration_binders;
#[cfg(test)]
#[path = "test_parser_parts/implementation_declarations.rs"]
mod implementation_declarations;
#[cfg(test)]
#[path = "test_parser_parts/inquiry_clauses.rs"]
mod inquiry_clauses;
#[cfg(test)]
#[path = "test_parser_parts/inquiry_target_structure.rs"]
mod inquiry_target_structure;
#[cfg(test)]
#[path = "test_parser_parts/invoke_expressions.rs"]
mod invoke_expressions;
#[cfg(test)]
#[path = "test_parser_parts/keyword_named_routines.rs"]
mod keyword_named_routines;
#[cfg(test)]
#[path = "test_parser_parts/keyword_named_type_members.rs"]
mod keyword_named_type_members;
#[cfg(test)]
#[path = "test_parser_parts/keyword_named_types.rs"]
mod keyword_named_types;
#[cfg(test)]
#[path = "test_parser_parts/quoted_type_names.rs"]
mod quoted_type_names;
#[cfg(test)]
#[path = "test_parser_parts/lab_declarations.rs"]
mod lab_declarations;
#[cfg(test)]
#[path = "test_parser_parts/leading_dot_builtin_calls.rs"]
mod leading_dot_builtin_calls;
#[cfg(test)]
#[path = "test_parser_parts/keyword_named_bindings.rs"]
mod keyword_named_bindings;
#[cfg(test)]
#[path = "test_parser_parts/keyword_named_parameters.rs"]
mod keyword_named_parameters;
#[cfg(test)]
#[path = "test_parser_parts/loops_use_bindings_and_ranges.rs"]
mod loops_use_bindings_and_ranges;
#[cfg(test)]
#[path = "test_parser_parts/method_receivers_and_branching.rs"]
mod method_receivers_and_branching;
#[cfg(test)]
#[path = "test_parser_parts/named_generics.rs"]
mod named_generics;
#[cfg(test)]
#[path = "test_parser_parts/named_function_types.rs"]
mod named_function_types;
#[cfg(test)]
#[path = "test_parser_parts/nil_literal_expressions.rs"]
mod nil_literal_expressions;
#[cfg(test)]
#[path = "test_parser_parts/object_type_markers.rs"]
mod object_type_markers;
#[cfg(test)]
#[path = "test_parser_parts/report_call_resolution.rs"]
mod report_call_resolution;
#[cfg(test)]
#[path = "test_parser_parts/reference_keywords.rs"]
mod reference_keywords;
#[cfg(test)]
#[path = "test_parser_parts/range_expressions.rs"]
mod range_expressions;
#[cfg(test)]
#[path = "test_parser_parts/rolling_expressions.rs"]
mod rolling_expressions;
#[cfg(test)]
#[path = "test_parser_parts/quoted_routine_names.rs"]
mod quoted_routine_names;
#[cfg(test)]
#[path = "test_parser_parts/quoted_member_access.rs"]
mod quoted_member_access;
#[cfg(test)]
#[path = "test_parser_parts/quoted_call_expressions.rs"]
mod quoted_call_expressions;
#[cfg(test)]
#[path = "test_parser_parts/quoted_report_call_resolution.rs"]
mod quoted_report_call_resolution;
#[cfg(test)]
#[path = "test_parser_parts/quoted_bindings.rs"]
mod quoted_bindings;
#[cfg(test)]
#[path = "test_parser_parts/quoted_binding_type_hints.rs"]
mod quoted_binding_type_hints;
#[cfg(test)]
#[path = "test_parser_parts/quoted_alias_names.rs"]
mod quoted_alias_names;
#[cfg(test)]
#[path = "test_parser_parts/quoted_root_statements.rs"]
mod quoted_root_statements;
#[cfg(test)]
#[path = "test_parser_parts/quoted_parameters.rs"]
mod quoted_parameters;
#[cfg(test)]
#[path = "test_parser_parts/quoted_use_names.rs"]
mod quoted_use_names;
#[cfg(test)]
#[path = "test_parser_parts/pipe_expressions.rs"]
mod pipe_expressions;
#[cfg(test)]
#[path = "test_parser_parts/pipe_lambda_expressions.rs"]
mod pipe_lambda_expressions;
#[cfg(test)]
#[path = "test_parser_parts/pattern_access_expressions.rs"]
mod pattern_access_expressions;
#[cfg(test)]
#[path = "test_parser_parts/quoted_type_members.rs"]
mod quoted_type_members;
#[cfg(test)]
#[path = "test_parser_parts/quoted_receiver_types.rs"]
mod quoted_receiver_types;
#[cfg(test)]
#[path = "test_parser_parts/quoted_type_references.rs"]
mod quoted_type_references;
#[cfg(test)]
#[path = "test_parser_parts/qualified_quoted_type_references.rs"]
mod qualified_quoted_type_references;
#[cfg(test)]
#[path = "test_parser_parts/qualified_path_expressions.rs"]
mod qualified_path_expressions;
#[cfg(test)]
#[path = "test_parser_parts/segment_declarations.rs"]
mod segment_declarations;
#[cfg(test)]
#[path = "test_parser_parts/single_quoted_names.rs"]
mod single_quoted_names;
#[cfg(test)]
#[path = "test_parser_parts/slice_access_expressions.rs"]
mod slice_access_expressions;
#[cfg(test)]
#[path = "test_parser_parts/slice_assignment_targets.rs"]
mod slice_assignment_targets;
#[cfg(test)]
#[path = "test_parser_parts/standard_declarations.rs"]
mod standard_declarations;
#[cfg(test)]
#[path = "test_parser_parts/source_kind_types.rs"]
mod source_kind_types;
#[cfg(test)]
#[path = "test_parser_parts/special_type_references.rs"]
mod special_type_references;
#[cfg(test)]
#[path = "test_parser_parts/scalar_type_option_separators.rs"]
mod scalar_type_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/test_block_declarations.rs"]
mod test_block_declarations;
#[cfg(test)]
#[path = "test_parser_parts/routine_error_types.rs"]
mod routine_error_types;
#[cfg(test)]
#[path = "test_parser_parts/routine_flow_bodies.rs"]
mod routine_flow_bodies;
#[cfg(test)]
#[path = "test_parser_parts/routine_option_separators.rs"]
mod routine_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/routine_closure_captures.rs"]
mod routine_closure_captures;
#[cfg(test)]
#[path = "test_parser_parts/use_options.rs"]
mod use_options;
#[cfg(test)]
#[path = "test_parser_parts/use_paths.rs"]
mod use_paths;
#[cfg(test)]
#[path = "test_parser_parts/variadic_parameters.rs"]
mod variadic_parameters;
#[cfg(test)]
#[path = "test_parser_parts/while_loops.rs"]
mod while_loops;
#[cfg(test)]
#[path = "test_parser_parts/routine_headers_and_when_forms.rs"]
mod routine_headers_and_when_forms;
#[cfg(test)]
#[path = "test_parser_parts/top_level_control_flow_and_calls.rs"]
mod top_level_control_flow_and_calls;
#[cfg(test)]
#[path = "test_parser_parts/type_definition_validation.rs"]
mod type_definition_validation;
#[cfg(test)]
#[path = "test_parser_parts/type_option_separators.rs"]
mod type_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/type_argument_separators.rs"]
mod type_argument_separators;
#[cfg(test)]
#[path = "test_parser_parts/type_member_metadata.rs"]
mod type_member_metadata;
#[cfg(test)]
#[path = "test_parser_parts/type_member_bodies.rs"]
mod type_member_bodies;
#[cfg(test)]
#[path = "test_parser_parts/type_contracts.rs"]
mod type_contracts;
#[cfg(test)]
#[path = "test_parser_parts/type_group_declarations.rs"]
mod type_group_declarations;
#[cfg(test)]
#[path = "test_parser_parts/type_forms_and_function_decls.rs"]
mod type_forms_and_function_decls;
#[cfg(test)]
#[path = "test_parser_parts/type_reference_diagnostics.rs"]
mod type_reference_diagnostics;
#[cfg(test)]
#[path = "test_parser_parts/typed_iteration_binders.rs"]
mod typed_iteration_binders;
#[cfg(test)]
#[path = "test_parser_parts/unary_and_call_argument_errors.rs"]
mod unary_and_call_argument_errors;
#[cfg(test)]
#[path = "test_parser_parts/unmatched_paren_errors.rs"]
mod unmatched_paren_errors;
