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
    use super::{BuildDefaultStepKind, BuildStepExecutionRequest, BuildStepExecutionResult};

    #[test]
    fn build_default_step_kind_covers_phase_six_defaults() {
        assert_eq!(BuildDefaultStepKind::Build.as_str(), "build");
        assert_eq!(BuildDefaultStepKind::Run.as_str(), "run");
        assert_eq!(BuildDefaultStepKind::Test.as_str(), "test");
        assert_eq!(BuildDefaultStepKind::Install.as_str(), "install");
        assert_eq!(BuildDefaultStepKind::Check.as_str(), "check");
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
}
