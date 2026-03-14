use crate::{LoweredWorkspace, LoweringError, LoweringErrorKind, LoweringResult};

#[derive(Debug)]
pub struct LoweringSession {
    typed: fol_typecheck::TypedWorkspace,
}

impl LoweringSession {
    pub fn new(typed: fol_typecheck::TypedWorkspace) -> Self {
        Self { typed }
    }

    pub fn typed_workspace(&self) -> &fol_typecheck::TypedWorkspace {
        &self.typed
    }

    pub fn lower_workspace(self) -> LoweringResult<LoweredWorkspace> {
        Err(vec![LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "lowering session is not implemented yet",
        )])
    }
}

#[cfg(test)]
mod tests {
    use super::LoweringSession;
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_workspace;
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn lowering_session_keeps_typed_workspace_identity_and_size() {
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

        let session = LoweringSession::new(typed);

        assert_eq!(session.typed_workspace().entry_identity().display_name, "parser");
        assert_eq!(session.typed_workspace().package_count(), 1);
    }

    #[test]
    fn lowering_session_stub_reports_explicit_boundary_error() {
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

        let error = LoweringSession::new(typed)
            .lower_workspace()
            .expect_err("Session shell should still stop at the explicit stub");

        assert_eq!(error.len(), 1);
        assert_eq!(error[0].message(), "lowering session is not implemented yet");
    }
}
