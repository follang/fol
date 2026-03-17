#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildArtifactModelKind {
    Executable,
    StaticLibrary,
    SharedLibrary,
    TestBundle,
    GeneratedSourceBundle,
    DocsBundle,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactDefinition {
    pub name: String,
    pub kind: BuildArtifactModelKind,
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
    use super::{BuildArtifactDefinition, BuildArtifactModelKind, BuildArtifactSet};

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
}
