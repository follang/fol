//! Lowering from typed `V1` FOL workspaces into a backend-oriented IR.

mod errors;

pub use errors::{LoweringError, LoweringErrorKind};

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
    use super::{Lowerer, LoweringError, LoweringErrorKind, LoweringResult};

    #[test]
    fn lowering_api_exposes_constructor_and_result_alias() {
        let _ = Lowerer::new();

        let result: LoweringResult<()> = Ok(());
        assert!(result.is_ok());
    }

    #[test]
    fn lowering_api_exposes_basic_error_surface() {
        let error = LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "lowering shell is not implemented yet",
        );
        assert_eq!(error.message(), "lowering shell is not implemented yet");
        assert_eq!(
            error.to_string(),
            "LoweringUnsupported: lowering shell is not implemented yet"
        );
    }
}
