#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStdlibModuleKind {
    Root,
    Types,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStdlibModulePath {
    pub package: String,
    pub module: String,
    pub kind: BuildStdlibModuleKind,
}

impl BuildStdlibModulePath {
    pub fn root() -> Self {
        Self {
            package: "fol/build".to_string(),
            module: "build".to_string(),
            kind: BuildStdlibModuleKind::Root,
        }
    }

    pub fn types() -> Self {
        Self {
            package: "fol/build".to_string(),
            module: "build/types".to_string(),
            kind: BuildStdlibModuleKind::Types,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStdlibImportSurface {
    pub canonical_import_alias: String,
    pub root_module: BuildStdlibModulePath,
}

impl BuildStdlibImportSurface {
    pub fn canonical() -> Self {
        Self {
            canonical_import_alias: "build".to_string(),
            root_module: BuildStdlibModulePath::root(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildStdlibImportSurface, BuildStdlibModuleKind, BuildStdlibModulePath};

    #[test]
    fn build_stdlib_module_paths_keep_canonical_package_and_module_names() {
        let root = BuildStdlibModulePath::root();
        let types = BuildStdlibModulePath::types();

        assert_eq!(root.package, "fol/build");
        assert_eq!(root.module, "build");
        assert_eq!(root.kind, BuildStdlibModuleKind::Root);

        assert_eq!(types.package, "fol/build");
        assert_eq!(types.module, "build/types");
        assert_eq!(types.kind, BuildStdlibModuleKind::Types);
    }

    #[test]
    fn build_stdlib_import_surface_keeps_the_canonical_build_alias() {
        let surface = BuildStdlibImportSurface::canonical();

        assert_eq!(surface.canonical_import_alias, "build");
        assert_eq!(surface.root_module, BuildStdlibModulePath::root());
    }
}
