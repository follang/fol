use crate::eval::{BuildEvaluationOperation};
use crate::runtime::{BuildRuntimeDependencyQuery, BuildRuntimeGeneratedFile};
use std::collections::BTreeMap;
use super::types::ExecArtifact;

// ---- Execution output container ---

#[derive(Debug, Default)]
pub struct ExecutionOutput {
    pub operations: Vec<BuildEvaluationOperation>,
    pub executable_artifacts: Vec<ExecArtifact>,
    pub test_artifacts: Vec<ExecArtifact>,
    pub generated_files: Vec<BuildRuntimeGeneratedFile>,
    pub dependency_queries: Vec<BuildRuntimeDependencyQuery>,
    pub run_steps: BTreeMap<String, String>,
}
