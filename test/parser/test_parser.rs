// Parser tests - to be expanded when full parser is implemented

use fol_lexer::lexer::stage3::Elements;
use fol_lexer::token::KEYWORD;
use fol_parser::ast::{
    AstNode, AstParser, BindingPattern, CharEncoding, CommentKind, FloatSize, FolType, InquiryTarget,
    IntSize, Literal, Parameter, ParseError, ParsedPackage, ParsedSourceUnit, ParsedTopLevel,
    QualifiedPath, SyntaxOrigin, TypeDefinition,
};
use fol_stream::FileStream;

fn inquiry_target_key(target: &InquiryTarget) -> String {
    target.duplicate_key()
}

fn inquiry_target_is(target: &InquiryTarget, expected: &str) -> bool {
    inquiry_target_key(target) == expected
}

fn program_root_nodes<'a>(declarations: &'a [AstNode]) -> Vec<&'a AstNode> {
    declarations.iter().collect()
}

fn only_root_routine_body_nodes<'a>(declarations: &'a [AstNode]) -> Vec<&'a AstNode> {
    assert_eq!(
        declarations.len(),
        1,
        "Expected a single root node before inspecting a routine body"
    );
    declarations[0]
        .routine_body()
        .expect("Expected the only root node to be a routine declaration")
        .iter()
        .collect()
}

fn fol_type_has_qualified_segments(typ: &FolType, expected: &[&str]) -> bool {
    matches!(
        typ,
        FolType::QualifiedNamed { path }
            if path.segments
                == expected
                    .iter()
                    .map(|segment| segment.to_string())
                    .collect::<Vec<_>>()
    )
}

fn fol_type_named_text_is(typ: &FolType, expected: &str) -> bool {
    typ.named_text().as_deref() == Some(expected)
}

fn use_path_segments_text(
    path_segments: &[fol_parser::ast::UsePathSegment],
) -> String {
    let mut path = String::new();

    for segment in path_segments {
        if let Some(separator) = &segment.separator {
            path.push_str(match separator {
                fol_parser::ast::UsePathSeparator::Slash => "/",
                fol_parser::ast::UsePathSeparator::DoubleColon => "::",
            });
        }
        path.push_str(&segment.spelling);
    }

    path
}

fn use_decl_path_text(node: &AstNode) -> Option<String> {
    match node {
        AstNode::UseDecl { path_segments, .. } => Some(use_path_segments_text(path_segments)),
        _ => None,
    }
}

fn use_decl_matches_path(node: &AstNode, expected_name: &str, expected_path: &str) -> bool {
    matches!(node, AstNode::UseDecl { name, .. } if name == expected_name)
        && use_decl_path_text(node).as_deref() == Some(expected_path)
}

fn parse_package_from_file(path: &str) -> ParsedPackage {
    let mut file_stream = FileStream::from_file(path).expect("Should read parser package test file");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse_package(&mut lexer)
        .expect("Parser should produce a parsed package")
}

fn parse_decl_package_from_file(path: &str) -> ParsedPackage {
    let mut file_stream =
        FileStream::from_file(path).expect("Should read parser declaration-package test file");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse_decl_package(&mut lexer)
        .expect("Parser should produce a declaration-only parsed package")
}

fn parse_decl_package_from_folder(path: &str) -> ParsedPackage {
    let mut file_stream = FileStream::from_folder(path)
        .expect("Should read parser declaration-package test folder");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse_decl_package(&mut lexer)
        .expect("Parser should produce a declaration-only parsed package")
}

fn parse_package_from_folder(path: &str) -> ParsedPackage {
    let mut file_stream =
        FileStream::from_folder(path).expect("Should read parser package test folder");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    parser
        .parse_package(&mut lexer)
        .expect("Parser should produce a parsed package")
}

fn source_unit_nodes<'a>(source_unit: &'a ParsedSourceUnit) -> Vec<&'a AstNode> {
    source_unit.items.iter().map(|item| &item.node).collect()
}

fn parsed_top_level_origin<'a>(
    package: &'a ParsedPackage,
    item: &ParsedTopLevel,
) -> &'a SyntaxOrigin {
    package
        .syntax_index
        .origin(item.node_id)
        .expect("Parsed top-level node should have a syntax origin")
}

fn ast_node_origin<'a>(package: &'a ParsedPackage, node: &AstNode) -> &'a SyntaxOrigin {
    let syntax_id = node
        .syntax_id()
        .expect("AST node should retain a syntax id in parsed-package mode");
    package
        .syntax_index
        .origin(syntax_id)
        .expect("AST node syntax id should resolve in the package syntax index")
}

