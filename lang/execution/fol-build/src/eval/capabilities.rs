#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildEvaluationBoundary {
    GraphConstructionSubset,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AllowedBuildTimeOperation {
    GraphMutation,
    OptionRead,
    PathJoin,
    PathNormalize,
    StringBasic,
    ContainerBasic,
    ControlledFileGeneration,
    ControlledProcessExecution,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ForbiddenBuildTimeOperation {
    ArbitraryFilesystemRead,
    ArbitraryFilesystemWrite,
    ArbitraryNetworkAccess,
    WallClockAccess,
    AmbientEnvironmentAccess,
    UncontrolledProcessExecution,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeCapabilityModel {
    pub allowed_operations: Vec<AllowedBuildTimeOperation>,
    pub forbidden_operations: Vec<ForbiddenBuildTimeOperation>,
}

impl BuildRuntimeCapabilityModel {
    pub fn new(
        allowed_operations: Vec<AllowedBuildTimeOperation>,
        forbidden_operations: Vec<ForbiddenBuildTimeOperation>,
    ) -> Self {
        Self {
            allowed_operations,
            forbidden_operations,
        }
    }
}

pub fn canonical_graph_construction_capabilities() -> BuildRuntimeCapabilityModel {
    BuildRuntimeCapabilityModel::new(
        vec![
            AllowedBuildTimeOperation::GraphMutation,
            AllowedBuildTimeOperation::OptionRead,
            AllowedBuildTimeOperation::PathJoin,
            AllowedBuildTimeOperation::PathNormalize,
            AllowedBuildTimeOperation::StringBasic,
            AllowedBuildTimeOperation::ContainerBasic,
            AllowedBuildTimeOperation::ControlledFileGeneration,
            AllowedBuildTimeOperation::ControlledProcessExecution,
        ],
        vec![
            ForbiddenBuildTimeOperation::ArbitraryFilesystemRead,
            ForbiddenBuildTimeOperation::ArbitraryFilesystemWrite,
            ForbiddenBuildTimeOperation::ArbitraryNetworkAccess,
            ForbiddenBuildTimeOperation::WallClockAccess,
            ForbiddenBuildTimeOperation::AmbientEnvironmentAccess,
            ForbiddenBuildTimeOperation::UncontrolledProcessExecution,
        ],
    )
}
