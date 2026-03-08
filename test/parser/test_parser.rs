// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{
    AstNode, AstParser, CharEncoding, FloatSize, FolType, IntSize, Literal, Parameter, ParseError,
    TypeDefinition,
};
use fol_stream::FileStream;

#[cfg(test)]
#[path = "test_parser_parts/assignments_and_logical_expressions.rs"]
mod assignments_and_logical_expressions;
#[cfg(test)]
#[path = "test_parser_parts/basic_declarations.rs"]
mod basic_declarations;
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
#[path = "test_parser_parts/comparison_keyword_expressions.rs"]
mod comparison_keyword_expressions;
#[cfg(test)]
#[path = "test_parser_parts/container_and_unary_expressions.rs"]
mod container_and_unary_expressions;
#[cfg(test)]
#[path = "test_parser_parts/custom_error_report_validation.rs"]
mod custom_error_report_validation;
#[cfg(test)]
#[path = "test_parser_parts/definition_declarations.rs"]
mod definition_declarations;
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
#[path = "test_parser_parts/segment_declarations.rs"]
mod segment_declarations;
#[cfg(test)]
#[path = "test_parser_parts/single_quoted_names.rs"]
mod single_quoted_names;
#[cfg(test)]
#[path = "test_parser_parts/standard_declarations.rs"]
mod standard_declarations;
#[cfg(test)]
#[path = "test_parser_parts/test_block_declarations.rs"]
mod test_block_declarations;
#[cfg(test)]
#[path = "test_parser_parts/routine_error_types.rs"]
mod routine_error_types;
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
#[path = "test_parser_parts/routine_headers_and_when_forms.rs"]
mod routine_headers_and_when_forms;
#[cfg(test)]
#[path = "test_parser_parts/top_level_control_flow_and_calls.rs"]
mod top_level_control_flow_and_calls;
#[cfg(test)]
#[path = "test_parser_parts/type_definition_validation.rs"]
mod type_definition_validation;
#[cfg(test)]
#[path = "test_parser_parts/type_forms_and_function_decls.rs"]
mod type_forms_and_function_decls;
#[cfg(test)]
#[path = "test_parser_parts/typed_iteration_binders.rs"]
mod typed_iteration_binders;
#[cfg(test)]
#[path = "test_parser_parts/unary_and_call_argument_errors.rs"]
mod unary_and_call_argument_errors;
#[cfg(test)]
#[path = "test_parser_parts/unmatched_paren_errors.rs"]
mod unmatched_paren_errors;
