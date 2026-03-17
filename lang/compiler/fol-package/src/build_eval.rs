use crate::build_graph::BuildGraph;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvaluationBoundary {
    GraphConstructionSubset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedBuildTimeOperation {
    GraphMutation,
    OptionRead,
    PathJoin,
    PathNormalize,
    StringBasic,
    ContainerBasic,
    ControlledFileGeneration,
    ControlledProcessExecution,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildEvaluationRequest {
    pub package_root: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildEvaluationResult {
    pub boundary: BuildEvaluationBoundary,
    pub allowed_operations: Vec<AllowedBuildTimeOperation>,
    pub package_root: String,
    pub graph: BuildGraph,
}

impl BuildEvaluationResult {
    pub fn new(
        boundary: BuildEvaluationBoundary,
        allowed_operations: Vec<AllowedBuildTimeOperation>,
        package_root: impl Into<String>,
        graph: BuildGraph,
    ) -> Self {
        Self {
            boundary,
            allowed_operations,
            package_root: package_root.into(),
            graph,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AllowedBuildTimeOperation, BuildEvaluationBoundary, BuildEvaluationRequest,
        BuildEvaluationResult,
    };
    use crate::build_graph::BuildGraph;

    #[test]
    fn build_evaluation_request_defaults_to_an_empty_package_root() {
        let request = BuildEvaluationRequest::default();

        assert!(request.package_root.is_empty());
    }

    #[test]
    fn build_evaluation_result_carries_the_constructed_graph() {
        let graph = BuildGraph::new();
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            vec![AllowedBuildTimeOperation::GraphMutation],
            "app",
            graph.clone(),
        );

        assert_eq!(result.package_root, "app");
        assert_eq!(result.graph, graph);
    }

    #[test]
    fn build_evaluation_result_keeps_boundary_and_allowed_operation_metadata() {
        let result = BuildEvaluationResult::new(
            BuildEvaluationBoundary::GraphConstructionSubset,
            vec![
                AllowedBuildTimeOperation::GraphMutation,
                AllowedBuildTimeOperation::OptionRead,
            ],
            "pkg",
            BuildGraph::new(),
        );

        assert_eq!(
            result.boundary,
            BuildEvaluationBoundary::GraphConstructionSubset
        );
        assert_eq!(
            result.allowed_operations,
            vec![
                AllowedBuildTimeOperation::GraphMutation,
                AllowedBuildTimeOperation::OptionRead,
            ]
        );
    }
}
