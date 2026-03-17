#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DependencyBuildSurface {
    pub alias: String,
    pub modules: Vec<DependencyModuleSurface>,
    pub source_roots: Vec<DependencySourceRootSurface>,
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

#[cfg(test)]
mod tests {
    use super::{
        DependencyBuildSurface, DependencyBuildSurfaceSet, DependencyModuleSurface,
        DependencySourceRootSurface,
    };

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
        });

        assert_eq!(set.surfaces().len(), 1);
        assert_eq!(set.surfaces()[0].alias, "logtiny");
        assert_eq!(set.surfaces()[0].modules.len(), 1);
        assert_eq!(set.surfaces()[0].source_roots.len(), 1);
    }
}
