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
    pub package_name: String,
    pub mode: FrontendBuildWorkflowMode,
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

#[cfg(test)]
mod tests {
    use super::{FrontendBuildStep, FrontendBuildWorkflowMode, FrontendMemberBuildRoute};

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
            package_name: "app".to_string(),
            mode: FrontendBuildWorkflowMode::Hybrid,
        };

        assert_eq!(route.package_name, "app");
        assert_eq!(route.mode, FrontendBuildWorkflowMode::Hybrid);
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
}
