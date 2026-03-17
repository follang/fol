use crate::{
    FrontendError, FrontendErrorKind, FrontendProfile, FrontendResult, FrontendWorkspace,
};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendBuildWorkflowMode {
    Compatibility,
    Modern,
    Hybrid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendBuildStep {
    Build,
    Run,
    Test,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendMemberBuildRoute {
    pub member_root: PathBuf,
    pub package_name: String,
    pub mode: FrontendBuildWorkflowMode,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendWorkspaceBuildRoute {
    pub requested_step: String,
    pub members: Vec<FrontendMemberBuildRoute>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendCompatibilityBuildRequest {
    pub requested_step: String,
    pub profile: FrontendProfile,
    pub run_args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontendMemberExecutionPlan {
    steps: Vec<FrontendMemberPlannedStep>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct FrontendMemberPlannedStep {
    name: String,
    execution: Option<FrontendStepExecutionKind>,
    selection: Option<crate::compile::FrontendArtifactExecutionSelection>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontendStepExecutionKind {
    Build,
    Run,
    Test,
    Check,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ResolvedWorkspaceStepExecution {
    execution: FrontendStepExecutionKind,
    selections: Vec<crate::compile::FrontendArtifactExecutionSelection>,
}

impl FrontendBuildWorkflowMode {
    pub fn from_package_build_mode(mode: fol_package::PackageBuildMode) -> Option<Self> {
        match mode {
            fol_package::PackageBuildMode::Empty
            | fol_package::PackageBuildMode::CompatibilityOnly => {
                Some(FrontendBuildWorkflowMode::Compatibility)
            }
            fol_package::PackageBuildMode::ModernOnly => Some(FrontendBuildWorkflowMode::Modern),
            fol_package::PackageBuildMode::Hybrid => Some(FrontendBuildWorkflowMode::Hybrid),
        }
    }
}

impl FrontendBuildStep {
    pub fn as_str(self) -> &'static str {
        match self {
            FrontendBuildStep::Build => "build",
            FrontendBuildStep::Run => "run",
            FrontendBuildStep::Test => "test",
            FrontendBuildStep::Check => "check",
        }
    }

    pub fn default_for_code_subcommand(command: &crate::CodeSubcommand) -> Self {
        match command {
            crate::CodeSubcommand::Build(_) => FrontendBuildStep::Build,
            crate::CodeSubcommand::Run(_) => FrontendBuildStep::Run,
            crate::CodeSubcommand::Test(_) => FrontendBuildStep::Test,
            crate::CodeSubcommand::Check(_) => FrontendBuildStep::Check,
            crate::CodeSubcommand::Emit(_) => FrontendBuildStep::Build,
        }
    }
}

pub fn plan_workspace_build_route(
    workspace: &FrontendWorkspace,
    requested_step: impl Into<String>,
) -> FrontendResult<FrontendWorkspaceBuildRoute> {
    let members = workspace
        .members
        .iter()
        .map(|member| {
            let metadata = fol_package::parse_package_metadata(&member.manifest_file)?;
            let mode = load_member_build_mode(&member.root.join("build.fol"))?;
            Ok(FrontendMemberBuildRoute {
                member_root: member.root.clone(),
                package_name: metadata.name,
                mode,
            })
        })
        .collect::<FrontendResult<Vec<_>>>()?;

    Ok(FrontendWorkspaceBuildRoute {
        requested_step: requested_step.into(),
        members,
    })
}

fn load_member_build_mode(build_path: &std::path::Path) -> FrontendResult<FrontendBuildWorkflowMode> {
    let build = fol_package::parse_package_build(build_path)?;
    FrontendBuildWorkflowMode::from_package_build_mode(build.mode()).ok_or_else(|| {
        FrontendError::new(
            FrontendErrorKind::Internal,
            format!(
                "workspace member build '{}' has an unmappable build mode",
                build_path.display()
            ),
        )
    })
}

pub fn execute_workspace_build_route(
    workspace: &FrontendWorkspace,
    config: &crate::FrontendConfig,
    request: &FrontendCompatibilityBuildRequest,
) -> FrontendResult<crate::FrontendCommandResult> {
    let route = plan_workspace_build_route(workspace, request.requested_step.clone())?;
    let member_plans = route
        .members
        .iter()
        .map(plan_member_execution)
        .collect::<FrontendResult<Vec<_>>>()?;
    let requested_step = request.requested_step.as_str();
    if !member_plans
        .iter()
        .any(|plan| plan.steps.iter().any(|step| step.name == requested_step))
    {
        return Err(unknown_workspace_build_step_error(requested_step, &route));
    }

    let resolved = resolve_requested_step_execution(requested_step, &member_plans)?;

    match resolved.execution {
        FrontendStepExecutionKind::Build => {
            if resolved.selections.is_empty() {
                crate::build_workspace_for_profile_with_config(workspace, config, request.profile)
            } else {
                crate::compile::build_selected_artifacts_for_profile_with_config(
                    workspace,
                    config,
                    request.profile,
                    &resolved.selections,
                )
            }
        }
        FrontendStepExecutionKind::Check => crate::check_workspace_with_config(workspace, config),
        FrontendStepExecutionKind::Run => match resolved.selections.as_slice() {
            [] => crate::run_workspace_with_args_and_config(workspace, config, &request.run_args),
            [selection] => crate::compile::run_selected_artifact_with_args_and_config(
                workspace,
                config,
                request.profile,
                selection,
                &request.run_args,
            ),
            selections => Err(FrontendError::new(
                FrontendErrorKind::InvalidInput,
                format!(
                    "workspace build execution step '{}' resolved to {} runnable artifacts",
                    requested_step,
                    selections.len()
                ),
            )),
        },
        FrontendStepExecutionKind::Test => {
            if resolved.selections.is_empty() {
                crate::test_workspace_with_config(workspace, config)
            } else {
                crate::compile::test_selected_artifacts_with_config(
                    workspace,
                    config,
                    request.profile,
                    &resolved.selections,
                )
            }
        }
    }
}

fn plan_member_execution(
    member: &FrontendMemberBuildRoute,
) -> FrontendResult<FrontendMemberExecutionPlan> {
    if let Some(plan) = plan_member_execution_from_build_source(member)? {
        return Ok(plan);
    }

    plan_member_default_execution(member)
}

fn plan_member_default_execution(
    member: &FrontendMemberBuildRoute,
) -> FrontendResult<FrontendMemberExecutionPlan> {
    let requested_step = fol_package::BuildRequestedStep::Default(fol_package::BuildDefaultStepKind::Check);
    let mut graph = fol_package::BuildGraph::new();
    let check = graph.add_step(
        fol_package::BuildStepKind::Check,
        requested_step.name().to_string(),
    );
    let mut steps = vec![FrontendMemberPlannedStep {
        name: requested_step.name().to_string(),
        execution: Some(FrontendStepExecutionKind::Check),
        selection: None,
    }];

    if let Some(root_module) = default_runnable_root_module(&member.member_root)? {
        let selection = crate::compile::FrontendArtifactExecutionSelection {
            package_root: member.member_root.clone(),
            label: member.package_name.clone(),
            root_module: Some(root_module.clone()),
        };
        let build = graph.add_step(fol_package::BuildStepKind::Default, "build");
        graph.add_step_dependency(check, build);

        let artifact = graph.add_artifact(fol_package::BuildArtifactKind::Executable, member.package_name.clone());
        let module = graph.add_module(fol_package::BuildModuleKind::Source, root_module);
        graph.add_artifact_module_input(artifact, module);

        let run = graph.add_step(fol_package::BuildStepKind::Run, "run");
        graph.add_step_dependency(run, build);
        let test = graph.add_step(fol_package::BuildStepKind::Test, "test");
        graph.add_step_dependency(test, build);

        for step in [build, run, test] {
            let order = fol_package::plan_step_order(&graph, step).map_err(|error| {
                FrontendError::new(
                    FrontendErrorKind::Internal,
                    format!(
                        "failed to plan build graph for package '{}': {error:?}",
                        member.package_name
                    ),
                )
            })?;
            if order.is_empty() {
                return Err(FrontendError::new(
                    FrontendErrorKind::Internal,
                    format!(
                        "planned build graph for package '{}' produced an empty step order",
                        member.package_name
                    ),
                ));
            }
        }

        steps.extend([
            FrontendMemberPlannedStep {
                name: "build".to_string(),
                execution: Some(FrontendStepExecutionKind::Build),
                selection: Some(selection.clone()),
            },
            FrontendMemberPlannedStep {
                name: "run".to_string(),
                execution: Some(FrontendStepExecutionKind::Run),
                selection: Some(selection.clone()),
            },
            FrontendMemberPlannedStep {
                name: "test".to_string(),
                execution: Some(FrontendStepExecutionKind::Test),
                selection: Some(selection),
            },
        ]);
    } else {
        let order = fol_package::plan_step_order(&graph, check).map_err(|error| {
            FrontendError::new(
                FrontendErrorKind::Internal,
                format!(
                    "failed to plan build graph for package '{}': {error:?}",
                    member.package_name
                ),
            )
        })?;
        if order.is_empty() {
            return Err(FrontendError::new(
                FrontendErrorKind::Internal,
                format!(
                    "planned build graph for package '{}' produced an empty step order",
                    member.package_name
                ),
            ));
        }
    }

    Ok(FrontendMemberExecutionPlan { steps })
}

fn plan_member_execution_from_build_source(
    member: &FrontendMemberBuildRoute,
) -> FrontendResult<Option<FrontendMemberExecutionPlan>> {
    let build_path = member.member_root.join("build.fol");
    let source = std::fs::read_to_string(&build_path).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::CommandFailed,
            format!("failed to read build file '{}': {error}", build_path.display()),
        )
    })?;
    let extracted = extract_build_program_from_source(member, &build_path, &source)?;
    if extracted.operations.is_empty() {
        return Ok(None);
    }
    let operations = extracted.operations.clone();

    let result = fol_package::evaluate_build_plan(&fol_package::BuildEvaluationRequest {
        package_root: member.member_root.display().to_string(),
        inputs: fol_package::BuildEvaluationInputs {
            working_directory: member.member_root.display().to_string(),
            ..fol_package::BuildEvaluationInputs::default()
        },
        operations,
    })
    .map_err(|error| FrontendError::new(FrontendErrorKind::InvalidInput, error.to_string()))?;

    let mut steps = fol_package::project_graph_steps(&result.graph)
        .into_iter()
        .map(|step| FrontendMemberPlannedStep {
            selection: extracted.selection_for_step(&step.name, step.default_kind),
            name: step.name,
            execution: step.default_kind.and_then(step_execution_kind_from_default),
        })
        .collect::<Vec<_>>();
    for step in extracted.synthesized_default_steps() {
        if !steps.iter().any(|existing| existing.name == step.name) {
            steps.push(step);
        }
    }
    if !steps.iter().any(|step| step.name == "check") {
        steps.push(FrontendMemberPlannedStep {
            name: "check".to_string(),
            execution: Some(FrontendStepExecutionKind::Check),
            selection: None,
        });
    }
    if steps.is_empty() {
        return Ok(None);
    }
    steps.sort_by(|left, right| left.name.cmp(&right.name));
    steps.dedup_by(|left, right| {
        left.name == right.name && left.execution == right.execution && left.selection == right.selection
    });
    Ok(Some(FrontendMemberExecutionPlan { steps }))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExtractedBuildProgram {
    operations: Vec<fol_package::BuildEvaluationOperation>,
    executable_artifacts: Vec<crate::compile::FrontendArtifactExecutionSelection>,
    test_artifacts: Vec<crate::compile::FrontendArtifactExecutionSelection>,
    run_steps: std::collections::BTreeMap<String, crate::compile::FrontendArtifactExecutionSelection>,
}

impl ExtractedBuildProgram {
    fn new() -> Self {
        Self {
            operations: Vec::new(),
            executable_artifacts: Vec::new(),
            test_artifacts: Vec::new(),
            run_steps: std::collections::BTreeMap::new(),
        }
    }

    fn selection_for_step(
        &self,
        step_name: &str,
        default_kind: Option<fol_package::BuildDefaultStepKind>,
    ) -> Option<crate::compile::FrontendArtifactExecutionSelection> {
        if let Some(selection) = self.run_steps.get(step_name) {
            return Some(selection.clone());
        }
        match default_kind {
            Some(fol_package::BuildDefaultStepKind::Build)
            | Some(fol_package::BuildDefaultStepKind::Run) => {
                single_selection(&self.executable_artifacts)
            }
            Some(fol_package::BuildDefaultStepKind::Test) => single_selection(&self.test_artifacts),
            _ => None,
        }
    }

    fn synthesized_default_steps(&self) -> Vec<FrontendMemberPlannedStep> {
        let mut steps = Vec::new();
        if let Some(selection) = single_selection(&self.executable_artifacts) {
            steps.push(FrontendMemberPlannedStep {
                name: "build".to_string(),
                execution: Some(FrontendStepExecutionKind::Build),
                selection: Some(selection.clone()),
            });
            steps.push(FrontendMemberPlannedStep {
                name: "run".to_string(),
                execution: Some(FrontendStepExecutionKind::Run),
                selection: Some(selection),
            });
        }
        if let Some(selection) = single_selection(&self.test_artifacts) {
            steps.push(FrontendMemberPlannedStep {
                name: "test".to_string(),
                execution: Some(FrontendStepExecutionKind::Test),
                selection: Some(selection),
            });
        }
        steps
    }
}

fn single_selection(
    selections: &[crate::compile::FrontendArtifactExecutionSelection],
) -> Option<crate::compile::FrontendArtifactExecutionSelection> {
    (selections.len() == 1).then(|| selections[0].clone())
}

fn extract_build_program_from_source(
    member: &FrontendMemberBuildRoute,
    build_path: &std::path::Path,
    source: &str,
) -> FrontendResult<ExtractedBuildProgram> {
    let Some((param_name, body, body_line)) = extract_build_body(source) else {
        return Ok(ExtractedBuildProgram::new());
    };
    let mut extracted = ExtractedBuildProgram::new();
    let mut executable_artifacts = std::collections::BTreeMap::new();
    let mut test_artifacts = std::collections::BTreeMap::new();
    for (offset, raw_line) in body.lines().enumerate() {
        let line_number = body_line + offset;
        let line = raw_line
            .split_once("//")
            .map_or(raw_line, |(prefix, _)| prefix)
            .trim()
            .trim_end_matches(';')
            .trim();
        if line.is_empty() || line == "return ." {
            continue;
        }
        let Some(call) = line.strip_prefix(&format!("{param_name}.")) else {
            continue;
        };
        let Some((method, raw_args)) = call.split_once('(') else {
            continue;
        };
        let args_text = raw_args.trim_end_matches(')').trim();
        let args = parse_build_string_args(args_text);
        let origin = fol_parser::ast::SyntaxOrigin {
            file: Some(build_path.display().to_string()),
            line: line_number,
            column: 1,
            length: raw_line.len(),
        };
        let kind = match (method.trim(), args.as_slice()) {
            ("standard_target", [name]) => fol_package::BuildEvaluationOperationKind::StandardTarget(
                fol_package::StandardTargetRequest::new(name.clone()),
            ),
            ("standard_optimize", [name]) => {
                fol_package::BuildEvaluationOperationKind::StandardOptimize(
                    fol_package::StandardOptimizeRequest::new(name.clone()),
                )
            }
            ("add_exe", [name, root_module]) => {
                executable_artifacts.insert(
                    name.clone(),
                    crate::compile::FrontendArtifactExecutionSelection {
                        package_root: member.member_root.clone(),
                        label: name.clone(),
                        root_module: Some(root_module.clone()),
                    },
                );
                fol_package::BuildEvaluationOperationKind::AddExe(fol_package::ExecutableRequest {
                    name: name.clone(),
                    root_module: root_module.clone(),
                })
            }
            ("add_static_lib", [name, root_module]) => {
                fol_package::BuildEvaluationOperationKind::AddStaticLib(
                    fol_package::StaticLibraryRequest {
                        name: name.clone(),
                        root_module: root_module.clone(),
                    },
                )
            }
            ("add_shared_lib", [name, root_module]) => {
                fol_package::BuildEvaluationOperationKind::AddSharedLib(
                    fol_package::SharedLibraryRequest {
                        name: name.clone(),
                        root_module: root_module.clone(),
                    },
                )
            }
            ("add_test", [name, root_module]) => {
                test_artifacts.insert(
                    name.clone(),
                    crate::compile::FrontendArtifactExecutionSelection {
                        package_root: member.member_root.clone(),
                        label: name.clone(),
                        root_module: Some(root_module.clone()),
                    },
                );
                fol_package::BuildEvaluationOperationKind::AddTest(
                    fol_package::TestArtifactRequest {
                        name: name.clone(),
                        root_module: root_module.clone(),
                    },
                )
            }
            ("step", [name, depends_on @ ..]) => fol_package::BuildEvaluationOperationKind::Step(
                fol_package::BuildEvaluationStepRequest {
                    name: name.clone(),
                    depends_on: depends_on.to_vec(),
                },
            ),
            ("add_run", [name, artifact, depends_on @ ..]) => {
                if let Some(selection) = executable_artifacts.get(artifact) {
                    extracted.run_steps.insert(name.clone(), selection.clone());
                }
                fol_package::BuildEvaluationOperationKind::AddRun(
                    fol_package::BuildEvaluationRunRequest {
                        name: name.clone(),
                        artifact: artifact.clone(),
                        depends_on: depends_on.to_vec(),
                    },
                )
            }
            ("install", [name, artifact]) => {
                fol_package::BuildEvaluationOperationKind::InstallArtifact(
                    fol_package::BuildEvaluationInstallArtifactRequest {
                        name: name.clone(),
                        artifact: artifact.clone(),
                    },
                )
            }
            ("dependency", [alias, package]) => {
                fol_package::BuildEvaluationOperationKind::Dependency(
                    fol_package::DependencyRequest {
                        alias: alias.clone(),
                        package: package.clone(),
                    },
                )
            }
            _ => {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "unsupported build API call in '{}': {}",
                        build_path.display(),
                        raw_line.trim()
                    ),
                ))
            }
        };
        extracted.operations.push(fol_package::BuildEvaluationOperation {
            origin: Some(origin),
            kind,
        });
    }
    extracted.executable_artifacts = executable_artifacts.into_values().collect();
    extracted.test_artifacts = test_artifacts.into_values().collect();
    Ok(extracted)
}

fn extract_build_body(source: &str) -> Option<(String, String, usize)> {
    let start = source.find("def build(")?;
    let rest = &source[start + "def build(".len()..];
    let param_end = rest.find(':')?;
    let param_name = rest[..param_end].trim().to_string();
    if param_name.is_empty() {
        return None;
    }
    let after_equals = rest.find('=')?;
    let body_start = start + "def build(".len() + after_equals + 1;
    let body_source = source[body_start..].trim_start();
    let body_line = source[..body_start].chars().filter(|ch| *ch == '\n').count() + 1;
    if let Some(stripped) = body_source.strip_prefix('{') {
        let block_end = stripped.rfind('}')?;
        return Some((param_name, stripped[..block_end].to_string(), body_line + 1));
    }
    Some((param_name, body_source.trim_end_matches(';').to_string(), body_line))
}

fn parse_build_string_args(args: &str) -> Vec<String> {
    args.split(',')
        .map(str::trim)
        .filter_map(|arg| arg.strip_prefix('"').and_then(|arg| arg.strip_suffix('"')))
        .map(str::to_string)
        .collect()
}

fn step_execution_kind_from_default(
    kind: fol_package::BuildDefaultStepKind,
) -> Option<FrontendStepExecutionKind> {
    match kind {
        fol_package::BuildDefaultStepKind::Build => Some(FrontendStepExecutionKind::Build),
        fol_package::BuildDefaultStepKind::Run => Some(FrontendStepExecutionKind::Run),
        fol_package::BuildDefaultStepKind::Test => Some(FrontendStepExecutionKind::Test),
        fol_package::BuildDefaultStepKind::Check => Some(FrontendStepExecutionKind::Check),
        fol_package::BuildDefaultStepKind::Install => None,
    }
}

fn resolve_requested_step_execution(
    requested_step: &str,
    member_plans: &[FrontendMemberExecutionPlan],
) -> FrontendResult<ResolvedWorkspaceStepExecution> {
    let mut resolved = None;
    let mut selections = Vec::new();
    let mut saw_untargeted = false;
    for step in member_plans
        .iter()
        .flat_map(|plan| plan.steps.iter())
        .filter(|step| step.name == requested_step)
    {
        let Some(execution_kind) = step.execution else {
            continue;
        };
        match &step.selection {
            Some(selection) => {
                if saw_untargeted {
                    return Err(FrontendError::new(
                        FrontendErrorKind::InvalidInput,
                        format!(
                            "workspace build execution step '{}' mixes targeted and untargeted members",
                            requested_step
                        ),
                    ));
                }
                if !selections.contains(selection) {
                    selections.push(selection.clone());
                }
            }
            None => {
                if !selections.is_empty() {
                    return Err(FrontendError::new(
                        FrontendErrorKind::InvalidInput,
                        format!(
                            "workspace build execution step '{}' mixes targeted and untargeted members",
                            requested_step
                        ),
                    ));
                }
                saw_untargeted = true;
            }
        }
        match resolved {
            None => resolved = Some(execution_kind),
            Some(current) if current == execution_kind => {}
            Some(current) => {
                return Err(FrontendError::new(
                    FrontendErrorKind::InvalidInput,
                    format!(
                        "workspace build execution step '{}' resolves to incompatible execution kinds: {:?} and {:?}",
                        requested_step, current, execution_kind
                    ),
                ))
            }
        }
    }

    resolved
        .map(|execution| ResolvedWorkspaceStepExecution { execution, selections })
        .ok_or_else(|| {
            FrontendError::new(
                FrontendErrorKind::InvalidInput,
                format!("workspace build execution does not support step '{requested_step}'"),
            )
        })
}

fn default_runnable_root_module(member_root: &std::path::Path) -> FrontendResult<Option<String>> {
    let main_path = member_root.join("src/main.fol");
    if !main_path.is_file() {
        return Ok(None);
    }
    let relative = main_path.strip_prefix(member_root).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::Internal,
            format!(
                "failed to compute build graph root module for '{}': {error}",
                member_root.display()
            ),
        )
    })?;
    Ok(Some(relative.to_string_lossy().replace('\\', "/")))
}

