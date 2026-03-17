macro_rules! define_graph_id {
    ($name:ident, $label:literal) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(pub usize);

        impl $name {
            pub fn index(self) -> usize {
                self.0
            }

            pub fn from_index(index: usize) -> Self {
                Self(index)
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}{}", $label, self.0)
            }
        }
    };
}

define_graph_id!(BuildStepId, "step:");
define_graph_id!(BuildArtifactId, "artifact:");
define_graph_id!(BuildModuleId, "module:");
define_graph_id!(BuildGeneratedFileId, "generated:");
define_graph_id!(BuildOptionId, "option:");
define_graph_id!(BuildInstallId, "install:");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildStepKind {
    Default,
    Install,
    Run,
    Test,
    Check,
    CustomCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildArtifactKind {
    Executable,
    StaticLibrary,
    SharedLibrary,
    Object,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildModuleKind {
    Source,
    Generated,
    Imported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildGeneratedFileKind {
    Write,
    Copy,
    CaptureOutput,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildOptionKind {
    Target,
    Optimize,
    Bool,
    String,
    Enum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildInstallKind {
    Artifact,
    File,
    Directory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildStep {
    pub id: BuildStepId,
    pub kind: BuildStepKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildArtifact {
    pub id: BuildArtifactId,
    pub kind: BuildArtifactKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildModule {
    pub id: BuildModuleId,
    pub kind: BuildModuleKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildGeneratedFile {
    pub id: BuildGeneratedFileId,
    pub kind: BuildGeneratedFileKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildOption {
    pub id: BuildOptionId,
    pub kind: BuildOptionKind,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildInstall {
    pub id: BuildInstallId,
    pub kind: BuildInstallKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildStepDependency {
    pub step: BuildStepId,
    pub depends_on: BuildStepId,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BuildGraph {
    steps: Vec<BuildStep>,
    artifacts: Vec<BuildArtifact>,
    modules: Vec<BuildModule>,
    generated_files: Vec<BuildGeneratedFile>,
    options: Vec<BuildOption>,
    installs: Vec<BuildInstall>,
    step_dependencies: Vec<BuildStepDependency>,
}

impl BuildGraph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn steps(&self) -> &[BuildStep] {
        &self.steps
    }

    pub fn artifacts(&self) -> &[BuildArtifact] {
        &self.artifacts
    }

    pub fn modules(&self) -> &[BuildModule] {
        &self.modules
    }

    pub fn generated_files(&self) -> &[BuildGeneratedFile] {
        &self.generated_files
    }

    pub fn options(&self) -> &[BuildOption] {
        &self.options
    }

    pub fn installs(&self) -> &[BuildInstall] {
        &self.installs
    }

    pub fn step_dependencies(&self) -> &[BuildStepDependency] {
        &self.step_dependencies
    }

    pub fn add_step(&mut self, kind: BuildStepKind, name: impl Into<String>) -> BuildStepId {
        let id = BuildStepId::from_index(self.steps.len());
        self.steps.push(BuildStep {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_artifact(
        &mut self,
        kind: BuildArtifactKind,
        name: impl Into<String>,
    ) -> BuildArtifactId {
        let id = BuildArtifactId::from_index(self.artifacts.len());
        self.artifacts.push(BuildArtifact {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_module(&mut self, kind: BuildModuleKind, name: impl Into<String>) -> BuildModuleId {
        let id = BuildModuleId::from_index(self.modules.len());
        self.modules.push(BuildModule {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_generated_file(
        &mut self,
        kind: BuildGeneratedFileKind,
        name: impl Into<String>,
    ) -> BuildGeneratedFileId {
        let id = BuildGeneratedFileId::from_index(self.generated_files.len());
        self.generated_files.push(BuildGeneratedFile {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_option(&mut self, kind: BuildOptionKind, name: impl Into<String>) -> BuildOptionId {
        let id = BuildOptionId::from_index(self.options.len());
        self.options.push(BuildOption {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_install(
        &mut self,
        kind: BuildInstallKind,
        name: impl Into<String>,
    ) -> BuildInstallId {
        let id = BuildInstallId::from_index(self.installs.len());
        self.installs.push(BuildInstall {
            id,
            kind,
            name: name.into(),
        });
        id
    }

    pub fn add_step_dependency(&mut self, step: BuildStepId, depends_on: BuildStepId) {
        self.step_dependencies.push(BuildStepDependency { step, depends_on });
    }

    pub fn step_dependencies_for(&self, step: BuildStepId) -> impl Iterator<Item = BuildStepId> + '_ {
        self.step_dependencies
            .iter()
            .filter(move |edge| edge.step == step)
            .map(|edge| edge.depends_on)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BuildArtifactId, BuildArtifactKind, BuildGeneratedFileId, BuildGeneratedFileKind,
        BuildGraph, BuildInstallId, BuildInstallKind, BuildModuleId, BuildModuleKind,
        BuildOptionId, BuildOptionKind, BuildStepDependency, BuildStepId, BuildStepKind,
    };

    #[test]
    fn build_graph_ids_round_trip_their_raw_indexes() {
        assert_eq!(BuildStepId::from_index(3).index(), 3);
        assert_eq!(BuildArtifactId::from_index(5).index(), 5);
        assert_eq!(BuildModuleId::from_index(7).index(), 7);
        assert_eq!(BuildGeneratedFileId::from_index(11).index(), 11);
        assert_eq!(BuildOptionId::from_index(13).index(), 13);
        assert_eq!(BuildInstallId::from_index(17).index(), 17);
    }

    #[test]
    fn build_graph_ids_render_with_stable_family_prefixes() {
        assert_eq!(BuildStepId(0).to_string(), "step:0");
        assert_eq!(BuildArtifactId(1).to_string(), "artifact:1");
        assert_eq!(BuildModuleId(2).to_string(), "module:2");
        assert_eq!(BuildGeneratedFileId(3).to_string(), "generated:3");
        assert_eq!(BuildOptionId(4).to_string(), "option:4");
        assert_eq!(BuildInstallId(5).to_string(), "install:5");
    }

    #[test]
    fn build_graph_kind_enums_cover_the_round_two_ir_vocab() {
        assert_eq!(BuildStepKind::Run, BuildStepKind::Run);
        assert_eq!(BuildArtifactKind::Executable, BuildArtifactKind::Executable);
        assert_eq!(BuildModuleKind::Generated, BuildModuleKind::Generated);
        assert_eq!(BuildGeneratedFileKind::CaptureOutput, BuildGeneratedFileKind::CaptureOutput);
        assert_eq!(BuildOptionKind::Optimize, BuildOptionKind::Optimize);
        assert_eq!(BuildInstallKind::Directory, BuildInstallKind::Directory);
    }

    #[test]
    fn build_graph_allocators_assign_dense_ids_per_node_family() {
        let mut graph = BuildGraph::new();

        let compile_step = graph.add_step(BuildStepKind::Default, "compile");
        let run_step = graph.add_step(BuildStepKind::Run, "run");
        let exe = graph.add_artifact(BuildArtifactKind::Executable, "app");
        let module = graph.add_module(BuildModuleKind::Source, "app.main");
        let generated = graph.add_generated_file(BuildGeneratedFileKind::Write, "version.rs");
        let option = graph.add_option(BuildOptionKind::Target, "target");
        let install = graph.add_install(BuildInstallKind::Artifact, "install-app");

        assert_eq!(compile_step, BuildStepId(0));
        assert_eq!(run_step, BuildStepId(1));
        assert_eq!(exe, BuildArtifactId(0));
        assert_eq!(module, BuildModuleId(0));
        assert_eq!(generated, BuildGeneratedFileId(0));
        assert_eq!(option, BuildOptionId(0));
        assert_eq!(install, BuildInstallId(0));
    }

    #[test]
    fn build_graph_storage_tables_preserve_inserted_records() {
        let mut graph = BuildGraph::new();

        graph.add_step(BuildStepKind::Test, "test");
        graph.add_artifact(BuildArtifactKind::StaticLibrary, "support");
        graph.add_module(BuildModuleKind::Imported, "dep.math");
        graph.add_generated_file(BuildGeneratedFileKind::Copy, "config.json");
        graph.add_option(BuildOptionKind::Bool, "enable-logs");
        graph.add_install(BuildInstallKind::Directory, "install-assets");

        assert_eq!(graph.steps()[0].name, "test");
        assert_eq!(graph.artifacts()[0].kind, BuildArtifactKind::StaticLibrary);
        assert_eq!(graph.modules()[0].kind, BuildModuleKind::Imported);
        assert_eq!(
            graph.generated_files()[0].kind,
            BuildGeneratedFileKind::Copy
        );
        assert_eq!(graph.options()[0].kind, BuildOptionKind::Bool);
        assert_eq!(graph.installs()[0].kind, BuildInstallKind::Directory);
    }

    #[test]
    fn build_graph_records_explicit_step_dependencies() {
        let mut graph = BuildGraph::new();
        let compile = graph.add_step(BuildStepKind::Default, "compile");
        let test = graph.add_step(BuildStepKind::Test, "test");
        let run = graph.add_step(BuildStepKind::Run, "run");

        graph.add_step_dependency(test, compile);
        graph.add_step_dependency(run, compile);

        assert_eq!(
            graph.step_dependencies(),
            &[
                BuildStepDependency {
                    step: test,
                    depends_on: compile,
                },
                BuildStepDependency {
                    step: run,
                    depends_on: compile,
                },
            ]
        );
    }

    #[test]
    fn build_graph_can_query_dependencies_for_one_step() {
        let mut graph = BuildGraph::new();
        let compile = graph.add_step(BuildStepKind::Default, "compile");
        let install = graph.add_step(BuildStepKind::Install, "install");
        let run = graph.add_step(BuildStepKind::Run, "run");

        graph.add_step_dependency(install, compile);
        graph.add_step_dependency(run, compile);

        let install_dependencies = graph.step_dependencies_for(install).collect::<Vec<_>>();
        let run_dependencies = graph.step_dependencies_for(run).collect::<Vec<_>>();

        assert_eq!(install_dependencies, vec![compile]);
        assert_eq!(run_dependencies, vec![compile]);
    }
}
