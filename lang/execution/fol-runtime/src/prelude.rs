//! Small public prelude for generated code and runtime users.

pub use crate::abi::{check_recoverable, recoverable_succeeded, FolRecover};
pub use crate::aggregate::{
    render_entry, render_entry_debug, render_record, render_record_debug, FolEntry, FolNamedValue,
    FolRecord,
};
pub use crate::alloc::{FolSeq, FolStr, FolVec};
pub use crate::builtins::{len, pow, pow_float, FolLength};
pub use crate::containers::{
    index_array, index_seq, index_vec, lookup_map, render_array, render_map, render_seq,
    render_set, render_vec, slice_seq, slice_vec, FolArray, FolMap, FolSet,
};
pub use crate::std::{
    echo, failure_outcome_from_error, outcome_from_recoverable, printable_outcome_message,
    render_echo, FolEchoFormat, FolProcessOutcome, FOL_EXIT_FAILURE, FOL_EXIT_SUCCESS,
};
pub use crate::shell::{
    unwrap_error_shell, unwrap_error_shell_ref, unwrap_optional_shell, unwrap_optional_shell_ref,
    FolError, FolOption,
};
pub use crate::value::{impossible, FolBool, FolChar, FolFloat, FolInt, FolNever};
pub use crate::{crate_name, CRATE_NAME};
