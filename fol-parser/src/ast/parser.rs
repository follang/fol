// AST Parser Implementation for FOL

use super::{
    AstNode, BinaryOperator, CharEncoding, ContainerType, DeclOption, EntryVariantMeta,
    FloatSize, FolType, FunOption, Generic, InquiryTarget, IntSize, Literal, LoopCondition,
    Parameter, QualifiedPath, RecordFieldMeta, RollingBinding, StandardKind, TypeDefinition, TypeOption,
    UnaryOperator, UseOption, VarOption, WhenCase,
};
use fol_lexer::token::{BUILDIN, KEYWORD, LITERAL, OPERATOR, SYMBOL, VOID};
use fol_types::*;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::fmt;

#[derive(Debug, Clone)]
pub struct ParseError {
    message: String,
    file: Option<String>,
    line: usize,
    column: usize,
    length: usize,
}

impl ParseError {
    pub fn from_token(token: &fol_lexer::lexer::stage3::element::Element, message: String) -> Self {
        let loc = token.loc();
        Self {
            message,
            file: loc.source().map(|src| src.path(true)),
            line: loc.row(),
            column: loc.col(),
            length: loc.len(),
        }
    }

    pub fn file(&self) -> Option<String> {
        self.file.clone()
    }

    pub fn line(&self) -> usize {
        self.line
    }

    pub fn column(&self) -> usize {
        self.column
    }

    pub fn length(&self) -> usize {
        self.length
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ParseError {}

impl Glitch for ParseError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

pub(super) struct ParseDepthGuard<'a> {
    depth: &'a Cell<usize>,
}

impl Drop for ParseDepthGuard<'_> {
    fn drop(&mut self) {
        let current = self.depth.get();
        debug_assert!(current > 0, "parser context depth underflow");
        self.depth.set(current.saturating_sub(1));
    }
}

/// Simple AST Parser for FOL
pub struct AstParser {
    routine_depth: Cell<usize>,
    loop_depth: Cell<usize>,
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new()
    }
}

impl AstParser {
    fn enter_depth<'a>(&'a self, depth: &'a Cell<usize>) -> ParseDepthGuard<'a> {
        depth.set(depth.get() + 1);
        ParseDepthGuard { depth }
    }

    pub(super) fn enter_routine_context(&self) -> ParseDepthGuard<'_> {
        self.enter_depth(&self.routine_depth)
    }

    pub(super) fn is_inside_routine(&self) -> bool {
        self.routine_depth.get() > 0
    }

    pub(super) fn enter_loop_context(&self) -> ParseDepthGuard<'_> {
        self.enter_depth(&self.loop_depth)
    }

    pub(super) fn is_inside_loop(&self) -> bool {
        self.loop_depth.get() > 0
    }
}

pub(super) fn canonical_identifier_key(name: &str) -> String {
    name.chars()
        .filter(|ch| *ch != '_')
        .map(|ch| {
            if ch.is_ascii() {
                ch.to_ascii_lowercase()
            } else {
                ch
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::canonical_identifier_key;

    #[test]
    fn canonical_identifier_key_normalizes_ascii_case_and_underscores() {
        assert_eq!(canonical_identifier_key("Foo_Bar"), "foobar");
        assert_eq!(canonical_identifier_key("foo__bar"), "foobar");
        assert_eq!(canonical_identifier_key("MIXED_Case_Name"), "mixedcasename");
    }

    #[test]
    fn canonical_identifier_key_preserves_non_ascii_while_normalizing_ascii() {
        assert_eq!(canonical_identifier_key("Straße_Name"), "straßename");
        assert_eq!(canonical_identifier_key("Δelta_Name"), "Δeltaname");
    }
}

#[path = "parser_parts/binding_alternative_parsers.rs"]
mod binding_alternative_parsers;
#[path = "parser_parts/binding_option_parsers.rs"]
mod binding_option_parsers;
#[path = "parser_parts/binding_value_parsers.rs"]
mod binding_value_parsers;
#[path = "parser_parts/declaration_parsers.rs"]
mod declaration_parsers;
#[path = "parser_parts/routine_declaration_parsers.rs"]
mod routine_declaration_parsers;
#[path = "parser_parts/declaration_option_parsers.rs"]
mod declaration_option_parsers;
#[path = "parser_parts/type_definition_parsers.rs"]
mod type_definition_parsers;
#[path = "parser_parts/expression_atoms_and_literal_lowering.rs"]
mod expression_atoms_and_literal_lowering;
#[path = "parser_parts/access_expression_parsers.rs"]
mod access_expression_parsers;
#[path = "parser_parts/expression_parsers.rs"]
mod expression_parsers;
#[path = "parser_parts/pipe_expression_parsers.rs"]
mod pipe_expression_parsers;
#[path = "parser_parts/pipe_lambda_parsers.rs"]
mod pipe_lambda_parsers;
#[path = "parser_parts/rolling_expression_parsers.rs"]
mod rolling_expression_parsers;
#[path = "parser_parts/grouped_binding_parsers.rs"]
mod grouped_binding_parsers;
#[path = "parser_parts/grouped_type_parsers.rs"]
mod grouped_type_parsers;
#[path = "parser_parts/program_and_bindings.rs"]
mod program_and_bindings;
#[path = "parser_parts/routine_headers_and_type_lowering.rs"]
mod routine_headers_and_type_lowering;
#[path = "parser_parts/routine_capture_parsers.rs"]
mod routine_capture_parsers;
#[path = "parser_parts/routine_body_parsers.rs"]
mod routine_body_parsers;
#[path = "parser_parts/statement_parsers.rs"]
mod statement_parsers;
#[path = "parser_parts/type_references_and_blocks.rs"]
mod type_references_and_blocks;
#[path = "parser_parts/use_declaration_parsers.rs"]
mod use_declaration_parsers;
#[path = "parser_parts/use_option_parsers.rs"]
mod use_option_parsers;
#[path = "parser_parts/segment_declaration_parsers.rs"]
mod segment_declaration_parsers;
#[path = "parser_parts/implementation_declaration_parsers.rs"]
mod implementation_declaration_parsers;
#[path = "parser_parts/inquiry_clause_parsers.rs"]
mod inquiry_clause_parsers;
#[path = "parser_parts/flow_body_parsers.rs"]
mod flow_body_parsers;
#[path = "parser_parts/postfix_expression_parsers.rs"]
mod postfix_expression_parsers;
#[path = "parser_parts/primary_expression_parsers.rs"]
mod primary_expression_parsers;
#[path = "parser_parts/standard_declaration_parsers.rs"]
mod standard_declaration_parsers;
#[path = "parser_parts/special_type_parsers.rs"]
mod special_type_parsers;
#[path = "parser_parts/source_kind_type_parsers.rs"]
mod source_kind_type_parsers;
#[path = "parser_parts/test_type_parsers.rs"]
mod test_type_parsers;
