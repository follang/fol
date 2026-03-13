use fol_diagnostics::{DiagnosticLocation, ToDiagnosticLocation};
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

#[cfg(test)]
mod tests {
    use super::{PackageError, PackageErrorKind};
    use fol_diagnostics::DiagnosticReport;
    use fol_parser::ast::SyntaxOrigin;

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
}
