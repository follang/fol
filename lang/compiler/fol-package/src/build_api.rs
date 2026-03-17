use crate::build_graph::BuildGraph;
use crate::build_graph::{BuildOptionId, BuildOptionKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardTargetRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeRequest {
    pub name: String,
    pub default: Option<String>,
}

impl StandardOptimizeRequest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default: None,
        }
    }

    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardTargetOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StandardOptimizeOption {
    pub id: BuildOptionId,
    pub name: String,
    pub default: Option<String>,
}

#[derive(Debug)]
pub struct BuildApi<'a> {
    graph: &'a mut BuildGraph,
}

impl<'a> BuildApi<'a> {
    pub fn new(graph: &'a mut BuildGraph) -> Self {
        Self { graph }
    }

    pub fn graph(&self) -> &BuildGraph {
        self.graph
    }

    pub fn graph_mut(&mut self) -> &mut BuildGraph {
        self.graph
    }

    pub fn standard_target(&mut self, request: StandardTargetRequest) -> StandardTargetOption {
        let option_id = self.graph.add_option(BuildOptionKind::Target, request.name.clone());
        StandardTargetOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }

    pub fn standard_optimize(
        &mut self,
        request: StandardOptimizeRequest,
    ) -> StandardOptimizeOption {
        let option_id = self.graph.add_option(BuildOptionKind::Optimize, request.name.clone());
        StandardOptimizeOption {
            id: option_id,
            name: request.name,
            default: request.default,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildApi, StandardOptimizeRequest, StandardTargetRequest};
    use crate::build_graph::BuildGraph;
    use crate::build_graph::BuildOptionKind;

    #[test]
    fn build_api_wraps_a_graph_reference() {
        let mut graph = BuildGraph::new();
        let api = BuildApi::new(&mut graph);

        assert!(api.graph().steps().is_empty());
    }

    #[test]
    fn build_api_exposes_mutable_graph_access() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        api.graph_mut().add_step(crate::build_graph::BuildStepKind::Default, "build");

        assert_eq!(api.graph().steps().len(), 1);
    }

    #[test]
    fn build_api_records_standard_target_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option = api.standard_target(StandardTargetRequest::new("target").with_default("native"));

        assert_eq!(option.name, "target");
        assert_eq!(option.default.as_deref(), Some("native"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Target);
    }

    #[test]
    fn build_api_records_standard_optimize_options_in_the_graph() {
        let mut graph = BuildGraph::new();
        let mut api = BuildApi::new(&mut graph);

        let option =
            api.standard_optimize(StandardOptimizeRequest::new("optimize").with_default("debug"));

        assert_eq!(option.name, "optimize");
        assert_eq!(option.default.as_deref(), Some("debug"));
        assert_eq!(api.graph().options()[0].id, option.id);
        assert_eq!(api.graph().options()[0].kind, BuildOptionKind::Optimize);
    }
}
