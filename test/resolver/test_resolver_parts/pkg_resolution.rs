use super::{resolve_package_from_folder_with_config, try_resolve_package_from_folder_with_config, unique_temp_root};
use fol_resolver::{ReferenceKind, ResolverConfig, ResolverErrorKind, ScopeKind, SymbolKind};
use std::fs;

#[test]
fn test_resolver_resolves_pkg_imports_from_the_configured_package_store_root() {
    let temp_root = unique_temp_root("pkg_import_root");
    let store_root = temp_root.join("store");
    let app_root = temp_root.join("app");
    fs::create_dir_all(store_root.join("json"))
        .expect("Should create the package-store fixture directory");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        store_root.join("json/package.yaml"),
        "name: json\nversion: 1.0.0\n",
    )
    .expect("Should write the installed package metadata fixture");
    fs::create_dir_all(store_root.join("json/src"))
        .expect("Should create the installed package export root fixture");
    fs::write(store_root.join("json/build.fol"), "def root: loc = \"src\";\n")
        .expect("Should write the installed package build fixture");
    fs::write(store_root.join("json/src/lib.fol"), "var[exp] answer: int = 42;\n")
        .expect("Should write the installed package export fixture");
    fs::write(
        app_root.join("main.fol"),
        "use json: pkg = {json};\nfun[] main(): int = {\n    return answer;\n}\n",
    )
    .expect("Should write the pkg import fixture");

    let resolved = resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Temporary package-store fixture path should be valid UTF-8")
                    .to_string(),
            ),
        },
    );
    let import = resolved
        .imports_in_scope(resolved.program_scope)
        .into_iter()
        .find(|import| import.alias_name == "json")
        .expect("Resolver should keep the pkg import record");
    let target_scope = import
        .target_scope
        .expect("Configured pkg imports should resolve to a mounted root scope");
    let answer_symbol = resolved
        .symbols_in_scope(target_scope)
        .into_iter()
        .find(|symbol| symbol.name == "answer" && symbol.kind == SymbolKind::ValueBinding)
        .expect("Mounted pkg roots should expose exported root symbols");
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
        .expect("Routine scope should record the plain pkg-imported identifier reference");

    assert!(
        matches!(
            resolved.scope(target_scope).map(|scope| &scope.kind),
            Some(ScopeKind::ProgramRoot { package }) if package == "json"
        ),
        "Configured pkg imports should mount the installed package root as the imported root scope",
    );
    assert_eq!(answer_reference.resolved, Some(answer_symbol.id));

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_pkg_imports_hide_non_exported_internal_roots() {
    let temp_root = unique_temp_root("pkg_hidden_internal_root");
    let store_root = temp_root.join("store");
    let app_root = temp_root.join("app");
    fs::create_dir_all(store_root.join("json/src/public"))
        .expect("Should create the exported source root fixture");
    fs::create_dir_all(store_root.join("json/src/internal"))
        .expect("Should create the internal source root fixture");
    fs::create_dir_all(&app_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        store_root.join("json/package.yaml"),
        "name: json\nversion: 1.0.0\n",
    )
    .expect("Should write the installed package metadata fixture");
    fs::write(store_root.join("json/build.fol"), "def root: loc = \"src/public\";\n")
        .expect("Should write the installed package build fixture");
    fs::write(
        store_root.join("json/src/public/value.fol"),
        "var[exp] answer: int = 42;\n",
    )
    .expect("Should write the exported source fixture");
    fs::write(
        store_root.join("json/src/internal/secret.fol"),
        "var[exp] secret: int = 7;\n",
    )
    .expect("Should write the internal source fixture");
    fs::write(
        app_root.join("main.fol"),
        "use json: pkg = {json};\nfun[] main(): int = {\n    return secret;\n}\n",
    )
    .expect("Should write the internal pkg import fixture");

    let errors = try_resolve_package_from_folder_with_config(
        app_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig {
            std_root: None,
            package_store_root: Some(
                store_root
                    .to_str()
                    .expect("Temporary package-store fixture path should be valid UTF-8")
                    .to_string(),
            ),
        },
    )
    .expect_err("Resolver should hide non-exported internal pkg roots from consumers");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::UnresolvedName
                && error.to_string().contains("could not resolve name 'secret'")
        }),
        "Pkg imports should not expose exported symbols from internal roots that build.fol does not export",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}

#[test]
fn test_resolver_reports_missing_package_store_roots_for_pkg_imports() {
    let temp_root = unique_temp_root("pkg_missing_store_root");
    fs::create_dir_all(&temp_root)
        .expect("Should create the importing package root fixture directory");
    fs::write(
        temp_root.join("main.fol"),
        "use json: pkg = {json};\nfun[] main(): int = {\n    return 0;\n}\n",
    )
    .expect("Should write the pkg import fixture");

    let errors = try_resolve_package_from_folder_with_config(
        temp_root
            .to_str()
            .expect("Temporary resolver fixture path should be valid UTF-8"),
        ResolverConfig::default(),
    )
    .expect_err("Resolver should reject pkg imports without an explicit package store root");

    assert!(
        errors.iter().any(|error| {
            error.kind() == ResolverErrorKind::InvalidInput
                && error
                    .to_string()
                    .contains("requires an explicit package store root")
        }),
        "Resolver should report missing package-store roots explicitly for pkg imports",
    );

    fs::remove_dir_all(&temp_root)
        .expect("Temporary resolver fixture directory should be removable after the test");
}
