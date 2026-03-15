//! Small public prelude for generated code and runtime users.

pub use crate::{crate_name, CRATE_NAME};
pub use crate::abi::{check_recoverable, recoverable_succeeded, FolRecover};
pub use crate::strings::FolStr;
pub use crate::value::{impossible, FolBool, FolChar, FolFloat, FolInt, FolNever};
