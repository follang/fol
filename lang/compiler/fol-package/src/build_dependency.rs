pub use fol_build::dependency::*;

use crate::{
    BuildEvaluationInputs, BuildEvaluationOperationKind, BuildEvaluationRequest, PackageError,
    PackageErrorKind, PreparedExportMount,
};
use fol_build::executor::BuildBodyExecutor;
use fol_parser::ast::{ParsedPackage, ParsedSourceUnitKind};
use std::path::Path;

pub fn dependency_modules_from_exports(
    alias: &str,
    exports: &[PreparedExportMount],
) -> Vec<DependencyModuleSurface> {
    exports
        .iter()
        .map(|export| DependencyModuleSurface {
            name: export
                .mounted_namespace_suffix
                .as_deref()
                .unwrap_or(alias)
                .to_string(),
            source_namespace: export.source_namespace.clone(),
        })
        .collect()
}

pub fn project_dependency_surface(
    alias: &str,
    package_root: &Path,
    syntax: &ParsedPackage,
) -> Result<DependencyBuildSurface, PackageError> {
    let build_path = package_root.join("build.fol");
    let (executor, body) = BuildBodyExecutor::from_file(&build_path).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency surface projection could not parse build '{}': {}",
                build_path.display(),
                error
            ),
        )
    })?
    .ok_or_else(|| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency surface projection requires canonical build entry in '{}'",
                build_path.display()
            ),
        )
    })?;
    let exec_output = executor.execute(&body).map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency surface projection could not evaluate build '{}': {}",
                build_path.display(),
                error
            ),
        )
    })?;
    let plan = fol_build::evaluate_build_plan(&BuildEvaluationRequest {
        package_root: package_root.display().to_string(),
        inputs: BuildEvaluationInputs {
            working_directory: package_root.display().to_string(),
            ..BuildEvaluationInputs::default()
        },
        operations: exec_output.operations.clone(),
    })
    .map_err(|error| {
        PackageError::new(
            PackageErrorKind::InvalidInput,
            format!(
                "package dependency surface projection could not plan build '{}': {}",
                build_path.display(),
                error
            ),
        )
    })?;

    let mut source_roots = syntax
        .source_units
        .iter()
        .filter(|unit| unit.kind == ParsedSourceUnitKind::Ordinary)
        .map(|unit| {
            let relative_path = std::path::Path::new(&unit.path)
                .parent()
                .map(|parent| {
                    let rendered = parent.to_string_lossy().to_string();
                    if rendered.is_empty() {
                        ".".to_string()
                    } else {
                        rendered
                    }
                })
                .unwrap_or_else(|| ".".to_string());
            DependencySourceRootSurface {
                namespace_prefix: alias.to_string(),
                relative_path,
            }
        })
        .collect::<Vec<_>>();
    source_roots.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));
    source_roots.dedup_by(|left, right| left.relative_path == right.relative_path);

    let mut modules = exec_output
        .operations
        .iter()
        .filter_map(|operation| match &operation.kind {
            BuildEvaluationOperationKind::AddModule(request) => Some(DependencyModuleSurface {
                name: request.name.clone(),
                source_namespace: request.root_module.clone(),
            }),
            _ => None,
        })
        .collect::<Vec<_>>();
    modules.sort_by(|left, right| left.name.cmp(&right.name));
    modules.dedup_by(|left, right| left.name == right.name);

    let mut artifacts = fol_build::project_graph_artifacts(&plan.graph)
        .into_iter()
        .map(|artifact| DependencyArtifactSurface {
            name: artifact.name,
            artifact_kind: match artifact.kind {
                fol_build::BuildArtifactModelKind::Executable => "exe".to_string(),
                fol_build::BuildArtifactModelKind::StaticLibrary => "static-lib".to_string(),
                fol_build::BuildArtifactModelKind::SharedLibrary => "shared-lib".to_string(),
                fol_build::BuildArtifactModelKind::TestBundle => "test".to_string(),
                fol_build::BuildArtifactModelKind::GeneratedSourceBundle => {
                    "generated-source".to_string()
                }
                fol_build::BuildArtifactModelKind::DocsBundle => "docs".to_string(),
            },
        })
        .collect::<Vec<_>>();
    artifacts.sort_by(|left, right| left.name.cmp(&right.name));
    artifacts.dedup_by(|left, right| left.name == right.name);

    let mut steps = fol_build::project_graph_steps(&plan.graph)
        .into_iter()
        .map(|step| DependencyStepSurface {
            step_kind: step
                .default_kind
                .map(|kind| kind.as_str().to_string())
                .unwrap_or_else(|| "custom".to_string()),
            name: step.name,
        })
        .collect::<Vec<_>>();
    steps.sort_by(|left, right| left.name.cmp(&right.name));
    steps.dedup_by(|left, right| left.name == right.name);

    let mut generated_outputs = exec_output
        .generated_files
        .iter()
        .map(|generated| DependencyGeneratedOutputSurface {
            name: generated.name.clone(),
            relative_path: generated.relative_path.clone(),
        })
        .collect::<Vec<_>>();
    generated_outputs.sort_by(|left, right| left.name.cmp(&right.name));
    generated_outputs.dedup_by(|left, right| left.name == right.name);

    Ok(DependencyBuildSurface {
        alias: alias.to_string(),
        modules,
        source_roots,
        artifacts,
        steps,
        generated_outputs,
    })
}

