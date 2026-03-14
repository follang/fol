//! Whole-program type checking for the `V1` FOL language subset.
//!
//! This crate is introduced in stages. The early foundation slices only provide
//! the workspace boundary and a small public API surface so later commits can
//! grow semantic types, typed results, and diagnostics incrementally.

pub mod builtins;
pub mod errors;
pub mod types;

pub use builtins::BuiltinTypeIds;
pub use errors::{TypecheckError, TypecheckErrorKind};
pub use types::{BuiltinType, CheckedType, CheckedTypeId, TypeTable};

pub type TypecheckResult<T> = Result<T, Vec<TypecheckError>>;

#[derive(Debug, Default)]
pub struct Typechecker;

impl Typechecker {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{TypecheckError, TypecheckErrorKind, Typechecker};
    use fol_parser::ast::SyntaxOrigin;

    #[test]
    fn typechecker_foundation_can_be_constructed() {
        let _ = Typechecker::new();
    }

    #[test]
    fn typechecker_foundation_exposes_typecheck_error_surface() {
        let error = TypecheckError::with_origin(
            TypecheckErrorKind::Unsupported,
            "generics are not part of the V1 typecheck milestone",
            SyntaxOrigin {
                file: Some("pkg/main.fol".to_string()),
                line: 3,
                column: 5,
                length: 7,
            },
        );

        assert_eq!(error.kind(), TypecheckErrorKind::Unsupported);
        assert_eq!(
            error.diagnostic_location()
                .expect("Typecheck error should expose its syntax origin")
                .line,
            3
        );
    }
}
