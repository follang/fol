use super::{resolve_package_from_folder, unique_temp_root};
use fol_resolver::ScopeKind;
use std::fs;

#[test]
fn test_resolver_preserves_deterministic_source_unit_order_for_folder_packages() {
    let temp_root = unique_temp_root("source_unit_order");
    fs::create_dir_all(temp_root.join("alpha_10")).expect("Should create alpha namespace dir");
    fs::create_dir_all(temp_root.join("beta_20")).expect("Should create beta namespace dir");
    fs::write(temp_root.join("00_root.fol"), "var rootValue: int = 1;")
        .expect("Should write root fixture");
    fs::write(
        temp_root.join("alpha_10/entry.fol"),
        "var alphaValue: int = 2;",
    )
    .expect("Should write alpha fixture");
    fs::write(
        temp_root.join("beta_20/entry.fol"),
        "var betaValue: int = 3;",
    )
    .expect("Should write beta fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Resolver folder fixture path should be utf-8"),
    );
    let package = temp_root
        .file_name()
        .expect("Temp resolver root should have a folder name")
        .to_string_lossy()
        .to_string();

    let ordered_units = resolved
        .source_units
        .iter_with_ids()
        .map(|(id, unit)| {
            (
                id.0,
                unit.namespace.clone(),
                std::path::Path::new(&unit.path)
                    .file_name()
                    .expect("Source unit path should have a file name")
                    .to_string_lossy()
                    .to_string(),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        ordered_units,
        vec![
            (0, package.clone(), "00_root.fol".to_string()),
            (1, format!("{}::alpha_10", package), "entry.fol".to_string()),
            (2, format!("{}::beta_20", package), "entry.fol".to_string()),
        ],
        "Resolver source-unit lowering should preserve the parser's deterministic folder traversal order"
    );

    fs::remove_dir_all(&temp_root).ok();
}

#[test]
fn test_resolver_builds_namespace_and_source_unit_scope_chain() {
    let temp_root = unique_temp_root("scope_chain");
    fs::create_dir_all(temp_root.join("net/http")).expect("Should create nested namespace dirs");
    fs::write(temp_root.join("main.fol"), "var rootValue: int = 1;")
        .expect("Should write root fixture");
    fs::write(
        temp_root.join("net/http/route.fol"),
        "var routeValue: int = 2;",
    )
    .expect("Should write nested fixture");

    let resolved = resolve_package_from_folder(
        temp_root
            .to_str()
            .expect("Resolver folder fixture path should be utf-8"),
    );
    let package = temp_root
        .file_name()
        .expect("Temp resolver root should have a folder name")
        .to_string_lossy()
        .to_string();

    let program_scope = resolved
        .scope(resolved.program_scope)
        .expect("Resolved program should expose its root scope");
    assert!(matches!(
        program_scope.kind,
        ScopeKind::ProgramRoot { package: ref scope_package } if scope_package == &package
    ));
    assert_eq!(program_scope.parent, None);

    let net_scope = resolved
        .namespace_scope(&format!("{}::net", package))
        .expect("Nested namespace should have its own scope");
    let http_scope = resolved
        .namespace_scope(&format!("{}::net::http", package))
        .expect("Deep namespace should have its own scope");
    assert_eq!(
        resolved
            .scope(net_scope)
            .expect("Net namespace scope should exist")
            .parent,
        Some(resolved.program_scope)
    );
    assert_eq!(
        resolved
            .scope(http_scope)
            .expect("HTTP namespace scope should exist")
            .parent,
        Some(net_scope)
    );

    let nested_unit = resolved
        .source_units
        .iter()
        .find(|unit| unit.path.ends_with("net/http/route.fol"))
        .expect("Nested source unit should exist");
    let nested_scope = resolved
        .scope(nested_unit.scope_id)
        .expect("Nested source unit scope should exist");

    assert!(matches!(
        nested_scope.kind,
        ScopeKind::SourceUnitRoot { .. }
    ));
    assert_eq!(nested_scope.parent, Some(http_scope));
    assert_eq!(nested_scope.source_unit, Some(nested_unit.id));

    fs::remove_dir_all(&temp_root).ok();
}