fn qualified_path_origin<'a>(
    package: &'a ParsedPackage,
    path: &QualifiedPath,
) -> &'a SyntaxOrigin {
    let syntax_id = path
        .syntax_id()
        .expect("Qualified path should retain a syntax id in parsed-package mode");
    package
        .syntax_index
        .origin(syntax_id)
        .expect("Qualified path syntax id should resolve in the package syntax index")
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
#[path = "test_parser_parts/anonymous_logical_diagnostics.rs"]
mod anonymous_logical_diagnostics;
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
#[path = "test_parser_parts/book_processor_examples.rs"]
mod book_processor_examples;
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
#[path = "test_parser_parts/chaining_sugar.rs"]
mod chaining_sugar;
#[cfg(test)]
#[path = "test_parser_parts/channel_access_expressions.rs"]
mod channel_access_expressions;
#[cfg(test)]
#[path = "test_parser_parts/availability_access_expressions.rs"]
mod availability_access_expressions;
#[cfg(test)]
#[path = "test_parser_parts/access_pattern_captures.rs"]
mod access_pattern_captures;
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
#[path = "test_parser_parts/comment_nodes.rs"]
mod comment_nodes;
#[cfg(test)]
#[path = "test_parser_parts/source_origins.rs"]
mod source_origins;
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
#[path = "test_parser_parts/definition_declarations.rs"]
mod definition_declarations;
#[cfg(test)]
#[path = "test_parser_parts/definition_meta_declarations.rs"]
mod definition_meta_declarations;
#[cfg(test)]
#[path = "test_parser_parts/declaration_option_separators.rs"]
mod declaration_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/destructuring_bindings.rs"]
mod destructuring_bindings;
#[cfg(test)]
#[path = "test_parser_parts/flow_bodies.rs"]
mod flow_bodies;
#[cfg(test)]
#[path = "test_parser_parts/control_flow_contexts.rs"]
mod control_flow_contexts;
#[cfg(test)]
#[path = "test_parser_parts/file_boundaries.rs"]
mod file_boundaries;
#[cfg(test)]
#[path = "test_parser_parts/package_source_units.rs"]
mod package_source_units;
#[cfg(test)]
#[path = "test_parser_parts/package_root_contract.rs"]
mod package_root_contract;
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
#[path = "test_parser_parts/illegal_token_contexts.rs"]
mod illegal_token_contexts;
#[cfg(test)]
#[path = "test_parser_parts/inquiry_clauses.rs"]
mod inquiry_clauses;
#[cfg(test)]
#[path = "test_parser_parts/logical_identity_ast.rs"]
mod logical_identity_ast;
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
#[path = "test_parser_parts/literal_lowering.rs"]
mod literal_lowering;
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
#[path = "test_parser_parts/mutex_parameters.rs"]
mod mutex_parameters;
#[cfg(test)]
#[path = "test_parser_parts/matching_expressions.rs"]
mod matching_expressions;
#[cfg(test)]
#[path = "test_parser_parts/named_generics.rs"]
mod named_generics;
#[cfg(test)]
#[path = "test_parser_parts/named_function_types.rs"]
mod named_function_types;
#[cfg(test)]
#[path = "test_parser_parts/name_path_shapes.rs"]
mod name_path_shapes;
#[cfg(test)]
#[path = "test_parser_parts/nil_literal_expressions.rs"]
mod nil_literal_expressions;
#[cfg(test)]
#[path = "test_parser_parts/object_type_markers.rs"]
mod object_type_markers;
#[cfg(test)]
#[path = "test_parser_parts/record_initializers.rs"]
mod record_initializers;
#[cfg(test)]
#[path = "test_parser_parts/report_syntax.rs"]
mod report_syntax;
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
#[path = "test_parser_parts/quoted_bindings.rs"]
mod quoted_bindings;
#[cfg(test)]
#[path = "test_parser_parts/quoted_binding_type_hints.rs"]
mod quoted_binding_type_hints;
#[cfg(test)]
#[path = "test_parser_parts/quoted_alias_names.rs"]
mod quoted_alias_names;
#[cfg(test)]
#[path = "test_parser_parts/qualified_path_nodes.rs"]
mod qualified_path_nodes;
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
#[path = "test_parser_parts/select_statements.rs"]
mod select_statements;
#[cfg(test)]
#[path = "test_parser_parts/spawn_expressions.rs"]
mod spawn_expressions;
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
#[path = "test_parser_parts/top_level_ast_shapes.rs"]
mod top_level_ast_shapes;
#[cfg(test)]
#[path = "test_parser_parts/statement_expression_boundaries.rs"]
mod statement_expression_boundaries;
#[cfg(test)]
#[path = "test_parser_parts/parser_diagnostic_consistency.rs"]
mod parser_diagnostic_consistency;
#[cfg(test)]
#[path = "test_parser_parts/type_definition_validation.rs"]
mod type_definition_validation;
#[cfg(test)]
#[path = "test_parser_parts/type_option_separators.rs"]
mod type_option_separators;
#[cfg(test)]
#[path = "test_parser_parts/template_calls.rs"]
mod template_calls;
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
