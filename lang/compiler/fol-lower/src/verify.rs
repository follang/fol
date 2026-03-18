use crate::{LoweredTerminator, LoweredWorkspace, LoweringError, LoweringErrorKind};
use std::collections::{BTreeSet, VecDeque};

pub(crate) fn verify_workspace(workspace: &LoweredWorkspace) -> Result<(), Vec<LoweringError>> {
    let mut errors = Vec::new();
    let valid_global_ids = workspace
        .packages()
        .flat_map(|package| package.global_decls.keys().copied())
        .collect::<BTreeSet<_>>();
    let valid_routine_ids = workspace
        .packages()
        .flat_map(|package| package.routine_decls.keys().copied())
        .collect::<BTreeSet<_>>();

    for package in workspace.packages() {
        for ownership in package.symbol_ownership.values() {
            match ownership.mounted_from.as_ref() {
                Some(provenance) => {
                    if ownership.owning_package != provenance.package_identity {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered package '{}' recorded mounted symbol {} with conflicting owning package '{}'",
                                package.identity.display_name,
                                ownership.symbol_id.0,
                                ownership.owning_package.display_name,
                            ),
                        ));
                    }
                }
                None => {
                    if ownership.owning_package != package.identity {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered package '{}' recorded local symbol {} as owned by foreign package '{}'",
                                package.identity.display_name,
                                ownership.symbol_id.0,
                                ownership.owning_package.display_name,
                            ),
                        ));
                    }
                }
            }
        }

        for global in package.global_decls.values() {
            if workspace.type_table().get(global.type_id).is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered global '{}' references missing type {}",
                        global.name, global.type_id.0
                    ),
                ));
            }
            if let Some(error_type) = global.recoverable_error_type {
                if workspace.type_table().get(error_type).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered global '{}' references missing recoverable error type {}",
                            global.name, error_type.0
                        ),
                    ));
                }
            }
        }

        for type_decl in package.type_decls.values() {
            if workspace.type_table().get(type_decl.runtime_type).is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered type '{}' references missing runtime type {}",
                        type_decl.name, type_decl.runtime_type.0
                    ),
                ));
            }
        }

        for routine in package.routine_decls.values() {
            if let Some(signature) = routine.signature {
                if workspace.type_table().get(signature).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' references missing signature type {}",
                            routine.name, signature.0
                        ),
                    ));
                }
            }
            if let Some(receiver_type) = routine.receiver_type {
                if workspace.type_table().get(receiver_type).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' references missing receiver type {}",
                            routine.name, receiver_type.0
                        ),
                    ));
                }
            }
            for local in routine.locals.iter() {
                if let Some(type_id) = local.type_id {
                    if workspace.type_table().get(type_id).is_none() {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered routine '{}' local {:?} references missing type {}",
                                routine.name, local.name, type_id.0
                            ),
                        ));
                    }
                }
                if let Some(error_type) = local.recoverable_error_type {
                    if workspace.type_table().get(error_type).is_none() {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered routine '{}' local {:?} references missing recoverable error type {}",
                                routine.name, local.name, error_type.0
                            ),
                        ));
                    }
                }
            }
            for param in &routine.params {
                if routine.locals.get(*param).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' parameter local {} is missing",
                            routine.name, param.0
                        ),
                    ));
                }
            }
            if let Some(body_result) = routine.body_result {
                if routine.locals.get(body_result).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' body result local {} is missing",
                            routine.name, body_result.0
                        ),
                    ));
                }
            }
            for (symbol_id, local_id) in &routine.local_symbols {
                if routine.locals.get(*local_id).is_none() {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' maps symbol {} to missing local {}",
                            routine.name, symbol_id.0, local_id.0
                        ),
                    ));
                }
            }

            let mut reachable = BTreeSet::new();
            let mut queue = VecDeque::from([routine.entry_block]);

            while let Some(block_id) = queue.pop_front() {
                if !reachable.insert(block_id) {
                    continue;
                }
                let Some(block) = routine.blocks.get(block_id) else {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' references a missing block {}",
                            routine.name, block_id.0
                        ),
                    ));
                    continue;
                };

                for instr_id in &block.instructions {
                    let Some(instr) = routine.instructions.get(*instr_id) else {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered routine '{}' block {} references missing instruction {}",
                                routine.name, block_id.0, instr_id.0
                            ),
                        ));
                        continue;
                    };
                    if let Some(result) = instr.result {
                        if routine.locals.get(result).is_none() {
                            errors.push(LoweringError::with_kind(
                                LoweringErrorKind::InvalidInput,
                                format!(
                                    "lowered routine '{}' instruction {} writes to missing local {}",
                                    routine.name, instr_id.0, result.0
                                ),
                            ));
                        }
                    }
                    verify_instruction(
                        workspace,
                        package,
                        routine,
                        instr,
                        &valid_global_ids,
                        &valid_routine_ids,
                        &mut errors,
                    );
                }

                match block.terminator.as_ref() {
                    Some(LoweredTerminator::Jump { target }) => {
                        enqueue_target(
                            routine.name.as_str(),
                            &routine.blocks,
                            &mut errors,
                            &mut queue,
                            *target,
                        );
                    }
                    Some(LoweredTerminator::Branch {
                        condition,
                        then_block,
                        else_block,
                    }) => {
                        if routine.locals.get(*condition).is_none() {
                            errors.push(LoweringError::with_kind(
                                LoweringErrorKind::InvalidInput,
                                format!(
                                    "lowered routine '{}' branch uses missing local {}",
                                    routine.name, condition.0
                                ),
                            ));
                        }
                        enqueue_target(
                            routine.name.as_str(),
                            &routine.blocks,
                            &mut errors,
                            &mut queue,
                            *then_block,
                        );
                        enqueue_target(
                            routine.name.as_str(),
                            &routine.blocks,
                            &mut errors,
                            &mut queue,
                            *else_block,
                        );
                    }
                    Some(LoweredTerminator::Return { value }) => {
                        if let Some(local) = value {
                            if routine.locals.get(*local).is_none() {
                                errors.push(LoweringError::with_kind(
                                    LoweringErrorKind::InvalidInput,
                                    format!(
                                        "lowered routine '{}' return uses missing local {}",
                                        routine.name, local.0
                                    ),
                                ));
                            }
                        }
                    }
                    Some(LoweredTerminator::Report { value }) => {
                        if let Some(local) = value {
                            if routine.locals.get(*local).is_none() {
                                errors.push(LoweringError::with_kind(
                                    LoweringErrorKind::InvalidInput,
                                    format!(
                                        "lowered routine '{}' report uses missing local {}",
                                        routine.name, local.0
                                    ),
                                ));
                            }
                        }
                    }
                    Some(LoweredTerminator::Panic { value }) => {
                        if let Some(local) = value {
                            if routine.locals.get(*local).is_none() {
                                errors.push(LoweringError::with_kind(
                                    LoweringErrorKind::InvalidInput,
                                    format!(
                                        "lowered routine '{}' panic uses missing local {}",
                                        routine.name, local.0
                                    ),
                                ));
                            }
                        }
                    }
                    Some(LoweredTerminator::Unreachable) => {}
                    None => {}
                }
            }

            let reachable_unterminated = reachable
                .iter()
                .filter(|block_id| {
                    routine
                        .blocks
                        .get(**block_id)
                        .map(|block| block.terminator.is_none())
                        .unwrap_or(false)
                })
                .count();
            if reachable_unterminated > 1 {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' retains {} reachable unterminated blocks; V1 lowering allows at most one fallthrough block",
                        routine.name, reachable_unterminated
                    ),
                ));
            }

            for (block_id, _block) in routine.blocks.iter_with_ids() {
                if !reachable.contains(&block_id) {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' produced unreachable block {}",
                            routine.name, block_id.0
                        ),
                    ));
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn verify_instruction(
    workspace: &LoweredWorkspace,
    package: &crate::LoweredPackage,
    routine: &crate::LoweredRoutine,
    instr: &crate::LoweredInstr,
    valid_global_ids: &BTreeSet<crate::LoweredGlobalId>,
    valid_routine_ids: &BTreeSet<crate::LoweredRoutineId>,
    errors: &mut Vec<LoweringError>,
) {
    match &instr.kind {
        crate::LoweredInstrKind::Const(_) => {}
        crate::LoweredInstrKind::LoadGlobal { global } => {
            if !valid_global_ids.contains(global) {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' loads missing global {}",
                        routine.name, global.0
                    ),
                ));
            }
        }
        crate::LoweredInstrKind::LoadLocal { local }
        | crate::LoweredInstrKind::UnwrapShell { operand: local }
        | crate::LoweredInstrKind::CheckRecoverable { operand: local }
        | crate::LoweredInstrKind::UnwrapRecoverable { operand: local }
        | crate::LoweredInstrKind::ExtractRecoverableError { operand: local } => {
            verify_local_reference(routine, instr.id.0, "operand", *local, errors);
        }
        crate::LoweredInstrKind::StoreLocal { local, value } => {
            verify_local_reference(routine, instr.id.0, "store target", *local, errors);
            verify_local_reference(routine, instr.id.0, "store value", *value, errors);
        }
        crate::LoweredInstrKind::StoreGlobal { global, value } => {
            if !valid_global_ids.contains(global) {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' stores to missing global {}",
                        routine.name, global.0
                    ),
                ));
            }
            verify_local_reference(routine, instr.id.0, "store value", *value, errors);
        }
        crate::LoweredInstrKind::Call {
            callee,
            args,
            error_type,
        } => {
            if !valid_routine_ids.contains(callee) {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' calls missing routine {}",
                        routine.name, callee.0
                    ),
                ));
            }
            for arg in args {
                verify_local_reference(routine, instr.id.0, "call arg", *arg, errors);
            }
            if let Some(error_type) = error_type {
                verify_type_reference(
                    workspace,
                    package,
                    routine,
                    instr.id.0,
                    "call error type",
                    *error_type,
                    errors,
                );
            }
            if let Some(result) = instr.result {
                let local_effect = routine
                    .locals
                    .get(result)
                    .and_then(|local| local.recoverable_error_type);
                if local_effect != *error_type {
                    errors.push(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!(
                            "lowered routine '{}' call instruction {} writes recoverable effect {:?} but local {} stores {:?}",
                            routine.name, instr.id.0, error_type, result.0, local_effect
                        ),
                    ));
                }
            }
        }
        crate::LoweredInstrKind::IntrinsicCall { intrinsic, args } => {
            if fol_intrinsics::intrinsic_by_id(*intrinsic).is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' uses missing intrinsic {}",
                        routine.name,
                        intrinsic.index()
                    ),
                ));
            } else if fol_intrinsics::backend_role_for_intrinsic(*intrinsic)
                != Some(fol_intrinsics::IntrinsicBackendRole::PureOp)
            {
                let intrinsic_name = fol_intrinsics::intrinsic_by_id(*intrinsic)
                    .map(|entry| entry.name)
                    .unwrap_or("<missing>");
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' instruction {} uses intrinsic '.{}' as an IntrinsicCall even though it is not a pure-op intrinsic",
                        routine.name, instr.id.0, intrinsic_name
                    ),
                ));
            }
            if instr.result.is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' intrinsic instruction {} must write a result local",
                        routine.name, instr.id.0
                    ),
                ));
            }
            for arg in args {
                verify_local_reference(routine, instr.id.0, "intrinsic arg", *arg, errors);
            }
        }
        crate::LoweredInstrKind::RuntimeHook { intrinsic, args } => {
            if fol_intrinsics::intrinsic_by_id(*intrinsic).is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' uses missing runtime hook intrinsic {}",
                        routine.name,
                        intrinsic.index()
                    ),
                ));
            } else if fol_intrinsics::backend_role_for_intrinsic(*intrinsic)
                != Some(fol_intrinsics::IntrinsicBackendRole::RuntimeHook)
            {
                let intrinsic_name = fol_intrinsics::intrinsic_by_id(*intrinsic)
                    .map(|entry| entry.name)
                    .unwrap_or("<missing>");
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' instruction {} uses intrinsic '.{}' as a RuntimeHook even though it is not a runtime-hook intrinsic",
                        routine.name, instr.id.0, intrinsic_name
                    ),
                ));
            }
            if let Some(result) = instr.result {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' runtime hook instruction {} must not write result local {}",
                        routine.name, instr.id.0, result.0
                    ),
                ));
            }
            for arg in args {
                verify_local_reference(routine, instr.id.0, "runtime hook arg", *arg, errors);
            }
        }
        crate::LoweredInstrKind::LengthOf { operand } => {
            if instr.result.is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' length helper instruction {} must write a result local",
                        routine.name, instr.id.0
                    ),
                ));
            }
            verify_local_reference(routine, instr.id.0, "length operand", *operand, errors);
        }
        crate::LoweredInstrKind::ConstructRecord { type_id, fields } => {
            verify_type_reference(
                workspace,
                package,
                routine,
                instr.id.0,
                "record type",
                *type_id,
                errors,
            );
            for (_, value) in fields {
                verify_local_reference(routine, instr.id.0, "record field", *value, errors);
            }
        }
        crate::LoweredInstrKind::ConstructEntry {
            type_id, payload, ..
        } => {
            verify_type_reference(
                workspace,
                package,
                routine,
                instr.id.0,
                "entry type",
                *type_id,
                errors,
            );
            if let Some(payload) = payload {
                verify_local_reference(routine, instr.id.0, "entry payload", *payload, errors);
            }
        }
        crate::LoweredInstrKind::ConstructLinear {
            type_id, elements, ..
        } => {
            verify_type_reference(
                workspace,
                package,
                routine,
                instr.id.0,
                "linear type",
                *type_id,
                errors,
            );
            for element in elements {
                verify_local_reference(routine, instr.id.0, "linear element", *element, errors);
            }
        }
        crate::LoweredInstrKind::ConstructSet { type_id, members } => {
            verify_type_reference(
                workspace, package, routine, instr.id.0, "set type", *type_id, errors,
            );
            for member in members {
                verify_local_reference(routine, instr.id.0, "set member", *member, errors);
            }
        }
        crate::LoweredInstrKind::ConstructMap { type_id, entries } => {
            verify_type_reference(
                workspace, package, routine, instr.id.0, "map type", *type_id, errors,
            );
            for (key, value) in entries {
                verify_local_reference(routine, instr.id.0, "map key", *key, errors);
                verify_local_reference(routine, instr.id.0, "map value", *value, errors);
            }
        }
        crate::LoweredInstrKind::ConstructOptional { type_id, value }
        | crate::LoweredInstrKind::ConstructError { type_id, value } => {
            verify_type_reference(
                workspace,
                package,
                routine,
                instr.id.0,
                "shell type",
                *type_id,
                errors,
            );
            if let Some(value) = value {
                verify_local_reference(routine, instr.id.0, "shell value", *value, errors);
            }
        }
        crate::LoweredInstrKind::FieldAccess { base, .. } => {
            verify_local_reference(routine, instr.id.0, "field base", *base, errors);
        }
        crate::LoweredInstrKind::IndexAccess { container, index } => {
            verify_local_reference(routine, instr.id.0, "index container", *container, errors);
            verify_local_reference(routine, instr.id.0, "index value", *index, errors);
        }
        crate::LoweredInstrKind::Cast {
            operand,
            target_type,
        } => {
            verify_local_reference(routine, instr.id.0, "cast operand", *operand, errors);
            verify_type_reference(
                workspace,
                package,
                routine,
                instr.id.0,
                "cast type",
                *target_type,
                errors,
            );
        }
    }

    match &instr.kind {
        crate::LoweredInstrKind::CheckRecoverable { operand }
        | crate::LoweredInstrKind::UnwrapRecoverable { operand }
        | crate::LoweredInstrKind::ExtractRecoverableError { operand } => {
            let operand_effect = routine
                .locals
                .get(*operand)
                .and_then(|local| local.recoverable_error_type);
            if operand_effect.is_none() {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' instruction {} expects a recoverable operand local {}",
                        routine.name, instr.id.0, operand.0
                    ),
                ));
            }
        }
        _ => {}
    }
}

