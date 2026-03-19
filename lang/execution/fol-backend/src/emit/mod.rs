mod build;
mod skeleton;
mod tests;

pub use build::{
    build_generated_crate, emit_backend_artifact, prepare_generated_build_dir,
    summarize_emitted_artifact, write_generated_crate,
};
pub use skeleton::{
    emit_cargo_toml, emit_generated_crate_skeleton, emit_main_rs, emit_namespace_module_shells,
    emit_package_module_shells,
};
