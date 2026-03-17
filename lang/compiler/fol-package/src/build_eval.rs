use crate::build_graph::BuildGraph;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEvaluationRequest {
    pub package_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationResult {
    pub package_root: String,
    pub graph: BuildGraph,
}

impl BuildEvaluationResult {
    pub fn new(package_root: impl Into<String>, graph: BuildGraph) -> Self {
        Self {
            package_root: package_root.into(),
            graph,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildEvaluationRequest, BuildEvaluationResult};
    use crate::build_graph::BuildGraph;

    #[test]
    fn build_evaluation_request_defaults_to_an_empty_package_root() {
        let request = BuildEvaluationRequest::default();

        assert!(request.package_root.is_empty());
    }

    #[test]
    fn build_evaluation_result_carries_the_constructed_graph() {
        let graph = BuildGraph::new();
        let result = BuildEvaluationResult::new("app", graph.clone());

        assert_eq!(result.package_root, "app");
        assert_eq!(result.graph, graph);
    }
}
