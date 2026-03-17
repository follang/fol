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

#[cfg(test)]
mod tests {
    use super::BuildEntrySignatureExpectation;

    #[test]
    fn canonical_build_entry_signature_expectation_keeps_graph_names() {
        let expectation = BuildEntrySignatureExpectation::canonical();

        assert!(expectation.accepts_parameter_type("Graph"));
        assert!(expectation.accepts_parameter_type("build::Graph"));
        assert!(expectation.accepts_return_type("Graph"));
        assert!(expectation.accepts_return_type("build::Graph"));
        assert!(!expectation.accepts_parameter_type("int"));
    }
}
