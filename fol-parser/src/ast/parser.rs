// AST Parser Implementation for FOL

use super::{
    AstNode, BinaryOperator, CharEncoding, ContainerType, FloatSize, FolType, FunOption, Generic,
    IntSize, Literal, LoopCondition, Parameter, RollingBinding, TypeDefinition, TypeOption,
    UnaryOperator, UseOption, VarOption, WhenCase,
};
use fol_lexer::token::{BUILDIN, KEYWORD, LITERAL, OPERATOR, SYMBOL, VOID};
use fol_types::*;
use std::cell::RefCell;
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

/// Simple AST Parser for FOL
pub struct AstParser {
    routine_return_types: RefCell<HashMap<String, FolType>>,
}

impl Default for AstParser {
    fn default() -> Self {
        Self::new()
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
#[path = "parser_parts/expression_atoms_and_report_validation.rs"]
mod expression_atoms_and_report_validation;
#[path = "parser_parts/expression_parsers.rs"]
mod expression_parsers;
#[path = "parser_parts/rolling_expression_parsers.rs"]
mod rolling_expression_parsers;
#[path = "parser_parts/grouped_binding_parsers.rs"]
mod grouped_binding_parsers;
#[path = "parser_parts/program_and_bindings.rs"]
mod program_and_bindings;
#[path = "parser_parts/routine_headers_and_type_lowering.rs"]
mod routine_headers_and_type_lowering;
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
