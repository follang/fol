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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFileInstallProjection {
    pub generated_file_name: String,
    pub install_name: String,
    pub install_path: String,
}

impl GeneratedFileInstallProjection {
    pub fn new(
        generated_file_name: impl Into<String>,
        install_name: impl Into<String>,
        install_path: impl Into<String>,
    ) -> Self {
        Self {
            generated_file_name: generated_file_name.into(),
            install_name: install_name.into(),
            install_path: install_path.into(),
        }
    }
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
    use super::{
        GeneratedFileAction, GeneratedFileDefinition, GeneratedFileInstallProjection,
        GeneratedFileSet,
    };

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

    #[test]
    fn generated_file_install_projection_keeps_install_helper_metadata() {
        let projection = GeneratedFileInstallProjection::new(
            "config",
            "install-config",
            "share/config.json",
        );

        assert_eq!(projection.generated_file_name, "config");
        assert_eq!(projection.install_name, "install-config");
        assert_eq!(projection.install_path, "share/config.json");
    }
}
