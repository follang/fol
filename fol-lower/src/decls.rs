use crate::{
    LoweredPackage, LoweredTypeDecl, LoweredTypeDeclKind, LoweringError, LoweringErrorKind,
    LoweringResult,
};
use fol_parser::ast::AstNode;
use fol_resolver::{SourceUnitId, SymbolId, SymbolKind};

pub fn lower_routine_signatures(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let Some(name) = routine_name(&item.node) else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) {
                Some(symbol_id) => match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
                    Ok(signature_type) => {
                        lowered_package.routine_signatures.insert(symbol_id, signature_type);
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("top-level routine '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

pub fn lower_alias_declarations(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &mut LoweredPackage,
) -> LoweringResult<()> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let AstNode::AliasDecl { name, .. } = &item.node else {
                continue;
            };

            match find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Alias,
                name,
            ) {
                Some(symbol_id) => match lower_symbol_signature(typed_package, lowered_package, symbol_id) {
                    Ok(target_type) => {
                        lowered_package.type_decls.insert(
                            symbol_id,
                            LoweredTypeDecl {
                                symbol_id,
                                source_unit_id,
                                name: name.clone(),
                                runtime_type: target_type,
                                kind: LoweredTypeDeclKind::Alias { target_type },
                            },
                        );
                    }
                    Err(error) => errors.push(error),
                },
                None => errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("alias '{name}' does not retain a resolved symbol"),
                )),
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_symbol_signature(
    typed_package: &fol_typecheck::TypedPackage,
    lowered_package: &LoweredPackage,
    symbol_id: SymbolId,
) -> Result<crate::LoweredTypeId, LoweringError> {
    let checked_signature = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|typed_symbol| typed_symbol.declared_type)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("routine symbol {} does not retain a typed signature", symbol_id.0),
            )
        })?;

    lowered_package
        .checked_type_map
        .get(&checked_signature)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "routine symbol {} does not map to a lowering-owned signature type",
                    symbol_id.0
                ),
            )
        })
}

fn routine_name(node: &AstNode) -> Option<&str> {
    match node {
        AstNode::FunDecl { name, .. }
        | AstNode::ProDecl { name, .. }
        | AstNode::LogDecl { name, .. } => Some(name.as_str()),
        _ => None,
    }
}

fn find_local_symbol_id(
    typed_program: &fol_typecheck::TypedProgram,
    source_unit_id: SourceUnitId,
    kind: SymbolKind,
    name: &str,
) -> Option<SymbolId> {
    typed_program
        .resolved()
        .symbols
        .iter_with_ids()
        .find(|(_, symbol)| {
            symbol.source_unit == source_unit_id
                && symbol.kind == kind
                && symbol.name == name
                && symbol.mounted_from.is_none()
        })
        .map(|(symbol_id, _)| symbol_id)
}

#[cfg(test)]
mod tests {
    use super::{lower_alias_declarations, lower_routine_signatures};
    use crate::{types::LoweredType, LoweredBuiltinType, LoweredPackage, LoweredTypeDeclKind};
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_workspace;
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn declaration_lowering_maps_top_level_routine_signatures_to_lowered_types() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_routine_sig_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] greet(count: int): str = { return \"ok\" }",
        )
        .expect("should write lowering signature fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_routine_signatures(typed_package, &mut lowered_package)
            .expect("top-level routine signatures should lower cleanly");

        let lowered_signature = lowered_package
            .routine_signatures
            .values()
            .next()
            .expect("routine signature should be recorded");
        let signature = lowered_workspace
            .type_table()
            .get(*lowered_signature)
            .expect("lowered signature type should exist");

        match signature {
            LoweredType::Routine(signature) => {
                assert_eq!(
                    lowered_workspace.type_table().get(signature.params[0]),
                    Some(&LoweredType::Builtin(LoweredBuiltinType::Int))
                );
                assert_eq!(
                    signature
                        .return_type
                        .and_then(|type_id| lowered_workspace.type_table().get(type_id)),
                    Some(&LoweredType::Builtin(LoweredBuiltinType::Str))
                );
            }
            other => panic!("expected lowered routine type, got {other:?}"),
        }
    }

    #[test]
    fn declaration_lowering_records_aliases_as_erased_runtime_shapes() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_alias_decl_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "ali Counter: int\nfun[] main(value: Counter): Counter = { return value }",
        )
        .expect("should write lowering alias fixture");

        let mut stream = FileStream::from_file(fixture.to_str().expect("utf8 temp path"))
            .expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered_workspace = crate::LoweringSession::new(typed.clone())
            .lower_workspace()
            .expect("workspace lowering should succeed");
        let typed_package = typed.entry_package();
        let mut lowered_package =
            LoweredPackage::new(crate::LoweredPackageId(0), typed_package.identity.clone());
        lowered_package.checked_type_map = lowered_workspace.entry_package().checked_type_map.clone();

        lower_alias_declarations(typed_package, &mut lowered_package)
            .expect("aliases should lower as erased runtime declarations");

        let alias_decl = lowered_package
            .type_decls
            .values()
            .next()
            .expect("alias declaration should be recorded");
        assert_eq!(alias_decl.name, "Counter");
        assert_eq!(
            lowered_workspace.type_table().get(alias_decl.runtime_type),
            Some(&LoweredType::Builtin(LoweredBuiltinType::Int))
        );
        assert_eq!(
            alias_decl.kind,
            LoweredTypeDeclKind::Alias {
                target_type: alias_decl.runtime_type
            }
        );
    }
}
