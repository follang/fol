#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildExecutionRepresentation {
    RestrictedRuntimeIr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildRuntimeProgram {
    representation: BuildExecutionRepresentation,
}

impl BuildRuntimeProgram {
    pub fn new(representation: BuildExecutionRepresentation) -> Self {
        Self { representation }
    }

    pub fn representation(&self) -> BuildExecutionRepresentation {
        self.representation
    }
}

#[cfg(test)]
mod tests {
    use super::{BuildExecutionRepresentation, BuildRuntimeProgram};

    #[test]
    fn runtime_programs_record_the_chosen_execution_representation() {
        let program = BuildRuntimeProgram::new(BuildExecutionRepresentation::RestrictedRuntimeIr);

        assert_eq!(
            program.representation(),
            BuildExecutionRepresentation::RestrictedRuntimeIr
        );
    }
}
