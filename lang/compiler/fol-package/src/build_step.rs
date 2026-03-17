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

#[cfg(test)]
mod tests {
    use super::{BuildStepExecutionRequest, BuildStepExecutionResult};

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
}
