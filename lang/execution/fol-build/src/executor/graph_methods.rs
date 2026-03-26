use crate::api::{
    CopyFileRequest, DependencyRequest, ExecutableRequest, InstallDirRequest, InstallFileRequest,
    SharedLibraryRequest, StaticLibraryRequest, TestArtifactRequest, WriteFileRequest,
};
use crate::artifact::BuildArtifactFolModel;
use crate::codegen::{CodegenRequest, SystemToolRequest};
use crate::eval::{
    BuildEvaluationError, BuildEvaluationErrorKind, BuildEvaluationInstallArtifactRequest,
    BuildEvaluationOperation, BuildEvaluationOperationKind,
    BuildEvaluationRunRequest, BuildEvaluationStepRequest,
};
use crate::runtime::{BuildRuntimeGeneratedFile, BuildRuntimeGeneratedFileKind};
use fol_parser::ast::AstNode;

use super::core::{BuildBodyExecutor, is_valid_identifier};
use super::types::{ExecArtifact, ExecValue};
use super::option_helpers::{
    build_option_kind, option_exec_value, parse_option_default, parse_option_kind,
};

impl BuildBodyExecutor {
    fn resolve_source_file_config(
        &self,
        method: &str,
        fields: &[fol_parser::ast::RecordInitField],
        field_name: &str,
    ) -> Result<String, BuildEvaluationError> {
        let field = fields
            .iter()
            .find(|field| field.name == field_name)
            .ok_or_else(|| {
                BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!("{method} config is invalid: missing '{field_name}'"),
                )
            })?;
        match &field.value {
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::SourceFile { path }) => Ok(path.clone()),
                Some(ExecValue::SourceDir { .. }) => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-file handle, not a source-dir handle"
                    ),
                )),
                Some(ExecValue::GeneratedFile { .. }) => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-file handle, not a generated-output handle"
                    ),
                )),
                _ => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-file handle"
                    ),
                )),
            },
            _ => Err(BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!("{method} config is invalid: '{field_name}' must be a source-file handle"),
            )),
        }
    }

    fn resolve_source_dir_config(
        &self,
        method: &str,
        fields: &[fol_parser::ast::RecordInitField],
        field_name: &str,
    ) -> Result<String, BuildEvaluationError> {
        let field = fields
            .iter()
            .find(|field| field.name == field_name)
            .ok_or_else(|| {
                BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!("{method} config is invalid: missing '{field_name}'"),
                )
            })?;
        match &field.value {
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::SourceDir { path }) => Ok(path.clone()),
                Some(ExecValue::SourceFile { .. }) => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-dir handle, not a source-file handle"
                    ),
                )),
                Some(ExecValue::GeneratedFile { .. }) => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-dir handle, not a generated-output handle"
                    ),
                )),
                _ => Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::InvalidInput,
                    format!(
                        "{method} config is invalid: '{field_name}' must be a source-dir handle"
                    ),
                )),
            },
            _ => Err(BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!("{method} config is invalid: '{field_name}' must be a source-dir handle"),
            )),
        }
    }

    pub(super) fn eval_graph_method(
        &mut self,
        method: &str,
        args: &[AstNode],
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        let origin = Some(fol_parser::ast::SyntaxOrigin {
            file: Some(self.build_path_str.clone()),
            line: 1,
            column: 1,
            length: method.len(),
        });

        match method {
            "standard_target" => {
                let name = match args {
                    [] => "target".to_string(),
                    [arg] => self
                        .resolve_string(arg)
                        .ok_or_else(|| self.unsupported(method))?,
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::StandardTarget(
                        crate::api::StandardTargetRequest::new(name.clone()),
                    ),
                });
                Ok(Some(ExecValue::Target(name)))
            }

            "standard_optimize" => {
                let name = match args {
                    [] => "optimize".to_string(),
                    [arg] => self
                        .resolve_string(arg)
                        .ok_or_else(|| self.unsupported(method))?,
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::StandardOptimize(
                        crate::api::StandardOptimizeRequest::new(name.clone()),
                    ),
                });
                Ok(Some(ExecValue::Optimize(name)))
            }

            "option" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                if !is_valid_identifier(&name) {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("invalid option name '{}': names must match [a-z][a-z0-9_-]*", name),
                    ));
                }
                let kind_str = self
                    .resolve_field_string(fields, "kind")
                    .ok_or_else(|| self.unsupported(method))?;
                let kind = parse_option_kind(&kind_str).ok_or_else(|| self.unsupported(method))?;
                let default = parse_option_default(kind, fields, |f| self.resolve_string(f));
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Option(crate::api::UserOptionRequest {
                        name: name.clone(),
                        kind: build_option_kind(kind),
                        default,
                    }),
                });
                Ok(Some(option_exec_value(kind, name)))
            }

            "add_exe" | "add_static_lib" | "add_shared_lib" | "add_test" => {
                self.eval_artifact_method(method, args, origin)
            }

            "step" => {
                let name = args
                    .first()
                    .and_then(|a| self.resolve_string(a))
                    .ok_or_else(|| self.unsupported(method))?;
                if !is_valid_identifier(&name) {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("invalid step name '{}': names must match [a-z][a-z0-9_-]*", name),
                    ));
                }
                let depends_on = args
                    .iter()
                    .skip(1)
                    .filter_map(|a| self.resolve_step_ref(a))
                    .collect::<Vec<_>>();
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Step(BuildEvaluationStepRequest {
                        name: name.clone(),
                        depends_on,
                    }),
                });
                Ok(Some(ExecValue::Step { name }))
            }

            "add_run" => {
                let (step_name, artifact_name) = match args {
                    [artifact_arg] => {
                        let artifact = self
                            .resolve_artifact_ref(artifact_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        let step_name = if self.next_run_index == 0 {
                            "run".to_string()
                        } else {
                            format!("run-{}", artifact.name)
                        };
                        self.next_run_index += 1;
                        (step_name, artifact.name.clone())
                    }
                    [name_arg, artifact_arg, ..] => {
                        let step_name = self
                            .resolve_string(name_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        let artifact = self
                            .resolve_artifact_ref(artifact_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        (step_name, artifact.name.clone())
                    }
                    _ => return Err(self.unsupported(method)),
                };
                self.output
                    .run_steps
                    .insert(step_name.clone(), artifact_name.clone());
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddRun(BuildEvaluationRunRequest {
                        name: step_name.clone(),
                        artifact: artifact_name,
                        depends_on: Vec::new(),
                    }),
                });
                Ok(Some(ExecValue::Run { name: step_name }))
            }

            "install" => {
                let (step_name, artifact_name) = match args {
                    [artifact_arg] => {
                        let artifact = self
                            .resolve_artifact_ref(artifact_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        let step_name = if self.next_install_index == 0 {
                            "install".to_string()
                        } else {
                            format!("install-{}", artifact.name)
                        };
                        self.next_install_index += 1;
                        (step_name, artifact.name.clone())
                    }
                    [name_arg, artifact_arg] => {
                        let step_name = self
                            .resolve_string(name_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        let artifact = self
                            .resolve_artifact_ref(artifact_arg)
                            .ok_or_else(|| self.unsupported(method))?;
                        (step_name, artifact.name.clone())
                    }
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::InstallArtifact(
                        BuildEvaluationInstallArtifactRequest {
                            name: step_name.clone(),
                            artifact: artifact_name,
                            depends_on: Vec::new(),
                        },
                    ),
                });
                Ok(Some(ExecValue::Install { name: step_name }))
            }

            "install_file" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let source_field = fields
                    .iter()
                    .find(|field| field.name == "path" || field.name == "source")
                    .ok_or_else(|| {
                        BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "install_file config is invalid: missing 'source'".to_string(),
                        )
                    })?;
                let kind = match &source_field.value {
                    AstNode::Identifier {
                        name: handle_name, ..
                    } => match self.scope.get(handle_name.as_str()) {
                        Some(ExecValue::GeneratedFile {
                            name: generated_name,
                            ..
                        }) => {
                            BuildEvaluationOperationKind::InstallGeneratedFile {
                                name: name.clone(),
                                generated_name: generated_name.clone(),
                            }
                        }
                        Some(ExecValue::SourceFile { path }) => {
                            BuildEvaluationOperationKind::InstallFile(InstallFileRequest {
                                name: name.clone(),
                                path: path.clone(),
                                depends_on: Vec::new(),
                            })
                        }
                        Some(ExecValue::SourceDir { .. }) => {
                            return Err(BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                "install_file config is invalid: 'source' must be a source-file handle or generated-output handle, not a source-dir handle".to_string(),
                            ))
                        }
                        _ => {
                            return Err(BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                "install_file config is invalid: 'source' must be a source-file handle or generated-output handle".to_string(),
                            ))
                        }
                    },
                    _ => {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "install_file config is invalid: 'source' must be a source-file handle or generated-output handle".to_string(),
                        ))
                    }
                };
                self.output.operations.push(BuildEvaluationOperation { origin, kind });
                Ok(Some(ExecValue::Install { name }))
            }

            "install_dir" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let path = self.resolve_source_dir_config(method, fields, "source")?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::InstallDir(InstallDirRequest {
                        name: name.clone(),
                        path,
                        depends_on: Vec::new(),
                    }),
                });
                Ok(Some(ExecValue::Install { name }))
            }

            "write_file" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let path = self
                    .resolve_field_string(fields, "path")
                    .ok_or_else(|| self.unsupported(method))?;
                let contents = self
                    .resolve_field_string(fields, "contents")
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::WriteFile(WriteFileRequest {
                        name: name.clone(),
                        path: path.clone(),
                        contents,
                    }),
                });
                let gen = BuildRuntimeGeneratedFile::new(
                    name.clone(),
                    path.clone(),
                    BuildRuntimeGeneratedFileKind::Write,
                );
                self.output.generated_files.push(gen);
                Ok(Some(ExecValue::GeneratedFile {
                    name,
                    path,
                    kind: BuildRuntimeGeneratedFileKind::Write,
                }))
            }

            "copy_file" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let source_path = self.resolve_source_file_config(method, fields, "source")?;
                let destination_path = self
                    .resolve_field_string(fields, "path")
                    .or_else(|| self.resolve_field_string(fields, "destination"))
                    .or_else(|| self.resolve_field_string(fields, "destination_path"))
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::CopyFile(CopyFileRequest {
                        name: name.clone(),
                        source_path,
                        destination_path: destination_path.clone(),
                    }),
                });
                let gen = BuildRuntimeGeneratedFile::new(
                    name.clone(),
                    destination_path.clone(),
                    BuildRuntimeGeneratedFileKind::Copy,
                );
                self.output.generated_files.push(gen);
                Ok(Some(ExecValue::GeneratedFile {
                    name,
                    path: destination_path,
                    kind: BuildRuntimeGeneratedFileKind::Copy,
                }))
            }

            "add_system_tool" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let tool = self
                    .resolve_field_string(fields, "tool")
                    .ok_or_else(|| self.unsupported(method))?;
                let output = self
                    .resolve_field_string(fields, "output")
                    .or_else(|| self.resolve_field_string(fields, "path"))
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::SystemTool(SystemToolRequest {
                        tool: tool.clone(),
                        args: Vec::new(),
                        outputs: vec![output.clone()],
                    }),
                });
                let gen = BuildRuntimeGeneratedFile::new(
                    tool.clone(),
                    output.clone(),
                    BuildRuntimeGeneratedFileKind::ToolOutput,
                );
                self.output.generated_files.push(gen);
                Ok(Some(ExecValue::GeneratedFile {
                    name: output.clone(),
                    path: output,
                    kind: BuildRuntimeGeneratedFileKind::ToolOutput,
                }))
            }

            "add_codegen" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let kind_str = self
                    .resolve_field_string(fields, "kind")
                    .ok_or_else(|| self.unsupported(method))?;
                let input = self
                    .resolve_field_string(fields, "input")
                    .ok_or_else(|| self.unsupported(method))?;
                let output = self
                    .resolve_field_string(fields, "output")
                    .or_else(|| self.resolve_field_string(fields, "path"))
                    .ok_or_else(|| self.unsupported(method))?;
                let codegen_kind = match kind_str.as_str() {
                    "fol" | "fol-to-fol" => crate::CodegenKind::FolToFol,
                    "schema" => crate::CodegenKind::Schema,
                    "asset" | "asset-preprocess" => crate::CodegenKind::AssetPreprocess,
                    _ => return Err(self.unsupported(method)),
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Codegen(CodegenRequest {
                        kind: codegen_kind,
                        input,
                        output: output.clone(),
                    }),
                });
                let gen = BuildRuntimeGeneratedFile::new(
                    output.clone(),
                    output.clone(),
                    BuildRuntimeGeneratedFileKind::CodegenOutput,
                );
                self.output.generated_files.push(gen);
                Ok(Some(ExecValue::GeneratedFile {
                    name: output.clone(),
                    path: output,
                    kind: BuildRuntimeGeneratedFileKind::CodegenOutput,
                }))
            }

            "dependency" => {
                let (alias, package, forwarded_args, evaluation_mode) = if let [AstNode::RecordInit {
                    fields, ..
                }] = args
                {
                    let alias = self
                        .resolve_field_string(fields, "alias")
                        .ok_or_else(|| self.unsupported(method))?;
                    let package = self
                        .resolve_field_string(fields, "package")
                        .ok_or_else(|| self.unsupported(method))?;
                    let forwarded_args = self.resolve_dependency_args(fields)?.unwrap_or_default();
                    let mode = self
                        .resolve_field_string(fields, "mode")
                        .and_then(|v| crate::DependencyBuildEvaluationMode::parse(v.as_str()));
                    (alias, package, forwarded_args, mode)
                } else if let [alias_arg, package_arg] = args {
                    let alias = self
                        .resolve_string(alias_arg)
                        .ok_or_else(|| self.unsupported(method))?;
                    let package = self
                        .resolve_string(package_arg)
                        .ok_or_else(|| self.unsupported(method))?;
                    (alias, package, std::collections::BTreeMap::<String, crate::api::DependencyArgValue>::new(), None)
                } else {
                    return Err(self.unsupported(method));
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Dependency(DependencyRequest {
                        alias: alias.clone(),
                        package,
                        args: forwarded_args,
                        evaluation_mode,
                        surface: None,
                    }),
                });
                Ok(Some(ExecValue::Dependency { alias }))
            }

            "add_module" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let root_module = self
                    .resolve_field_string(fields, "root")
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddModule(crate::api::AddModuleRequest {
                        name: name.clone(),
                        root_module,
                    }),
                });
                Ok(Some(ExecValue::Module { name }))
            }

            "file_from_root" => {
                let subpath = match args {
                    [arg] => self.resolve_string(arg).ok_or_else(|| {
                        BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "file_from_root requires one string path argument".to_string(),
                        )
                    })?,
                    _ => {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "file_from_root requires one string path argument".to_string(),
                        ))
                    }
                };
                if subpath.trim().is_empty() {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        "file_from_root requires a non-empty relative path".to_string(),
                    ));
                }
                Ok(Some(ExecValue::SourceFile { path: subpath }))
            }

            "dir_from_root" => {
                let subpath = match args {
                    [arg] => self.resolve_string(arg).ok_or_else(|| {
                        BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "dir_from_root requires one string path argument".to_string(),
                        )
                    })?,
                    _ => {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            "dir_from_root requires one string path argument".to_string(),
                        ))
                    }
                };
                if subpath.trim().is_empty() {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        "dir_from_root requires a non-empty relative path".to_string(),
                    ));
                }
                Ok(Some(ExecValue::SourceDir { path: subpath }))
            }

            "build_root" => Ok(Some(ExecValue::Str(self.package_root_str.clone()))),

            "install_prefix" => Ok(Some(ExecValue::Str(self.install_prefix_str.clone()))),

            _ => Err(self.unsupported(method)),
        }
    }

    pub(super) fn eval_artifact_method(
        &mut self,
        method: &str,
        args: &[AstNode],
        origin: Option<fol_parser::ast::SyntaxOrigin>,
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        let (name, root_module, fol_model, target, optimize) = match args {
            [AstNode::RecordInit { fields, .. }] => {
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("{method} config is invalid: artifact config requires string field 'name'"),
                    ))?;
                let root_field = fields
                    .iter()
                    .find(|f| f.name == "root" || f.name == "root_module")
                    .ok_or_else(|| BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("{method} config is invalid: artifact config requires 'root'"),
                    ))?;
                let root_module = self
                    .parse_config_value(&root_field.value, &["path", "string", "target"])
                    .ok_or_else(|| BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("{method} config is invalid: artifact 'root' must be a string path or path-like option"),
                    ))?;
                if root_module.placeholder_string().trim().is_empty() {
                    return Err(BuildEvaluationError::new(
                        BuildEvaluationErrorKind::InvalidInput,
                        format!("{method} config is invalid: artifact 'root' must not be empty"),
                    ));
                }
                let target = fields
                    .iter()
                    .find(|f| f.name == "target")
                    .map(|f| {
                        self.parse_config_value(&f.value, &["target", "string"])
                            .ok_or_else(|| BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                format!("{method} config is invalid: artifact 'target' must be a target handle or string triple"),
                            ))
                    })
                    .transpose()?;
                let fol_model = match fields.iter().find(|f| f.name == "fol_model") {
                    Some(field) => {
                        let raw = self
                            .resolve_string(&field.value)
                            .ok_or_else(|| BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                format!("{method} config is invalid: artifact 'fol_model' must be a string"),
                            ))?;
                        BuildArtifactFolModel::parse(raw.as_str()).ok_or_else(|| {
                            BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                format!(
                                    "artifact fol_model must be one of: core, alloc, std (got '{}')",
                                    raw
                                ),
                            )
                        })?
                    }
                    None => BuildArtifactFolModel::Std,
                };
                let optimize = fields
                    .iter()
                    .find(|f| f.name == "optimize")
                    .map(|f| {
                        self.parse_config_value(&f.value, &["optimize", "string"])
                            .ok_or_else(|| BuildEvaluationError::new(
                                BuildEvaluationErrorKind::InvalidInput,
                                format!("{method} config is invalid: artifact 'optimize' must be an optimize handle or string mode"),
                            ))
                    })
                    .transpose()?;
                (name, root_module, fol_model, target, optimize)
            }
            [name_arg, root_arg] => {
                let name = self
                    .resolve_string(name_arg)
                    .ok_or_else(|| self.unsupported(method))?;
                let root_module = self
                    .parse_config_value(root_arg, &["path", "string"])
                    .ok_or_else(|| self.unsupported(method))?;
                (name, root_module, BuildArtifactFolModel::Std, None, None)
            }
            _ => return Err(self.unsupported(method)),
        };

        let artifact = ExecArtifact {
            name: name.clone(),
            root_module: root_module.clone(),
            fol_model,
            target: target.clone(),
            optimize: optimize.clone(),
        };
        let root_placeholder = root_module.placeholder_string();

        match method {
            "add_exe" => {
                self.output.executable_artifacts.push(artifact.clone());
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddExe(ExecutableRequest {
                        name: name.clone(),
                        root_module: root_placeholder,
                    }),
                });
            }
            "add_static_lib" => {
                self.output.static_library_artifacts.push(artifact.clone());
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddStaticLib(StaticLibraryRequest {
                        name: name.clone(),
                        root_module: root_placeholder,
                    }),
                });
            }
            "add_shared_lib" => {
                self.output.shared_library_artifacts.push(artifact.clone());
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddSharedLib(SharedLibraryRequest {
                        name: name.clone(),
                        root_module: root_placeholder,
                    }),
                });
            }
            "add_test" => {
                self.output.test_artifacts.push(artifact.clone());
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddTest(TestArtifactRequest {
                        name: name.clone(),
                        root_module: root_placeholder,
                    }),
                });
            }
            _ => {
                return Err(BuildEvaluationError::new(
                    BuildEvaluationErrorKind::Internal,
                    format!("eval_artifact_method called with unexpected method '{method}'"),
                ));
            }
        }

        Ok(Some(ExecValue::Artifact(artifact)))
    }
}
