// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{
    AstNode, AstParser, CharEncoding, FloatSize, FolType, IntSize, Literal, Parameter,
    ParseError, TypeDefinition,
};
use fol_stream::FileStream;

#[cfg(test)]
#[path = "test_parser_parts/basic_declarations.rs"]
mod basic_declarations;
#[cfg(test)]
#[path = "test_parser_parts/definition_declarations.rs"]
mod definition_declarations;
#[cfg(test)]
#[path = "test_parser_parts/type_definition_validation.rs"]
mod type_definition_validation;
#[cfg(test)]
#[path = "test_parser_parts/type_forms_and_function_decls.rs"]
mod type_forms_and_function_decls;
#[cfg(test)]
#[path = "test_parser_parts/routine_headers_and_when_forms.rs"]
mod routine_headers_and_when_forms;
#[cfg(test)]
#[path = "test_parser_parts/routine_error_types.rs"]
mod routine_error_types;
#[cfg(test)]
#[path = "test_parser_parts/report_call_resolution.rs"]
mod report_call_resolution;
#[cfg(test)]
#[path = "test_parser_parts/method_receivers_and_branching.rs"]
mod method_receivers_and_branching;
#[cfg(test)]
#[path = "test_parser_parts/top_level_control_flow_and_calls.rs"]
mod top_level_control_flow_and_calls;
#[cfg(test)]
#[path = "test_parser_parts/call_and_postfix_expressions.rs"]
mod call_and_postfix_expressions;
#[cfg(test)]
#[path = "test_parser_parts/loops_use_bindings_and_ranges.rs"]
mod loops_use_bindings_and_ranges;
#[cfg(test)]
#[path = "test_parser_parts/container_and_unary_expressions.rs"]
mod container_and_unary_expressions;
#[cfg(test)]
#[path = "test_parser_parts/assignments_and_logical_expressions.rs"]
mod assignments_and_logical_expressions;
#[cfg(test)]
#[path = "test_parser_parts/unary_and_call_argument_errors.rs"]
mod unary_and_call_argument_errors;
#[cfg(test)]
#[path = "test_parser_parts/unmatched_paren_errors.rs"]
mod unmatched_paren_errors;
