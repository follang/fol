use super::{
    resolve_package_from_folder_with_config, try_resolve_package_from_folder_with_config,
    unique_temp_root,
};
use fol_resolver::{ReferenceKind, ResolverConfig, ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_std_package_roots_from_the_configured_std_root() {
    let temp_root = unique_temp_root("std_package_root");
    let std_root = temp_root.join("std");
    let app_root = temp_root.join("app");
    fs::create_dir_all(std_root.join("fmt"))
        .expect("Should create the standard library package-root fixture directory");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(std_root.join("fmt/values.fol"), "var[exp] answer: int = 42;\n")
        .expect("Should write the standard library exported value fixture");
    fs::write(
        app_root.join("main.fol"),
        "use fmt: std = {fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("Should write the std import fixture");

    let resolved = resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("Temporary std-root fixture path should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    );
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "fmt")
        .expect("Resolver should keep the std import record");
    let target_scope = import
        .target_scope
        .expect("Configured std imports should resolve to a mounted root scope");
    let answer_symbol = resolved
        .symbols_in_scope(target_scope)
        .into_iter()
        .find(|symbol| symbol.name == "answer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Mounted std roots should expose exported root symbols");
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");
    let answer_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| {
            reference.kind == ReferenceKind::Identifier && reference.name == "answer"
        })
        .expect("Routine scope should record the plain std-imported identifier reference");

    assert!(
        matches!(
            resolved.scope(target_scope).map(|scope| &scope.kind),
            Some(ScopeKind::ProgramRoot { package }) if package == "fmt"
        ),
        "Configured std imports should mount the exact standard-library directory as the imported root scope",
    );
    assert_eq!(answer_reference.resolved, Some(answer_symbol.id));

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_resolves_std_namespace_roots_from_the_configured_std_root() {
    let temp_root = unique_temp_root("std_namespace_root");
    let std_root = temp_root.join("std");
    let app_root = temp_root.join("app");
    fs::create_dir_all(std_root.join("core/fmt"))
        .expect("Should create the standard-library namespace fixture directory");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        std_root.join("core/fmt/values.fol"),
        "var[exp] answer: int = 7;\n",
    )
    .expect("Should write the standard-library namespace export fixture");
    fs::write(
        app_root.join("main.fol"),
        "use fmt: std = {core/fmt};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("Should write the std namespace import fixture");

    let resolved = resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("Temporary std-root fixture path should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    );
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "fmt")
        .expect("Resolver should keep the std namespace import record");
    let target_scope = import
        .target_scope
        .expect("Configured std namespace imports should resolve to a mounted root scope");
    let answer_symbol = resolved
        .symbols_in_scope(target_scope)
        .into_iter()
        .find(|symbol| symbol.name == "answer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Mounted std namespace roots should expose exported root symbols");
    let routine_scope_id = resolved
        .scopes
        .iter_with_ids()
        .find_map(|(scope_id, scope)| matches!(scope.kind, ScopeKind::Routine).then_some(scope_id))
        .expect("Resolver should create a routine scope");
    let answer_reference = resolved
        .references_in_scope(routine_scope_id)
        .into_iter()
        .find(|reference| {
            reference.kind == ReferenceKind::Identifier && reference.name == "answer"
        })
        .expect("Routine scope should record the plain std-namespace identifier reference");

    assert!(
        matches!(
            resolved.scope(target_scope).map(|scope| &scope.kind),
            Some(ScopeKind::ProgramRoot { package }) if package == "fmt"
        ),
        "Configured std imports should mount the exact standard-library directory as the imported root scope",
    );
    assert_eq!(answer_reference.resolved, Some(answer_symbol.id));

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_missing_std_targets_explicitly() {
    let temp_root = unique_temp_root("std_missing_target");
    let std_root = temp_root.join("std");
    let app_root = temp_root.join("app");
    fs::create_dir_all(&std_root)
        .expect("Should create the configured std-root fixture directory");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        app_root.join("main.fol"),
        "use fmt: std = {fmt};\nfun[] main(): int = {\n    return 0;\n}\n",
    )
    .expect("Should write the missing std-target fixture");

    let errors = try_resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("Temporary std-root fixture path should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    )
    .expect_err("Resolver should reject missing std directory targets");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::InvalidInput
                && error.to_string().contains("resolver std import target")
                && error.to_string().contains("does not exist")
        }),
        "Resolver should report missing std directory targets explicitly",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_rejects_std_file_targets_explicitly() {
    let temp_root = unique_temp_root("std_file_target");
    let std_root = temp_root.join("std");
    let app_root = temp_root.join("app");
    fs::create_dir_all(&std_root)
        .expect("Should create the configured std-root fixture directory");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        std_root.join("fmt.fol"),
        "var[exp] answer: int = 1;\n",
    )
    .expect("Should write the std file target fixture");
    fs::write(
        app_root.join("main.fol"),
        "use fmt: std = {fmt.fol};\nfun[] main(): int = {\n    return 0;\n}\n",
    )
    .expect("Should write the std file-target import fixture");

    let errors = try_resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: Some(
                std_root
                    .to_str()
                    .expect("Temporary std-root fixture path should be valid UTF-8")
                    .to_string(),
            ),
            package_store_root: None,
        },
    )
    .expect_err("Resolver should reject direct file targets for std imports");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::InvalidInput
                && error
                    .to_string()
                    .contains("must point to a directory, not a file")
        }),
        "Resolver should report direct file std imports explicitly",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
