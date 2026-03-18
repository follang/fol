use crate::api::{
    CopyFileRequest, DependencyRequest, ExecutableRequest, InstallDirRequest, InstallFileRequest,
    SharedLibraryRequest, StandardOptimizeRequest, StandardTargetRequest, StaticLibraryRequest,
    TestArtifactRequest, UserOptionRequest, WriteFileRequest,
};
use crate::codegen::{CodegenRequest, SystemToolRequest};
use crate::eval::{
    BuildEvaluationError, BuildEvaluationErrorKind, BuildEvaluationInstallArtifactRequest,
    BuildEvaluationOperation, BuildEvaluationOperationKind, BuildEvaluationRunArgKind,
    BuildEvaluationRunRequest, BuildEvaluationStepRequest,
};
use crate::runtime::{
    BuildRuntimeDependencyQuery, BuildRuntimeDependencyQueryKind, BuildRuntimeGeneratedFile,
    BuildRuntimeGeneratedFileKind,
};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{
    AstNode, BinaryOperator, CallSurface, Literal, LoopCondition, ParsedPackage, RecordInitField,
    WhenCase,
};
use fol_stream::FileStream;
use std::collections::BTreeMap;
use std::path::Path;

// ---- Extraction output types (public so eval.rs can build EvaluatedBuildProgram) ---

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecConfigValue {
    Literal(String),
    OptionRef(String),
}

impl ExecConfigValue {
    pub fn placeholder_string(&self) -> String {
        match self {
            Self::Literal(value) => value.clone(),
            Self::OptionRef(name) => name.clone(),
        }
    }

