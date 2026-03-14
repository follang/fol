use crate::{
    decls, exprs, CheckedType, CheckedTypeId, TypecheckError, TypecheckErrorKind,
    TypecheckResult, TypedPackage, TypedProgram, TypedWorkspace,
};
use fol_resolver::{MountedSymbolProvenance, PackageIdentity, SymbolId};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Default)]
pub struct TypecheckSession;

impl TypecheckSession {
    pub fn new() -> Self {
        Self
    }

    pub fn check_resolved_program(
        &mut self,
        resolved: fol_resolver::ResolvedProgram,
    ) -> TypecheckResult<TypedProgram> {
        let mut typed = TypedProgram::from_resolved(resolved);
        decls::lower_declaration_signatures(&mut typed)?;
        exprs::type_program(&mut typed)?;
        Ok(typed)
    }

    pub fn check_resolved_workspace(
        &mut self,
        resolved: fol_resolver::ResolvedWorkspace,
    ) -> TypecheckResult<TypedWorkspace> {
        let mut typed_packages = BTreeMap::new();
        let mut in_progress = BTreeSet::new();
        let identities = resolved
            .packages()
            .map(|package| package.identity.clone())
            .collect::<Vec<_>>();
        let mut errors = Vec::new();

        for identity in identities {
            if let Err(mut package_errors) = self.check_workspace_package(
                &resolved,
                &identity,
                &mut typed_packages,
                &mut in_progress,
            ) {
                errors.append(&mut package_errors);
            }
        }

        if errors.is_empty() {
            Ok(TypedWorkspace::new(
                resolved.entry_identity().clone(),
                typed_packages,
            ))
        } else {
            Err(errors)
        }
    }

