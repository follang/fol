use crate::{FrontendError, FrontendErrorKind, FrontendResult, FrontendWorkspace};
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
            let build = fol_package::parse_package_build(&member.root.join("build.fol"))?;
            let mode = FrontendBuildWorkflowMode::from_package_build_mode(build.mode())
                .ok_or_else(|| {
                    FrontendError::new(
                        FrontendErrorKind::Internal,
                        format!(
                            "workspace member '{}' has an unmappable build mode",
                            member.root.display()
                        ),
                    )
                })?;
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

#[cfg(test)]
mod tests {
    use super::{
        plan_workspace_build_route, FrontendBuildStep, FrontendBuildWorkflowMode,
        FrontendMemberBuildRoute, FrontendWorkspaceBuildRoute,
    };
    use crate::{FrontendWorkspace, PackageRoot, WorkspaceRoot};
    use std::{fs, path::PathBuf};

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
        fs::write(compatibility.join("build.fol"), "def root: loc = \"src\"\n").unwrap();
        fs::write(
            hybrid.join("build.fol"),
            "def root: loc = \"src\"\ndef build(graph: int): int = graph;\n",
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
}
