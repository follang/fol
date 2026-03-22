mod build;
mod runtime;
mod skeleton;
mod tests;

pub use build::{
    backend_build_paths, build_generated_crate, build_generated_crate_with_cargo,
    build_generated_crate_with_cargo_for_profile, build_generated_crate_with_rustc,
    build_runtime_rlib_with_rustc, emit_backend_artifact, prepare_backend_build_paths,
    prepare_generated_build_dir, summarize_emitted_artifact, write_generated_crate,
};
pub use runtime::{
    backend_runtime_build_dir, backend_runtime_manifest_path, backend_runtime_source_entry,
    backend_runtime_source_root, prepare_backend_runtime_build_dir,
};
pub use skeleton::{
    emit_cargo_toml, emit_generated_crate_skeleton, emit_main_rs, emit_namespace_module_shells,
    emit_package_module_shells,
};
