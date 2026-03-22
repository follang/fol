#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmittedRustFile {
    pub path: String,
    pub module_name: String,
    pub contents: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendBuildPaths {
    pub output_root: String,
    pub build_root: String,
    pub bin_root: String,
    pub runtime_root: String,
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
