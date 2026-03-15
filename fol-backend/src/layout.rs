use crate::{mangle_package_module_name, BackendSession};
use fol_resolver::PackageIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageLayoutPlan {
    pub package_identity: PackageIdentity,
    pub module_name: String,
    pub relative_dir: String,
}

pub fn plan_package_layouts(session: &BackendSession) -> Vec<PackageLayoutPlan> {
    session
        .package_graph()
        .iter()
        .cloned()
        .map(|package_identity| {
            let module_name = mangle_package_module_name(&package_identity);
            PackageLayoutPlan {
                package_identity,
                relative_dir: format!("src/packages/{module_name}"),
                module_name,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::plan_package_layouts;
    use crate::{testing::sample_lowered_workspace, BackendSession};

    #[test]
    fn package_layout_plans_follow_package_graph_order() {
        let session = BackendSession::new(sample_lowered_workspace());

        let plans = plan_package_layouts(&session);

        assert_eq!(plans.len(), 2);
        assert_eq!(plans[0].package_identity.display_name, "app");
        assert_eq!(plans[0].module_name, "pkg__entry__app");
        assert_eq!(plans[0].relative_dir, "src/packages/pkg__entry__app");
        assert_eq!(plans[1].package_identity.display_name, "shared");
        assert_eq!(plans[1].module_name, "pkg__local__shared");
        assert_eq!(plans[1].relative_dir, "src/packages/pkg__local__shared");
    }
}
