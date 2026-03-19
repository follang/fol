// FOL Parser - Clean AST-only implementation

pub mod ast;

// Re-export the AST parser
pub use ast::*;

/// Source kind names used by import declarations and tree-sitter.
pub const SOURCE_KIND_NAMES: &[&str] = &["loc", "std", "pkg"];

/// Container type names used by syntax and tree-sitter.
pub const CONTAINER_TYPE_NAMES: &[&str] = &["arr", "vec", "seq", "set", "map"];

/// Shell type names used by syntax and tree-sitter.
pub const SHELL_TYPE_NAMES: &[&str] = &["opt", "err"];
