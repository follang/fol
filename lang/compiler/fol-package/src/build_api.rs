use crate::build_graph::BuildGraph;
use crate::build_graph::{BuildOptionId, BuildOptionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardTargetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardOptimizeRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildOptionValue {
    Bool(bool),
    String(String),
    Enum(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOptionRequest {
    pub name: String,
    pub kind: BuildOptionKind,
    pub default: Option<BuildOptionValue>,
}

impl UserOptionRequest {
    pub fn bool(name: impl Into<String>, default: bool) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::Bool,
            default: Some(BuildOptionValue::Bool(default)),
        }
    }

    pub fn string(name: impl Into<String>, default: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::String,
            default: Some(BuildOptionValue::String(default.into())),
        }
    }

    pub fn enumeration(name: impl Into<String>, default: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            kind: BuildOptionKind::Enum,
            default: Some(BuildOptionValue::Enum(default.into())),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserOption {
    pub id: BuildOptionId,
    pub name: String,
    pub kind: BuildOptionKind,
    pub default: Option<BuildOptionValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutableRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StaticLibraryRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SharedLibraryRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestArtifactRequest {
    pub name: String,
    pub root_module: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildApiNameError {
    Empty,
    InvalidCharacter(char),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildApiError {
    InvalidName(BuildApiNameError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactHandle {
    pub artifact_id: crate::build_graph::BuildArtifactId,
    pub root_module_id: crate::build_graph::BuildModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepRequest {
    pub name: String,
    pub depends_on: Vec<crate::build_graph::BuildStepId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepHandle {
    pub step_id: crate::build_graph::BuildStepId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunRequest {
    pub name: String,
    pub artifact: BuildArtifactHandle,
    pub depends_on: Vec<crate::build_graph::BuildStepId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunHandle {
    pub step_id: crate::build_graph::BuildStepId,
    pub artifact_id: crate::build_graph::BuildArtifactId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallArtifactRequest {
    pub name: String,
    pub artifact: BuildArtifactHandle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallFileRequest {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallDirRequest {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallHandle {
    pub install_id: crate::build_graph::BuildInstallId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyRequest {
    pub alias: String,
    pub package: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyHandle {
    pub alias: String,
    pub package: String,
    pub root_module_id: crate::build_graph::BuildModuleId,
}

pub fn validate_build_name(name: &str) -> Result<(), BuildApiNameError> {
    if name.is_empty() {
        return Err(BuildApiNameError::Empty);
    }

    for ch in name.chars() {
        if ch.is_ascii_lowercase() || ch.is_ascii_digit() || matches!(ch, '-' | '_' | '.') {
            continue;
        }
        return Err(BuildApiNameError::InvalidCharacter(ch));
    }

    Ok(())
}

#[derive(Debug)]
pub struct BuildApi<'a> {
    graph: &'a mut BuildGraph,
}

impl<'a> BuildApi<'a> {
    pub fn new(graph: &'a mut BuildGraph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &BuildGraph {
        self.graph
    }

    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        self.graph
    }

    pub fn standard_target(&mut self, request: StandardTargetRequest) -> StandardTargetOption {
        let option_id = self.graph.add_option(BuildOptionKind::Target, request.name.clone());
        StandardTargetOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }

    pub fn standard_optimize(
        &mut self,
        request: StandardOptimizeRequest,
    ) -> StandardOptimizeOption {
        let option_id = self.graph.add_option(BuildOptionKind::Optimize, request.name.clone());
        StandardOptimizeOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }

    pub fn option(&mut self, request: UserOptionRequest) -> UserOption {
        let option_id = self.graph.add_option(request.kind, request.name.clone());
        UserOption {
            id: option_id,
            name: request.name,
            kind: request.kind,
            default: request.default,
        }
    }

    pub fn add_exe(
        &mut self,
        request: ExecutableRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::build_graph::BuildArtifactKind::Executable,
        )
    }

    pub fn add_static_lib(
        &mut self,
        request: StaticLibraryRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::build_graph::BuildArtifactKind::StaticLibrary,
        )
    }

    pub fn add_shared_lib(
        &mut self,
        request: SharedLibraryRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::build_graph::BuildArtifactKind::SharedLibrary,
        )
    }

    pub fn add_test(
        &mut self,
        request: TestArtifactRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::build_graph::BuildArtifactKind::Executable,
        )
    }

    fn add_named_artifact(
        &mut self,
        name: String,
        root_module: String,
        kind: crate::build_graph::BuildArtifactKind,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        validate_build_name(&name).map_err(BuildApiError::InvalidName)?;
        let module_id = self
            .graph
            .add_module(crate::build_graph::BuildModuleKind::Source, root_module);
        let artifact_id = self.graph.add_artifact(kind, name);
        self.graph.add_artifact_module_input(artifact_id, module_id);
        Ok(BuildArtifactHandle {
            artifact_id,
            root_module_id: module_id,
        })
    }

    pub fn step(&mut self, request: StepRequest) -> Result<StepHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(BuildApiError::InvalidName)?;
        let step_id = self
            .graph
            .add_step(crate::build_graph::BuildStepKind::Default, request.name.clone());
        for dependency in request.depends_on {
            self.graph.add_step_dependency(step_id, dependency);
        }
        Ok(StepHandle {
            step_id,
            name: request.name,
        })
    }

    pub fn add_run(&mut self, request: RunRequest) -> Result<RunHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(BuildApiError::InvalidName)?;
        let step_id = self
            .graph
            .add_step(crate::build_graph::BuildStepKind::Run, request.name);
        for dependency in request.depends_on {
            self.graph.add_step_dependency(step_id, dependency);
        }
        Ok(RunHandle {
            step_id,
            artifact_id: request.artifact.artifact_id,
        })
    }

    pub fn install(
        &mut self,
        request: InstallArtifactRequest,
    ) -> Result<InstallHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(BuildApiError::InvalidName)?;
        let install_id = self.graph.add_install_with_target(
            crate::build_graph::BuildInstallKind::Artifact,
            request.name,
            Some(crate::build_graph::BuildInstallTarget::Artifact(
                request.artifact.artifact_id,
            )),
        );
        Ok(InstallHandle { install_id })
    }

    pub fn install_file(&mut self, request: InstallFileRequest) -> Result<InstallHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(BuildApiError::InvalidName)?;
        let generated = self.graph.add_generated_file(
            crate::build_graph::BuildGeneratedFileKind::Copy,
            request.path,
        );
        let install_id = self.graph.add_install_with_target(
            crate::build_graph::BuildInstallKind::File,
            request.name,
            Some(crate::build_graph::BuildInstallTarget::GeneratedFile(generated)),
        );
        Ok(InstallHandle { install_id })
    }

    pub fn install_dir(&mut self, request: InstallDirRequest) -> Result<InstallHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(BuildApiError::InvalidName)?;
        let install_id = self.graph.add_install_with_target(
            crate::build_graph::BuildInstallKind::Directory,
            request.name,
            Some(crate::build_graph::BuildInstallTarget::DirectoryPath(
                request.path,
            )),
        );
        Ok(InstallHandle { install_id })
    }

    pub fn dependency(&mut self, request: DependencyRequest) -> Result<DependencyHandle, BuildApiError> {
        validate_build_name(&request.alias).map_err(BuildApiError::InvalidName)?;
        let module_id = self.graph.add_module(
            crate::build_graph::BuildModuleKind::Imported,
            format!("{}:{}", request.alias, request.package),
        );
        Ok(DependencyHandle {
            alias: request.alias,
            package: request.package,
            root_module_id: module_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{
        validate_build_name, BuildApi, BuildApiError, BuildApiNameError, BuildOptionValue,
        DependencyRequest, ExecutableRequest, InstallArtifactRequest, InstallDirRequest,
        InstallFileRequest, RunRequest, SharedLibraryRequest, StandardOptimizeRequest,
        StandardTargetRequest, StaticLibraryRequest, StepRequest, TestArtifactRequest,
        UserOptionRequest,
    };
    use crate::build_graph::BuildGraph;
    use crate::build_graph::{
        BuildArtifactInput, BuildArtifactKind, BuildInstallKind, BuildInstallTarget,
        BuildModuleKind, BuildOptionKind, BuildStepKind,
    };

    #[test]
    fn build_api_wraps_a_graph_reference() {
        let mut graph = BuildGraph::new();
        let api = BuildApi::new(&mut graph);

        assert!(api.graph().steps().is_empty());
    }

    #[test]
    fn build_api_exposes_mutable_graph_access() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        api.graph_mut().add_step(crate::build_graph::BuildStepKind::Default, "build");

        assert_eq!(api.graph().steps().len(), 1);
    }

    #[test]
    fn build_api_records_standard_target_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option = api.standard_target(StandardTargetRequest::new("target").with_default("native"));

        assert_eq!(option.name, "target");
        assert_eq!(option.default.as_deref(), Some("native"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Target);
    }

    #[test]
    fn build_api_records_standard_optimize_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option =
            api.standard_optimize(StandardOptimizeRequest::new("optimize").with_default("debug"));

        assert_eq!(option.name, "optimize");
        assert_eq!(option.default.as_deref(), Some("debug"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Optimize);
    }

    #[test]
    fn build_api_records_boolean_user_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option = api.option(UserOptionRequest::bool("strip", false));

        assert_eq!(option.name, "strip");
        assert_eq!(option.kind, BuildOptionKind::Bool);
        assert_eq!(option.default, Some(BuildOptionValue::Bool(false)));
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Bool);
    }

    #[test]
    fn build_api_records_string_and_enum_user_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let prefix = api.option(UserOptionRequest::string("prefix", "/usr/local"));
        let flavor = api.option(UserOptionRequest::enumeration("flavor", "release"));

        assert_eq!(
            prefix.default,
            Some(BuildOptionValue::String("/usr/local".to_string()))
        );
        assert_eq!(
            flavor.default,
            Some(BuildOptionValue::Enum("release".to_string()))
        );
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::String);
        assert_eq!(api.graph().options()[1].kind, BuildOptionKind::Enum);
    }

    #[test]
    fn build_name_validation_accepts_the_draft_public_naming_rules() {
        assert_eq!(validate_build_name("app"), Ok(()));
        assert_eq!(validate_build_name("app-main"), Ok(()));
        assert_eq!(validate_build_name("app.main_1"), Ok(()));
    }

    #[test]
    fn build_name_validation_rejects_empty_and_mixed_case_names() {
        assert_eq!(validate_build_name(""), Err(BuildApiNameError::Empty));
        assert_eq!(
            validate_build_name("App"),
            Err(BuildApiNameError::InvalidCharacter('A'))
        );
    }

    #[test]
    fn structured_artifact_requests_keep_name_and_root_module_fields() {
        let exe = ExecutableRequest {
            name: "app".to_string(),
            root_module: "src/app.fol".to_string(),
        };
        let static_lib = StaticLibraryRequest {
            name: "support".to_string(),
            root_module: "src/support.fol".to_string(),
        };
        let shared_lib = SharedLibraryRequest {
            name: "plugin".to_string(),
            root_module: "src/plugin.fol".to_string(),
        };
        let tests = TestArtifactRequest {
            name: "app-tests".to_string(),
            root_module: "test/app.fol".to_string(),
        };

        assert_eq!(exe.root_module, "src/app.fol");
        assert_eq!(static_lib.name, "support");
        assert_eq!(shared_lib.name, "plugin");
        assert_eq!(tests.root_module, "test/app.fol");
    }

    #[test]
    fn build_api_add_exe_and_lib_methods_create_graph_artifacts_and_modules() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let exe = api
            .add_exe(ExecutableRequest {
                name: "app".to_string(),
                root_module: "src/app.fol".to_string(),
            })
            .expect("valid executable request should succeed");
        let static_lib = api
            .add_static_lib(StaticLibraryRequest {
                name: "support".to_string(),
                root_module: "src/support.fol".to_string(),
            })
            .expect("valid static library request should succeed");
        let shared_lib = api
            .add_shared_lib(SharedLibraryRequest {
                name: "plugin".to_string(),
                root_module: "src/plugin.fol".to_string(),
            })
            .expect("valid shared library request should succeed");

        assert_eq!(api.graph().artifacts()[0].id, exe.artifact_id);
        assert_eq!(api.graph().artifacts()[0].kind, BuildArtifactKind::Executable);
        assert_eq!(api.graph().artifacts()[1].id, static_lib.artifact_id);
        assert_eq!(api.graph().artifacts()[1].kind, BuildArtifactKind::StaticLibrary);
        assert_eq!(api.graph().artifacts()[2].id, shared_lib.artifact_id);
        assert_eq!(api.graph().artifacts()[2].kind, BuildArtifactKind::SharedLibrary);
        assert_eq!(
            api.graph().artifact_inputs_for(exe.artifact_id).collect::<Vec<_>>(),
            vec![BuildArtifactInput::Module(exe.root_module_id)]
        );
    }

    #[test]
    fn build_api_add_test_uses_the_executable_artifact_shape() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let tests = api
            .add_test(TestArtifactRequest {
                name: "app-tests".to_string(),
                root_module: "test/app.fol".to_string(),
            })
            .expect("valid test artifact request should succeed");

        assert_eq!(api.graph().artifacts()[0].id, tests.artifact_id);
        assert_eq!(api.graph().artifacts()[0].kind, BuildArtifactKind::Executable);
    }

    #[test]
    fn build_api_artifact_methods_reject_invalid_names() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let error = api
            .add_exe(ExecutableRequest {
                name: "App".to_string(),
                root_module: "src/app.fol".to_string(),
            })
            .expect_err("mixed-case names should be rejected");

        assert_eq!(
            error,
            BuildApiError::InvalidName(BuildApiNameError::InvalidCharacter('A'))
        );
    }

    #[test]
    fn build_api_step_adds_named_default_steps_and_dependencies() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);
        let base = api
            .step(StepRequest {
                name: "build".to_string(),
                depends_on: Vec::new(),
            })
            .expect("valid step request should succeed");
        let check = api
            .step(StepRequest {
                name: "check".to_string(),
                depends_on: vec![base.step_id],
            })
            .expect("valid dependent step should succeed");

        assert_eq!(api.graph().steps()[0].kind, BuildStepKind::Default);
        assert_eq!(api.graph().steps()[1].id, check.step_id);
        assert_eq!(
            api.graph().step_dependencies_for(check.step_id).collect::<Vec<_>>(),
            vec![base.step_id]
        );
    }

    #[test]
    fn build_api_add_run_creates_a_run_step() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);
        let build = api
            .step(StepRequest {
                name: "build".to_string(),
                depends_on: Vec::new(),
            })
            .expect("build step should succeed");
        let exe = api
            .add_exe(ExecutableRequest {
                name: "app".to_string(),
                root_module: "src/app.fol".to_string(),
            })
            .expect("valid executable request should succeed");

        let run = api
            .add_run(RunRequest {
                name: "run".to_string(),
                artifact: exe.clone(),
                depends_on: vec![build.step_id],
            })
            .expect("valid run request should succeed");

        assert_eq!(run.artifact_id, exe.artifact_id);
        assert_eq!(api.graph().steps()[1].kind, BuildStepKind::Run);
        assert_eq!(
            api.graph().step_dependencies_for(run.step_id).collect::<Vec<_>>(),
            vec![build.step_id]
        );
    }

    #[test]
    fn build_api_install_methods_record_install_targets_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);
        let exe = api
            .add_exe(ExecutableRequest {
                name: "app".to_string(),
                root_module: "src/app.fol".to_string(),
            })
            .expect("valid executable request should succeed");

        let artifact_install = api
            .install(InstallArtifactRequest {
                name: "install-app".to_string(),
                artifact: exe.clone(),
            })
            .expect("valid artifact install should succeed");
        let file_install = api
            .install_file(InstallFileRequest {
                name: "install-config".to_string(),
                path: "share/config.json".to_string(),
            })
            .expect("valid file install should succeed");
        let dir_install = api
            .install_dir(InstallDirRequest {
                name: "install-assets".to_string(),
                path: "share/assets".to_string(),
            })
            .expect("valid directory install should succeed");

        assert_eq!(api.graph().installs()[0].id, artifact_install.install_id);
        assert_eq!(api.graph().installs()[0].kind, BuildInstallKind::Artifact);
        assert_eq!(
            api.graph().installs()[0].target,
            Some(BuildInstallTarget::Artifact(exe.artifact_id))
        );
        assert_eq!(api.graph().installs()[1].id, file_install.install_id);
        assert_eq!(api.graph().installs()[1].kind, BuildInstallKind::File);
        assert_eq!(api.graph().installs()[2].id, dir_install.install_id);
        assert_eq!(api.graph().installs()[2].kind, BuildInstallKind::Directory);
        assert_eq!(
            api.graph().installs()[2].target,
            Some(BuildInstallTarget::DirectoryPath("share/assets".to_string()))
        );
    }

    #[test]
    fn build_api_dependency_creates_an_imported_module_placeholder() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let dependency = api
            .dependency(DependencyRequest {
                alias: "logtiny".to_string(),
                package: "org/logtiny".to_string(),
            })
            .expect("valid dependency request should succeed");

        assert_eq!(dependency.alias, "logtiny");
        assert_eq!(dependency.package, "org/logtiny");
        assert_eq!(api.graph().modules()[0].id, dependency.root_module_id);
        assert_eq!(api.graph().modules()[0].kind, BuildModuleKind::Imported);
        assert_eq!(api.graph().modules()[0].name, "logtiny:org/logtiny");
    }
}
