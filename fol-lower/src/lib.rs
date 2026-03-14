//! Lowering from typed `V1` FOL workspaces into a backend-oriented IR.

mod errors;

pub use errors::LoweringError;

pub type LoweringResult<T> = Result<T, Vec<LoweringError>>;

#[derive(Debug, Default)]
pub struct Lowerer;

impl Lowerer {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
mod tests {
    use super::{Lowerer, LoweringError, LoweringResult};

    #[test]
    fn lowering_api_exposes_constructor_and_result_alias() {
        let _ = Lowerer::new();

        let result: LoweringResult<()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn lowering_api_exposes_basic_error_surface() {
        let error = LoweringError::new("lowering shell is not implemented yet");
        assert_eq!(error.message(), "lowering shell is not implemented yet");
        assert_eq!(
            error.to_string(),
            "LoweringError: lowering shell is not implemented yet"
        );
    }
}
