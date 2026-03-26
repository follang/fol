mod build_api;
mod tests;
pub mod types;

pub use build_api::BuildApi;
pub use types::{
    validate_build_name, AddModuleRequest, BuildApiError, BuildApiNameError, BuildArtifactHandle,
    BuildOptionValue, CopyFileRequest, DependencyArgValue, DependencyHandle, DependencyRequest,
    ExecutableRequest,
    GeneratedFileHandle, InstallArtifactRequest, InstallDirRequest, InstallFileRequest,
    InstallHandle, ModuleHandle, OutputHandle, OutputHandleKind, OutputHandleLocator, RunHandle,
    RunRequest, SharedLibraryRequest, SourceDirHandle, SourceFileHandle,
    StandardOptimizeOption, StandardOptimizeRequest, StandardTargetOption,
    StandardTargetRequest, StaticLibraryRequest, StepHandle, StepRequest,
    TestArtifactRequest, UserOption, UserOptionRequest, WriteFileRequest,
};
