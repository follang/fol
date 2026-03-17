#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyBuildSurface {
    pub alias: String,
    pub modules: Vec<DependencyModuleSurface>,
    pub source_roots: Vec<DependencySourceRootSurface>,
    pub artifacts: Vec<DependencyArtifactSurface>,
    pub steps: Vec<DependencyStepSurface>,
    pub generated_outputs: Vec<DependencyGeneratedOutputSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyBuildHandle {
    pub alias: String,
    pub package: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyModuleSurfaceSet {
    pub modules: Vec<DependencyModuleSurface>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyArtifactSurfaceSet {
    pub artifacts: Vec<DependencyArtifactSurface>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyStepSurfaceSet {
    pub steps: Vec<DependencyStepSurface>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyGeneratedOutputSurfaceSet {
    pub generated_outputs: Vec<DependencyGeneratedOutputSurface>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyModuleSurface {
    pub name: String,
    pub source_namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencySourceRootSurface {
    pub relative_path: String,
    pub namespace_prefix: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyArtifactSurface {
    pub name: String,
    pub artifact_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyStepSurface {
    pub name: String,
    pub step_kind: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyGeneratedOutputSurface {
    pub name: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DependencyBuildSurfaceSet {
    surfaces: Vec<DependencyBuildSurface>,
}

impl DependencyBuildSurfaceSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn surfaces(&self) -> &[DependencyBuildSurface] {
        &self.surfaces
    }

    pub fn add(&mut self, surface: DependencyBuildSurface) {
        self.surfaces.push(surface);
    }
}

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
    use super::{
        dependency_modules_from_exports, DependencyArtifactSurface,
        DependencyArtifactSurfaceSet, DependencyBuildHandle, DependencyBuildSurface,
        DependencyBuildSurfaceSet, DependencyGeneratedOutputSurface,
        DependencyGeneratedOutputSurfaceSet, DependencyModuleSurface, DependencyModuleSurfaceSet,
        DependencySourceRootSurface, DependencyStepSurface, DependencyStepSurfaceSet,
    };
    use crate::PreparedExportMount;

    #[test]
    fn dependency_build_surface_set_starts_empty() {
        let set = DependencyBuildSurfaceSet::new();

        assert!(set.surfaces().is_empty());
    }

    #[test]
    fn dependency_build_surface_set_preserves_inserted_shell_surfaces() {
        let mut set = DependencyBuildSurfaceSet::new();
        set.add(DependencyBuildSurface {
            alias: "logtiny".to_string(),
            modules: vec![DependencyModuleSurface {
                name: "logtiny".to_string(),
                source_namespace: "logtiny::src".to_string(),
            }],
            source_roots: vec![DependencySourceRootSurface {
                relative_path: "src".to_string(),
                namespace_prefix: "logtiny::src".to_string(),
            }],
            artifacts: vec![DependencyArtifactSurface {
                name: "logtiny".to_string(),
                artifact_kind: "static-lib".to_string(),
            }],
            steps: vec![DependencyStepSurface {
                name: "test".to_string(),
                step_kind: "test".to_string(),
            }],
            generated_outputs: vec![DependencyGeneratedOutputSurface {
                name: "bindings".to_string(),
                relative_path: "gen/bindings.fol".to_string(),
            }],
        });

        assert_eq!(set.surfaces().len(), 1);
        assert_eq!(set.surfaces()[0].alias, "logtiny");
        assert_eq!(set.surfaces()[0].modules.len(), 1);
        assert_eq!(set.surfaces()[0].source_roots.len(), 1);
        assert_eq!(set.surfaces()[0].artifacts.len(), 1);
        assert_eq!(set.surfaces()[0].steps.len(), 1);
        assert_eq!(set.surfaces()[0].generated_outputs.len(), 1);
    }

    #[test]
    fn dependency_build_handle_keeps_alias_and_package_identity() {
        let handle = DependencyBuildHandle {
            alias: "logtiny".to_string(),
            package: "org/logtiny".to_string(),
        };

        assert_eq!(handle.alias, "logtiny");
        assert_eq!(handle.package, "org/logtiny");
    }

    #[test]
    fn dependency_surface_collection_types_preserve_inserted_items() {
        let modules = DependencyModuleSurfaceSet {
            modules: vec![DependencyModuleSurface {
                name: "logtiny".to_string(),
                source_namespace: "logtiny::src".to_string(),
            }],
        };
        let artifacts = DependencyArtifactSurfaceSet {
            artifacts: vec![DependencyArtifactSurface {
                name: "logtiny".to_string(),
                artifact_kind: "static-lib".to_string(),
            }],
        };
        let steps = DependencyStepSurfaceSet {
            steps: vec![DependencyStepSurface {
                name: "test".to_string(),
                step_kind: "test".to_string(),
            }],
        };
        let outputs = DependencyGeneratedOutputSurfaceSet {
            generated_outputs: vec![DependencyGeneratedOutputSurface {
                name: "bindings".to_string(),
                relative_path: "gen/bindings.fol".to_string(),
            }],
        };

        assert_eq!(modules.modules.len(), 1);
        assert_eq!(artifacts.artifacts.len(), 1);
        assert_eq!(steps.steps.len(), 1);
        assert_eq!(outputs.generated_outputs.len(), 1);
    }

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
use crate::PreparedExportMount;
