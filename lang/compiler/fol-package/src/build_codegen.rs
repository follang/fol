#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFileDefinition {
    pub name: String,
    pub relative_path: String,
    pub action: GeneratedFileAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GeneratedFileAction {
    Write { contents: String },
    Copy { source_path: String },
    CaptureToolOutput { tool: String, args: Vec<String> },
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct GeneratedFileSet {
    definitions: Vec<GeneratedFileDefinition>,
}

impl GeneratedFileSet {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn definitions(&self) -> &[GeneratedFileDefinition] {
        &self.definitions
    }

    pub fn add(&mut self, definition: GeneratedFileDefinition) {
        self.definitions.push(definition);
    }
}

#[cfg(test)]
mod tests {
    use super::{GeneratedFileAction, GeneratedFileDefinition, GeneratedFileSet};

    #[test]
    fn generated_file_set_starts_empty() {
        let set = GeneratedFileSet::new();

        assert!(set.definitions().is_empty());
    }

    #[test]
    fn generated_file_set_preserves_inserted_shell_definitions() {
        let mut set = GeneratedFileSet::new();
        set.add(GeneratedFileDefinition {
            name: "version".to_string(),
            relative_path: "gen/version.fol".to_string(),
            action: GeneratedFileAction::Write {
                contents: "let version = \"0.1.0\"".to_string(),
            },
        });

        assert_eq!(set.definitions().len(), 1);
        assert_eq!(set.definitions()[0].name, "version");
        assert_eq!(set.definitions()[0].relative_path, "gen/version.fol");
        assert!(matches!(
            set.definitions()[0].action,
            GeneratedFileAction::Write { .. }
        ));
    }

    #[test]
    fn generated_file_actions_cover_write_copy_and_captured_outputs() {
        let write = GeneratedFileAction::Write {
            contents: "hello".to_string(),
        };
        let copy = GeneratedFileAction::Copy {
            source_path: "assets/logo.svg".to_string(),
        };
        let capture = GeneratedFileAction::CaptureToolOutput {
            tool: "schema-gen".to_string(),
            args: vec!["api.yaml".to_string()],
        };

        assert!(matches!(write, GeneratedFileAction::Write { .. }));
        assert!(matches!(copy, GeneratedFileAction::Copy { .. }));
        assert!(matches!(capture, GeneratedFileAction::CaptureToolOutput { .. }));
    }
}
