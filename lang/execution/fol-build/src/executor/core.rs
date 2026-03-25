use crate::eval::{
    BuildEvaluationError, BuildEvaluationErrorKind,
    BuildEvaluationOperationKind,
};
use fol_lexer::lexer::stage3::Elements;
use fol_parser::ast::{AstNode, CallSurface, LoopCondition, ParsedPackage, WhenCase};
use fol_stream::FileStream;
use std::collections::BTreeMap;
use std::path::Path;

use super::output::ExecutionOutput;
use super::types::{ExecValue, HelperRoutine};

pub(super) const MAX_EVAL_DEPTH: usize = 256;
pub(super) const MAX_SCOPE_SIZE: usize = 10_000;

pub(super) fn is_valid_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    match chars.next() {
        Some(c) if c.is_ascii_lowercase() => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-')
}

// ---- Main executor ---

pub struct BuildBodyExecutor {
    pub(super) scope: BTreeMap<String, ExecValue>,
    pub(super) helpers: BTreeMap<String, HelperRoutine>,
    pub(super) output: ExecutionOutput,
    pub(super) build_path_str: String,
    pub(super) next_run_index: usize,
    pub(super) next_install_index: usize,
    pub(super) last_value: Option<ExecValue>,
    /// Resolved values for standard options (target, optimize), used when evaluating `when` conditions.
    pub(super) resolved_inputs: BTreeMap<String, String>,
    pub(super) recursion_depth: usize,
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
        let parsed = parser.parse_package(&mut lexer).map_err(|diagnostics| {
            let message = diagnostics
                .into_iter()
                .next()
                .map(|d| d.message)
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
        let mut build_entry: Option<Vec<AstNode>> = None;

        for unit in &package.source_units {
            for item in &unit.items {
                match &item.node {
                    AstNode::ProDecl { name, body, .. } if name == "build" => {
                        build_entry = Some(body.clone());
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

        let Some(body) = build_entry else {
            return Ok(None);
        };

        let executor = BuildBodyExecutor {
            scope: BTreeMap::new(),
            helpers,
            output: ExecutionOutput::default(),
            build_path_str: build_path.display().to_string(),
            next_run_index: 0,
            next_install_index: 0,
            last_value: None,
            resolved_inputs: BTreeMap::new(),
            recursion_depth: 0,
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

    pub(super) fn exec_body(&mut self, stmts: &[AstNode]) -> Result<(), BuildEvaluationError> {
        self.recursion_depth += 1;
        if self.recursion_depth > MAX_EVAL_DEPTH {
            self.recursion_depth -= 1;
            return Err(BuildEvaluationError::new(
                BuildEvaluationErrorKind::InvalidInput,
                format!("build script exceeded maximum recursion depth ({MAX_EVAL_DEPTH})"),
            ));
        }
        for stmt in stmts {
            self.exec_stmt(stmt)?;
        }
        self.recursion_depth -= 1;
        Ok(())
    }

    pub(super) fn exec_stmt(&mut self, stmt: &AstNode) -> Result<(), BuildEvaluationError> {
        match stmt {
            AstNode::VarDecl { name, value, .. } => {
                let Some(value) = value.as_deref() else {
                    self.last_value = None;
                    return Ok(());
                };
                if let Some(v) = self.eval_expr(value)? {
                    if !is_valid_identifier(name) {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            format!("invalid variable name '{}': names must match [a-z][a-z0-9_-]*", name),
                        ));
                    }
                    if self.scope.len() >= MAX_SCOPE_SIZE {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            format!("build script exceeded maximum scope size ({MAX_SCOPE_SIZE})"),
                        ));
                    }
                    self.scope.insert(name.clone(), v.clone());
                    self.last_value = Some(v);
                } else {
                    self.last_value = None;
                }
                Ok(())
            }

            AstNode::FunctionCall {
                surface: CallSurface::DotIntrinsic,
                name: _,
                args: _,
                ..
            } => {
                self.last_value = self.eval_expr(stmt)?;
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

    pub(super) fn exec_when(
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

    pub(super) fn exec_loop(
        &mut self,
        condition: &LoopCondition,
        body: &[AstNode],
    ) -> Result<(), BuildEvaluationError> {
        match condition {
            LoopCondition::Iteration { var, iterable, .. } => {
                let items = self.eval_iterable(iterable)?;
                for item in items {
                    if self.scope.len() >= MAX_SCOPE_SIZE {
                        return Err(BuildEvaluationError::new(
                            BuildEvaluationErrorKind::InvalidInput,
                            format!("build script exceeded maximum scope size ({MAX_SCOPE_SIZE})"),
                        ));
                    }
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

    pub(super) fn exec_body_with_return(
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

    pub(super) fn append_step_dependencies(
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

    pub(super) fn unsupported(&self, name: &str) -> BuildEvaluationError {
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
