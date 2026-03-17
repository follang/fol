#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FrontendBuildWorkflowMode {
    Compatibility,
    Modern,
    Hybrid,
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

#[cfg(test)]
mod tests {
    use super::{FrontendBuildWorkflowMode, FrontendMemberBuildRoute};

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
}
