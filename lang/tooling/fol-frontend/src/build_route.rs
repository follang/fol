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
    available_steps: Vec<String>,
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
    match fol_package::parse_package_build(build_path) {
        Ok(build) => FrontendBuildWorkflowMode::from_package_build_mode(build.mode()).ok_or_else(|| {
            FrontendError::new(
                FrontendErrorKind::Internal,
                format!(
                    "workspace member build '{}' has an unmappable build mode",
                    build_path.display()
                ),
            )
        }),
        Err(error) => {
            let contents = std::fs::read_to_string(build_path)
                .map_err(|_| FrontendError::from(error.clone()))?;
            classify_build_mode_from_source(&contents).ok_or_else(|| FrontendError::from(error))
        }
    }
}

fn classify_build_mode_from_source(contents: &str) -> Option<FrontendBuildWorkflowMode> {
    let mut has_compatibility_controls = false;
    let mut has_build_entry = false;

    for line in contents.lines().map(str::trim) {
        if line.is_empty() || line.starts_with("//") || line.starts_with('#') {
            continue;
        }
        if line.contains("def build(") {
            has_build_entry = true;
        }
        if line.starts_with("def ") && (line.contains(": pkg") || line.contains(": loc")) {
            has_compatibility_controls = true;
        }
    }

    match (has_compatibility_controls, has_build_entry) {
        (false, false) => None,
        (true, false) => Some(FrontendBuildWorkflowMode::Compatibility),
        (false, true) => Some(FrontendBuildWorkflowMode::Modern),
        (true, true) => Some(FrontendBuildWorkflowMode::Hybrid),
    }
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
        .any(|plan| plan.available_steps.iter().any(|step| step == requested_step))
    {
        return Err(unknown_workspace_build_step_error(requested_step, &route));
    }

    match request.requested_step.as_str() {
        "build" => crate::build_workspace_for_profile_with_config(workspace, config, request.profile),
        "check" => crate::check_workspace_with_config(workspace, config),
        "run" => crate::run_workspace_with_args_and_config(workspace, config, &request.run_args),
        "test" => crate::test_workspace_with_config(workspace, config),
        step => Err(FrontendError::new(
            FrontendErrorKind::InvalidInput,
            format!("workspace build execution does not support step '{step}'"),
        )),
    }
}

fn plan_member_execution(
    member: &FrontendMemberBuildRoute,
) -> FrontendResult<FrontendMemberExecutionPlan> {
    let requested_step = fol_package::BuildRequestedStep::Default(fol_package::BuildDefaultStepKind::Check);
    let mut graph = fol_package::BuildGraph::new();
    let check = graph.add_step(
        fol_package::BuildStepKind::Check,
        requested_step.name().to_string(),
    );
    let mut available_steps = vec![requested_step.name().to_string()];

    if let Some(root_module) = default_runnable_root_module(&member.member_root)? {
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

        available_steps.extend(["build", "run", "test"].into_iter().map(str::to_string));
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

    Ok(FrontendMemberExecutionPlan { available_steps })
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
        execute_workspace_build_route, plan_workspace_build_route,
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
