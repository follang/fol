//! Lowering from typed `V1` FOL workspaces into a backend-oriented IR.

mod errors;
pub mod control;
pub mod decls;
pub mod exprs;
pub mod ids;
pub mod model;
pub mod session;
pub mod types;
mod verify;

use fol_resolver::{PackageIdentity, PackageSourceKind};

pub use control::{
    LoweredBlock, LoweredInstr, LoweredInstrKind, LoweredLocal, LoweredOperand, LoweredRoutine,
    LoweredTerminator,
};
pub use errors::{LoweringError, LoweringErrorKind};
pub use ids::{
    IdTable, LoweredBlockId, LoweredGlobalId, LoweredInstrId, LoweredLocalId, LoweredPackageId,
    LoweredRoutineId, LoweredTypeId,
};
pub use model::{
    LoweredEntryCandidate, LoweredExportMount, LoweredGlobal, LoweredPackage, LoweredSourceMap,
    LoweredSourceMapEntry, LoweredSourceSymbol, LoweredSourceUnit, LoweredSymbolOwnership, LoweredTypeDecl,
    LoweredTypeDeclKind, LoweredFieldLayout, LoweredVariantLayout, LoweredWorkspace,
};
pub use session::LoweringSession;
pub use types::{
    LoweredBuiltinType, LoweredRoutineType, LoweredType, LoweredTypeTable,
};

pub type LoweringResult<T> = Result<T, Vec<LoweringError>>;

#[derive(Debug, Default)]
pub struct Lowerer;

impl Lowerer {
    pub fn new() -> Self {
        Self
    }

    pub fn lower_typed_workspace(
        &mut self,
        typed: fol_typecheck::TypedWorkspace,
    ) -> LoweringResult<LoweredWorkspace> {
        LoweringSession::new(typed).lower_workspace()
    }

    pub fn lower_typed_program(
        &mut self,
        typed: fol_typecheck::TypedProgram,
    ) -> LoweringResult<LoweredWorkspace> {
        let identity = compatibility_identity_for_program(&typed);
        self.lower_typed_workspace(fol_typecheck::TypedWorkspace::single(identity, typed))
    }
}

fn compatibility_identity_for_program(typed: &fol_typecheck::TypedProgram) -> PackageIdentity {
    let canonical_root = typed
        .source_units()
        .first()
        .map(|source_unit| source_unit.path.clone())
        .unwrap_or_else(|| typed.package_name().to_string());

    PackageIdentity {
        source_kind: PackageSourceKind::Entry,
        canonical_root,
        display_name: typed.package_name().to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{LoweredWorkspace, Lowerer, LoweringError, LoweringErrorKind, LoweringResult};
    use fol_parser::ast::AstParser;
    use fol_resolver::{resolve_package, resolve_workspace};
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn lowering_api_exposes_constructor_and_result_alias() {
        let _ = Lowerer::new();

        let result: LoweringResult<()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn lowering_api_exposes_basic_error_surface() {
        let error = LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "lowering shell is not implemented yet",
        );
        assert_eq!(error.message(), "lowering shell is not implemented yet");
        assert_eq!(
            error.to_string(),
            "LoweringUnsupported: lowering shell is not implemented yet"
        );
    }

    #[test]
    fn lowering_smoke_accepts_typed_workspace_inputs() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");

        let lowered = Lowerer::new()
            .lower_typed_workspace(typed)
            .expect("Lowering shell should accept typed workspaces");

        assert_eq!(lowered.package_count(), 1);
    }

    #[test]
    fn lowering_compatibility_shim_wraps_typed_programs_as_single_entry_workspaces() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_package(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_program(resolved)
            .expect("Lowering fixture should typecheck");

        let lowered = Lowerer::new()
            .lower_typed_program(typed)
            .expect("Compatibility shim should lower through workspace lowering");

        assert_eq!(lowered.package_count(), 1);
        assert_eq!(lowered.entry_identity().display_name, "parser");
    }

    #[test]
    fn lowered_workspace_shell_is_constructible() {
        let _ = LoweredWorkspace;
    }
}
