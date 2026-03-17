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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildSemanticParameterShape {
    Scalar,
    Record,
    Handle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSemanticMethodParameter {
    pub name: String,
    pub shape: BuildSemanticParameterShape,
    pub value_type: Option<BuildSemanticTypeFamily>,
    pub optional: bool,
    pub variadic: bool,
}

impl BuildSemanticMethodParameter {
    pub fn scalar(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shape: BuildSemanticParameterShape::Scalar,
            value_type: None,
            optional: false,
            variadic: false,
        }
    }

    pub fn record(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            shape: BuildSemanticParameterShape::Record,
            value_type: None,
            optional: false,
            variadic: false,
        }
    }

    pub fn handle(name: impl Into<String>, family: BuildSemanticTypeFamily) -> Self {
        Self {
            name: name.into(),
            shape: BuildSemanticParameterShape::Handle,
            value_type: Some(family),
            optional: false,
            variadic: false,
        }
    }

    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    pub fn variadic(mut self) -> Self {
        self.variadic = true;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildSemanticMethodSignature {
    pub receiver: BuildSemanticTypeFamily,
    pub name: String,
    pub params: Vec<BuildSemanticMethodParameter>,
    pub returns: Option<BuildSemanticTypeFamily>,
    pub chainable: bool,
}

impl BuildSemanticMethodSignature {
    pub fn new(receiver: BuildSemanticTypeFamily, name: impl Into<String>) -> Self {
        Self {
            receiver,
            name: name.into(),
            params: Vec::new(),
            returns: None,
            chainable: false,
        }
    }

    pub fn with_param(mut self, param: BuildSemanticMethodParameter) -> Self {
        self.params.push(param);
        self
    }

    pub fn returning(mut self, family: BuildSemanticTypeFamily) -> Self {
        self.returns = Some(family);
        self
    }

    pub fn chainable(mut self) -> Self {
        self.chainable = true;
        self
    }
}

pub fn canonical_graph_method_signatures() -> Vec<BuildSemanticMethodSignature> {
    vec![
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "standard_target")
            .with_param(BuildSemanticMethodParameter::record("config").optional()),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "standard_optimize")
            .with_param(BuildSemanticMethodParameter::record("config").optional()),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "option")
            .with_param(BuildSemanticMethodParameter::record("config")),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_exe")
            .with_param(BuildSemanticMethodParameter::record("config"))
            .returning(BuildSemanticTypeFamily::ArtifactHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_static_lib")
            .with_param(BuildSemanticMethodParameter::record("config"))
            .returning(BuildSemanticTypeFamily::ArtifactHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_shared_lib")
            .with_param(BuildSemanticMethodParameter::record("config"))
            .returning(BuildSemanticTypeFamily::ArtifactHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_test")
            .with_param(BuildSemanticMethodParameter::record("config"))
            .returning(BuildSemanticTypeFamily::ArtifactHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "step")
            .with_param(BuildSemanticMethodParameter::scalar("name"))
            .with_param(BuildSemanticMethodParameter::scalar("description").optional())
            .returning(BuildSemanticTypeFamily::StepHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_run")
            .with_param(BuildSemanticMethodParameter::handle(
                "artifact",
                BuildSemanticTypeFamily::ArtifactHandle,
            ))
            .returning(BuildSemanticTypeFamily::RunHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "install")
            .with_param(BuildSemanticMethodParameter::handle(
                "artifact",
                BuildSemanticTypeFamily::ArtifactHandle,
            ))
            .returning(BuildSemanticTypeFamily::InstallHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "install_file")
            .with_param(BuildSemanticMethodParameter::scalar("path"))
            .returning(BuildSemanticTypeFamily::InstallHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "install_dir")
            .with_param(BuildSemanticMethodParameter::scalar("path"))
            .returning(BuildSemanticTypeFamily::InstallHandle),
        BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "dependency")
            .with_param(BuildSemanticMethodParameter::scalar("alias"))
            .with_param(BuildSemanticMethodParameter::scalar("package"))
            .returning(BuildSemanticTypeFamily::DependencyHandle),
    ]
}

#[cfg(test)]
mod tests {
    use super::{
        canonical_graph_method_signatures,
        BuildSemanticMethodParameter, BuildSemanticMethodSignature, BuildSemanticParameterShape,
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

    #[test]
    fn semantic_method_parameters_capture_scalar_record_and_handle_shapes() {
        let scalar = BuildSemanticMethodParameter::scalar("name");
        let record = BuildSemanticMethodParameter::record("config").optional();
        let handle = BuildSemanticMethodParameter::handle(
            "artifact",
            BuildSemanticTypeFamily::ArtifactHandle,
        )
        .variadic();

        assert_eq!(scalar.shape, BuildSemanticParameterShape::Scalar);
        assert_eq!(record.shape, BuildSemanticParameterShape::Record);
        assert!(record.optional);
        assert_eq!(handle.shape, BuildSemanticParameterShape::Handle);
        assert_eq!(handle.value_type, Some(BuildSemanticTypeFamily::ArtifactHandle));
        assert!(handle.variadic);
    }

    #[test]
    fn semantic_method_signatures_keep_receiver_return_and_chainability() {
        let signature = BuildSemanticMethodSignature::new(BuildSemanticTypeFamily::Graph, "add_exe")
            .with_param(BuildSemanticMethodParameter::record("config"))
            .returning(BuildSemanticTypeFamily::ArtifactHandle)
            .chainable();

        assert_eq!(signature.receiver, BuildSemanticTypeFamily::Graph);
        assert_eq!(signature.name, "add_exe");
        assert_eq!(signature.params.len(), 1);
        assert_eq!(signature.returns, Some(BuildSemanticTypeFamily::ArtifactHandle));
        assert!(signature.chainable);
    }

    #[test]
    fn canonical_graph_methods_cover_build_graph_entrypoints() {
        let signatures = canonical_graph_method_signatures();
        let names = signatures
            .iter()
            .map(|signature| signature.name.as_str())
            .collect::<Vec<_>>();

        assert!(names.contains(&"standard_target"));
        assert!(names.contains(&"standard_optimize"));
        assert!(names.contains(&"add_exe"));
        assert!(names.contains(&"step"));
        assert!(names.contains(&"dependency"));
    }

    #[test]
    fn canonical_graph_methods_return_expected_handle_families() {
        let signatures = canonical_graph_method_signatures();
        let add_exe = signatures.iter().find(|signature| signature.name == "add_exe");
        let add_run = signatures.iter().find(|signature| signature.name == "add_run");
        let install = signatures.iter().find(|signature| signature.name == "install");

        assert_eq!(
            add_exe.and_then(|signature| signature.returns),
            Some(BuildSemanticTypeFamily::ArtifactHandle)
        );
        assert_eq!(
            add_run.and_then(|signature| signature.returns),
            Some(BuildSemanticTypeFamily::RunHandle)
        );
        assert_eq!(
            install.and_then(|signature| signature.returns),
            Some(BuildSemanticTypeFamily::InstallHandle)
        );
    }
}
