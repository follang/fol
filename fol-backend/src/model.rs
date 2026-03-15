#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedRustFile {
    pub path: String,
    pub module_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BackendArtifact {
    RustSourceCrate {
        root: String,
        files: Vec<EmittedRustFile>,
    },
    CompiledBinary {
        crate_root: String,
        binary_path: String,
    },
}
