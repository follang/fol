// FOL Types - Shared types, traits, and core abstractions

#[macro_use]
pub mod r#mod;
pub mod error;

// Basic types
pub use error::*;
pub use r#mod::*;

// Placeholder error trait for now
pub trait Glitch: std::error::Error + Send + Sync + 'static {
    fn clone_box(&self) -> Box<dyn Glitch>;
    fn as_any(&self) -> &dyn std::any::Any;
}

// Basic implementations will be added later
#[derive(Debug, Clone)]
pub struct BasicError {
    pub message: String,
}

impl std::fmt::Display for BasicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for BasicError {}
impl Glitch for BasicError {
    fn clone_box(&self) -> Box<dyn Glitch> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl Clone for Box<dyn Glitch> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
