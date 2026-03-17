#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildDefaultStepKind {
    Build,
    Run,
    Test,
    Install,
    Check,
}

impl BuildDefaultStepKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Build => "build",
            Self::Run => "run",
            Self::Test => "test",
            Self::Install => "install",
            Self::Check => "check",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildRequestedStep {
    Default(BuildDefaultStepKind),
    Named(String),
}

impl BuildRequestedStep {
    pub fn name(&self) -> &str {
        match self {
            Self::Default(kind) => kind.as_str(),
            Self::Named(name) => name.as_str(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStepDefinition {
    pub name: String,
    pub default_kind: Option<BuildDefaultStepKind>,
    pub dependencies: Vec<String>,
}

impl BuildStepDefinition {
    pub fn custom(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            default_kind: None,
            dependencies: Vec::new(),
        }
    }

    pub fn default(kind: BuildDefaultStepKind) -> Self {
        Self {
            name: kind.as_str().to_string(),
            default_kind: Some(kind),
            dependencies: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildStepPlanError {
    UnknownStep(BuildStepId),
    DependencyCycle(BuildStepId),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildStepExecutionRequest {
    pub requested_step: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStepExecutionResult {
    pub requested_step: String,
}

impl BuildStepExecutionResult {
    pub fn new(requested_step: impl Into<String>) -> Self {
        Self {
            requested_step: requested_step.into(),
        }
    }
}

pub fn plan_step_order(
    graph: &BuildGraph,
    requested_step: BuildStepId,
) -> Result<Vec<BuildStepId>, BuildStepPlanError> {
    if requested_step.index() >= graph.steps().len() {
        return Err(BuildStepPlanError::UnknownStep(requested_step));
    }

    let mut visiting = vec![false; graph.steps().len()];
    let mut visited = vec![false; graph.steps().len()];
    let mut order = Vec::new();
    visit_step_order(
        graph,
        requested_step,
        &mut visiting,
        &mut visited,
        &mut order,
    )?;
    Ok(order)
}

fn visit_step_order(
    graph: &BuildGraph,
    step: BuildStepId,
    visiting: &mut [bool],
    visited: &mut [bool],
    order: &mut Vec<BuildStepId>,
) -> Result<(), BuildStepPlanError> {
    let index = step.index();
    if index >= graph.steps().len() {
        return Err(BuildStepPlanError::UnknownStep(step));
    }
    if visited[index] {
        return Ok(());
    }
    if visiting[index] {
        return Err(BuildStepPlanError::DependencyCycle(step));
    }

    visiting[index] = true;
    for dependency in graph.step_dependencies_for(step) {
        visit_step_order(graph, dependency, visiting, visited, order)?;
    }
    visiting[index] = false;
    visited[index] = true;
    order.push(step);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        plan_step_order, BuildDefaultStepKind, BuildRequestedStep, BuildStepDefinition,
        BuildStepExecutionRequest, BuildStepExecutionResult, BuildStepPlanError,
    };
    use crate::build_graph::{BuildGraph, BuildStepId, BuildStepKind};

    #[test]
    fn build_default_step_kind_covers_phase_six_defaults() {
        assert_eq!(BuildDefaultStepKind::Build.as_str(), "build");
        assert_eq!(BuildDefaultStepKind::Run.as_str(), "run");
        assert_eq!(BuildDefaultStepKind::Test.as_str(), "test");
        assert_eq!(BuildDefaultStepKind::Install.as_str(), "install");
        assert_eq!(BuildDefaultStepKind::Check.as_str(), "check");
    }

    #[test]
    fn requested_steps_preserve_default_and_custom_names() {
        assert_eq!(
            BuildRequestedStep::Default(BuildDefaultStepKind::Build).name(),
            "build"
        );
        assert_eq!(
            BuildRequestedStep::Named("docs".to_string()).name(),
            "docs"
        );
    }

    #[test]
    fn build_step_definitions_cover_default_and_custom_shapes() {
        let build = BuildStepDefinition::default(BuildDefaultStepKind::Build);
        let docs = BuildStepDefinition::custom("docs");

        assert_eq!(build.name, "build");
        assert_eq!(build.default_kind, Some(BuildDefaultStepKind::Build));
        assert!(build.dependencies.is_empty());

        assert_eq!(docs.name, "docs");
        assert_eq!(docs.default_kind, None);
        assert!(docs.dependencies.is_empty());
    }

    #[test]
    fn build_step_execution_request_defaults_to_an_empty_step_name() {
        let request = BuildStepExecutionRequest::default();

        assert!(request.requested_step.is_empty());
    }

    #[test]
    fn build_step_execution_result_keeps_the_requested_step_name() {
        let result = BuildStepExecutionResult::new("build");

        assert_eq!(result.requested_step, "build");
    }

    #[test]
    fn step_order_planning_runs_dependencies_before_requested_step() {
        let mut graph = BuildGraph::new();
        let build = graph.add_step(BuildStepKind::Default, "build");
        let generate = graph.add_step(BuildStepKind::CustomCommand, "generate");
        let check = graph.add_step(BuildStepKind::Check, "check");
        graph.add_step_dependency(build, generate);
        graph.add_step_dependency(build, check);

        let order = plan_step_order(&graph, build).expect("build order should plan");

        assert_eq!(order, vec![generate, check, build]);
    }

    #[test]
    fn step_order_planning_reports_unknown_requested_steps() {
        let graph = BuildGraph::new();

        let error = plan_step_order(&graph, BuildStepId::from_index(0)).unwrap_err();

        assert_eq!(error, BuildStepPlanError::UnknownStep(BuildStepId::from_index(0)));
    }

    #[test]
    fn step_order_planning_reports_dependency_cycles() {
        let mut graph = BuildGraph::new();
        let build = graph.add_step(BuildStepKind::Default, "build");
        let check = graph.add_step(BuildStepKind::Check, "check");
        graph.add_step_dependency(build, check);
        graph.add_step_dependency(check, build);

        let error = plan_step_order(&graph, build).unwrap_err();

        assert_eq!(error, BuildStepPlanError::DependencyCycle(build));
    }
}
use crate::build_graph::{BuildGraph, BuildStepId};