    fn check_workspace_package(
        &mut self,
        workspace: &fol_resolver::ResolvedWorkspace,
        identity: &fol_resolver::PackageIdentity,
        typed_packages: &mut BTreeMap<fol_resolver::PackageIdentity, TypedPackage>,
        in_progress: &mut BTreeSet<fol_resolver::PackageIdentity>,
    ) -> TypecheckResult<()> {
        if typed_packages.contains_key(identity) {
            return Ok(());
        }

        if !in_progress.insert(identity.clone()) {
            return Err(vec![TypecheckError::new(
                TypecheckErrorKind::Internal,
                format!(
                    "typecheck workspace entered a package cycle at '{}'",
                    identity.canonical_root
                ),
            )]);
        }

        let package = workspace.package(identity).ok_or_else(|| {
            vec![TypecheckError::new(
                TypecheckErrorKind::Internal,
                format!(
                    "resolved workspace lost package '{}'",
                    identity.canonical_root
                ),
            )]
        })?;

        let dependency_identities = package
            .program
            .symbols
            .iter()
            .filter_map(|symbol| symbol.mounted_from.as_ref())
            .map(|provenance| provenance.package_identity.clone())
            .filter(|dependency| dependency != identity)
            .collect::<BTreeSet<_>>();

        let mut errors = Vec::new();
        for dependency in dependency_identities {
            if workspace.package(&dependency).is_none() {
                continue;
            }
            if let Err(mut dependency_errors) =
                self.check_workspace_package(workspace, &dependency, typed_packages, in_progress)
            {
                errors.append(&mut dependency_errors);
            }
        }

        if errors.is_empty() {
            let mut typed = TypedProgram::from_resolved(package.program.clone());
            if let Err(mut package_errors) = decls::lower_declaration_signatures(&mut typed) {
                errors.append(&mut package_errors);
            } else if let Err(mut package_errors) =
                self.hydrate_mounted_symbol_types(&mut typed, typed_packages)
            {
                errors.append(&mut package_errors);
            } else if let Err(mut package_errors) = exprs::type_program(&mut typed) {
                errors.append(&mut package_errors);
            } else {
                typed_packages.insert(identity.clone(), TypedPackage::new(identity.clone(), typed));
            }
        }

        in_progress.remove(identity);

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn hydrate_mounted_symbol_types(
        &mut self,
        typed: &mut TypedProgram,
        typed_packages: &BTreeMap<PackageIdentity, TypedPackage>,
    ) -> TypecheckResult<()> {
        let mounted_symbols = typed
            .resolved()
            .symbols
            .iter_with_ids()
            .filter_map(|(symbol_id, symbol)| {
                symbol
                    .mounted_from
                    .as_ref()
                    .map(|provenance| (symbol_id, provenance.clone()))
            })
            .collect::<Vec<_>>();

        if mounted_symbols.is_empty() {
            return Ok(());
        }

        let mounted_symbol_map = mounted_symbols
            .iter()
            .map(|(local_symbol_id, provenance)| {
                (
                    (
                        provenance.package_identity.clone(),
                        provenance.foreign_symbol,
                    ),
                    *local_symbol_id,
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut imported_cache = BTreeMap::new();
        let mut errors = Vec::new();

        for (local_symbol_id, provenance) in mounted_symbols {
            match self.import_mounted_symbol_type(
                typed,
                typed_packages,
                &mounted_symbol_map,
                &mut imported_cache,
                local_symbol_id,
                &provenance,
            ) {
                Ok(()) => {}
                Err(error) => errors.push(error),
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn import_mounted_symbol_type(
        &mut self,
        typed: &mut TypedProgram,
        typed_packages: &BTreeMap<PackageIdentity, TypedPackage>,
        mounted_symbol_map: &BTreeMap<(PackageIdentity, SymbolId), SymbolId>,
        imported_cache: &mut BTreeMap<(PackageIdentity, CheckedTypeId), CheckedTypeId>,
        local_symbol_id: SymbolId,
        provenance: &MountedSymbolProvenance,
    ) -> Result<(), TypecheckError> {
        let foreign_package = typed_packages
            .get(&provenance.package_identity)
            .ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::Internal,
                    format!(
                        "typed workspace lost loaded package '{}'",
                        provenance.package_identity.canonical_root
                    ),
                )
            })?;
        let foreign_type = foreign_package
            .program
            .typed_symbol(provenance.foreign_symbol)
            .ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::Internal,
                    format!(
                        "mounted imported symbol {} does not retain foreign typed facts",
                        provenance.foreign_symbol.0
                    ),
                )
            })?;
        let foreign_declared_type = foreign_type
            .declared_type
            .ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::Internal,
                    format!(
                        "mounted imported symbol {} does not retain foreign typed facts",
                        provenance.foreign_symbol.0
                    ),
                )
            })?;
        let translated = self.import_type_id(
            typed,
            &foreign_package.identity,
            &foreign_package.program,
            foreign_declared_type,
            mounted_symbol_map,
            imported_cache,
        )?;
        let translated_receiver = foreign_type
            .receiver_type
            .map(|receiver_type| {
                self.import_type_id(
                    typed,
                    &foreign_package.identity,
                    &foreign_package.program,
                    receiver_type,
                    mounted_symbol_map,
                    imported_cache,
                )
            })
            .transpose()?;
        let typed_symbol = typed.typed_symbol_mut(local_symbol_id).ok_or_else(|| {
            TypecheckError::new(
                TypecheckErrorKind::Internal,
                "typed symbol table lost a mounted imported symbol",
            )
        })?;
        typed_symbol.declared_type = Some(translated);
        typed_symbol.receiver_type = translated_receiver;
        Ok(())
    }

    fn import_type_id(
        &mut self,
        target_program: &mut TypedProgram,
        source_identity: &PackageIdentity,
        source_program: &TypedProgram,
        source_type_id: CheckedTypeId,
        mounted_symbol_map: &BTreeMap<(PackageIdentity, SymbolId), SymbolId>,
        imported_cache: &mut BTreeMap<(PackageIdentity, CheckedTypeId), CheckedTypeId>,
    ) -> Result<CheckedTypeId, TypecheckError> {
        if let Some(existing) = imported_cache.get(&(source_identity.clone(), source_type_id)) {
            return Ok(*existing);
        }

        let source_type = source_program
            .type_table()
            .get(source_type_id)
            .cloned()
            .ok_or_else(|| {
                TypecheckError::new(
                    TypecheckErrorKind::Internal,
                    format!("foreign typed package lost type {}", source_type_id.0),
                )
            })?;

        let translated = match source_type {
            CheckedType::Builtin(builtin) => target_program.type_table_mut().intern_builtin(builtin),
            CheckedType::Declared { symbol, name, kind } => {
                let translated_symbol = translated_symbol_id(
                    source_identity,
                    source_program,
                    symbol,
                    mounted_symbol_map,
                )
                .unwrap_or(symbol);
                target_program.type_table_mut().intern(CheckedType::Declared {
                    symbol: translated_symbol,
                    name,
                    kind,
                })
            }
            CheckedType::Array { element_type, size } => {
                let element_type = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    element_type,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Array { element_type, size })
            }
            CheckedType::Vector { element_type } => {
                let element_type = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    element_type,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Vector { element_type })
            }
            CheckedType::Sequence { element_type } => {
                let element_type = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    element_type,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Sequence { element_type })
            }
            CheckedType::Set { member_types } => {
                let member_types = member_types
                    .into_iter()
                    .map(|member| {
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            member,
                            mounted_symbol_map,
                            imported_cache,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Set { member_types })
            }
            CheckedType::Map {
                key_type,
                value_type,
            } => {
                let key_type = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    key_type,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                let value_type = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    value_type,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                target_program.type_table_mut().intern(CheckedType::Map {
                    key_type,
                    value_type,
                })
            }
            CheckedType::Optional { inner } => {
                let inner = self.import_type_id(
                    target_program,
                    source_identity,
                    source_program,
                    inner,
                    mounted_symbol_map,
                    imported_cache,
                )?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Optional { inner })
            }
            CheckedType::Error { inner } => {
                let inner = inner
                    .map(|inner| {
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            inner,
                            mounted_symbol_map,
                            imported_cache,
                        )
                    })
                    .transpose()?;
                target_program.type_table_mut().intern(CheckedType::Error { inner })
            }
            CheckedType::Record { fields } => {
                let mut translated_fields = BTreeMap::new();
                for (field_name, field_type) in fields {
                    translated_fields.insert(
                        field_name,
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            field_type,
                            mounted_symbol_map,
                            imported_cache,
                        )?,
                    );
                }
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Record { fields: translated_fields })
            }
            CheckedType::Entry { variants } => {
                let mut translated_variants = BTreeMap::new();
                for (variant_name, variant_type) in variants {
                    translated_variants.insert(
                        variant_name,
                        variant_type
                            .map(|variant| {
                                self.import_type_id(
                                    target_program,
                                    source_identity,
                                    source_program,
                                    variant,
                                    mounted_symbol_map,
                                    imported_cache,
                                )
                            })
                            .transpose()?,
                    );
                }
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Entry {
                        variants: translated_variants,
                    })
            }
            CheckedType::Routine(signature) => {
                let params = signature
                    .params
                    .into_iter()
                    .map(|param| {
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            param,
                            mounted_symbol_map,
                            imported_cache,
                        )
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                let return_type = signature
                    .return_type
                    .map(|return_type| {
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            return_type,
                            mounted_symbol_map,
                            imported_cache,
                        )
                    })
                    .transpose()?;
                let error_type = signature
                    .error_type
                    .map(|error_type| {
                        self.import_type_id(
                            target_program,
                            source_identity,
                            source_program,
                            error_type,
                            mounted_symbol_map,
                            imported_cache,
                        )
                    })
                    .transpose()?;
                target_program
                    .type_table_mut()
                    .intern(CheckedType::Routine(crate::RoutineType {
                        params,
                        return_type,
                        error_type,
                    }))
            }
        };

        imported_cache.insert((source_identity.clone(), source_type_id), translated);
        Ok(translated)
    }
}

fn translated_symbol_id(
    source_identity: &PackageIdentity,
    source_program: &TypedProgram,
    symbol_id: SymbolId,
    mounted_symbol_map: &BTreeMap<(PackageIdentity, SymbolId), SymbolId>,
) -> Option<SymbolId> {
    let resolved_symbol = source_program.resolved().symbol(symbol_id)?;
    let translation_key = resolved_symbol
        .mounted_from
        .as_ref()
        .map(|provenance| {
            (
                provenance.package_identity.clone(),
                provenance.foreign_symbol,
            )
        })
        .unwrap_or_else(|| (source_identity.clone(), symbol_id));

    mounted_symbol_map.get(&translation_key).copied()
}
