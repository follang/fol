//! Re-exports the compiler-owned LSP diagnostic adapter from `fol_diagnostics::lsp`.
//!
//! The conversion contract (severity mapping, message format, location
//! indexing, dedup layers) is defined and tested in `fol_diagnostics::lsp`.
//! This module exists only to maintain the crate's public API surface.

pub use fol_diagnostics::lsp::{
    dedup_lsp_diagnostics, diagnostic_to_lsp, location_to_range, LspDiagnostic,
    LspDiagnosticRelatedInformation, LspDiagnosticSeverity, LspLocation, LspPosition, LspRange,
};