fn verify_local_reference(
    routine: &crate::LoweredRoutine,
    instr_id: usize,
    label: &str,
    local: crate::LoweredLocalId,
    errors: &mut Vec<LoweringError>,
) {
    if routine.locals.get(local).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered routine '{}' instruction {} uses missing {} local {}",
                routine.name, instr_id, label, local.0
            ),
        ));
    }
}

fn verify_type_reference(
    workspace: &LoweredWorkspace,
    package: &crate::LoweredPackage,
    routine: &crate::LoweredRoutine,
    instr_id: usize,
    label: &str,
    type_id: crate::LoweredTypeId,
    errors: &mut Vec<LoweringError>,
) {
    if workspace.type_table().get(type_id).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered package '{}' routine '{}' instruction {} uses missing {} {}",
                package.identity.display_name, routine.name, instr_id, label, type_id.0
            ),
        ));
    }
}

fn enqueue_target(
    routine_name: &str,
    blocks: &crate::IdTable<crate::LoweredBlockId, crate::LoweredBlock>,
    errors: &mut Vec<LoweringError>,
    queue: &mut VecDeque<crate::LoweredBlockId>,
    target: crate::LoweredBlockId,
) {
    if blocks.get(target).is_none() {
        errors.push(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "lowered routine '{}' terminator targets missing block {}",
                routine_name, target.0
            ),
        ));
        return;
    }
    queue.push_back(target);
}