    pub fn resolve(&self, options: &crate::option::ResolvedBuildOptionSet) -> String {
        match self {
            Self::Literal(value) => value.clone(),
            Self::OptionRef(name) => options
                .get(name.as_str())
                .map(str::to_string)
                .unwrap_or_else(|| name.clone()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecArtifact {
    pub name: String,
    pub root_module: ExecConfigValue,
    pub target: Option<ExecConfigValue>,
    pub optimize: Option<ExecConfigValue>,
}

// ---- Internal value type for the execution scope ---

#[derive(Debug, Clone, PartialEq, Eq)]
enum ExecValue {
    Target(String),
    Optimize(String),
    OptionRef(String),
    Str(String),
    Bool(bool),
    Artifact(ExecArtifact),
    Module {
        name: String,
    },
    GeneratedFile {
        name: String,
        path: String,
        kind: BuildRuntimeGeneratedFileKind,
    },
    Step {
        name: String,
    },
    Run {
        name: String,
    },
    Install {
        name: String,
    },
    Dependency {
        alias: String,
    },
    DependencyModule {
        alias: String,
        query_name: String,
    },
    DependencyArtifact {
        alias: String,
        query_name: String,
    },
    DependencyStep {
        alias: String,
        query_name: String,
    },
    DependencyGenerated {
        alias: String,
        query_name: String,
    },
    List(Vec<ExecValue>),
}


// ---- Helper routine representation ---

struct HelperRoutine {
    params: Vec<String>,
    body: Vec<AstNode>,
}

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

// ---- Main executor ---

pub struct BuildBodyExecutor {
    graph_param: String,
    scope: BTreeMap<String, ExecValue>,
    helpers: BTreeMap<String, HelperRoutine>,
    output: ExecutionOutput,
    build_path_str: String,
    next_run_index: usize,
    next_install_index: usize,
    last_value: Option<ExecValue>,
    /// Resolved values for standard options (target, optimize), used when evaluating `when` conditions.
    resolved_inputs: BTreeMap<String, String>,
}

impl BuildBodyExecutor {
    /// Parse a build.fol file and create an executor ready to run the `build` routine.
    /// Returns None if no `build` entry is found.
    pub fn from_file(
        build_path: &Path,
    ) -> Result<Option<(BuildBodyExecutor, Vec<AstNode>)>, BuildEvaluationError> {
        let path_str = build_path.to_str().ok_or_else(|| {
            BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!(
                    "build file path '{}' is not valid UTF-8",
                    build_path.display()
                ),
            )
        })?;

        let mut stream = FileStream::from_file(path_str).map_err(|error| {
            BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!(
                    "could not open build file '{}': {}",
                    build_path.display(),
                    error
                ),
            )
        })?;

        let mut lexer = Elements::init(&mut stream);
        let mut parser = fol_parser::ast::AstParser::new();
        let parsed = parser.parse_package(&mut lexer).map_err(|errors| {
            let message = errors
                .into_iter()
                .next()
                .map(|e| e.to_string())
                .unwrap_or_else(|| "unknown parse error".to_string());
            BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!(
                    "build file '{}' failed to parse: {}",
                    build_path.display(),
                    message
                ),
            )
        })?;

        Self::from_parsed_package(&parsed, build_path)
    }

    /// Build an executor from an already-parsed package.
    /// Returns None if no `build` entry is found.
    pub fn from_parsed_package(
        package: &ParsedPackage,
        build_path: &Path,
    ) -> Result<Option<(BuildBodyExecutor, Vec<AstNode>)>, BuildEvaluationError> {
        let mut helpers: BTreeMap<String, HelperRoutine> = BTreeMap::new();
        let mut build_entry: Option<(String, Vec<AstNode>)> = None;

        for unit in &package.source_units {
            for item in &unit.items {
                match &item.node {
                    AstNode::ProDecl { name, params, body, .. } if name == "build" => {
                        let graph_param = params
                            .first()
                            .map(|p| p.name.clone())
                            .unwrap_or_else(|| "graph".to_string());
                        build_entry = Some((graph_param, body.clone()));
                    }
                    AstNode::FunDecl { name, params, body, .. }
                    | AstNode::ProDecl { name, params, body, .. }
                        if name != "build" =>
                    {
                        helpers.insert(
                            name.clone(),
                            HelperRoutine {
                                params: params.iter().map(|p| p.name.clone()).collect(),
                                body: body.clone(),
                            },
                        );
                    }
                    _ => {}
                }
            }
        }

        let Some((graph_param, body)) = build_entry else {
            return Ok(None);
        };

        let executor = BuildBodyExecutor {
            graph_param,
            scope: BTreeMap::new(),
            helpers,
            output: ExecutionOutput::default(),
            build_path_str: build_path.display().to_string(),
            next_run_index: 0,
            next_install_index: 0,
            last_value: None,
            resolved_inputs: BTreeMap::new(),
        };

        Ok(Some((executor, body)))
    }

    /// Execute the build routine body and return the collected output.
    pub fn execute(mut self, body: &[AstNode]) -> Result<ExecutionOutput, BuildEvaluationError> {
        self.exec_body(body)?;
        Ok(self.output)
    }

    /// Set resolved option values used when evaluating `when` conditions.
    pub fn with_resolved_inputs(mut self, inputs: BTreeMap<String, String>) -> Self {
        self.resolved_inputs = inputs;
        self
    }

    fn exec_body(&mut self, stmts: &[AstNode]) -> Result<(), BuildEvaluationError> {
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        Ok(())
    }

    fn exec_stmt(&mut self, stmt: &AstNode) -> Result<(), BuildEvaluationError> {
        match stmt {
            AstNode::VarDecl { name, value, .. } => {
                let Some(value) = value.as_deref() else {
                    self.last_value = None;
                    return Ok(());
                };
                if let Some(v) = self.eval_expr(value)? {
                    self.scope.insert(name.clone(), v.clone());
                    self.last_value = Some(v);
                } else {
                    self.last_value = None;
                }
                Ok(())
            }

            AstNode::FunctionCall {
                surface: CallSurface::DotIntrinsic,
                name,
                args,
                ..
            } => {
                let Some(receiver) = self.last_value.clone() else {
                    return Err(self.unsupported(name));
                };
                self.last_value = self.eval_handle_method(receiver, name, args)?;
                Ok(())
            }

            AstNode::Return { .. } | AstNode::Break => {
                self.last_value = None;
                Ok(())
            }

            AstNode::When { expr, cases, default } => {
                self.exec_when(expr, cases, default.as_deref())?;
                self.last_value = None;
                Ok(())
            }

            AstNode::Loop { condition, body } => {
                self.exec_loop(condition, body)?;
                self.last_value = None;
                Ok(())
            }

            other => {
                self.last_value = self.eval_expr(other)?;
                Ok(())
            }
        }
    }

    fn exec_when(
        &mut self,
        expr: &AstNode,
        cases: &[WhenCase],
        default: Option<&[AstNode]>,
    ) -> Result<(), BuildEvaluationError> {
        // When there are no case sub-clauses, the outer `expr` acts as a boolean gate:
        // `when(condition) { { body } }` — run body only when condition is truthy.
        // `when(condition) { stmts }` — the parser puts direct statements into `default`.
        if cases.is_empty() {
            if self.eval_condition(expr)? {
                if let Some(default_body) = default {
                    self.exec_body(default_body)?;
                }
            }
            return Ok(());
        }

        // When case sub-clauses are present, the outer expr is the match subject.
        // Each `case(condition)` is evaluated; the first match wins.
        let mut matched = false;
        for case in cases {
            match case {
                WhenCase::Case { condition, body } => {
                    if self.eval_condition(condition)? {
                        self.exec_body(body)?;
                        matched = true;
                        break;
                    }
                }
                _ => {
                    // Is/In/Has/On — not supported in build evaluation
                }
            }
        }
        if !matched {
            if let Some(default_body) = default {
                self.exec_body(default_body)?;
            }
        }
        Ok(())
    }

    fn exec_loop(
        &mut self,
        condition: &LoopCondition,
        body: &[AstNode],
    ) -> Result<(), BuildEvaluationError> {
        match condition {
            LoopCondition::Iteration { var, iterable, .. } => {
                let items = self.eval_iterable(iterable)?;
                for item in items {
                    self.scope.insert(var.clone(), item);
                    self.exec_body(body)?;
                }
            }
            LoopCondition::Condition(_) => {
                // While-like loops are not supported in build evaluation
            }
        }
        Ok(())
    }

    fn eval_iterable(&self, node: &AstNode) -> Result<Vec<ExecValue>, BuildEvaluationError> {
        match node {
            AstNode::ContainerLiteral { elements, .. } => {
                let mut result = Vec::with_capacity(elements.len());
                for elem in elements {
                    match elem {
                        AstNode::Literal(Literal::String(s)) => {
                            result.push(ExecValue::Str(s.clone()));
                        }
                        AstNode::Literal(Literal::Boolean(b)) => {
                            result.push(ExecValue::Bool(*b));
                        }
                        AstNode::Identifier { name, .. } => {
                            if let Some(v) = self.scope.get(name.as_str()) {
                                result.push(v.clone());
                            }
                        }
                        _ => {}
                    }
                }
                Ok(result)
            }
            AstNode::Identifier { name, .. } => {
                match self.scope.get(name.as_str()) {
                    Some(ExecValue::List(items)) => Ok(items.clone()),
                    _ => Ok(Vec::new()),
                }
            }
            _ => Ok(Vec::new()),
        }
    }

    fn eval_condition(&self, cond: &AstNode) -> Result<bool, BuildEvaluationError> {
        match cond {
            AstNode::Literal(Literal::Boolean(b)) => Ok(*b),
            AstNode::Identifier { name, .. } => {
                if let Some(v) = self.scope.get(name.as_str()) {
                    match v {
                        ExecValue::Bool(b) => Ok(*b),
                        _ => Ok(true),
                    }
                } else {
                    Ok(false)
                }
            }
            AstNode::BinaryOp { op: BinaryOperator::Eq, left, right } => {
                let lhs = self.eval_value_str(left);
                let rhs = self.eval_value_str(right);
                match (lhs, rhs) {
                    (Some(l), Some(r)) => Ok(l == r),
                    _ => Ok(false),
                }
            }
            AstNode::BinaryOp { op: BinaryOperator::Ne, left, right } => {
                let lhs = self.eval_value_str(left);
                let rhs = self.eval_value_str(right);
                match (lhs, rhs) {
                    (Some(l), Some(r)) => Ok(l != r),
                    _ => Ok(false),
                }
            }
            AstNode::BinaryOp { op: BinaryOperator::And, left, right } => {
                Ok(self.eval_condition(left)? && self.eval_condition(right)?)
            }
            AstNode::BinaryOp { op: BinaryOperator::Or, left, right } => {
                Ok(self.eval_condition(left)? || self.eval_condition(right)?)
            }
            AstNode::UnaryOp { op: fol_parser::ast::UnaryOperator::Not, operand } => {
                Ok(!self.eval_condition(operand)?)
            }
            _ => Ok(false),
        }
    }

    fn eval_value_str(&self, node: &AstNode) -> Option<String> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(s.clone()),
            AstNode::Literal(Literal::Boolean(b)) => Some(b.to_string()),
            AstNode::Identifier { name, .. } => {
                match self.scope.get(name.as_str()) {
                    Some(ExecValue::Target(option_name)) => {
                        // Resolve actual target value from inputs if available
                        self.resolved_inputs
                            .get(option_name.as_str())
                            .cloned()
                            .or_else(|| Some(option_name.clone()))
                    }
                    Some(ExecValue::Optimize(option_name)) => {
                        self.resolved_inputs
                            .get(option_name.as_str())
                            .cloned()
                            .or_else(|| Some(option_name.clone()))
                    }
                    Some(ExecValue::OptionRef(option_name)) => {
                        self.resolved_inputs
                            .get(option_name.as_str())
                            .cloned()
                    }
                    Some(ExecValue::Str(s)) => Some(s.clone()),
                    Some(ExecValue::Bool(b)) => Some(b.to_string()),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn eval_expr(&mut self, expr: &AstNode) -> Result<Option<ExecValue>, BuildEvaluationError> {
        match expr {
            AstNode::Identifier { name, .. } if name == &self.graph_param => Ok(None),
            AstNode::Identifier { name, .. } => Ok(self.scope.get(name.as_str()).cloned()),

            AstNode::MethodCall { object, method, args } => {
                if let AstNode::Identifier { name, .. } = object.as_ref() {
                    if name == &self.graph_param {
                        return self.eval_graph_method(method, args);
                    }
                }
                let Some(receiver) = self.eval_expr(object)? else {
                    return Ok(None);
                };
                self.eval_handle_method(receiver, method, args)
            }

            AstNode::FunctionCall { surface: CallSurface::Plain, name, args, .. } => {
                // Could be a helper routine call
                if self.helpers.contains_key(name.as_str()) {
                    self.eval_helper_call(name, args)
                } else {
                    Ok(None)
                }
            }

            AstNode::ContainerLiteral { elements, .. } => {
                let items = self.eval_iterable(&AstNode::ContainerLiteral {
                    container_type: fol_parser::ast::ContainerType::Array,
                    elements: elements.clone(),
                })?;
                Ok(Some(ExecValue::List(items)))
            }

            AstNode::Literal(Literal::String(s)) => Ok(Some(ExecValue::Str(s.clone()))),
            AstNode::Literal(Literal::Boolean(b)) => Ok(Some(ExecValue::Bool(*b))),

            _ => Ok(None),
        }
    }

    fn eval_helper_call(
        &mut self,
        name: &str,
        args: &[AstNode],
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        // Evaluate args in current scope
        let evaluated_args: Vec<Option<ExecValue>> = args
            .iter()
            .map(|arg| self.eval_expr(arg))
            .collect::<Result<Vec<_>, _>>()?;

        let Some(helper) = self.helpers.get(name) else {
            return Ok(None);
        };

        // Build a child scope with parameter bindings
        let mut child_scope: BTreeMap<String, ExecValue> = BTreeMap::new();

        // Add the graph parameter (first param) if present
        // For helpers that take `graph: Graph` as first param, bind it to a sentinel
        for (param_name, value) in helper.params.iter().zip(evaluated_args.iter()) {
            if let Some(v) = value {
                child_scope.insert(param_name.clone(), v.clone());
            }
        }

        // We need the graph_param name to be accessible in helper execution
        let helper_body = helper.body.clone();
        let helper_params = helper.params.clone();

        // Save current scope and last_value, install helper scope
        let saved_scope = std::mem::replace(&mut self.scope, child_scope);
        let saved_last = self.last_value.take();

        // If helper takes a graph param, wire it up
        if let Some(first_param) = helper_params.first() {
            let saved_graph_param = self.graph_param.clone();
            // Check if any arg was None (the graph arg)
            for (param_name, value) in helper_params.iter().zip(evaluated_args.iter()) {
                if value.is_none() && param_name == first_param {
                    // This is the graph parameter — update the executor's graph_param
                    // so method calls inside the helper know what the graph is named
                    self.graph_param = param_name.clone();
                }
            }
            let result = self.exec_body_with_return(&helper_body);
            self.graph_param = saved_graph_param;
            self.scope = saved_scope;
            self.last_value = saved_last;
            result
        } else {
            let result = self.exec_body_with_return(&helper_body);
            self.scope = saved_scope;
            self.last_value = saved_last;
            result
        }
    }

    fn exec_body_with_return(
        &mut self,
        stmts: &[AstNode],
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        for stmt in stmts {
            match stmt {
                AstNode::Return { value } => {
                    if let Some(value) = value.as_deref() {
                        return self.eval_expr(value);
                    }
                    return Ok(None);
                }
                _ => {
                    self.exec_stmt(stmt)?;
                }
            }
        }
        Ok(self.last_value.clone())
    }

    fn eval_graph_method(
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
                        StandardTargetRequest::new(name.clone()),
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
                        StandardOptimizeRequest::new(name.clone()),
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
                let kind_str = self
                    .resolve_field_string(fields, "kind")
                    .ok_or_else(|| self.unsupported(method))?;
                let kind = parse_option_kind(&kind_str).ok_or_else(|| self.unsupported(method))?;
                let default = parse_option_default(kind, fields, |f| self.resolve_string(f));
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Option(UserOptionRequest {
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
                let path = self
                    .resolve_field_string(fields, "path")
                    .or_else(|| self.resolve_field_string(fields, "source"))
                    .ok_or_else(|| self.unsupported(method))?;
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::InstallFile(InstallFileRequest {
                        name: name.clone(),
                        path,
                        depends_on: Vec::new(),
                    }),
                });
                Ok(Some(ExecValue::Install { name }))
            }

            "install_dir" => {
                let [AstNode::RecordInit { fields, .. }] = args else {
                    return Err(self.unsupported(method));
                };
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let path = self
                    .resolve_field_string(fields, "path")
                    .or_else(|| self.resolve_field_string(fields, "source"))
                    .ok_or_else(|| self.unsupported(method))?;
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
                let source_path = self
                    .resolve_field_string(fields, "source")
                    .or_else(|| self.resolve_field_string(fields, "source_path"))
                    .ok_or_else(|| self.unsupported(method))?;
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
                    name: tool,
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
                let (alias, package, evaluation_mode) = if let [AstNode::RecordInit {
                    fields, ..
                }] = args
                {
                    let alias = self
                        .resolve_field_string(fields, "alias")
                        .ok_or_else(|| self.unsupported(method))?;
                    let package = self
                        .resolve_field_string(fields, "package")
                        .ok_or_else(|| self.unsupported(method))?;
                    let mode = self
                        .resolve_field_string(fields, "mode")
                        .and_then(|v| crate::DependencyBuildEvaluationMode::parse(v.as_str()));
                    (alias, package, mode)
                } else if let [alias_arg, package_arg] = args {
                    let alias = self
                        .resolve_string(alias_arg)
                        .ok_or_else(|| self.unsupported(method))?;
                    let package = self
                        .resolve_string(package_arg)
                        .ok_or_else(|| self.unsupported(method))?;
                    (alias, package, None)
                } else {
                    return Err(self.unsupported(method));
                };
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::Dependency(DependencyRequest {
                        alias: alias.clone(),
                        package,
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

            "path_from_root" => {
                let subpath = match args {
                    [arg] => self.resolve_string(arg).unwrap_or_default(),
                    _ => String::new(),
                };
                Ok(Some(ExecValue::Str(format!("$root/{subpath}"))))
            }

            "build_root" => Ok(Some(ExecValue::Str("$root".to_string()))),

            "install_prefix" => Ok(Some(ExecValue::Str("$prefix".to_string()))),

            _ => Err(self.unsupported(method)),
        }
    }

    fn eval_artifact_method(
        &mut self,
        method: &str,
        args: &[AstNode],
        origin: Option<fol_parser::ast::SyntaxOrigin>,
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        let (name, root_module, target, optimize) = match args {
            [AstNode::RecordInit { fields, .. }] => {
                let name = self
                    .resolve_field_string(fields, "name")
                    .ok_or_else(|| self.unsupported(method))?;
                let root_module = fields
                    .iter()
                    .find(|f| f.name == "root" || f.name == "root_module")
                    .and_then(|f| self.parse_config_value(&f.value, &["path", "string", "target"]))
                    .ok_or_else(|| self.unsupported(method))?;
                let target = fields
                    .iter()
                    .find(|f| f.name == "target")
                    .and_then(|f| self.parse_config_value(&f.value, &["target", "string"]));
                let optimize = fields
                    .iter()
                    .find(|f| f.name == "optimize")
                    .and_then(|f| self.parse_config_value(&f.value, &["optimize", "string"]));
                (name, root_module, target, optimize)
            }
            [name_arg, root_arg] => {
                let name = self
                    .resolve_string(name_arg)
                    .ok_or_else(|| self.unsupported(method))?;
                let root_module = self
                    .parse_config_value(root_arg, &["path", "string"])
                    .ok_or_else(|| self.unsupported(method))?;
                (name, root_module, None, None)
            }
            _ => return Err(self.unsupported(method)),
        };

        let artifact = ExecArtifact {
            name: name.clone(),
            root_module: root_module.clone(),
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
                self.output.operations.push(BuildEvaluationOperation {
                    origin,
                    kind: BuildEvaluationOperationKind::AddStaticLib(StaticLibraryRequest {
                        name: name.clone(),
                        root_module: root_placeholder,
                    }),
                });
            }
            "add_shared_lib" => {
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
            _ => unreachable!("eval_artifact_method called with non-artifact method"),
        }

        Ok(Some(ExecValue::Artifact(artifact)))
    }

    fn eval_handle_method(
        &mut self,
        receiver: ExecValue,
        method: &str,
        args: &[AstNode],
    ) -> Result<Option<ExecValue>, BuildEvaluationError> {
        match &receiver {
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
                    _ => unreachable!(),
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
                    "generated" => ExecValue::DependencyGenerated { alias, query_name },
                    _ => unreachable!(),
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

    fn append_step_dependencies(
        &mut self,
        step_name: &str,
        depends_on: &[String],
    ) -> Result<(), BuildEvaluationError> {
        let Some(op) = self.output.operations.iter_mut().rev().find(|op| {
            match &op.kind {
                BuildEvaluationOperationKind::Step(r) => r.name == step_name,
                BuildEvaluationOperationKind::AddRun(r) => r.name == step_name,
                BuildEvaluationOperationKind::InstallArtifact(r) => r.name == step_name,
                _ => false,
            }
        }) else {
            return Err(BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!("unknown chained step '{step_name}'"),
            ));
        };
        match &mut op.kind {
            BuildEvaluationOperationKind::Step(r) => r.depends_on.extend(depends_on.iter().cloned()),
            BuildEvaluationOperationKind::AddRun(r) => r.depends_on.extend(depends_on.iter().cloned()),
            BuildEvaluationOperationKind::InstallArtifact(r) => r.depends_on.extend(depends_on.iter().cloned()),
            _ => {}
        }
        Ok(())
    }

    // ---- Value resolution helpers ---

    fn resolve_string(&self, node: &AstNode) -> Option<String> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(s.clone()),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Target(s)) => Some(s.clone()),
                Some(ExecValue::Optimize(s)) => Some(s.clone()),
                Some(ExecValue::Str(s)) => Some(s.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn resolve_field_string(&self, fields: &[RecordInitField], field_name: &str) -> Option<String> {
        fields
            .iter()
            .find(|f| f.name == field_name)
            .and_then(|f| self.resolve_string(&f.value))
    }

    fn parse_config_value(&self, node: &AstNode, _allowed_kinds: &[&str]) -> Option<ExecConfigValue> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(ExecConfigValue::Literal(s.clone())),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Target(option_name))
                | Some(ExecValue::Optimize(option_name))
                | Some(ExecValue::OptionRef(option_name)) => {
                    Some(ExecConfigValue::OptionRef(option_name.clone()))
                }
                Some(ExecValue::Str(s)) => Some(ExecConfigValue::Literal(s.clone())),
                _ => None,
            },
            _ => None,
        }
    }

    fn resolve_artifact_ref(&self, node: &AstNode) -> Option<ExecArtifact> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(ExecArtifact {
                name: s.clone(),
                root_module: ExecConfigValue::Literal(String::new()),
                target: None,
                optimize: None,
            }),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Artifact(a)) => Some(a.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn resolve_step_ref(&self, node: &AstNode) -> Option<String> {
        match node {
            AstNode::Literal(Literal::String(s)) => Some(s.clone()),
            AstNode::Identifier { name, .. } => match self.scope.get(name.as_str()) {
                Some(ExecValue::Step { name }) => Some(name.clone()),
                Some(ExecValue::Run { name }) => Some(name.clone()),
                Some(ExecValue::Install { name }) => Some(name.clone()),
                _ => None,
            },
            _ => None,
        }
    }

    fn unsupported(&self, name: &str) -> BuildEvaluationError {
        BuildEvaluationError::with_origin(
            BuildEvaluationErrorKind::Unsupported,
            format!(
                "unsupported build API call in '{}': {}",
                self.build_path_str,
                name.trim()
            ),
            fol_parser::ast::SyntaxOrigin {
                file: Some(self.build_path_str.clone()),
                line: 1,
                column: 1,
                length: name.len(),
            },
        )
    }
}

// ---- Option kind helpers ---

#[derive(Clone, Copy)]
enum OptionKind {
    Target,
    Optimize,
    Bool,
    Int,
    String,
    Enum,
    Path,
}

fn parse_option_kind(raw: &str) -> Option<OptionKind> {
    match raw {
        "target" => Some(OptionKind::Target),
        "optimize" => Some(OptionKind::Optimize),
        "bool" => Some(OptionKind::Bool),
        "int" => Some(OptionKind::Int),
        "string" => Some(OptionKind::String),
        "enum" => Some(OptionKind::Enum),
        "path" => Some(OptionKind::Path),
        _ => None,
    }
}

fn build_option_kind(kind: OptionKind) -> crate::BuildOptionKind {
    match kind {
        OptionKind::Target => crate::BuildOptionKind::Target,
        OptionKind::Optimize => crate::BuildOptionKind::Optimize,
        OptionKind::Bool => crate::BuildOptionKind::Bool,
        OptionKind::Int => crate::BuildOptionKind::Int,
        OptionKind::String => crate::BuildOptionKind::String,
        OptionKind::Enum => crate::BuildOptionKind::Enum,
        OptionKind::Path => crate::BuildOptionKind::Path,
    }
}

fn option_exec_value(kind: OptionKind, name: String) -> ExecValue {
    match kind {
        OptionKind::Target => ExecValue::Target(name),
        OptionKind::Optimize => ExecValue::Optimize(name),
        OptionKind::Bool => ExecValue::Bool(false), // default until resolved
        OptionKind::Int | OptionKind::String | OptionKind::Enum | OptionKind::Path => {
            ExecValue::OptionRef(name)
        }
    }
}

fn parse_option_default(
    kind: OptionKind,
    fields: &[RecordInitField],
    resolve_str: impl Fn(&AstNode) -> Option<String>,
) -> Option<crate::BuildOptionValue> {
    let field = fields.iter().find(|f| f.name == "default")?;
    match (kind, &field.value) {
        (OptionKind::Bool, AstNode::Literal(Literal::Boolean(b))) => {
            Some(crate::BuildOptionValue::Bool(*b))
        }
        (OptionKind::Int, AstNode::Literal(Literal::Integer(i))) => {
            Some(crate::BuildOptionValue::Int(*i))
        }
        (OptionKind::String, _) => {
            resolve_str(&field.value).map(crate::BuildOptionValue::String)
        }
        (OptionKind::Enum, _) => resolve_str(&field.value).map(crate::BuildOptionValue::Enum),
        (OptionKind::Path, _) => resolve_str(&field.value).map(crate::BuildOptionValue::Path),
        _ => None,
    }
}
