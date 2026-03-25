use crate::codegen::{CodegenRequest, GeneratedFileInstallProjection, SystemToolRequest};
use crate::dependency::{
    DependencyArtifactSurfaceSet, DependencyBuildHandle, DependencyGeneratedOutputSurfaceSet,
    DependencyModuleSurfaceSet, DependencyStepSurfaceSet,
};
use crate::graph::BuildGraph;
use crate::graph::{BuildOptionKind};

use super::types::{
    validate_build_name, AddModuleRequest, BuildApiError, BuildArtifactHandle,
    CopyFileRequest, DependencyHandle, DependencyRequest, ExecutableRequest, GeneratedFileHandle,
    InstallArtifactRequest, InstallDirRequest, InstallFileRequest, InstallHandle, ModuleHandle,
    OutputHandle, OutputHandleKind, OutputHandleLocator, RunHandle, RunRequest,
    SharedLibraryRequest, StandardOptimizeOption, StandardOptimizeRequest, StandardTargetOption,
    StandardTargetRequest, StaticLibraryRequest, StepHandle, StepRequest, TestArtifactRequest,
    UserOption, UserOptionRequest, WriteFileRequest,
};

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
        let option_id = self
            .graph
            .add_option(BuildOptionKind::Target, request.name.clone());
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
        let option_id = self
            .graph
            .add_option(BuildOptionKind::Optimize, request.name.clone());
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
            crate::graph::BuildArtifactKind::Executable,
        )
    }

    pub fn add_static_lib(
        &mut self,
        request: StaticLibraryRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::graph::BuildArtifactKind::StaticLibrary,
        )
    }

    pub fn add_shared_lib(
        &mut self,
        request: SharedLibraryRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::graph::BuildArtifactKind::SharedLibrary,
        )
    }

    pub fn add_test(
        &mut self,
        request: TestArtifactRequest,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        self.add_named_artifact(
            request.name,
            request.root_module,
            crate::graph::BuildArtifactKind::Executable,
        )
    }

    fn add_named_artifact(
        &mut self,
        name: String,
        root_module: String,
        kind: crate::graph::BuildArtifactKind,
    ) -> Result<BuildArtifactHandle, BuildApiError> {
        validate_build_name(&name).map_err(super::types::BuildApiError::InvalidName)?;
        let module_id = self
            .graph
            .add_module(crate::graph::BuildModuleKind::Source, root_module);
        let artifact_id = self.graph.add_artifact(kind, name);
        self.graph.add_artifact_module_input(artifact_id, module_id);
        Ok(BuildArtifactHandle {
            artifact_id,
            root_module_id: module_id,
        })
    }

    pub fn step(&mut self, request: StepRequest) -> Result<StepHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let step_id = self.graph.add_step(
            crate::graph::BuildStepKind::Default,
            request.name.clone(),
        );
        for dependency in request.depends_on {
            self.graph.add_step_dependency(step_id, dependency);
        }
        Ok(StepHandle {
            step_id,
            name: request.name,
        })
    }

    pub fn add_run(&mut self, request: RunRequest) -> Result<RunHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let step_id = self
            .graph
            .add_step(crate::graph::BuildStepKind::Run, request.name);
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
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let step_id = self.graph.add_step(
            crate::graph::BuildStepKind::Install,
            request.name.clone(),
        );
        for dependency in &request.depends_on {
            self.graph.add_step_dependency(step_id, *dependency);
        }
        let install_id = self.graph.add_install_with_target(
            crate::graph::BuildInstallKind::Artifact,
            request.name.clone(),
            Some(crate::graph::BuildInstallTarget::Artifact(
                request.artifact.artifact_id,
            )),
        );
        Ok(InstallHandle {
            install_id,
            step_id,
            name: request.name,
        })
    }

    pub fn install_file(
        &mut self,
        request: InstallFileRequest,
    ) -> Result<InstallHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let step_id = self.graph.add_step(
            crate::graph::BuildStepKind::Install,
            request.name.clone(),
        );
        for dependency in &request.depends_on {
            self.graph.add_step_dependency(step_id, *dependency);
        }
        let generated = self.graph.add_generated_file(
            crate::graph::BuildGeneratedFileKind::Copy,
            request.path,
        );
        let install_id = self.graph.add_install_with_target(
            crate::graph::BuildInstallKind::File,
            request.name.clone(),
            Some(crate::graph::BuildInstallTarget::GeneratedFile(
                generated,
            )),
        );
        Ok(InstallHandle {
            install_id,
            step_id,
            name: request.name,
        })
    }

    pub fn write_file(
        &mut self,
        request: WriteFileRequest,
    ) -> Result<OutputHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let generated_file_id = self.graph.add_generated_file(
            crate::graph::BuildGeneratedFileKind::Write,
            request.path,
        );
        Ok(OutputHandle {
            kind: OutputHandleKind::WrittenFile,
            locator: OutputHandleLocator::GeneratedFile(generated_file_id),
        })
    }

    pub fn copy_file(
        &mut self,
        request: CopyFileRequest,
    ) -> Result<OutputHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let generated_file_id = self.graph.add_generated_file(
            crate::graph::BuildGeneratedFileKind::Copy,
            request.destination_path,
        );
        Ok(OutputHandle {
            kind: OutputHandleKind::CopiedFile,
            locator: OutputHandleLocator::GeneratedFile(generated_file_id),
        })
    }

    pub fn add_system_tool(
        &mut self,
        request: SystemToolRequest,
    ) -> Result<Vec<GeneratedFileHandle>, BuildApiError> {
        validate_build_name(&request.tool.replace('_', "-")).map_err(super::types::BuildApiError::InvalidName)?;
        Ok(request
            .outputs
            .into_iter()
            .map(|output| GeneratedFileHandle {
                generated_file_id: self.graph.add_generated_file(
                    crate::graph::BuildGeneratedFileKind::CaptureOutput,
                    output,
                ),
            })
            .collect())
    }

    pub fn add_codegen(
        &mut self,
        request: CodegenRequest,
    ) -> Result<GeneratedFileHandle, BuildApiError> {
        let generated_file_id = self.graph.add_generated_file(
            crate::graph::BuildGeneratedFileKind::Write,
            request.output,
        );
        Ok(GeneratedFileHandle { generated_file_id })
    }

    pub fn project_install_file(
        &mut self,
        projection: GeneratedFileInstallProjection,
    ) -> Result<InstallHandle, BuildApiError> {
        self.install_file(InstallFileRequest {
            name: projection.install_name,
            path: projection.install_path,
            depends_on: Vec::new(),
        })
    }

    pub fn install_dir(
        &mut self,
        request: InstallDirRequest,
    ) -> Result<InstallHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let step_id = self.graph.add_step(
            crate::graph::BuildStepKind::Install,
            request.name.clone(),
        );
        for dependency in &request.depends_on {
            self.graph.add_step_dependency(step_id, *dependency);
        }
        let install_id = self.graph.add_install_with_target(
            crate::graph::BuildInstallKind::Directory,
            request.name.clone(),
            Some(crate::graph::BuildInstallTarget::DirectoryPath(
                request.path,
            )),
        );
        Ok(InstallHandle {
            install_id,
            step_id,
            name: request.name,
        })
    }

    pub fn add_module(
        &mut self,
        request: AddModuleRequest,
    ) -> Result<ModuleHandle, BuildApiError> {
        validate_build_name(&request.name).map_err(super::types::BuildApiError::InvalidName)?;
        let module_id = self
            .graph
            .add_module(crate::graph::BuildModuleKind::Source, request.root_module);
        Ok(ModuleHandle {
            module_id,
            name: request.name,
        })
    }

    pub fn artifact_link(
        &mut self,
        artifact_id: crate::graph::BuildArtifactId,
        linked_id: crate::graph::BuildArtifactId,
    ) {
        self.graph.add_artifact_link(artifact_id, linked_id);
    }

    pub fn artifact_import(
        &mut self,
        artifact_id: crate::graph::BuildArtifactId,
        module_id: crate::graph::BuildModuleId,
    ) {
        self.graph.add_artifact_module_import(artifact_id, module_id);
    }

    pub fn artifact_add_generated(
        &mut self,
        artifact_id: crate::graph::BuildArtifactId,
        generated_file_id: crate::graph::BuildGeneratedFileId,
    ) {
        self.graph
            .add_artifact_generated_file_input(artifact_id, generated_file_id);
    }

    pub fn run_add_arg(
        &mut self,
        step_id: crate::graph::BuildStepId,
        arg: crate::graph::BuildRunArg,
    ) {
        self.graph.run_config_mut(step_id).args.push(arg);
    }

    pub fn run_capture_stdout(
        &mut self,
        step_id: crate::graph::BuildStepId,
        output_name: impl Into<String>,
    ) -> GeneratedFileHandle {
        let generated_file_id = self.graph.add_generated_file(
            crate::graph::BuildGeneratedFileKind::CaptureOutput,
            output_name,
        );
        self.graph.run_config_mut(step_id).capture_stdout = Some(generated_file_id);
        GeneratedFileHandle { generated_file_id }
    }

    pub fn run_set_env(
        &mut self,
        step_id: crate::graph::BuildStepId,
        key: impl Into<String>,
        value: impl Into<String>,
    ) {
        self.graph
            .run_config_mut(step_id)
            .env
            .push((key.into(), value.into()));
    }

    pub fn step_attach(
        &mut self,
        step_id: crate::graph::BuildStepId,
        generated_file_id: crate::graph::BuildGeneratedFileId,
    ) {
        self.graph.add_step_attachment(step_id, generated_file_id);
    }

    pub fn dependency(
        &mut self,
        request: DependencyRequest,
    ) -> Result<DependencyHandle, BuildApiError> {
        validate_build_name(&request.alias).map_err(super::types::BuildApiError::InvalidName)?;
        let alias = request.alias;
        let package = request.package;
        let evaluation_mode = request.evaluation_mode;
        let surface = request.surface;
        let module_id = self.graph.add_module(
            crate::graph::BuildModuleKind::Imported,
            format!("{alias}:{package}"),
        );
        Ok(DependencyHandle {
            alias: alias.clone(),
            package: package.clone(),
            root_module_id: module_id,
            evaluation_mode,
            build: DependencyBuildHandle { alias, package },
            modules: surface
                .as_ref()
                .map(|surface| DependencyModuleSurfaceSet {
                    modules: surface.modules.clone(),
                })
                .unwrap_or_default(),
            artifacts: surface
                .as_ref()
                .map(|surface| DependencyArtifactSurfaceSet {
                    artifacts: surface.artifacts.clone(),
                })
                .unwrap_or_default(),
            steps: surface
                .as_ref()
                .map(|surface| DependencyStepSurfaceSet {
                    steps: surface.steps.clone(),
                })
                .unwrap_or_default(),
            generated_outputs: surface
                .as_ref()
                .map(|surface| DependencyGeneratedOutputSurfaceSet {
                    generated_outputs: surface.generated_outputs.clone(),
                })
                .unwrap_or_default(),
        })
    }
}
