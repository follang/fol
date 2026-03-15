use crate::{
    decls,
    ids::{LoweredPackageId, LoweredTypeId},
    types::{LoweredBuiltinType, LoweredRoutineType, LoweredType, LoweredTypeTable},
    LoweredPackage, LoweredSourceMap, LoweredSourceMapEntry, LoweredSourceSymbol,
    LoweredSourceUnit, LoweredSymbolOwnership, LoweredWorkspace, LoweringError,
    LoweringErrorKind, LoweringResult,
};
use fol_resolver::PackageIdentity;
use fol_typecheck::{BuiltinType, CheckedType, CheckedTypeId};
use std::collections::BTreeMap;

#[derive(Debug)]
pub struct LoweringSession {
    typed: fol_typecheck::TypedWorkspace,
}

impl LoweringSession {
    pub fn new(typed: fol_typecheck::TypedWorkspace) -> Self {
        Self { typed }
    }

    pub fn typed_workspace(&self) -> &fol_typecheck::TypedWorkspace {
        &self.typed
    }

    pub fn lower_workspace(self) -> LoweringResult<LoweredWorkspace> {
        let entry_identity = self.typed.entry_identity().clone();
        let mut packages = BTreeMap::new();
        let mut type_table = LoweredTypeTable::new();
        let mut lowered_type_cache = BTreeMap::new();

        for (index, package) in self.typed.packages().enumerate() {
            let mut lowered = LoweredPackage::new(LoweredPackageId(index), package.identity.clone());
            lowered.source_units = package
                .program
                .source_units()
                .iter()
                .map(|unit| LoweredSourceUnit {
                    source_unit_id: unit.source_unit_id,
                    path: unit.path.clone(),
                    package: unit.package.clone(),
                    namespace: unit.namespace.clone(),
                })
                .collect();
            lowered.symbol_ownership = package
                .program
                .resolved()
                .symbols
                .iter_with_ids()
                .map(|(symbol_id, symbol)| {
                    let mounted_from = symbol.mounted_from.clone();
                    (
                        symbol_id,
                        LoweredSymbolOwnership {
                            symbol_id,
                            source_unit_id: symbol.source_unit,
                            owning_package: mounted_from
                                .as_ref()
                                .map(|provenance| provenance.package_identity.clone())
                                .unwrap_or_else(|| package.identity.clone()),
                            mounted_from,
                        },
                    )
                })
                .collect();
            lowered.checked_type_map = (0..package.program.type_table().len())
                .map(|raw_type_id| {
                    let checked_type_id = CheckedTypeId(raw_type_id);
                    translate_checked_type(
                        &mut type_table,
                        &mut lowered_type_cache,
                        &package.identity,
                        &package.program,
                        checked_type_id,
                    )
                    .map(|lowered_type_id| (checked_type_id, lowered_type_id))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            decls::lower_routine_signatures(package, &mut lowered)?;
            decls::lower_alias_declarations(package, &mut lowered)?;
            packages.insert(package.identity.clone(), lowered);
        }

        let source_map = build_workspace_source_map(&self.typed, &packages);

        Ok(LoweredWorkspace::new(entry_identity, packages, type_table, source_map))
    }
}

fn build_workspace_source_map(
    typed: &fol_typecheck::TypedWorkspace,
    packages: &BTreeMap<PackageIdentity, LoweredPackage>,
) -> LoweredSourceMap {
    let mut source_map = LoweredSourceMap::new();

    for typed_package in typed.packages() {
        let Some(lowered_package) = packages.get(&typed_package.identity) else {
            continue;
        };

        for source_unit in typed_package.program.source_units() {
            for syntax_id in &source_unit.top_level_nodes {
                if let Some(origin) = typed_package.program.resolved().syntax_index().origin(*syntax_id) {
                    source_map.push(LoweredSourceMapEntry {
                        symbol: LoweredSourceSymbol::Package(lowered_package.id),
                        origin: origin.clone(),
                    });
                }
            }
        }
    }

    source_map
}

fn translate_checked_type(
    lowered_types: &mut LoweredTypeTable,
    cache: &mut BTreeMap<(PackageIdentity, CheckedTypeId), LoweredTypeId>,
    package_identity: &PackageIdentity,
    program: &fol_typecheck::TypedProgram,
    checked_type_id: CheckedTypeId,
) -> Result<LoweredTypeId, Vec<LoweringError>> {
    if let Some(existing) = cache.get(&(package_identity.clone(), checked_type_id)) {
        return Ok(*existing);
    }

    let checked_type = program
        .type_table()
        .get(checked_type_id)
        .cloned()
        .ok_or_else(|| {
            vec![LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("typed program lost checked type {}", checked_type_id.0),
            )]
        })?;

    let lowered_type_id = match checked_type {
        CheckedType::Builtin(builtin) => lowered_types.intern_builtin(lower_builtin(builtin)),
        CheckedType::Declared { symbol, .. } => {
            let runtime_type = program
                .typed_symbol(symbol)
                .and_then(|typed_symbol| typed_symbol.declared_type)
                .ok_or_else(|| {
                    vec![LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "typed declared symbol {} does not retain a lowered runtime shape yet",
                            symbol.0
                        ),
                    )]
                })?;
            translate_checked_type(lowered_types, cache, package_identity, program, runtime_type)?
        }
        CheckedType::Array { element_type, size } => {
            let element_type = translate_checked_type(
                lowered_types,
                cache,
                package_identity,
                program,
                element_type,
            )?;
            lowered_types.intern(LoweredType::Array { element_type, size })
        }
        CheckedType::Vector { element_type } => {
            let element_type = translate_checked_type(
                lowered_types,
                cache,
                package_identity,
                program,
                element_type,
            )?;
            lowered_types.intern(LoweredType::Vector { element_type })
        }
        CheckedType::Sequence { element_type } => {
            let element_type = translate_checked_type(
                lowered_types,
                cache,
                package_identity,
                program,
                element_type,
            )?;
            lowered_types.intern(LoweredType::Sequence { element_type })
        }
        CheckedType::Set { member_types } => {
            let member_types = member_types
                .into_iter()
                .map(|member_type| {
                    translate_checked_type(
                        lowered_types,
                        cache,
                        package_identity,
                        program,
                        member_type,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            lowered_types.intern(LoweredType::Set { member_types })
        }
        CheckedType::Map {
            key_type,
            value_type,
        } => {
            let key_type = translate_checked_type(
                lowered_types,
                cache,
                package_identity,
                program,
                key_type,
            )?;
            let value_type = translate_checked_type(
                lowered_types,
                cache,
                package_identity,
                program,
                value_type,
            )?;
            lowered_types.intern(LoweredType::Map {
                key_type,
                value_type,
            })
        }
        CheckedType::Optional { inner } => {
            let inner = translate_checked_type(lowered_types, cache, package_identity, program, inner)?;
            lowered_types.intern(LoweredType::Optional { inner })
        }
        CheckedType::Error { inner } => {
            let inner = inner
                .map(|inner| {
                    translate_checked_type(lowered_types, cache, package_identity, program, inner)
                })
                .transpose()?;
            lowered_types.intern(LoweredType::Error { inner })
        }
        CheckedType::Record { fields } => {
            let fields = fields
                .into_iter()
                .map(|(field_name, field_type)| {
                    translate_checked_type(lowered_types, cache, package_identity, program, field_type)
                        .map(|lowered_field_type| (field_name, lowered_field_type))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            lowered_types.intern(LoweredType::Record { fields })
        }
        CheckedType::Entry { variants } => {
            let variants = variants
                .into_iter()
                .map(|(variant_name, variant_type)| {
                    variant_type
                        .map(|variant_type| {
                            translate_checked_type(
                                lowered_types,
                                cache,
                                package_identity,
                                program,
                                variant_type,
                            )
                        })
                        .transpose()
                        .map(|lowered_variant_type| (variant_name, lowered_variant_type))
                })
                .collect::<Result<BTreeMap<_, _>, _>>()?;
            lowered_types.intern(LoweredType::Entry { variants })
        }
        CheckedType::Routine(signature) => {
            let params = signature
                .params
                .into_iter()
                .map(|param_type| {
                    translate_checked_type(
                        lowered_types,
                        cache,
                        package_identity,
                        program,
                        param_type,
                    )
                })
                .collect::<Result<Vec<_>, _>>()?;
            let return_type = signature
                .return_type
                .map(|return_type| {
                    translate_checked_type(
                        lowered_types,
                        cache,
                        package_identity,
                        program,
                        return_type,
                    )
                })
                .transpose()?;
            let error_type = signature
                .error_type
                .map(|error_type| {
                    translate_checked_type(
                        lowered_types,
                        cache,
                        package_identity,
                        program,
                        error_type,
                    )
                })
                .transpose()?;
            lowered_types.intern(LoweredType::Routine(LoweredRoutineType {
                params,
                return_type,
                error_type,
            }))
        }
    };

    cache.insert((package_identity.clone(), checked_type_id), lowered_type_id);
    Ok(lowered_type_id)
}

fn lower_builtin(builtin: BuiltinType) -> LoweredBuiltinType {
    match builtin {
        BuiltinType::Int => LoweredBuiltinType::Int,
        BuiltinType::Float => LoweredBuiltinType::Float,
        BuiltinType::Bool => LoweredBuiltinType::Bool,
        BuiltinType::Char => LoweredBuiltinType::Char,
        BuiltinType::Str => LoweredBuiltinType::Str,
        BuiltinType::Never => LoweredBuiltinType::Never,
    }
}

#[cfg(test)]
mod tests {
    use super::LoweringSession;
    use crate::types::{LoweredBuiltinType, LoweredType};
    use fol_parser::ast::AstParser;
    use fol_resolver::resolve_workspace;
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;

    #[test]
    fn lowering_session_keeps_typed_workspace_identity_and_size() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");

        let session = LoweringSession::new(typed);

        assert_eq!(session.typed_workspace().entry_identity().display_name, "parser");
        assert_eq!(session.typed_workspace().package_count(), 1);
    }

    #[test]
    fn lowering_session_translates_single_package_source_units_and_symbols() {
        let fixture_path = concat!(env!("CARGO_MANIFEST_DIR"), "/../test/parser/simple_var.fol");
        let mut stream = FileStream::from_file(fixture_path).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");

        let lowered = LoweringSession::new(typed)
            .lower_workspace()
            .expect("Lowering shell should now translate workspace shells");

        assert_eq!(lowered.package_count(), 1);
        let entry = lowered.entry_package();
        assert_eq!(entry.source_units.len(), 1);
        assert!(!entry.symbol_ownership.is_empty());
        assert!(!entry.checked_type_map.is_empty());
        assert!(!lowered.source_map().is_empty());
        assert_eq!(
            lowered
                .type_table()
                .get(*entry.checked_type_map.get(&fol_typecheck::CheckedTypeId(0)).expect("int builtin should translate"))
                .expect("lowered builtin type should exist"),
            &LoweredType::Builtin(LoweredBuiltinType::Int)
        );
    }

    #[test]
    fn lowering_session_translates_loaded_package_identity_boundaries() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_workspace_{stamp}"));
        let app_dir = root.join("app");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&app_dir).expect("should create app dir");
        fs::create_dir_all(&shared_dir).expect("should create shared dir");
        fs::write(
            app_dir.join("main.fol"),
            "use shared: loc = {\"../shared\"}\nfun[] main(): int = { return answer }",
        )
        .expect("should write app entry");
        fs::write(
            shared_dir.join("lib.fol"),
            "var[exp] answer: int = 7",
        )
        .expect("should write shared library");

        let mut stream = FileStream::from_folder(app_dir.to_str().expect("utf8 temp path"))
            .expect("should open folder fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering folder fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering folder fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering folder fixture should typecheck");

        let lowered = LoweringSession::new(typed)
            .lower_workspace()
            .expect("Lowering shell should translate loaded package shells");

        assert_eq!(lowered.package_count(), 2);
        assert!(lowered
            .packages()
            .any(|package| package.identity.display_name == "shared"));
        let lowered_files = lowered
            .source_map()
            .entries()
            .iter()
            .filter_map(|entry| entry.origin.file.clone())
            .collect::<Vec<_>>();
        assert!(lowered_files.iter().any(|path| path.ends_with("app/main.fol")));
        assert!(lowered_files.iter().any(|path| path.ends_with("shared/lib.fol")));
        let app_package = lowered.entry_package();
        let imported_symbol = app_package
            .symbol_ownership
            .values()
            .find(|ownership| ownership.mounted_from.is_some())
            .expect("entry package should retain at least one mounted imported symbol");
        assert_eq!(imported_symbol.owning_package.display_name, "shared");
        assert_eq!(
            imported_symbol
                .mounted_from
                .as_ref()
                .expect("mounted symbol should keep foreign provenance")
                .package_identity
                .display_name,
            "shared"
        );
    }
}