#[cfg(test)]
mod tests {
    use super::verify_workspace;
    use crate::{
        control::{
            LoweredBlock, LoweredInstr, LoweredInstrKind, LoweredLocal, LoweredRoutine,
            LoweredTerminator,
        },
        ids::{
            LoweredBlockId, LoweredInstrId, LoweredLocalId, LoweredPackageId, LoweredRoutineId,
            LoweredTypeId,
        },
        model::{
            LoweredPackage, LoweredRecoverableAbi, LoweredSourceMap, LoweredSymbolOwnership,
            LoweredWorkspace,
        },
        types::{LoweredBuiltinType, LoweredTypeTable},
    };
    use fol_resolver::{
        MountedSymbolProvenance, PackageIdentity, PackageSourceKind, SourceUnitId, SymbolId,
    };
    use std::collections::BTreeMap;

    fn identity(name: &str) -> PackageIdentity {
        PackageIdentity {
            source_kind: PackageSourceKind::Entry,
            canonical_root: format!("/workspace/{name}"),
            display_name: name.to_string(),
        }
    }

    fn empty_workspace(identity: PackageIdentity, package: LoweredPackage) -> LoweredWorkspace {
        let mut type_table = LoweredTypeTable::new();
        let recoverable_abi =
            LoweredRecoverableAbi::v1(type_table.intern_builtin(LoweredBuiltinType::Bool));
        LoweredWorkspace::new(
            identity.clone(),
            BTreeMap::from([(identity, package)]),
            Vec::new(),
            type_table,
            LoweredSourceMap::new(),
            recoverable_abi,
        )
    }

