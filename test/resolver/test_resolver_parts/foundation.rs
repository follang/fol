use super::resolve_package_from_file;

#[test]
fn test_resolver_smoke_lowers_source_units() {
    let resolved = resolve_package_from_file("test/parser/simple_var.fol");

    assert_eq!(resolved.package_name(), "parser");
    assert_eq!(resolved.source_units.len(), 1);

    let source_unit = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolved program should expose a source unit");

    assert!(source_unit.path.ends_with("simple_var.fol"));
    assert_eq!(source_unit.top_level_nodes.len(), 1);
}

#[test]
fn test_resolver_keeps_parser_syntax_origins_available() {
    let resolved = resolve_package_from_file("test/parser/simple_var.fol");
    let source_unit = resolved
        .source_units
        .iter()
        .next()
        .expect("Resolved program should expose a source unit");
    let first_node = *source_unit
        .top_level_nodes
        .first()
        .expect("Resolved source unit should keep top-level syntax ids");
    let origin = resolved
        .syntax_index()
        .origin(first_node)
        .expect("Resolver-visible syntax ids should resolve back to syntax origins");

    assert!(origin
        .file
        .as_deref()
        .expect("Syntax origin should retain file path")
        .ends_with("simple_var.fol"));
    assert_eq!(origin.line, 1);
    assert_eq!(origin.column, 1);
    assert!(origin.length >= 3);
}
