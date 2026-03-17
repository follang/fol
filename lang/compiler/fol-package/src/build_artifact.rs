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
pub struct BuildArtifactTargetConfig {
    pub target: Option<String>,
    pub optimize: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildArtifactOutput {
    EmittedRustCrate {
        crate_root: String,
    },
    Binary {
        binary_path: String,
    },
    GeneratedSourceBundle {
        root: String,
    },
    DocsBundle {
        root: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactReport {
    pub artifact_name: String,
    pub output: BuildArtifactOutput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactPipelinePlan {
    pub definition: BuildArtifactDefinition,
    pub stages: Vec<BuildArtifactPipelineStage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildArtifactPipelineStage {
    Package,
    Resolver,
    Typecheck,
    Lower,
    Backend,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifactDefinition {
    pub name: String,
    pub kind: BuildArtifactModelKind,
    pub root_source: BuildArtifactRootSource,
    pub modules: BuildArtifactModuleConfig,
    pub output_name: String,
    pub linkage: BuildArtifactLinkage,
    pub target: BuildArtifactTargetConfig,
    pub native_artifacts: Vec<String>,
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
        BuildArtifactModuleConfig, BuildArtifactOutput, BuildArtifactReport,
        BuildArtifactPipelinePlan, BuildArtifactPipelineStage, BuildArtifactRootSource,
        BuildArtifactSet,
        BuildArtifactTargetConfig,
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
            target: BuildArtifactTargetConfig {
                target: None,
                optimize: None,
            },
            native_artifacts: Vec::new(),
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
            target: BuildArtifactTargetConfig {
                target: Some("x86_64-linux-gnu".to_string()),
                optimize: Some("release".to_string()),
            },
            native_artifacts: vec!["ssl".to_string(), "zlib".to_string()],
        };

        assert_eq!(definition.root_source.path, "src/plugin.fol");
        assert_eq!(definition.modules.roots.len(), 2);
        assert_eq!(definition.output_name, "fol_plugin");
        assert_eq!(definition.linkage, BuildArtifactLinkage::Shared);
        assert_eq!(definition.target.target.as_deref(), Some("x86_64-linux-gnu"));
        assert_eq!(definition.target.optimize.as_deref(), Some("release"));
        assert_eq!(definition.native_artifacts, vec!["ssl".to_string(), "zlib".to_string()]);
    }

    #[test]
    fn artifact_reports_cover_backend_and_bundle_outputs() {
        let emitted = BuildArtifactReport {
            artifact_name: "app".to_string(),
            output: BuildArtifactOutput::EmittedRustCrate {
                crate_root: ".fol/build/emit/rust/app".to_string(),
            },
        };
        let binary = BuildArtifactReport {
            artifact_name: "app".to_string(),
            output: BuildArtifactOutput::Binary {
                binary_path: ".fol/build/debug/app".to_string(),
            },
        };
        let docs = BuildArtifactReport {
            artifact_name: "docs".to_string(),
            output: BuildArtifactOutput::DocsBundle {
                root: ".fol/build/docs".to_string(),
            },
        };

        match emitted.output {
            BuildArtifactOutput::EmittedRustCrate { crate_root } => {
                assert!(crate_root.contains("emit/rust"));
            }
            other => panic!("unexpected emitted output: {other:?}"),
        }
        match binary.output {
            BuildArtifactOutput::Binary { binary_path } => {
                assert!(binary_path.ends_with("/app"));
            }
            other => panic!("unexpected binary output: {other:?}"),
        }
        match docs.output {
            BuildArtifactOutput::DocsBundle { root } => {
                assert!(root.ends_with("docs"));
            }
            other => panic!("unexpected docs output: {other:?}"),
        }
    }

    #[test]
    fn artifact_pipeline_plan_tracks_all_compiler_and_backend_stages() {
        let plan = BuildArtifactPipelinePlan {
            definition: BuildArtifactDefinition {
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
                target: BuildArtifactTargetConfig {
                    target: Some("native".to_string()),
                    optimize: Some("debug".to_string()),
                },
                native_artifacts: Vec::new(),
            },
            stages: vec![
                BuildArtifactPipelineStage::Package,
                BuildArtifactPipelineStage::Resolver,
                BuildArtifactPipelineStage::Typecheck,
                BuildArtifactPipelineStage::Lower,
                BuildArtifactPipelineStage::Backend,
            ],
        };

        assert_eq!(plan.definition.name, "app");
        assert_eq!(plan.stages.len(), 5);
        assert_eq!(plan.stages[0], BuildArtifactPipelineStage::Package);
        assert_eq!(plan.stages[4], BuildArtifactPipelineStage::Backend);
    }
}