    #[test]
    fn verifier_rejects_missing_branch_targets() {
        let identity = identity("app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: Vec::new(),
            terminator: Some(LoweredTerminator::Jump {
                target: LoweredBlockId(9),
            }),
        });
        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = empty_workspace(identity, package);

        let errors =
            verify_workspace(&workspace).expect_err("verifier should reject missing jump targets");

        assert!(errors
            .iter()
            .any(|error| error.message().contains("missing block 9")));
    }

    #[test]
    fn verifier_rejects_unreachable_blocks() {
        let identity = identity("app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: Vec::new(),
            terminator: Some(LoweredTerminator::Return { value: None }),
        });
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(1),
            instructions: Vec::new(),
            terminator: Some(LoweredTerminator::Return { value: None }),
        });
        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = empty_workspace(identity, package);

        let errors =
            verify_workspace(&workspace).expect_err("verifier should reject unreachable blocks");

        assert!(errors
            .iter()
            .any(|error| error.message().contains("unreachable block 1")));
    }

    #[test]
    fn verifier_rejects_dangling_locals_and_missing_type_ids() {
        let identity = identity("app");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(LoweredTypeId(9)),
            recoverable_error_type: None,
            name: Some("bad".to_string()),
        });
        routine.instructions.push(LoweredInstr {
            id: LoweredInstrId(0),
            result: Some(LoweredLocalId(1)),
            kind: LoweredInstrKind::LoadLocal {
                local: LoweredLocalId(2),
            },
        });
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: vec![LoweredInstrId(0)],
            terminator: Some(LoweredTerminator::Return {
                value: Some(LoweredLocalId(1)),
            }),
        });

        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = empty_workspace(identity, package);

        let errors = verify_workspace(&workspace)
            .expect_err("verifier should reject missing locals and missing lowered type ids");

        assert!(errors
            .iter()
            .any(|error| error.message().contains("references missing type 9")));
        assert!(errors
            .iter()
            .any(|error| error.message().contains("writes to missing local 1")));
        assert!(errors
            .iter()
            .any(|error| error.message().contains("uses missing operand local 2")));
        assert!(errors
            .iter()
            .any(|error| error.message().contains("return uses missing local 1")));
    }

    #[test]
    fn verifier_rejects_impossible_mounted_symbol_ownership() {
        let identity = identity("app");
        let foreign = identity("shared");
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: Vec::new(),
            terminator: Some(LoweredTerminator::Return { value: None }),
        });

        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.symbol_ownership.insert(
            SymbolId(7),
            LoweredSymbolOwnership {
                symbol_id: SymbolId(7),
                source_unit_id: SourceUnitId(0),
                owning_package: foreign.clone(),
                mounted_from: Some(MountedSymbolProvenance {
                    package_identity: identity.clone(),
                    foreign_symbol: SymbolId(3),
                }),
            },
        );
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = empty_workspace(identity, package);

        let errors = verify_workspace(&workspace)
            .expect_err("verifier should reject conflicting mounted symbol ownership");

        assert!(errors.iter().any(|error| {
            error
                .message()
                .contains("recorded mounted symbol 7 with conflicting owning package")
        }));
    }

    #[test]
    fn verifier_rejects_intrinsic_calls_using_non_pure_intrinsics() {
        let identity = identity("app");
        let mut type_table = LoweredTypeTable::new();
        let bool_type = type_table.intern_builtin(LoweredBuiltinType::Bool);
        let recoverable_abi = LoweredRecoverableAbi::v1(bool_type);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_type),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        routine.instructions.push(LoweredInstr {
            id: LoweredInstrId(0),
            result: Some(LoweredLocalId(0)),
            kind: LoweredInstrKind::IntrinsicCall {
                intrinsic: fol_intrinsics::intrinsic_by_canonical_name("echo")
                    .expect("echo should exist")
                    .id,
                args: vec![LoweredLocalId(0)],
            },
        });
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: vec![LoweredInstrId(0)],
            terminator: Some(LoweredTerminator::Return {
                value: Some(LoweredLocalId(0)),
            }),
        });

        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            BTreeMap::from([(identity, package)]),
            Vec::new(),
            type_table,
            LoweredSourceMap::new(),
            recoverable_abi,
        );

        let errors = verify_workspace(&workspace)
            .expect_err("verifier should reject runtime hooks lowered as intrinsic calls");

        assert!(errors.iter().any(|error| {
            error
                .message()
                .contains("uses intrinsic '.echo' as an IntrinsicCall")
        }));
    }

    #[test]
    fn verifier_rejects_runtime_hooks_with_results_and_helper_without_results() {
        let identity = identity("app");
        let mut type_table = LoweredTypeTable::new();
        let bool_type = type_table.intern_builtin(LoweredBuiltinType::Bool);
        let int_type = type_table.intern_builtin(LoweredBuiltinType::Int);
        let seq_type = type_table.intern_sequence(bool_type);
        let recoverable_abi = LoweredRecoverableAbi::v1(bool_type);
        let mut routine = LoweredRoutine::new(LoweredRoutineId(0), "main", LoweredBlockId(0));
        routine.locals.push(LoweredLocal {
            id: LoweredLocalId(0),
            type_id: Some(bool_type),
            recoverable_error_type: None,
            name: Some("flag".to_string()),
        });
        routine.locals.push(LoweredLocal {
            id: LoweredLocalId(1),
            type_id: Some(seq_type),
            recoverable_error_type: None,
            name: Some("items".to_string()),
        });
        routine.locals.push(LoweredLocal {
            id: LoweredLocalId(2),
            type_id: Some(int_type),
            recoverable_error_type: None,
            name: Some("count".to_string()),
        });
        routine.instructions.push(LoweredInstr {
            id: LoweredInstrId(0),
            result: Some(LoweredLocalId(2)),
            kind: LoweredInstrKind::RuntimeHook {
                intrinsic: fol_intrinsics::intrinsic_by_canonical_name("echo")
                    .expect("echo should exist")
                    .id,
                args: vec![LoweredLocalId(0)],
            },
        });
        routine.instructions.push(LoweredInstr {
            id: LoweredInstrId(1),
            result: None,
            kind: LoweredInstrKind::LengthOf {
                operand: LoweredLocalId(1),
            },
        });
        routine.blocks.push(LoweredBlock {
            id: LoweredBlockId(0),
            instructions: vec![LoweredInstrId(0), LoweredInstrId(1)],
            terminator: Some(LoweredTerminator::Return {
                value: Some(LoweredLocalId(0)),
            }),
        });

        let mut package = LoweredPackage::new(LoweredPackageId(0), identity.clone());
        package.routine_decls.insert(LoweredRoutineId(0), routine);
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            BTreeMap::from([(identity, package)]),
            Vec::new(),
            type_table,
            LoweredSourceMap::new(),
            recoverable_abi,
        );

        let errors = verify_workspace(&workspace)
            .expect_err("verifier should reject impossible runtime-hook and helper result shapes");

        assert!(errors.iter().any(|error| {
            error
                .message()
                .contains("runtime hook instruction 0 must not write result local 2")
        }));
        assert!(errors.iter().any(|error| {
            error
                .message()
                .contains("length helper instruction 1 must write a result local")
        }));
    }
}
