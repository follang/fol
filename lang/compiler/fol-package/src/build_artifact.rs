#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildArtifactModelKind {
    Executable,
    StaticLibrary,
    SharedLibrary,
    TestBundle,
    GeneratedSourceBundle,
    DocsBundle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildArtifactLinkage {
    Executable,
    Static,
    Shared,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactRootSource {
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactModuleConfig {
    pub roots: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactDefinition {
    pub name: String,
    pub kind: BuildArtifactModelKind,
    pub root_source: BuildArtifactRootSource,
    pub modules: BuildArtifactModuleConfig,
    pub output_name: String,
    pub linkage: BuildArtifactLinkage,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildArtifactSet {
    definitions: Vec<BuildArtifactDefinition>,
}

impl BuildArtifactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn definitions(&self) -> &[BuildArtifactDefinition] {
        &self.definitions
    }

    pub fn add_definition(&mut self, definition: BuildArtifactDefinition) {
        self.definitions.push(definition);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildArtifactDefinition, BuildArtifactLinkage, BuildArtifactModelKind,
        BuildArtifactModuleConfig, BuildArtifactRootSource, BuildArtifactSet,
    };

    #[test]
    fn build_artifact_set_starts_empty() {
        let set = BuildArtifactSet::new();

        assert!(set.definitions().is_empty());
    }

    #[test]
    fn build_artifact_set_preserves_inserted_definitions() {
        let mut set = BuildArtifactSet::new();
        set.add_definition(BuildArtifactDefinition {
            name: "app".to_string(),
            kind: BuildArtifactModelKind::Executable,
            root_source: BuildArtifactRootSource {
                path: "src/main.fol".to_string(),
            },
            modules: BuildArtifactModuleConfig {
                roots: vec!["src".to_string()],
            },
            output_name: "app".to_string(),
            linkage: BuildArtifactLinkage::Executable,
        });

        assert_eq!(set.definitions()[0].name, "app");
    }

    #[test]
    fn artifact_model_kinds_cover_phase_five_bundle_shapes() {
        assert_eq!(
            BuildArtifactModelKind::GeneratedSourceBundle,
            BuildArtifactModelKind::GeneratedSourceBundle
        );
        assert_eq!(BuildArtifactModelKind::DocsBundle, BuildArtifactModelKind::DocsBundle);
        assert_eq!(BuildArtifactModelKind::TestBundle, BuildArtifactModelKind::TestBundle);
    }

    #[test]
    fn artifact_definitions_keep_root_module_output_and_linkage_config() {
        let definition = BuildArtifactDefinition {
            name: "plugin".to_string(),
            kind: BuildArtifactModelKind::SharedLibrary,
            root_source: BuildArtifactRootSource {
                path: "src/plugin.fol".to_string(),
            },
            modules: BuildArtifactModuleConfig {
                roots: vec!["src".to_string(), "generated".to_string()],
            },
            output_name: "fol_plugin".to_string(),
            linkage: BuildArtifactLinkage::Shared,
        };

        assert_eq!(definition.root_source.path, "src/plugin.fol");
        assert_eq!(definition.modules.roots.len(), 2);
        assert_eq!(definition.output_name, "fol_plugin");
        assert_eq!(definition.linkage, BuildArtifactLinkage::Shared);
    }
}
