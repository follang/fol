use crate::{FrontendError, FrontendErrorKind, FrontendProfile, FrontendResult, FrontendWorkspace};
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendBuildWorkflowMode {
    Modern,
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
pub struct FrontendWorkspaceBuildRequest {
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
    ambiguous_selection: bool,
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
            fol_package::PackageBuildMode::ModernOnly => Some(FrontendBuildWorkflowMode::Modern),
            _ => None,
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

pub fn requested_workspace_step(
    command: &crate::CodeSubcommand,
    override_step: Option<&str>,
) -> String {
    override_step.map(str::to_string).unwrap_or_else(|| {
        FrontendBuildStep::default_for_code_subcommand(command)
            .as_str()
            .to_string()
    })
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

fn load_member_build_mode(
    build_path: &std::path::Path,
) -> FrontendResult<FrontendBuildWorkflowMode> {
    let mode = fol_package::parse_package_build_mode(build_path)?;
    FrontendBuildWorkflowMode::from_package_build_mode(mode).ok_or_else(|| {
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
    request: &FrontendWorkspaceBuildRequest,
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
    plan_member_execution_from_build_source(member)
}

fn plan_member_execution_from_build_source(
    member: &FrontendMemberBuildRoute,
) -> FrontendResult<FrontendMemberExecutionPlan> {
    let build_path = member.member_root.join("build.fol");
    let source = std::fs::read_to_string(&build_path).map_err(|error| {
        FrontendError::new(
            FrontendErrorKind::CommandFailed,
            format!(
                "failed to read build file '{}': {error}",
                build_path.display()
            ),
        )
    })?;
    let evaluated = fol_package::evaluate_build_source(
        &fol_package::BuildEvaluationRequest {
            package_root: member.member_root.display().to_string(),
            inputs: fol_package::BuildEvaluationInputs {
                working_directory: member.member_root.display().to_string(),
                ..fol_package::BuildEvaluationInputs::default()
            },
            operations: Vec::new(),
        },
        &build_path,
        &source,
    )
    .map_err(|error| FrontendError::new(FrontendErrorKind::InvalidInput, error.to_string()))?;
    let Some(evaluated) = evaluated else {
        return Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            format!(
                "workspace member '{}' must declare a semantic `pro[] build(graph: Graph): non` entry",
                member.package_name
            ),
        ));
    };

    plan_member_execution_from_graph(
        member,
        &evaluated.result.graph,
        &evaluated.evaluated,
        true,
    )
}

fn plan_member_execution_from_graph(
    member: &FrontendMemberBuildRoute,
    graph: &fol_package::BuildGraph,
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
    synthesize_defaults: bool,
) -> FrontendResult<FrontendMemberExecutionPlan> {
    let mut steps = fol_package::project_graph_steps(graph)
        .into_iter()
        .map(|step| FrontendMemberPlannedStep {
            selection: selection_for_step(member, evaluated, &step.name, step.default_kind),
            ambiguous_selection: step_has_ambiguous_default_selection(evaluated, step.default_kind),
            name: step.name,
            execution: step.default_kind.and_then(step_execution_kind_from_default),
        })
        .collect::<Vec<_>>();
    if synthesize_defaults {
        for step in synthesized_default_steps(member, evaluated) {
            if !steps.iter().any(|existing| existing.name == step.name) {
                steps.push(step);
            }
        }
    }
    if !steps.iter().any(|step| step.name == "check") {
        steps.push(FrontendMemberPlannedStep {
            name: "check".to_string(),
            execution: Some(FrontendStepExecutionKind::Check),
            selection: None,
            ambiguous_selection: false,
        });
    }
    if steps.is_empty() {
        return Err(FrontendError::new(
            FrontendErrorKind::Internal,
            format!(
                "workspace member '{}' produced an empty graph-driven step plan",
                member.package_name
            ),
        ));
    }
    steps.sort_by(|left, right| left.name.cmp(&right.name));
    steps.dedup_by(|left, right| {
        left.name == right.name
            && left.execution == right.execution
            && left.selection == right.selection
    });
    Ok(FrontendMemberExecutionPlan { steps })
}

fn selection_for_step(
    member: &FrontendMemberBuildRoute,
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
    step_name: &str,
    default_kind: Option<fol_package::BuildDefaultStepKind>,
) -> Option<crate::compile::FrontendArtifactExecutionSelection> {
    if let Some(binding) = evaluated
        .step_bindings
        .iter()
        .find(|binding| binding.step_name == step_name)
    {
        if let Some(artifact_name) = &binding.artifact_name {
            return evaluated
                .artifacts
                .iter()
                .find(|artifact| artifact.name == *artifact_name)
                .map(|artifact| artifact_selection(member, artifact));
        }
    }
    if let Some(artifact) = evaluated
        .artifacts
        .iter()
        .find(|artifact| artifact.name == step_name)
    {
        return Some(artifact_selection(member, artifact));
    }
    match default_kind {
        Some(fol_package::BuildDefaultStepKind::Build)
        | Some(fol_package::BuildDefaultStepKind::Run) => single_selection(
            member,
            evaluated,
            fol_package::build_runtime::BuildRuntimeArtifactKind::Executable,
        ),
        Some(fol_package::BuildDefaultStepKind::Test) => single_selection(
            member,
            evaluated,
            fol_package::build_runtime::BuildRuntimeArtifactKind::Test,
        ),
        _ => None,
    }
}

fn synthesized_default_steps(
    member: &FrontendMemberBuildRoute,
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
) -> Vec<FrontendMemberPlannedStep> {
    let mut steps = Vec::new();
    let executable_count = artifact_count(
        evaluated,
        fol_package::build_runtime::BuildRuntimeArtifactKind::Executable,
    );
    if executable_count > 0 {
        let selection = single_selection(
            member,
            evaluated,
            fol_package::build_runtime::BuildRuntimeArtifactKind::Executable,
        );
        steps.push(FrontendMemberPlannedStep {
            name: "build".to_string(),
            execution: Some(FrontendStepExecutionKind::Build),
            selection: selection.clone(),
            ambiguous_selection: executable_count > 1,
        });
        steps.push(FrontendMemberPlannedStep {
            name: "run".to_string(),
            execution: Some(FrontendStepExecutionKind::Run),
            selection,
            ambiguous_selection: executable_count > 1,
        });
    }
    let test_count = artifact_count(
        evaluated,
        fol_package::build_runtime::BuildRuntimeArtifactKind::Test,
    );
    if test_count > 0 {
        let selection = single_selection(
            member,
            evaluated,
            fol_package::build_runtime::BuildRuntimeArtifactKind::Test,
        );
        steps.push(FrontendMemberPlannedStep {
            name: "test".to_string(),
            execution: Some(FrontendStepExecutionKind::Test),
            selection,
            ambiguous_selection: test_count > 1,
        });
    }
    steps
}

fn step_has_ambiguous_default_selection(
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
    default_kind: Option<fol_package::BuildDefaultStepKind>,
) -> bool {
    match default_kind {
        Some(fol_package::BuildDefaultStepKind::Build)
        | Some(fol_package::BuildDefaultStepKind::Run) => {
            artifact_count(
                evaluated,
                fol_package::build_runtime::BuildRuntimeArtifactKind::Executable,
            ) > 1
        }
        Some(fol_package::BuildDefaultStepKind::Test) => {
            artifact_count(
                evaluated,
                fol_package::build_runtime::BuildRuntimeArtifactKind::Test,
            ) > 1
        }
        _ => false,
    }
}

fn artifact_count(
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
    kind: fol_package::build_runtime::BuildRuntimeArtifactKind,
) -> usize {
    evaluated
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .count()
}

fn single_selection(
    member: &FrontendMemberBuildRoute,
    evaluated: &fol_package::build_eval::EvaluatedBuildProgram,
    kind: fol_package::build_runtime::BuildRuntimeArtifactKind,
) -> Option<crate::compile::FrontendArtifactExecutionSelection> {
    let artifacts = evaluated
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind)
        .collect::<Vec<_>>();
    (artifacts.len() == 1).then(|| artifact_selection(member, artifacts[0]))
}

fn artifact_selection(
    member: &FrontendMemberBuildRoute,
    artifact: &fol_package::build_runtime::BuildRuntimeArtifact,
) -> crate::compile::FrontendArtifactExecutionSelection {
    crate::compile::FrontendArtifactExecutionSelection {
        package_root: member.member_root.clone(),
        label: artifact.name.clone(),
        root_module: Some(artifact.root_module.clone()),
    }
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
        if step.ambiguous_selection {
            return Err(FrontendError::new(
                FrontendErrorKind::InvalidInput,
                format!(
                    "workspace build execution step '{}' matches multiple artifacts and requires an explicit named step",
                    requested_step
                ),
            ));
        }
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
        .map(|execution| ResolvedWorkspaceStepExecution {
            execution,
            selections,
        })
        .ok_or_else(|| {
            FrontendError::new(
                FrontendErrorKind::InvalidInput,
                format!("workspace build execution does not support step '{requested_step}'"),
            )
        })
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
        FrontendBuildStep, FrontendBuildWorkflowMode, FrontendMemberBuildRoute,
        FrontendStepExecutionKind, FrontendWorkspaceBuildRequest, FrontendWorkspaceBuildRoute,
    };
    use crate::{FrontendConfig, FrontendProfile, FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::PathBuf};

    fn absorbed_build_workspace_fixture(label: &str) -> FrontendWorkspace {
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
        fs::write(
            app.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"app\", \"src/main.fol\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
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
            None
        );
        assert_eq!(
            FrontendBuildWorkflowMode::from_package_build_mode(
                fol_package::PackageBuildMode::CompatibilityOnly
            ),
            None
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
            None
        );
    }

    #[test]
    fn member_build_route_keeps_package_name_and_workflow_mode() {
        let route = FrontendMemberBuildRoute {
            member_root: PathBuf::from("/tmp/demo/app"),
            package_name: "app".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        };

        assert_eq!(route.member_root, PathBuf::from("/tmp/demo/app"));
        assert_eq!(route.package_name, "app");
        assert_eq!(route.mode, FrontendBuildWorkflowMode::Modern);
    }

    #[test]
    fn workspace_build_route_keeps_requested_step_and_members() {
        let route = FrontendWorkspaceBuildRoute {
            requested_step: "build".to_string(),
            members: vec![FrontendMemberBuildRoute {
                member_root: PathBuf::from("/tmp/demo/app"),
                package_name: "app".to_string(),
                mode: FrontendBuildWorkflowMode::Modern,
            }],
        };

        assert_eq!(route.requested_step, "build");
        assert_eq!(route.members.len(), 1);
        assert_eq!(route.members[0].package_name, "app");
    }

    #[test]
    fn shared_graph_projection_helper_keeps_graph_steps_and_synthesizes_check() {
        let mut graph = fol_package::BuildGraph::new();
        graph.add_step(fol_package::BuildStepKind::Default, "build");
        let member = FrontendMemberBuildRoute {
            member_root: PathBuf::from("/tmp/demo/app"),
            package_name: "app".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        };
        let evaluated = fol_package::build_eval::EvaluatedBuildProgram {
            program: fol_package::BuildRuntimeProgram::new(
                fol_package::BuildExecutionRepresentation::RestrictedRuntimeIr,
            ),
            artifacts: Vec::new(),
            generated_files: Vec::new(),
            dependencies: Vec::new(),
            dependency_queries: Vec::new(),
            step_bindings: Vec::new(),
            result: fol_package::BuildEvaluationResult::new(
                fol_package::BuildEvaluationBoundary::GraphConstructionSubset,
                fol_package::canonical_graph_construction_capabilities(),
                "/tmp/demo/app",
                fol_package::BuildOptionDeclarationSet::new(),
                fol_package::ResolvedBuildOptionSet::new(),
                Vec::new(),
                graph.clone(),
            ),
        };

        let plan = super::plan_member_execution_from_graph(&member, &graph, &evaluated, false)
            .expect("graph projection should succeed");

        assert!(plan.steps.iter().any(|step| step.name == "build"));
        assert!(plan.steps.iter().any(|step| step.name == "check"));
    }

    #[test]
    fn semantic_member_planning_uses_graph_projected_build_run_and_check_steps() {
        let workspace = absorbed_build_workspace_fixture("compat_graph_plan");

        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: workspace.members[0].root.clone(),
            package_name: "app".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("semantic member planning should succeed");

        assert!(plan.steps.iter().any(|step| step.name == "build"));
        assert!(plan.steps.iter().any(|step| step.name == "run"));
        assert!(plan.steps.iter().any(|step| step.name == "check"));

        fs::remove_dir_all(&workspace.root.root).ok();
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
    fn requested_workspace_step_prefers_explicit_override_and_falls_back_to_command_default() {
        let build = crate::CodeSubcommand::Build(crate::BuildCommand::default());
        let run = crate::CodeSubcommand::Run(crate::RunCommand::default());

        assert_eq!(super::requested_workspace_step(&build, None), "build");
        assert_eq!(super::requested_workspace_step(&run, None), "run");
        assert_eq!(
            super::requested_workspace_step(&build, Some("docs")),
            "docs"
        );
    }

    #[test]
    fn workspace_route_planner_accepts_only_semantic_members() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_plan_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        let modern = root.join("modern");
        fs::create_dir_all(modern.join("src")).unwrap();
        fs::write(
            modern.join("package.yaml"),
            "name: modern\nversion: 0.1.0\n",
        )
        .unwrap();
        fs::write(
            modern.join("build.fol"),
            "pro[] build(graph: Graph): non = graph;\n",
        )
        .unwrap();
        fs::write(
            modern.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        let workspace = FrontendWorkspace {
            root: WorkspaceRoot::new(root.clone()),
            members: vec![PackageRoot::new(modern.clone())],
            std_root_override: None,
            package_store_root_override: None,
            build_root: root.join(".fol/build"),
            cache_root: root.join(".fol/cache"),
            git_cache_root: root.join(".fol/cache/git"),
        };

        let route = plan_workspace_build_route(&workspace, "build").unwrap();

        assert_eq!(route.requested_step, "build");
        assert_eq!(route.members.len(), 1);
        assert_eq!(route.members[0].mode, FrontendBuildWorkflowMode::Modern);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_route_planner_rejects_old_compatibility_members() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_old_build_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: old\nversion: 0.1.0\n").unwrap();
        fs::write(root.join("build.fol"), "def root: loc = \"src\";\n").unwrap();

        let error = plan_workspace_build_route(
            &FrontendWorkspace {
                root: WorkspaceRoot::new(root.clone()),
                members: vec![PackageRoot::new(root.clone())],
                std_root_override: None,
                package_store_root_override: None,
                build_root: root.join(".fol/build"),
                cache_root: root.join(".fol/cache"),
                git_cache_root: root.join(".fol/cache/git"),
            },
            "build",
        )
        .expect_err("old compatibility-only build should be rejected");

        assert_eq!(error.kind(), crate::FrontendErrorKind::Internal);
        assert!(error.message().contains("unmappable build mode"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_route_planner_rejects_broken_modern_builds() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_broken_modern_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: modern\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            "pro[] build(graph: Graph): non = {\n",
        )
        .unwrap();

        let error = plan_workspace_build_route(
            &FrontendWorkspace {
                root: WorkspaceRoot::new(root.clone()),
                members: vec![PackageRoot::new(root.clone())],
                std_root_override: None,
                package_store_root_override: None,
                build_root: root.join(".fol/build"),
                cache_root: root.join(".fol/cache"),
                git_cache_root: root.join(".fol/cache/git"),
            },
            "build",
        )
        .expect_err("broken modern-only build should stay a parse failure");

        assert_eq!(error.kind(), crate::FrontendErrorKind::CommandFailed);
        assert!(error
            .message()
            .contains("package loader could not parse package build file"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn modern_members_plan_custom_steps_without_compatibility_controls() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_modern_custom_steps_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time before epoch")
                .as_nanos()
        ));
        fs::create_dir_all(root.join("src")).unwrap();
        fs::write(root.join("package.yaml"), "name: modern\nversion: 0.1.0\n").unwrap();
        fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(graph: Graph): non = {\n",
                "    graph.step(\"docs\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        let route = plan_workspace_build_route(
            &FrontendWorkspace {
                root: WorkspaceRoot::new(root.clone()),
                members: vec![PackageRoot::new(root.clone())],
                std_root_override: None,
                package_store_root_override: None,
                build_root: root.join(".fol/build"),
                cache_root: root.join(".fol/cache"),
                git_cache_root: root.join(".fol/cache/git"),
            },
            "docs",
        )
        .expect("modern workspace route should classify successfully");
        assert_eq!(route.members[0].mode, FrontendBuildWorkflowMode::Modern);

        let plan = plan_member_execution(&route.members[0])
            .expect("modern member should plan custom graph steps");
        let docs = plan
            .steps
            .iter()
            .find(|step| step.name == "docs")
            .expect("modern member should keep the custom docs step");
        assert_eq!(docs.execution, Some(FrontendStepExecutionKind::Build));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn absorbed_build_executor_maps_build_steps_back_onto_existing_workspace_commands() {
        let workspace = absorbed_build_workspace_fixture("compat_exec_build");

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendWorkspaceBuildRequest {
                requested_step: "build".to_string(),
                profile: crate::FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .unwrap();

        assert_eq!(result.command, "build");
        assert!(result
            .summary
            .contains("built 1 workspace package(s) into "));

        fs::remove_dir_all(&workspace.root.root).ok();
    }

    #[test]
    fn absorbed_build_executor_routes_run_steps_with_arguments() {
        let workspace = absorbed_build_workspace_fixture("compat_exec_run");

        let result = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendWorkspaceBuildRequest {
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
    fn absorbed_build_executor_rejects_unknown_named_steps() {
        let workspace = absorbed_build_workspace_fixture("compat_exec_unknown");

        let error = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendWorkspaceBuildRequest {
                requested_step: "docs".to_string(),
                profile: crate::FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect_err("unknown absorbed-build step should fail");

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(error
            .message()
            .contains("workspace build execution does not define step 'docs'"));

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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.step(\"docs\");\n",
                "    graph.step(\"lint\");\n",
                "    return graph\n",
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"app\", \"src/main.fol\");\n",
                "    graph.step(\"gen\");\n",
                "    graph.step(\"docs\", \"gen\");\n",
                "    graph.add_run(\"run\", \"app\", \"docs\");\n",
                "    return graph\n",
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
    fn custom_build_steps_plan_as_build_execution() {
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.step(\"docs\");\n",
                "    return graph\n",
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
        .expect("custom build-like step should plan successfully");

        let docs = plan
            .steps
            .iter()
            .find(|step| step.name == "docs")
            .expect("custom docs step should be present");
        assert_eq!(docs.execution, Some(FrontendStepExecutionKind::Build));
        assert!(docs.selection.is_none());

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn cli_selected_custom_graph_steps_flow_into_the_routed_member_plan() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_cli_step_{}_{}",
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
                "def root: loc = \"src\";\n",
                "pro[] build(graph: Graph): non = {\n",
                "    graph.step(\"docs\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/main.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let requested_step = super::requested_workspace_step(
            &crate::CodeSubcommand::Build(crate::BuildCommand::default()),
            Some("docs"),
        );
        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("member planning should surface the custom docs step");

        assert_eq!(requested_step, "docs");
        assert!(plan.steps.iter().any(|step| step.name == "docs"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn custom_run_steps_plan_as_run_execution() {
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"app\", \"src/main.fol\");\n",
                "    graph.add_run(\"serve\", \"app\");\n",
                "    return graph\n",
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
        .expect("custom run step should plan successfully");

        let serve = plan
            .steps
            .iter()
            .find(|step| step.name == "serve")
            .expect("custom run step should be present");
        assert_eq!(serve.execution, Some(FrontendStepExecutionKind::Run));
        assert_eq!(
            serve
                .selection
                .as_ref()
                .map(|selection| selection.label.as_str()),
            Some("app")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn explicit_named_run_steps_select_the_requested_artifact_when_multiple_runnables_exist() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_multi_run_{}_{}",
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
                "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
                "    graph.add_run(\"serve\", \"serve_app\");\n",
                "    graph.add_run(\"admin\", \"admin_app\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/serve.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/admin.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("member planning should keep named run step selections");

        let admin = plan
            .steps
            .iter()
            .find(|step| step.name == "admin")
            .expect("admin run step should be present");
        assert_eq!(admin.execution, Some(FrontendStepExecutionKind::Run));
        assert_eq!(
            admin
                .selection
                .as_ref()
                .map(|selection| selection.label.as_str()),
            Some("admin_app")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn named_build_steps_can_target_matching_artifacts_when_multiple_builds_exist() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_multi_build_{}_{}",
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
                "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
                "    graph.step(\"serve_app\");\n",
                "    graph.step(\"admin_app\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/serve.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/admin.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("member planning should keep named build step selections");

        let admin = plan
            .steps
            .iter()
            .find(|step| step.name == "admin_app")
            .expect("admin build step should be present");
        assert_eq!(admin.execution, Some(FrontendStepExecutionKind::Build));
        assert_eq!(
            admin
                .selection
                .as_ref()
                .map(|selection| selection.label.as_str()),
            Some("admin_app")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn default_build_step_is_marked_ambiguous_when_multiple_executables_exist() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_ambiguous_build_plan_{}_{}",
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
                "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/serve.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/admin.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();

        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("member planning should succeed");

        let build = plan
            .steps
            .iter()
            .find(|step| step.name == "build")
            .expect("default build step should be present");
        assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
        assert!(build.selection.is_none());
        assert!(build.ambiguous_selection);

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn ambiguous_default_multi_artifact_build_steps_fail_clearly() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_ambiguous_build_exec_{}_{}",
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"serve_app\", \"src/serve.fol\");\n",
                "    graph.add_exe(\"admin_app\", \"src/admin.fol\");\n",
                "    return graph\n",
                "}\n",
            ),
        )
        .unwrap();
        fs::write(
            root.join("src/serve.fol"),
            "fun[] main(): int = {\n    return 0\n}\n",
        )
        .unwrap();
        fs::write(
            root.join("src/admin.fol"),
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

        let error = execute_workspace_build_route(
            &workspace,
            &FrontendConfig::default(),
            &FrontendWorkspaceBuildRequest {
                requested_step: "build".to_string(),
                profile: FrontendProfile::Debug,
                run_args: Vec::new(),
            },
        )
        .expect_err("ambiguous default build step should fail");

        assert_eq!(error.kind(), crate::FrontendErrorKind::InvalidInput);
        assert!(error.message().contains("requires an explicit named step"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn configured_executable_roots_drive_default_build_and_run_step_planning() {
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
                "pro[] build(graph: Graph): non = {\n",
                "    graph.add_exe(\"app\", \"src/app.fol\");\n",
                "    graph.add_run(\"serve\", \"app\");\n",
                "    return graph\n",
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
        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("configured add_exe root should drive routed planning");

        let build = plan
            .steps
            .iter()
            .find(|step| step.name == "build")
            .expect("default build step should be present");
        assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
        assert_eq!(
            build
                .selection
                .as_ref()
                .and_then(|selection| selection.root_module.as_deref()),
            Some("src/app.fol")
        );

        let serve = plan
            .steps
            .iter()
            .find(|step| step.name == "serve")
            .expect("custom serve step should be present");
        assert_eq!(serve.execution, Some(FrontendStepExecutionKind::Run));
        assert_eq!(
            serve
                .selection
                .as_ref()
                .and_then(|selection| selection.root_module.as_deref()),
            Some("src/app.fol")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn object_style_artifact_build_bodies_drive_default_build_and_run_step_planning() {
        let root = std::env::temp_dir().join(format!(
            "fol_frontend_build_route_object_root_{}_{}",
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
                "pro[] build(graph: Graph): non = {\n",
                "    var target = graph.standard_target();\n",
                "    var optimize = graph.standard_optimize();\n",
                "    var app = graph.add_exe({\n",
                "        name = \"demo\",\n",
                "        root = \"src/app.fol\",\n",
                "        target = target,\n",
                "        optimize = optimize,\n",
                "    });\n",
                "    graph.install(app);\n",
                "    graph.add_run(app);\n",
                "    return graph\n",
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
        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("object style add_exe should drive routed planning");

        let build = plan
            .steps
            .iter()
            .find(|step| step.name == "build")
            .expect("default build step should be present");
        assert_eq!(build.execution, Some(FrontendStepExecutionKind::Build));
        assert_eq!(
            build
                .selection
                .as_ref()
                .and_then(|selection| selection.root_module.as_deref()),
            Some("src/app.fol")
        );

        let run = plan
            .steps
            .iter()
            .find(|step| step.name == "run")
            .expect("default run step should be present");
        assert_eq!(run.execution, Some(FrontendStepExecutionKind::Run));
        assert_eq!(
            run.selection
                .as_ref()
                .and_then(|selection| selection.root_module.as_deref()),
            Some("src/app.fol")
        );

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_route_plans_modern_build_members_through_default_graph_planning() {
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
            "pro[] build(graph: Graph): non = graph;\n",
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
        .expect("modern member should plan through the default graph");

        assert!(plan.steps.iter().any(|step| step.name == "build"));
        assert!(plan.steps.iter().any(|step| step.name == "run"));
        assert!(plan.steps.iter().any(|step| step.name == "test"));

        fs::remove_dir_all(root).ok();
    }

    #[test]
    fn workspace_route_plans_modern_check_steps_even_without_a_runnable_binary() {
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
            "pro[] build(graph: Graph): non = graph;\n",
        )
        .unwrap();
        fs::write(root.join("src/lib.fol"), "var[exp] answer: int = 42;\n").unwrap();
        let plan = plan_member_execution(&FrontendMemberBuildRoute {
            member_root: root.clone(),
            package_name: "demo".to_string(),
            mode: FrontendBuildWorkflowMode::Modern,
        })
        .expect("modern member without an executable should still plan check");

        let check = plan
            .steps
            .iter()
            .find(|step| step.name == "check")
            .expect("check step should be present");
        assert_eq!(check.execution, Some(FrontendStepExecutionKind::Check));
        assert!(check.selection.is_none());

        fs::remove_dir_all(root).ok();
    }
}
