use crate::eval::{
    BuildEvaluationError, BuildEvaluationOperation, BuildEvaluationOperationKind,
    BuildEvaluationRunArgKind,
};
use crate::api::DependencyRequest;
use crate::runtime::{BuildRuntimeDependencyQuery, BuildRuntimeDependencyQueryKind, BuildRuntimeGeneratedFileKind};
use fol_parser::ast::AstNode;

use super::core::BuildBodyExecutor;
use super::types::ExecValue;

impl BuildBodyExecutor {
    pub(super) fn eval_handle_method(
        &mut self,
        receiver: ExecValue,
        method: &str,
        args: &[AstNode],
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        match &receiver {
            ExecValue::Build if method == "meta" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                self.resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                self.resolve_field_string(fields, "version")
                    .ok_or_else(|| self.unsupported(method))?;
                Ok(Some(receiver))
            }
            ExecValue::Build if method == "add_dep" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let alias = self
                    .resolve_field_string(fields, "alias")
                    .ok_or_else(|| self.unsupported(method))?;
                let package = self
                    .resolve_field_string(fields, "target")
                    .ok_or_else(|| self.unsupported(method))?;
                let args = self.resolve_dependency_args(fields).unwrap_or_default();
                self.resolve_field_string(fields, "source")
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::Dependency(DependencyRequest {
                        alias: alias.clone(),
                        package,
                        args,
                        evaluation_mode: None,
                        surface: None,
                    }),
                });
                Ok(Some(ExecValue::Dependency { alias }))
            }
            ExecValue::Build if method == "graph" => {
                let [] = args else {
                    return Err(self.unsupported(method));
                };
                Ok(Some(ExecValue::Graph))
            }
            ExecValue::Build => Err(self.unsupported(method)),
            ExecValue::Dependency { alias }
                if matches!(method, "module" | "artifact" | "step" | "generated") =>
            {
                let alias = alias.clone();
                let [name_arg] = args else {
                    return Err(self.unsupported(method));
                };
                let query_name = self
                    .resolve_string(name_arg)
                    .ok_or_else(|| self.unsupported(method))?;
                let kind = match method {
                    "module" => BuildRuntimeDependencyQueryKind::Module,
                    "artifact" => BuildRuntimeDependencyQueryKind::Artifact,
                    "step" => BuildRuntimeDependencyQueryKind::Step,
                    "generated" => BuildRuntimeDependencyQueryKind::GeneratedOutput,
                    _ => return Err(self.unsupported(method)),
                };
                self.output.dependency_queries.push(BuildRuntimeDependencyQuery {
                    dependency_alias: alias.clone(),
                    query_name: query_name.clone(),
                    kind,
                });
                let result = match method {
                    "module" => ExecValue::DependencyModule { alias, query_name },
                    "artifact" => ExecValue::DependencyArtifact { alias, query_name },
                    "step" => ExecValue::DependencyStep { alias, query_name },
                    "generated" => ExecValue::GeneratedFile {
                        name: format!("dep::{alias}::generated::{query_name}"),
                        path: format!("$dep/{alias}/{query_name}"),
                        kind: BuildRuntimeGeneratedFileKind::ToolOutput,
                    },
                    _ => return Err(self.unsupported(method)),
                };
                Ok(Some(result))
            }

            ExecValue::Step { name } | ExecValue::Run { name } | ExecValue::Install { name }
                if method == "depend_on" =>
            {
                let step_name = name.clone();
                let receiver_clone = receiver.clone();
                let depends_on = args
                    .iter()
                    .filter_map(|a| self.resolve_step_ref(a))
                    .collect::<Vec<_>>();
                if depends_on.is_empty() || depends_on.len() != args.len() {
                    return Err(self.unsupported(method));
                }
                self.append_step_dependencies(&step_name, &depends_on)?;
                Ok(Some(receiver_clone))
            }

            // Artifact handle methods
            ExecValue::Artifact(artifact) if method == "link" => {
                let artifact_name = artifact.name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let linked_name = match arg {
                    AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                        Some(ExecValue::Artifact(a)) => a.name.clone(),
                        _ => return Err(self.unsupported(method)),
                    },
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::ArtifactLink {
                        artifact: artifact_name,
                        linked: linked_name,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Artifact(artifact) if method == "import" => {
                let artifact_name = artifact.name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let module_name = match arg {
                    AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                        Some(ExecValue::Module { name }) => name.clone(),
                        Some(ExecValue::DependencyModule { alias, query_name }) => {
                            format!("dep::{alias}::module::{query_name}")
                        }
                        _ => return Err(self.unsupported(method)),
                    },
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::ArtifactImport {
                        artifact: artifact_name,
                        module_name,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Artifact(artifact) if method == "add_generated" => {
                let artifact_name = artifact.name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let generated_name = match arg {
                    AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                        Some(ExecValue::GeneratedFile { name, .. }) => name.clone(),
                        _ => return Err(self.unsupported(method)),
                    },
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::ArtifactAddGenerated {
                        artifact: artifact_name,
                        generated_name,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Artifact { .. } => Err(self.unsupported(method)),

            // Run handle methods
            ExecValue::Run { name } if matches!(method, "add_arg" | "add_dir_arg") => {
                let run_name = name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let value = self
                    .resolve_string(arg)
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::RunAddArg {
                        run_name,
                        kind: BuildEvaluationRunArgKind::Literal,
                        value,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Run { name } if method == "add_file_arg" => {
                let run_name = name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let gen_name = match arg {
                    AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                        Some(ExecValue::GeneratedFile { name, .. }) => name.clone(),
                        _ => return Err(self.unsupported(method)),
                    },
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::RunAddArg {
                        run_name,
                        kind: BuildEvaluationRunArgKind::GeneratedFile,
                        value: gen_name,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Run { name } if method == "capture_stdout" => {
                let run_name = name.clone();
                let output_name = format!("{run_name}-stdout");
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::RunCapture {
                        run_name,
                        output_name: output_name.clone(),
                    },
                });
                Ok(Some(ExecValue::GeneratedFile {
                    name: output_name.clone(),
                    path: output_name,
                    kind: BuildRuntimeGeneratedFileKind::ToolOutput,
                }))
            }

            ExecValue::Run { name } if method == "set_env" => {
                let run_name = name.clone();
                let (key, value) = match args {
                    [key_arg, val_arg] => {
                        let k = self
                            .resolve_string(key_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        let v = self
                            .resolve_string(val_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        (k, v)
                    }
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::RunSetEnv {
                        run_name,
                        key,
                        value,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Run { .. } => Err(self.unsupported(method)),

            // Step handle method: attach
            ExecValue::Step { name } if method == "attach" => {
                let step_name = name.clone();
                let [arg] = args else {
                    return Err(self.unsupported(method));
                };
                let generated_name = match arg {
                    AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                        Some(ExecValue::GeneratedFile { name, .. }) => name.clone(),
                        _ => return Err(self.unsupported(method)),
                    },
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin: None,
                    kind: BuildEvaluationOperationKind::StepAttach {
                        step_name,
                        generated_name,
                    },
                });
                Ok(Some(receiver))
            }

            ExecValue::Step { .. }
            | ExecValue::Install { .. }
            | ExecValue::Dependency { .. } => Err(self.unsupported(method)),

            _ => Ok(None),
        }
    }
}
