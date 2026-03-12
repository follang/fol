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
