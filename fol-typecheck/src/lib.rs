//! Whole-program type checking for the `V1` FOL language subset.
//!
//! This crate is introduced in stages. The early foundation slices only provide
//! the workspace boundary and a small public API surface so later commits can
//! grow semantic types, typed results, and diagnostics incrementally.

pub mod builtins;
pub mod decls;
pub mod errors;
pub mod exprs;
pub mod model;
pub mod session;
pub mod types;

pub use builtins::BuiltinTypeIds;
pub use errors::{TypecheckError, TypecheckErrorKind};
pub use model::{
    TypedNode, TypedPackage, TypedProgram, TypedReference, TypedSourceUnit, TypedSymbol,
    TypedWorkspace,
};
pub use types::{
    BuiltinType, CheckedType, CheckedTypeId, DeclaredTypeKind, RoutineType, TypeTable,
};

pub type TypecheckResult<T> = Result<T, Vec<TypecheckError>>;

#[derive(Debug, Default)]
pub struct Typechecker;

impl Typechecker {
    pub fn new() -> Self {
        Self
    }

    pub fn check_resolved_program(
        &mut self,
        resolved: fol_resolver::ResolvedProgram,
    ) -> TypecheckResult<TypedProgram> {
        session::TypecheckSession::new().check_resolved_program(resolved)
    }

    pub fn check_resolved_workspace(
        &mut self,
        resolved: fol_resolver::ResolvedWorkspace,
    ) -> TypecheckResult<TypedWorkspace> {
        session::TypecheckSession::new().check_resolved_workspace(resolved)
    }
}

#[cfg(test)]
mod tests {
    use super::{TypecheckError, TypecheckErrorKind, Typechecker};
    use fol_parser::ast::SyntaxOrigin;
    use fol_resolver::resolve_package;
    use fol_stream::FileStream;

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

    #[test]
    fn typechecker_can_wrap_a_resolved_program_in_a_typed_shell() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open typecheck fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = fol_parser::ast::AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Typecheck fixture should parse");
        let resolved = resolve_package(syntax).expect("Typecheck fixture should resolve");

        let typed = Typechecker::new()
            .check_resolved_program(resolved)
            .expect("Typed shell should accept resolved programs");

        assert_eq!(typed.package_name(), "parser");
        assert_eq!(typed.type_table().len(), 6);
        assert_eq!(typed.resolved().source_units.len(), 1);
    }
}