fn unknown_workspace_build_step_error(
    requested_step: &str,
    route: &FrontendWorkspaceBuildRoute,
) -> FrontendError {
    let members = route
        .members
        .iter()
        .map(|member| member.package_name.as_str())
        .collect::<Vec<_>>();
    FrontendError::new(
        FrontendErrorKind::InvalidInput,
        format!(
            "workspace build execution does not define step '{requested_step}' for workspace members: {}",
            members.join(", ")
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        execute_workspace_build_route, plan_member_execution, plan_workspace_build_route,
        FrontendBuildStep, FrontendBuildWorkflowMode, FrontendCompatibilityBuildRequest,
        FrontendMemberBuildRoute, FrontendWorkspaceBuildRoute,
    };
    use crate::{FrontendConfig, FrontendProfile, FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::PathBuf};

    fn compatibility_workspace_fixture(label: &str) -> FrontendWorkspace {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_{label}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        let app = root.join("app");
        fs::create_dir_all(app.join("src")).unwrap();
        fs::write(app.join("package.yaml"), "name: app\nversion: 0.1.0\n").unwrap();
        fs::write(app.join("build.fol"), "def root: loc = \"src\";\n").unwrap();
        fs::write(
            app.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(app)],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        }
    }

    #[test]
    fn workflow_mode_maps_package_build_modes_into_frontend_route_modes() {
        assert_eq!(
            FrontendBuildWorkflowMode::from_package_build_mode(
                fol_package::PackageBuildMode::Empty
            ),
            Some(FrontendBuildWorkflowMode::Compatibility)
        );
        assert_eq!(
            FrontendBuildWorkflowMode::from_package_build_mode(
                fol_package::PackageBuildMode::CompatibilityOnly
            ),
            Some(FrontendBuildWorkflowMode::Compatibility)
        );
        assert_eq!(
            FrontendBuildWorkflowMode::from_package_build_mode(
                fol_package::PackageBuildMode::ModernOnly
            ),
            Some(FrontendBuildWorkflowMode::Modern)
        );
        assert_eq!(
            FrontendBuildWorkflowMode::from_package_build_mode(
                fol_package::PackageBuildMode::Hybrid
            ),
            Some(FrontendBuildWorkflowMode::Hybrid)
        );
    }

    #[test]
    fn member_build_route_keeps_package_name_and_workflow_mode() {
        let route = FrontendMemberBuildRoute {
            member_root: PathBuf::from("/tmp/demo/app"),
            package_name: "app".to_string(),
            mode: FrontendBuildWorkflowMode::Hybrid,
        };

        assert_eq!(route.member_root, PathBuf::from("/tmp/demo/app"));
        assert_eq!(route.package_name, "app");
        assert_eq!(route.mode, FrontendBuildWorkflowMode::Hybrid);
    }

    #[test]
    fn workspace_build_route_keeps_requested_step_and_members() {
        let route = FrontendWorkspaceBuildRoute {
            requested_step: "build".to_string(),
            members: vec![FrontendMemberBuildRoute {
                member_root: PathBuf::from("/tmp/demo/app"),
                package_name: "app".to_string(),
                mode: FrontendBuildWorkflowMode::Compatibility,
            }],
        };

        assert_eq!(route.requested_step, "build");
        assert_eq!(route.members.len(), 1);
        assert_eq!(route.members[0].package_name, "app");
    }

    #[test]
    fn build_steps_keep_stable_cli_facing_names() {
        assert_eq!(FrontendBuildStep::Build.as_str(), "build");
        assert_eq!(FrontendBuildStep::Run.as_str(), "run");
        assert_eq!(FrontendBuildStep::Test.as_str(), "test");
        assert_eq!(FrontendBuildStep::Check.as_str(), "check");
    }

    #[test]
    fn build_steps_map_code_subcommands_to_default_requested_steps() {
        assert_eq!(
            FrontendBuildStep::default_for_code_subcommand(&crate::CodeSubcommand::Build(
                crate::BuildCommand::default()
            )),
            FrontendBuildStep::Build
        );
        assert_eq!(
            FrontendBuildStep::default_for_code_subcommand(&crate::CodeSubcommand::Run(
                crate::RunCommand::default()
            )),
            FrontendBuildStep::Run
        );
        assert_eq!(
            FrontendBuildStep::default_for_code_subcommand(&crate::CodeSubcommand::Test(
                crate::TestCommand::default()
            )),
            FrontendBuildStep::Test
        );
        assert_eq!(
            FrontendBuildStep::default_for_code_subcommand(&crate::CodeSubcommand::Check(
                crate::CheckCommand::default()
            )),
            FrontendBuildStep::Check
        );
    }

    #[test]
    fn workspace_route_planner_classifies_workspace_members_by_build_mode() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_plan_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        let compatibility = root.join("compat");
        let hybrid = root.join("hybrid");
        let modern = root.join("modern");
        for package in [&compatibility, &hybrid, &modern] {
            fs::create_dir_all(package.join("src")).unwrap();
            let name = package.file_name().unwrap().to_string_lossy();
            fs::write(
                package.join("package.yaml"),
                format!("name: {name}\nversion: 0.1.0\n"),
            )
            .unwrap();
            fs::write(
                package.join("src/main.fol"),
                "fun[] main(): int = {\n    return 0\n}\n",
            )
            .unwrap();
        }
        fs::write(compatibility.join("build.fol"), "def root: loc = \"src\";\n").unwrap();
        fs::write(
            hybrid.join("build.fol"),
            "def root: loc = \"src\";\ndef build(graph: int): int = graph;\n",
        )
        .unwrap();
        fs::write(
            modern.join("build.fol"),
            "def build(graph: int): int = graph;\n",
        )
        .unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![
                PackageRoot::new(compatibility.clone()),
                PackageRoot::new(hybrid.clone()),
                PackageRoot::new(modern.clone()),
            ],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let route = plan_workspace_build_route(&workspace, "build").unwrap();

        assert_eq!(route.requested_step, "build");
        assert_eq!(route.members.len(), 3);
        assert_eq!(route.members[0].mode, FrontendBuildWorkflowMode::Compatibility);
        assert_eq!(route.members[1].mode, FrontendBuildWorkflowMode::Hybrid);
        assert_eq!(route.members[2].mode, FrontendBuildWorkflowMode::Modern);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn compatibility_executor_maps_build_steps_back_onto_existing_workspace_commands() {
        let workspace = compatibility_workspace_fixture("compat_exec_build");

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "build".to_string(),
                profile: crate::FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(result.command, "build");
        assert!(result.summary.contains("built 1 workspace package(s) into "));

        fs::remove_dir_all(&workspace.root.root).ok();
    }

    #[test]
    fn compatibility_executor_routes_run_steps_with_arguments() {
        let workspace = compatibility_workspace_fixture("compat_exec_run");

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "run".to_string(),
                profile: crate::FrontendProfile::Debug,
                run_args: vec!["--demo".to_string()],
            },
        )
        .unwrap();

        assert_eq!(result.command, "run");
        assert!(result.summary.contains("ran "));

        fs::remove_dir_all(&workspace.root.root).ok();
    }

    #[test]
    fn compatibility_executor_rejects_unknown_named_steps() {
        let workspace = compatibility_workspace_fixture("compat_exec_unknown");

        let error = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "docs".to_string(),
                profile: crate::FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect_err("unknown compatibility step should fail");

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(
            error
                .message()
                .contains("workspace build execution does not define step 'docs'")
        );

        fs::remove_dir_all(&workspace.root.root).ok();
    }

    #[test]
    fn build_body_step_calls_flow_into_member_execution_plans() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_body_steps_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "def build(graph: int): int = {\n",
                "    graph.step(\"docs\");\n",
                "    graph.step(\"lint\");\n",
                "    return .\n",
                "}\n",
            ),
        )
        .unwrap();

        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .unwrap();

        assert!(plan.steps.iter().any(|step| step.name == "docs"));
        assert!(plan.steps.iter().any(|step| step.name == "lint"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn build_body_step_dependencies_are_accepted_during_member_planning() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_body_step_deps_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "def build(graph: int): int = {\n",
                "    graph.add_exe(\"app\", \"src/main.fol\");\n",
                "    graph.step(\"gen\");\n",
                "    graph.step(\"docs\", \"gen\");\n",
                "    graph.add_run(\"run\", \"app\", \"docs\");\n",
                "    return .\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .unwrap();

        assert!(plan.steps.iter().any(|step| step.name == "gen"));
        assert!(plan.steps.iter().any(|step| step.name == "docs"));
        assert!(plan.steps.iter().any(|step| step.name == "run"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn custom_build_steps_execute_through_build_dispatch() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_custom_step_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "def build(graph: int): int = {\n",
                "    graph.step(\"docs\");\n",
                "    return .\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(root.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "docs".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect("custom build-like step should dispatch through build execution");

        assert_eq!(result.command, "build");
        assert!(result.summary.contains("built 1 workspace package(s) into "));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn custom_run_steps_execute_through_run_dispatch() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_custom_run_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "def build(graph: int): int = {\n",
                "    graph.add_exe(\"app\", \"src/main.fol\");\n",
                "    graph.add_run(\"serve\", \"app\");\n",
                "    return .\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(root.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "serve".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect("custom run step should dispatch through run execution");

        assert_eq!(result.command, "run");
        assert!(result.summary.contains("ran "));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn configured_executable_roots_drive_default_build_and_run_steps() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_targeted_root_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "def build(graph: int): int = {\n",
                "    graph.add_exe(\"app\", \"src/app.fol\");\n",
                "    graph.add_run(\"serve\", \"app\");\n",
                "    return .\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(root.join("src/main.fol"), "var[exp] ignored: int = 1;\n").unwrap();
        fs::write(
            root.join("src/app.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(root.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let build_result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "build".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect("configured add_exe root should drive default build execution");
        assert_eq!(build_result.command, "build");
        assert!(build_result.summary.contains("built 1 workspace package(s) into "));

        let run_result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "serve".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect("configured add_exe root should drive custom run execution");
        assert_eq!(run_result.command, "run");
        assert!(run_result.summary.contains("ran "));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_executor_routes_modern_build_members_through_default_graph_planning() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_modern_exec_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            "def build(graph: int): int = graph;\n",
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(root.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "build".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(result.command, "build");
        assert!(result.summary.contains("built 1 workspace package(s) into "));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_executor_routes_modern_check_steps_even_without_a_runnable_binary() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_modern_check_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: demo\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            "def build(graph: int): int = graph;\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.fol"), "var[exp] answer: int = 42;\n").unwrap();
        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(root.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendCompatibilityBuildRequest {
                requested_step: "check".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(result.command, "check");
        assert!(result.summary.contains("checked 1 workspace package(s)"));

        fs::remove_dir_all(root).ok();
    }
}
