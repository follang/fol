use super::*;

#[derive(Clone, Copy, Debug)]
enum RootDeclFamily {
    Use,
    Def,
    Seg,
    Imp,
    Var,
    Lab,
    Fun,
    Pro,
    Log,
    Type,
    Alias,
    Standard,
}

fn source_unit_has_root_decl_family(source_unit: &ParsedSourceUnit, family: RootDeclFamily) -> bool {
    source_unit.items.iter().any(|item| match (&item.node, family) {
        (AstNode::UseDecl { .. }, RootDeclFamily::Use)
        | (AstNode::DefDecl { .. }, RootDeclFamily::Def)
        | (AstNode::SegDecl { .. }, RootDeclFamily::Seg)
        | (AstNode::ImpDecl { .. }, RootDeclFamily::Imp)
        | (AstNode::VarDecl { .. }, RootDeclFamily::Var)
        | (AstNode::LabDecl { .. }, RootDeclFamily::Lab)
        | (AstNode::FunDecl { .. }, RootDeclFamily::Fun)
        | (AstNode::ProDecl { .. }, RootDeclFamily::Pro)
        | (AstNode::LogDecl { .. }, RootDeclFamily::Log)
        | (AstNode::TypeDecl { .. }, RootDeclFamily::Type)
        | (AstNode::AliasDecl { .. }, RootDeclFamily::Alias)
        | (AstNode::StdDecl { .. }, RootDeclFamily::Standard) => true,
        _ => false,
    })
}

fn parse_decl_package_errors(path: &str) -> Vec<ParseError> {
    let mut file_stream =
        FileStream::from_file(path).expect("Should read parser declaration-package error fixture");
    let mut lexer = Elements::init(&mut file_stream);
    let mut parser = AstParser::new();
    let errors = parser
        .parse_decl_package(&mut lexer)
        .expect_err("Declaration-only package parsing should reject forbidden file-root forms");

    errors
        .iter()
        .map(|error| {
            error
                .as_ref()
                .as_any()
                .downcast_ref::<ParseError>()
                .cloned()
                .expect("Declaration-only file-root rejection should produce ParseError values")
        })
        .collect()
}

#[test]
fn test_decl_package_accepts_supported_file_root_declaration_families() {
    for (path, family) in [
        ("test/parser/simple_use_bare_mod_type.fol", RootDeclFamily::Use),
        ("test/parser/simple_def_module.fol", RootDeclFamily::Def),
        ("test/parser/simple_seg_module.fol", RootDeclFamily::Seg),
        ("test/parser/simple_imp_basic.fol", RootDeclFamily::Imp),
        ("test/parser/simple_var.fol", RootDeclFamily::Var),
        ("test/parser/simple_lab_decl.fol", RootDeclFamily::Lab),
        ("test/parser/simple_fun.fol", RootDeclFamily::Fun),
        ("test/parser/simple_pro.fol", RootDeclFamily::Pro),
        ("test/parser/simple_log.fol", RootDeclFamily::Log),
        ("test/parser/simple_typ_object_marker.fol", RootDeclFamily::Type),
        ("test/parser/simple_ali.fol", RootDeclFamily::Alias),
        ("test/parser/simple_std_protocol.fol", RootDeclFamily::Standard),
    ] {
        let parsed = parse_decl_package_from_file(path);

        assert_eq!(
            parsed.source_units.len(),
            1,
            "Single-file declaration-only parsing should yield one source unit for {path}"
        );
        assert!(
            source_unit_has_root_decl_family(&parsed.source_units[0], family),
            "Declaration-only file root should accept {:?} fixtures from {}",
            family,
            path
        );
    }
}

#[test]
fn test_decl_package_rejects_top_level_executable_calls_as_one_root_error() {
    let errors = parse_decl_package_errors("test/parser/simple_call_top_level.fol");

    assert_eq!(
        errors.len(),
        1,
        "A forbidden top-level call should be rejected as one file-root error"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("Executable calls are not allowed at file root"),
        "Expected executable-call file-root diagnostic, got: {}",
        errors[0]
    );
    assert_eq!(errors[0].line(), 1);
    assert_eq!(errors[0].column(), 1);
}

#[test]
fn test_decl_package_rejects_top_level_assignments_as_one_root_error() {
    let errors = parse_decl_package_errors("test/parser/simple_top_level_keyword_call_and_assignment.fol");

    assert_eq!(
        errors.len(),
        2,
        "A file with one forbidden call and one forbidden assignment should yield two file-root errors"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("Executable calls are not allowed at file root"),
        "Expected executable-call file-root diagnostic first, got: {}",
        errors[0]
    );
    assert_eq!(errors[0].line(), 1);
    assert_eq!(errors[0].column(), 1);
    assert!(
        errors[1]
            .to_string()
            .contains("Assignments are not allowed at file root"),
        "Expected assignment file-root diagnostic second, got: {}",
        errors[1]
    );
    assert_eq!(errors[1].line(), 2);
    assert_eq!(errors[1].column(), 1);
}

#[test]
fn test_decl_package_rejects_top_level_when_statement_as_one_root_error() {
    let errors = parse_decl_package_errors("test/parser/simple_when_top_level_statement.fol");

    assert_eq!(
        errors.len(),
        1,
        "A forbidden top-level when statement should be rejected as one file-root error"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("Control-flow statements are not allowed at file root"),
        "Expected control-flow file-root diagnostic, got: {}",
        errors[0]
    );
    assert_eq!(errors[0].line(), 1);
    assert_eq!(errors[0].column(), 1);
}

#[test]
fn test_decl_package_rejects_top_level_loop_statement_as_one_root_error() {
    let errors = parse_decl_package_errors("test/parser/simple_loop_top_level.fol");

    assert_eq!(
        errors.len(),
        1,
        "A forbidden top-level loop statement should be rejected as one file-root error"
    );
    assert!(
        errors[0]
            .to_string()
            .contains("Control-flow statements are not allowed at file root"),
        "Expected control-flow file-root diagnostic, got: {}",
        errors[0]
    );
    assert_eq!(errors[0].line(), 1);
    assert_eq!(errors[0].column(), 1);
}

#[test]
fn test_decl_package_rejects_literal_roots_line_by_line() {
    let errors = parse_decl_package_errors("test/parser/simple_literal_logic.fol");

    assert_eq!(
        errors.len(),
        3,
        "Each forbidden literal root should produce one file-root error"
    );
    for (index, error) in errors.iter().enumerate() {
        assert!(
            error
                .to_string()
                .contains("Literal expressions are not allowed at file root"),
            "Expected literal file-root diagnostic, got: {}",
            error
        );
        assert_eq!(error.line(), index + 1);
        assert_eq!(error.column(), 1);
    }
}
