#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntrySignatureExpectation {
    pub parameter_type_names: Vec<String>,
    pub return_type_names: Vec<String>,
}

impl BuildEntrySignatureExpectation {
    pub fn canonical() -> Self {
        Self {
            parameter_type_names: vec!["Graph".to_string(), "build::Graph".to_string()],
            return_type_names: vec!["Graph".to_string(), "build::Graph".to_string()],
        }
    }

    pub fn accepts_parameter_type(&self, name: &str) -> bool {
        self.parameter_type_names.iter().any(|candidate| candidate == name)
    }

    pub fn accepts_return_type(&self, name: &str) -> bool {
        self.return_type_names.iter().any(|candidate| candidate == name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntryCandidate {
    pub source_unit_path: String,
    pub syntax_id: fol_parser::ast::SyntaxNodeId,
    pub name: String,
    pub parameter_names: Vec<String>,
    pub parameter_type_names: Vec<Option<String>>,
    pub return_type_name: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedBuildEntry {
    pub candidate: BuildEntryCandidate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEntryValidationErrorKind {
    MissingEntry,
    MultipleEntries,
    WrongParameterCount,
    WrongParameterType,
    WrongReturnType,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEntryValidationError {
    pub kind: BuildEntryValidationErrorKind,
    pub message: String,
    pub origin: Option<fol_parser::ast::SyntaxOrigin>,
}

impl BuildEntryValidationError {
    pub fn new(kind: BuildEntryValidationErrorKind, message: impl Into<String>) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: None,
        }
    }

    pub fn with_origin(
        kind: BuildEntryValidationErrorKind,
        message: impl Into<String>,
        origin: fol_parser::ast::SyntaxOrigin,
    ) -> Self {
        Self {
            kind,
            message: message.into(),
            origin: Some(origin),
        }
    }
}

pub fn collect_build_entry_candidates(
    syntax: &fol_parser::ast::ParsedPackage,
) -> Vec<BuildEntryCandidate> {
    let mut candidates = Vec::new();

    for source_unit in &syntax.source_units {
        if source_unit.kind != fol_parser::ast::ParsedSourceUnitKind::Build {
            continue;
        }

        for item in &source_unit.items {
            let fol_parser::ast::AstNode::DefDecl {
                name,
                params,
                def_type,
                ..
            } = &item.node
            else {
                continue;
            };

            if name != "build" {
                continue;
            }

            candidates.push(BuildEntryCandidate {
                source_unit_path: source_unit.path.clone(),
                syntax_id: item.node_id,
                name: name.clone(),
                parameter_names: params.iter().map(|param| param.name.clone()).collect(),
                parameter_type_names: params
                    .iter()
                    .map(|param| param.param_type.named_text())
                    .collect(),
                return_type_name: def_type.named_text(),
            });
        }
    }

    candidates
}

#[cfg(test)]
mod tests {
    use super::{
        collect_build_entry_candidates, BuildEntryCandidate, BuildEntrySignatureExpectation,
        BuildEntryValidationError,
        BuildEntryValidationErrorKind, ValidatedBuildEntry,
    };

    #[test]
    fn canonical_build_entry_signature_expectation_keeps_graph_names() {
        let expectation = BuildEntrySignatureExpectation::canonical();

        assert!(expectation.accepts_parameter_type("Graph"));
        assert!(expectation.accepts_parameter_type("build::Graph"));
        assert!(expectation.accepts_return_type("Graph"));
        assert!(expectation.accepts_return_type("build::Graph"));
        assert!(!expectation.accepts_parameter_type("int"));
    }

    #[test]
    fn build_entry_candidate_and_validation_error_models_capture_core_metadata() {
        let candidate = BuildEntryCandidate {
            source_unit_path: "build.fol".to_string(),
            syntax_id: fol_parser::ast::SyntaxNodeId(7),
            name: "build".to_string(),
            parameter_names: vec!["graph".to_string()],
            parameter_type_names: vec![Some("Graph".to_string())],
            return_type_name: Some("Graph".to_string()),
        };
        let validated = ValidatedBuildEntry {
            candidate: candidate.clone(),
        };
        let error = BuildEntryValidationError::new(
            BuildEntryValidationErrorKind::WrongReturnType,
            "wrong return type",
        );

        assert_eq!(validated.candidate, candidate);
        assert_eq!(error.kind, BuildEntryValidationErrorKind::WrongReturnType);
        assert!(error.origin.is_none());
    }

    #[test]
    fn candidate_collection_scans_only_build_source_units() {
        let syntax = fol_parser::ast::ParsedPackage {
            package: "demo".to_string(),
            source_units: vec![
                fol_parser::ast::ParsedSourceUnit {
                    path: "build.fol".to_string(),
                    package: "demo".to_string(),
                    namespace: "demo".to_string(),
                    kind: fol_parser::ast::ParsedSourceUnitKind::Build,
                    items: vec![fol_parser::ast::ParsedTopLevel {
                        node_id: fol_parser::ast::SyntaxNodeId(1),
                        node: fol_parser::ast::AstNode::DefDecl {
                            options: Vec::new(),
                            name: "build".to_string(),
                            params: vec![fol_parser::ast::Parameter {
                                name: "graph".to_string(),
                                param_type: fol_parser::ast::FolType::Named {
                                    syntax_id: None,
                                    name: "Graph".to_string(),
                                },
                                is_borrowable: false,
                                is_mutex: false,
                                default: None,
                            }],
                            def_type: fol_parser::ast::FolType::Named {
                                syntax_id: None,
                                name: "Graph".to_string(),
                            },
                            body: Vec::new(),
                        },
                        meta: fol_parser::ast::ParsedTopLevelMeta::default(),
                    }],
                },
                fol_parser::ast::ParsedSourceUnit {
                    path: "src/main.fol".to_string(),
                    package: "demo".to_string(),
                    namespace: "demo::src".to_string(),
                    kind: fol_parser::ast::ParsedSourceUnitKind::Ordinary,
                    items: Vec::new(),
                },
            ],
            syntax_index: fol_parser::ast::SyntaxIndex::default(),
        };

        let candidates = collect_build_entry_candidates(&syntax);

        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].source_unit_path, "build.fol");
        assert_eq!(candidates[0].parameter_names, vec!["graph".to_string()]);
        assert_eq!(candidates[0].return_type_name.as_deref(), Some("Graph"));
    }
}
