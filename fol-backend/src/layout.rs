use crate::{mangle_package_module_name, BackendSession};
use fol_resolver::{PackageIdentity, SourceUnitId};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageLayoutPlan {
    pub package_identity: PackageIdentity,
    pub module_name: String,
    pub relative_dir: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NamespaceLayoutPlan {
    pub package_identity: PackageIdentity,
    pub full_namespace: String,
    pub module_name: String,
    pub relative_file: String,
    pub source_unit_ids: Vec<SourceUnitId>,
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

pub fn plan_namespace_layouts(session: &BackendSession) -> Vec<NamespaceLayoutPlan> {
    let mut planned = Vec::new();
    for package_identity in session.package_graph() {
        let Some(package) = session.workspace().package(package_identity) else {
            continue;
        };

        let mut by_namespace: BTreeMap<String, Vec<SourceUnitId>> = BTreeMap::new();
        for source_unit in &package.source_units {
            by_namespace
                .entry(source_unit.namespace.clone())
                .or_default()
                .push(source_unit.source_unit_id);
        }

        for (namespace, source_unit_ids) in by_namespace {
            let relative_segments = namespace_segments(package_identity, &namespace);
            let module_name = relative_segments
                .last()
                .map(|segment| sanitize_segment(segment))
                .unwrap_or_else(|| "root".to_string());
            let relative_file = if relative_segments.len() <= 1 {
                format!("{module_name}.rs")
            } else {
                format!(
                    "{}/{}.rs",
                    relative_segments[..relative_segments.len() - 1].join("/"),
                    module_name
                )
            };

            planned.push(NamespaceLayoutPlan {
                package_identity: package_identity.clone(),
                full_namespace: namespace,
                module_name,
                relative_file,
                source_unit_ids,
            });
        }
    }
    planned
}

fn namespace_segments(package_identity: &PackageIdentity, namespace: &str) -> Vec<String> {
    let mut segments = namespace
        .split("::")
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if segments
        .first()
        .is_some_and(|segment| segment == &package_identity.display_name)
    {
        segments.remove(0);
    }
    segments
}

fn sanitize_segment(raw: &str) -> String {
    crate::sanitize_backend_ident(raw)
}

#[cfg(test)]
mod tests {
    use super::{plan_namespace_layouts, plan_package_layouts};
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

    #[test]
    fn namespace_layout_plans_group_source_units_by_namespace() {
        let session = BackendSession::new(sample_lowered_workspace());

        let plans = plan_namespace_layouts(&session);

        assert_eq!(plans.len(), 4);
        assert_eq!(plans[0].full_namespace, "app");
        assert_eq!(plans[0].module_name, "root");
        assert_eq!(plans[0].relative_file, "root.rs");
        assert_eq!(plans[1].full_namespace, "app::math");
        assert_eq!(plans[1].module_name, "math");
        assert_eq!(plans[1].relative_file, "math.rs");
        assert_eq!(plans[3].full_namespace, "shared::util");
        assert_eq!(plans[3].module_name, "util");
        assert_eq!(plans[3].relative_file, "util.rs");
    }
}
