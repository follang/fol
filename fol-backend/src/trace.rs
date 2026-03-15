use fol_lower::LoweredSourceSymbol;
use fol_parser::ast::SyntaxOrigin;
use fol_resolver::PackageIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendEmittedSourceMapEntry {
    pub emitted_path: String,
    pub line: usize,
    pub column: usize,
    pub symbol: LoweredSourceSymbol,
    pub origin: SyntaxOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackendEmittedSourceMap {
    entries: Vec<BackendEmittedSourceMapEntry>,
}

impl BackendEmittedSourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[BackendEmittedSourceMapEntry] {
        &self.entries
    }

    pub fn push(&mut self, entry: BackendEmittedSourceMapEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendTraceKind {
    Session,
    Layout,
    Emission,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendTraceRecord {
    pub kind: BackendTraceKind,
    pub emitted_path: Option<String>,
    pub package_identity: Option<PackageIdentity>,
    pub symbol: Option<LoweredSourceSymbol>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BackendTrace {
    records: Vec<BackendTraceRecord>,
}

impl BackendTrace {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn records(&self) -> &[BackendTraceRecord] {
        &self.records
    }

    pub fn push(&mut self, record: BackendTraceRecord) {
        self.records.push(record);
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BackendEmittedSourceMap, BackendEmittedSourceMapEntry, BackendTrace, BackendTraceKind,
        BackendTraceRecord,
    };
    use crate::testing::package_identity;
    use fol_lower::{LoweredGlobalId, LoweredSourceSymbol};
    use fol_parser::ast::SyntaxOrigin;
    use fol_resolver::PackageSourceKind;

    #[test]
    fn backend_trace_and_source_map_capture_backend_owned_metadata() {
        let mut source_map = BackendEmittedSourceMap::new();
        let mut trace = BackendTrace::new();

        source_map.push(BackendEmittedSourceMapEntry {
            emitted_path: "src/app.rs".to_string(),
            line: 4,
            column: 8,
            symbol: LoweredSourceSymbol::Global(LoweredGlobalId(0)),
            origin: SyntaxOrigin {
                file: Some("app/main.fol".to_string()),
                line: 2,
                column: 3,
                length: 5,
            },
        });
        trace.push(BackendTraceRecord {
            kind: BackendTraceKind::Layout,
            emitted_path: Some("src/app.rs".to_string()),
            package_identity: Some(package_identity(
                "app",
                PackageSourceKind::Entry,
                "/workspace/app",
            )),
            symbol: Some(LoweredSourceSymbol::Global(LoweredGlobalId(0))),
            detail: "planned app root module".to_string(),
        });

        assert_eq!(source_map.entries().len(), 1);
        assert_eq!(trace.records().len(), 1);
        assert_eq!(trace.records()[0].kind, BackendTraceKind::Layout);
        assert_eq!(source_map.entries()[0].origin.line, 2);
    }
}
