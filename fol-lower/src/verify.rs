use crate::{LoweredTerminator, LoweredWorkspace, LoweringError, LoweringErrorKind};
use std::collections::{BTreeSet, VecDeque};

pub(crate) fn verify_workspace(workspace: &LoweredWorkspace) -> Result<(), Vec<LoweringError>> {
    let mut errors = Vec::new();

    for package in workspace.packages() {
        for routine in package.routine_decls.values() {
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
                    if routine.instructions.get(*instr_id).is_none() {
                        errors.push(LoweringError::with_kind(
                            LoweringErrorKind::InvalidInput,
                            format!(
                                "lowered routine '{}' block {} references missing instruction {}",
                                routine.name, block_id.0, instr_id.0
                            ),
                        ));
                    }
                }

                match block.terminator.as_ref() {
                    Some(LoweredTerminator::Jump { target }) => {
                        enqueue_target(routine.name.as_str(), &routine.blocks, &mut errors, &mut queue, *target);
                    }
                    Some(LoweredTerminator::Branch {
                        then_block,
                        else_block,
                        ..
                    }) => {
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
                    Some(LoweredTerminator::Return { .. })
                    | Some(LoweredTerminator::Report { .. })
                    | Some(LoweredTerminator::Unreachable) => {}
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
        control::{LoweredBlock, LoweredRoutine, LoweredTerminator},
        ids::{LoweredBlockId, LoweredPackageId, LoweredRoutineId},
        model::{LoweredPackage, LoweredSourceMap, LoweredWorkspace},
        types::LoweredTypeTable,
    };
    use fol_resolver::{PackageIdentity, PackageSourceKind};
    use std::collections::BTreeMap;

    fn identity(name: &str) -> PackageIdentity {
        PackageIdentity {
            source_kind: PackageSourceKind::Entry,
            canonical_root: format!("/workspace/{name}"),
            display_name: name.to_string(),
        }
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
        package
            .routine_decls
            .insert(LoweredRoutineId(0), routine);
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            BTreeMap::from([(identity, package)]),
            LoweredTypeTable::new(),
            LoweredSourceMap::new(),
        );

        let errors = verify_workspace(&workspace).expect_err("verifier should reject missing jump targets");

        assert!(errors.iter().any(|error| error.message().contains("missing block 9")));
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
        package
            .routine_decls
            .insert(LoweredRoutineId(0), routine);
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            BTreeMap::from([(identity, package)]),
            LoweredTypeTable::new(),
            LoweredSourceMap::new(),
        );

        let errors = verify_workspace(&workspace).expect_err("verifier should reject unreachable blocks");

        assert!(errors.iter().any(|error| error.message().contains("unreachable block 1")));
    }
}
