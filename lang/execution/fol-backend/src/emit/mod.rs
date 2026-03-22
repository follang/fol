mod build;
mod skeleton;
mod tests;

pub use build::{
    backend_build_paths, build_generated_crate, build_generated_crate_with_cargo,
    build_generated_crate_with_cargo_for_profile, emit_backend_artifact,
    prepare_backend_build_paths, prepare_generated_build_dir, summarize_emitted_artifact,
    write_generated_crate,
};
pub use skeleton::{
    emit_cargo_toml, emit_generated_crate_skeleton, emit_main_rs, emit_namespace_module_shells,
    emit_package_module_shells,
};
