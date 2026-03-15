use fol_diagnostics::{Diagnostic, DiagnosticReport, ToDiagnostic};
use fol_lower::LoweringError;
use fol_package::PackageError;
use fol_parser::ast::ParseError;
use fol_resolver::ResolverError;
use fol_typecheck::TypecheckError;
use fol_types::Glitch;

pub fn add_compiler_glitch(report: &mut DiagnosticReport, error: &dyn Glitch) {
    if let Some(diagnostic) = lower_compiler_glitch(error) {
        report.add_diagnostic(diagnostic);
    } else {
        report.add_error(error, None);
    }
}

fn lower_compiler_glitch(error: &dyn Glitch) -> Option<Diagnostic> {
    error
        .as_any()
        .downcast_ref::<ParseError>()
        .map(ToDiagnostic::to_diagnostic)
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<PackageError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<ResolverError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<TypecheckError>()
                .map(ToDiagnostic::to_diagnostic)
        })
        .or_else(|| {
            error
                .as_any()
                .downcast_ref::<LoweringError>()
                .map(ToDiagnostic::to_diagnostic)
        })
}

#[cfg(test)]
mod tests {
    use super::{add_compiler_glitch, lower_compiler_glitch};
    use fol_diagnostics::DiagnosticReport;
    use fol_package::{PackageError, PackageErrorKind};
    use fol_parser::ast::AstParser;
    use fol_resolver::{ResolverError, ResolverErrorKind};
    use fol_stream::FileStream;
    use fol_lower::{LoweringError, LoweringErrorKind};
    use fol_typecheck::{TypecheckError, TypecheckErrorKind};
    use fol_types::{BasicError, Glitch};
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn lower_compiler_glitch_maps_parse_package_and_resolver_errors() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be stable enough for temp file names")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fol_compiler_diagnostics_{stamp}.fol"));
        fs::write(&path, "run(1, 2)\n").expect("parser fixture should be writable");
        let mut stream = FileStream::from_file(
            path.to_str()
                .expect("parser fixture path should be valid UTF-8"),
        )
        .expect("parser fixture should open");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let parse = parser
            .parse_package(&mut lexer)
            .expect_err("parser fixture should fail")
            .into_iter()
            .next()
            .expect("parser fixture should produce one error");
        let package = PackageError::new(PackageErrorKind::InvalidInput, "package issue");
        let resolver = ResolverError::new(ResolverErrorKind::Unsupported, "resolver issue");
        let typecheck = TypecheckError::new(TypecheckErrorKind::Unsupported, "typecheck issue");
        let lowering = LoweringError::with_kind(LoweringErrorKind::Unsupported, "lowering issue");

        assert_eq!(
            lower_compiler_glitch(parse.as_ref())
                .expect("parse errors should lower")
                .code
                .as_str(),
            "P1002"
        );
        assert_eq!(
            lower_compiler_glitch(&package as &dyn Glitch)
                .expect("package errors should lower")
                .code
                .as_str(),
            "K1001"
        );
        assert_eq!(
            lower_compiler_glitch(&resolver as &dyn Glitch)
                .expect("resolver errors should lower")
                .code
                .as_str(),
            "R1002"
        );
        assert_eq!(
            lower_compiler_glitch(&typecheck as &dyn Glitch)
                .expect("typecheck errors should lower")
                .code
                .as_str(),
            "T1002"
        );
        assert_eq!(
            lower_compiler_glitch(&lowering as &dyn Glitch)
                .expect("lowering errors should lower")
                .code
                .as_str(),
            "L1001"
        );

        let _ = fs::remove_file(path);
    }

    #[test]
    fn add_compiler_glitch_falls_back_for_generic_errors() {
        let mut report = DiagnosticReport::new();
        let error = BasicError {
            message: "generic fallback".to_string(),
        };

        add_compiler_glitch(&mut report, &error);

        assert_eq!(report.error_count, 1);
        assert_eq!(report.diagnostics.len(), 1);
        assert_eq!(report.diagnostics[0].code.as_str(), "E0000");
    }
}