#[cfg(test)]
mod tests {
    use super::{dependency_modules_from_exports, project_dependency_surface};
    use crate::PreparedExportMount;
    use fol_parser::ast::AstParser;
    use fol_stream::FileStream;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    fn unique_temp_root(label: &str) -> std::path::PathBuf {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        std::env::temp_dir().join(format!(
            "fol_pkg_dep_surface_{}_{}_{}",
            label,
            std::process::id(),
            NEXT_ID.fetch_add(1, Ordering::Relaxed)
        ))
    }

    #[test]
    fn dependency_module_bridge_projects_prepared_exports_into_module_surfaces() {
        let modules = dependency_modules_from_exports(
            "json",
            &[
                PreparedExportMount {
                    source_namespace: "json::src".to_string(),
                    mounted_namespace_suffix: None,
                },
                PreparedExportMount {
                    source_namespace: "json::src::codec".to_string(),
                    mounted_namespace_suffix: Some("codec".to_string()),
                },
            ],
        );

        assert_eq!(modules.len(), 2);
        assert_eq!(modules[0].name, "json");
        assert_eq!(modules[0].source_namespace, "json::src");
        assert_eq!(modules[1].name, "codec");
        assert_eq!(modules[1].source_namespace, "json::src::codec");
    }

    #[test]
    fn projected_dependency_surface_keeps_default_exposed_shapes_deterministic() {
        let root = unique_temp_root("projected_surface");
        fs::create_dir_all(root.join("src/root")).expect("fixture root should exist");
        fs::write(
            root.join("build.fol"),
            concat!(
                "pro[] build(): non = {\n",
                "    var build = .build();\n",
                "    build.meta({ name = \"json\", version = \"1.0.0\" });\n",
                "    var graph = build.graph();\n",
                "    var codec = graph.add_module({ name = \"codec\", root = \"src/root/codec.fol\" });\n",
                "    var app = graph.add_exe({ name = \"json\", root = \"src/root/main.fol\" });\n",
                "    var schema = graph.write_file({ name = \"schema\", path = \"gen/schema.fol\", contents = \"ok\" });\n",
                "    var docs = graph.step(\"docs\");\n",
                "    graph.install(app);\n",
                "    return;\n",
                "}\n",
            ),
        )
        .expect("build fixture should be written");
        fs::write(root.join("src/root/main.fol"), "var[exp] answer: int = 42;\n")
            .expect("source fixture should be written");
        fs::write(root.join("src/root/codec.fol"), "var[exp] codec: int = 7;\n")
            .expect("module fixture should be written");

        let mut stream =
            FileStream::from_folder(root.to_str().expect("temp path should be utf-8"))
                .expect("folder stream should open");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("package syntax should parse");

        let surface =
            project_dependency_surface("json", &root, &syntax).expect("surface should project");

        assert_eq!(surface.alias, "json");
        assert_eq!(
            surface
                .source_roots
                .iter()
                .map(|root| root.relative_path.as_str())
                .collect::<Vec<_>>(),
            vec!["src/root"]
        );
        assert_eq!(
            surface.modules.iter().map(|module| module.name.as_str()).collect::<Vec<_>>(),
            vec!["codec"]
        );
        assert_eq!(
            surface
                .artifacts
                .iter()
                .map(|artifact| artifact.name.as_str())
                .collect::<Vec<_>>(),
            vec!["json"]
        );
        assert!(surface.steps.iter().any(|step| step.name == "docs"));
        assert!(surface.steps.iter().any(|step| step.name == "install"));
        assert_eq!(
            surface
                .generated_outputs
                .iter()
                .map(|output| output.name.as_str())
                .collect::<Vec<_>>(),
            vec!["schema"]
        );

        fs::remove_dir_all(&root).expect("fixture root should be removable");
    }
}
