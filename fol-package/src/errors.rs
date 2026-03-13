use fol_diagnostics::{
    Diagnostic, DiagnosticCode, DiagnosticLocation, ToDiagnostic, ToDiagnosticLocation,
};
use fol_parser::ast::SyntaxOrigin;
use fol_types::Glitch;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageErrorKind {
    InvalidInput,
    Unsupported,
    ImportCycle,
    Internal,
}

impl PackageErrorKind {
    fn label(self) -> &'static str {
        match self {
            Self::InvalidInput => "PackageInvalidInput",
            Self::Unsupported => "PackageUnsupported",
            Self::ImportCycle => "PackageImportCycle",
            Self::Internal => "PackageInternal",
        }
    }

    pub fn diagnostic_code(self) -> DiagnosticCode {
        match self {
            Self::InvalidInput => DiagnosticCode::new("K1001"),
            Self::Unsupported => DiagnosticCode::new("K1002"),
            Self::ImportCycle => DiagnosticCode::new("K1003"),
            Self::Internal => DiagnosticCode::new("K1099"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageError {
    kind: PackageErrorKind,
    message: String,
    origin: Option<SyntaxOrigin>,
}

impl PackageError {
    pub fn new(kind: PackageErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: None,
        }
    }

    pub fn with_origin(
        kind: PackageErrorKind,
        message: impl Into<String>,
        origin: SyntaxOrigin,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: Some(origin),
        }
    }

    pub fn kind(&self) -> PackageErrorKind {
        self.kind
    }

    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn origin(&self) -> Option<&SyntaxOrigin> {
        self.origin.as_ref()
    }

    pub fn diagnostic_location(&self) -> Option<DiagnosticLocation> {
        self.origin.as_ref().map(|origin| DiagnosticLocation {
            file: origin.file.clone(),
            line: origin.line,
            column: origin.column,
            length: Some(origin.length),
        })
    }
}

impl std::fmt::Display for PackageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.kind.label(), self.message)
    }
}

impl std::error::Error for PackageError {}

impl Glitch for PackageError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl ToDiagnosticLocation for PackageError {
    fn to_diagnostic_location(&self, file: Option<String>) -> DiagnosticLocation {
        if let Some(origin) = &self.origin {
            DiagnosticLocation {
                file: file.or_else(|| origin.file.clone()),
                line: origin.line,
                column: origin.column,
                length: Some(origin.length),
            }
        } else {
            DiagnosticLocation {
                file,
                line: 1,
                column: 1,
                length: None,
            }
        }
    }
}

impl ToDiagnostic for PackageError {
    fn to_diagnostic(&self) -> Diagnostic {
        let mut diagnostic = Diagnostic::error(self.kind.diagnostic_code(), self.to_string());
        if let Some(location) = self.diagnostic_location() {
            diagnostic = diagnostic.with_primary_label(location);
        }
        diagnostic
    }
}

#[cfg(test)]
mod tests {
    use super::{PackageError, PackageErrorKind};
    use fol_diagnostics::{DiagnosticCode, DiagnosticReport};
    use fol_parser::ast::SyntaxOrigin;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn package_error_formats_with_kind_prefix() {
        let error = PackageError::new(
            PackageErrorKind::Unsupported,
            "package fetching is not implemented yet",
        );

        assert_eq!(
            error.to_string(),
            "PackageUnsupported: package fetching is not implemented yet"
        );
    }

    #[test]
    fn package_error_exposes_diagnostic_location_from_origin() {
        let error = PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            "invalid package metadata",
            SyntaxOrigin {
                file: Some("pkg/package.yaml".to_string()),
                line: 4,
                column: 1,
                length: 4,
            },
        );

        let location = error
            .diagnostic_location()
            .expect("Package errors with syntax origins should expose diagnostic locations");

        assert_eq!(location.file.as_deref(), Some("pkg/package.yaml"));
        assert_eq!(location.line, 4);
        assert_eq!(location.column, 1);
        assert_eq!(location.length, Some(4));
    }

    #[test]
    fn package_error_integrates_with_diagnostic_report() {
        let error = PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            "duplicate package metadata field",
            SyntaxOrigin {
                file: Some("pkg/package.yaml".to_string()),
                line: 2,
                column: 1,
                length: 4,
            },
        );
        let mut report = DiagnosticReport::new();

        report.add_error(&error, error.diagnostic_location());

        assert!(report.has_errors());
        let rendered = report.output(fol_diagnostics::OutputFormat::Json);
        assert!(rendered.contains("PackageInvalidInput"));
        assert!(rendered.contains("pkg/package.yaml"));
        assert!(rendered.contains("\"line\": 2"));
    }

    #[test]
    fn package_error_kinds_map_to_stable_diagnostic_codes() {
        assert_eq!(
            PackageErrorKind::InvalidInput.diagnostic_code(),
            DiagnosticCode::new("K1001")
        );
        assert_eq!(
            PackageErrorKind::Unsupported.diagnostic_code(),
            DiagnosticCode::new("K1002")
        );
        assert_eq!(
            PackageErrorKind::ImportCycle.diagnostic_code(),
            DiagnosticCode::new("K1003")
        );
        assert_eq!(
            PackageErrorKind::Internal.diagnostic_code(),
            DiagnosticCode::new("K1099")
        );
    }

    #[test]
    fn package_error_human_output_keeps_snippet_shape() {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be stable enough for temp file names")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("fol_package_diagnostic_{stamp}.yaml"));
        std::fs::write(&path, "name: json\nversion: 1.0.0\n")
            .expect("package diagnostic fixture should be writable");
        let error = PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            "duplicate package metadata field",
            SyntaxOrigin {
                file: Some(path.to_string_lossy().into_owned()),
                line: 1,
                column: 1,
                length: 4,
            },
        );
        let mut report = DiagnosticReport::new();

        report.add_error(&error, error.diagnostic_location());

        let rendered = report.output(fol_diagnostics::OutputFormat::Human);
        let _ = std::fs::remove_file(&path);

        assert!(rendered.contains("error: PackageInvalidInput: duplicate package metadata field"));
        assert!(rendered.contains("| name: json"));
        assert!(rendered.contains("| ^^^^"));
    }

    #[test]
    fn package_error_to_diagnostic_preserves_explicit_package_codes() {
        let error = PackageError::with_origin(
            PackageErrorKind::InvalidInput,
            "duplicate package metadata field",
            SyntaxOrigin {
                file: Some("pkg/package.yaml".to_string()),
                line: 2,
                column: 1,
                length: 4,
            },
        );
        let mut report = DiagnosticReport::new();

        report.add_from(&error);

        let rendered = report.output(fol_diagnostics::OutputFormat::Json);
        assert!(rendered.contains("\"code\": \"K1001\""));
        assert!(rendered.contains("PackageInvalidInput: duplicate package metadata field"));
    }
}
