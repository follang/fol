#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NativeArtifactDefinition {
    pub name: String,
    pub relative_path: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NativeArtifactSet {
    definitions: Vec<NativeArtifactDefinition>,
}

impl NativeArtifactSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn definitions(&self) -> &[NativeArtifactDefinition] {
        &self.definitions
    }

    pub fn add(&mut self, definition: NativeArtifactDefinition) {
        self.definitions.push(definition);
    }
}

#[cfg(test)]
mod tests {
    use super::{NativeArtifactDefinition, NativeArtifactSet};

    #[test]
    fn native_artifact_set_starts_empty() {
        let set = NativeArtifactSet::new();

        assert!(set.definitions().is_empty());
    }

    #[test]
    fn native_artifact_set_preserves_inserted_shell_definitions() {
        let mut set = NativeArtifactSet::new();
        set.add(NativeArtifactDefinition {
            name: "api".to_string(),
            relative_path: "include/api.h".to_string(),
        });

        assert_eq!(set.definitions().len(), 1);
        assert_eq!(set.definitions()[0].name, "api");
        assert_eq!(set.definitions()[0].relative_path, "include/api.h");
    }
}
