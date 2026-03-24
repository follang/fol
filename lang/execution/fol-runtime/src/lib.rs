//! Runtime support foundations for executable FOL `V1` programs.
//!
//! `fol-runtime` is the support crate that future generated programs will link
//! against. It is not a front-end phase and it is not the backend itself.
//!
//! The intended compiler split is:
//!
//! - `fol-runtime` owns runtime data/layout/helper semantics
//! - `fol-backend` will later own code generation
//!
//! Current `V1` runtime scope:
//!
//! - builtin scalar support
//! - alloc-tier strings
//! - alloc-tier heap containers and container helpers
//! - optional/error shells
//! - recoverable routine results
//! - backend-facing runtime hooks such as `.echo(...)`
//!
//! The runtime model is converging on explicit internal tiers:
//!
//! - [`core`]
//! - [`alloc`]
//! - [`std`]
//!
//! The heap-backed runtime families now belong to [`alloc`], while [`containers`]
//! remains the helper layer for indexing, slicing, and rendering.
//!
//! Explicitly out of scope for this milestone:
//!
//! - ownership / borrowing / pointers
//! - standards / generics
//! - concurrency runtime
//! - C ABI
//!
//! # Backend Mapping: Builtins
//!
//! The first backend should map lowered builtins using this rule:
//!
//! - prefer native Rust operators or expressions for pure scalar operations
//! - use `fol-runtime` helpers for runtime-sensitive or policy-sensitive behavior
//!
//! Current `V1` expectation:
//!
//! - `.eq`, `.nq`, `.lt`, `.gt`, `.ge`, `.le`
//!   - lower to native Rust comparisons on already-lowered scalar values
//! - `.not`
//!   - lower to native Rust boolean negation
//! - `.len`
//!   - lower through the active model module's `len(...)`
//! - `.echo`
//!   - lower through [`std::echo`]
//! - `check`
//!   - lower through the active model module's `check_recoverable(...)`
//! - recoverable top-level result handling
//!   - lower through [`std::outcome_from_recoverable`]
//!
//! The backend should not reimplement `.len` or `.echo` inline. Those are part
//! of the runtime contract so later backends can share the same behavior.
//!
//! # Backend Mapping: Lowered Instructions
//!
//! The current lowered `V1` IR mixes instructions that can become plain Rust
//! syntax with instructions that require stable `fol-runtime` support.
//!
//! Native-emission friendly instructions:
//!
//! - `Const`
//! - `LoadLocal`
//! - `StoreLocal`
//! - `Call`
//! - `IntrinsicCall` for scalar comparisons and boolean negation
//! - `FieldAccess` for backend-authored record layouts
//! - `Cast` once the backend implements the admitted `V1` cast policy
//! - control terminators such as `Jump`, `Branch`, and `Return`
//!
//! Runtime-backed instructions or lowered surfaces:
//!
//! - `LengthOf`
//!   - must call the active model module's `len(...)`
//! - `RuntimeHook`
//!   - currently `.echo(...)`, which must call [`std::echo`]
//! - `CheckRecoverable`
//!   - must inspect [`abi::FolRecover`] through the active model module
//! - `UnwrapRecoverable`
//!   - must unwrap the success lane of [`abi::FolRecover`]
//! - `ExtractRecoverableError`
//!   - must extract the error lane of [`abi::FolRecover`]
//! - `ConstructLinear`
//!   - sequence and vector lowering must map to [`alloc::FolSeq`] and
//!     [`alloc::FolVec`]
//! - `ConstructSet`
//!   - must map to [`alloc::FolSet`] to preserve deterministic ordering
//! - `ConstructMap`
//!   - must map to [`alloc::FolMap`] to preserve deterministic ordering
//! - `ConstructOptional`
//!   - must map to [`shell::FolOption`]
//! - `ConstructError`
//!   - must map to [`shell::FolError`]
//! - `IndexAccess`
//!   - for runtime containers, must use the runtime indexing contract
//! - `UnwrapShell`
//!   - must follow the runtime shell boundary rather than routine-recoverable
//!     semantics
//! - `Report`
//!   - must produce the error lane of [`abi::FolRecover`]
//! - `Panic`
//!   - must route through the backend's panic strategy while preserving the
//!     runtime printable-message contract
//!
//! Backend-authored records and entries may compile into plain Rust structs and
//! enums, but their public formatting behavior should still follow
//! [`aggregate::FolRecord`], [`aggregate::render_record`], and
//! [`aggregate::FolEntry`] so generated `.echo(...)` output stays stable.
//!
//! # Backend Mapping: Generated Crate Names And Imports
//!
//! The first backend should generate one temporary Rust crate per lowered FOL
//! workspace.
//!
//! Import expectations for that generated crate:
//!
//! - declare a dependency on the package named `fol-runtime`
//! - import runtime items through `fol_runtime`, matching Rust's crate-name
//!   hyphen-to-underscore rule
//! - prefer one stable model alias per emitted module, such as
//!   `use fol_runtime::core as rt;`
//!   - or `alloc` / `std` depending on the artifact's `fol_model`
//! - use fully qualified imports for less-common runtime modules when needed,
//!   for example:
//!   - `fol_runtime::alloc::FolSeq`
//!   - `fol_runtime::shell::FolOption`
//!   - `fol_runtime::abi::FolRecover`
//!
//! Generated Rust should not guess alternate runtime package names and should
//! not inline shadow copies of runtime types into emitted modules.
//!
//! Namespace/layout expectations:
//!
//! - group emitted Rust by lowered package and namespace, not by original `.fol`
//!   file count
//! - backend-generated local helper names may be mangled, but runtime imports
//!   should stay readable and stable
//! - `fol-runtime` remains the single support dependency for current `V1`
//!   semantics; generated crates should not split the runtime contract across
//!   multiple ad hoc support crates
//!
//! # Backend Integration Guide
//!
//! A first Rust backend should integrate with `fol-runtime` in this order:
//!
//! 1. Lower a full workspace through `fol-lower` and treat that lowered
//!    workspace as the only backend input.
//! 2. Create one generated Rust crate for that lowered workspace.
//! 3. Add `fol-runtime` as a dependency and import the artifact's model module
//!    as `rt` in each emitted module.
//! 4. Emit backend-authored Rust structs and enums for lowered records and
//!    entries, then implement [`aggregate::FolRecord`] or [`aggregate::FolEntry`]
//!    where runtime formatting needs to stay stable.
//! 5. Map lowered container, shell, and recoverable shapes onto the public
//!    runtime types:
//!    - [`alloc::FolVec`]
//!    - [`alloc::FolSeq`]
//!    - [`alloc::FolSet`]
//!    - [`alloc::FolMap`]
//!    - [`shell::FolOption`]
//!    - [`shell::FolError`]
//!    - [`abi::FolRecover`]
//! 6. Lower builtin/runtime-sensitive operations through the model helpers
//!    instead of inlining policy:
//!    - `rt::len(...)`
//!    - `rt::echo(...)`
//!    - `rt::check_recoverable(...)`
//!    - `rt::outcome_from_recoverable(...)`
//! 7. Keep pure scalar comparison and boolean negation native in the emitted
//!    Rust where possible.
//! 8. Lower top-level entry routines so recoverable outcomes become
//!    [`std::FolProcessOutcome`] values with the documented exit-code policy.
//! 9. Only after emitted Rust typechecks against `fol-runtime` should the
//!    backend invoke `cargo build` or `rustc`.
//!
//! Current `V1` backends should treat `fol-runtime` as stable support code, not
//! as an optimizer target. If a future backend wants to inline or replace a
//! runtime helper, it should first preserve the same public behavior and only
//! then optimize behind that contract.

