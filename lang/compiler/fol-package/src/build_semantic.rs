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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSemanticTypeFamily {
    Graph,
    ArtifactHandle,
    StepHandle,
    RunHandle,
    InstallHandle,
    DependencyHandle,
    GeneratedFileHandle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSemanticType {
    pub module: BuildStdlibModulePath,
    pub name: String,
    pub family: BuildSemanticTypeFamily,
}

impl BuildSemanticType {
    pub fn graph() -> Self {
        Self {
            module: BuildStdlibModulePath::root(),
            name: "Graph".to_string(),
            family: BuildSemanticTypeFamily::Graph,
        }
    }

    pub fn artifact_handle() -> Self {
        Self::types_named("Artifact", BuildSemanticTypeFamily::ArtifactHandle)
    }

    pub fn step_handle() -> Self {
        Self::types_named("Step", BuildSemanticTypeFamily::StepHandle)
    }

    pub fn run_handle() -> Self {
        Self::types_named("Run", BuildSemanticTypeFamily::RunHandle)
    }

    pub fn install_handle() -> Self {
        Self::types_named("Install", BuildSemanticTypeFamily::InstallHandle)
    }

    pub fn dependency_handle() -> Self {
        Self::types_named("Dependency", BuildSemanticTypeFamily::DependencyHandle)
    }

    pub fn generated_file_handle() -> Self {
        Self::types_named("GeneratedFile", BuildSemanticTypeFamily::GeneratedFileHandle)
    }

    fn types_named(name: &str, family: BuildSemanticTypeFamily) -> Self {
        Self {
            module: BuildStdlibModulePath::types(),
            name: name.to_string(),
            family,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildSemanticType, BuildSemanticTypeFamily, BuildStdlibImportSurface,
        BuildStdlibModuleKind, BuildStdlibModulePath,
    };

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

    #[test]
    fn semantic_build_surface_types_keep_canonical_modules() {
        let graph = BuildSemanticType::graph();
        let artifact = BuildSemanticType::artifact_handle();
        let step = BuildSemanticType::step_handle();

        assert_eq!(graph.module, BuildStdlibModulePath::root());
        assert_eq!(graph.family, BuildSemanticTypeFamily::Graph);
        assert_eq!(artifact.module, BuildStdlibModulePath::types());
        assert_eq!(artifact.family, BuildSemanticTypeFamily::ArtifactHandle);
        assert_eq!(step.module, BuildStdlibModulePath::types());
        assert_eq!(step.family, BuildSemanticTypeFamily::StepHandle);
    }

    #[test]
    fn semantic_build_surface_handle_names_stay_stable() {
        assert_eq!(BuildSemanticType::run_handle().name, "Run");
        assert_eq!(BuildSemanticType::install_handle().name, "Install");
        assert_eq!(BuildSemanticType::dependency_handle().name, "Dependency");
        assert_eq!(
            BuildSemanticType::generated_file_handle().name,
            "GeneratedFile"
        );
    }
}
