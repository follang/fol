use crate::build_graph::BuildGraph;

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
}

#[cfg(test)]
mod tests {
    use super::BuildApi;
    use crate::build_graph::BuildGraph;

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
}
