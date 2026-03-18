pub use fol_build::dependency::*;

use crate::PreparedExportMount;

pub fn dependency_modules_from_exports(
    alias: &str,
    exports: &[PreparedExportMount],
) -> Vec<DependencyModuleSurface> {
    exports
        .iter()
        .map(|export| DependencyModuleSurface {
            name: export
                .mounted_namespace_suffix
                .as_deref()
                .unwrap_or(alias)
                .to_string(),
            source_namespace: export.source_namespace.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::dependency_modules_from_exports;
    use crate::PreparedExportMount;

    #[test]
    fn dependency_module_bridge_projects_prepared_exports_into_module_surfaces() {
        let modules = dependency_modules_from_exports(
            "json",
            &[
                PreparedExportMount {
                    source_namespace: "json::src".to_string(),
                    mounted_namespace_suffix: None,
                },
                PreparedExportMount {
                    source_namespace: "json::src::codec".to_string(),
                    mounted_namespace_suffix: Some("codec".to_string()),
                },
            ],
        );

        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].name, "json");
        assert_eq!(modules[0].source_namespace, "json::src");
        assert_eq!(modules[1].name, "codec");
        assert_eq!(modules[1].source_namespace, "json::src::codec");
    }
}