pub mod abi;
pub mod alloc;
pub mod aggregate;
pub mod builtins;
pub mod containers;
pub mod core;
pub mod error;
pub mod shell;
pub mod std;
pub mod value;

pub const CRATE_NAME: &str = "fol-runtime";

pub fn crate_name() -> &'static str {
    CRATE_NAME
}

pub use error::{RuntimeError, RuntimeErrorKind};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crate_name_matches_expected_runtime_identity() {
        assert_eq!(crate_name(), "fol-runtime");
    }

    #[test]
    fn public_runtime_module_shell_is_importable() {
        assert_eq!(alloc::module_name(), "alloc");
        assert_eq!(abi::module_name(), "abi");
        assert_eq!(aggregate::module_name(), "aggregate");
        assert_eq!(builtins::module_name(), "builtins");
        assert_eq!(containers::module_name(), "containers");
        assert_eq!(core::module_name(), "core");
        assert_eq!(error::module_name(), "error");
        assert_eq!(shell::module_name(), "shell");
        assert_eq!(std::module_name(), "std");
        assert_eq!(value::module_name(), "value");
    }

    #[test]
    fn runtime_model_modules_expose_expected_capabilities() {
        assert_eq!(core::tier_name(), "core");
        assert!(!core::HAS_HEAP);
        assert!(!core::HAS_OS);

        assert_eq!(alloc::tier_name(), "alloc");
        assert!(alloc::HAS_HEAP);
        assert!(!alloc::HAS_OS);

        assert_eq!(std::tier_name(), "std");
        assert!(std::HAS_HEAP);
        assert!(std::HAS_OS);
    }

    #[test]
    fn runtime_errors_can_be_constructed_with_stable_kinds() {
        let error = RuntimeError::new(
            RuntimeErrorKind::InvariantViolation,
            "runtime invariant failed",
        );

        assert_eq!(error.kind(), RuntimeErrorKind::InvariantViolation);
        assert_eq!(error.message(), "runtime invariant failed");
    }

    #[test]
    fn public_recoverable_abi_freezes_success_path_through_model_modules() {
        let value = alloc::FolRecover::<alloc::FolInt, alloc::FolStr>::ok(7);

        assert!(!alloc::check_recoverable(&value));
        assert!(alloc::recoverable_succeeded(&value));
        assert_eq!(value.value_ref(), Some(&7));
        assert_eq!(Result::<alloc::FolInt, alloc::FolStr>::from(value), Ok(7));
    }

    #[test]
    fn public_recoverable_abi_freezes_failure_path_through_model_modules() {
        let value = alloc::FolRecover::<alloc::FolInt, alloc::FolStr>::err(
            alloc::FolStr::from("bad-input"),
        );

        assert!(alloc::check_recoverable(&value));
        assert!(!alloc::recoverable_succeeded(&value));
        assert_eq!(
            value.error_ref().map(|error| error.as_str()),
            Some("bad-input")
        );
        assert_eq!(Result::<alloc::FolInt, alloc::FolStr>::from(value), Err(alloc::FolStr::from("bad-input")));
    }

    #[test]
    fn public_shell_values_stay_distinct_from_recoverable_results() {
        let optional = alloc::FolOption::some(7);
        let error_shell = alloc::FolError::new(alloc::FolStr::from("broken"));
        let recoverable =
            alloc::FolRecover::<alloc::FolInt, alloc::FolStr>::err(alloc::FolStr::from("broken"));

        assert_eq!(
            ::std::any::type_name::<alloc::FolOption<alloc::FolInt>>(),
            "fol_runtime::shell::FolOption<i64>"
        );
        assert_eq!(
            ::std::any::type_name::<alloc::FolError<alloc::FolStr>>(),
            "fol_runtime::shell::FolError<fol_runtime::alloc::FolStr>"
        );
        assert_eq!(
            ::std::any::type_name::<alloc::FolRecover<alloc::FolInt, alloc::FolStr>>(),
            "fol_runtime::abi::FolRecover<i64, fol_runtime::alloc::FolStr>"
        );

        assert_eq!(alloc::unwrap_optional_shell(optional), Ok(7));
        assert_eq!(alloc::unwrap_error_shell(error_shell), alloc::FolStr::from("broken"));
        assert!(alloc::check_recoverable(&recoverable));
    }
}
