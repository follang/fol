use crate::{
    control::{LoweredInstr, LoweredInstrKind, LoweredLinearKind, LoweredLocal, LoweredOperand},
    ids::{LoweredBlockId, LoweredInstrId, LoweredLocalId, LoweredTypeId},
    LoweredGlobalId, LoweredPackage, LoweredRoutine, LoweredRoutineId, LoweredWorkspace,
    LoweringError, LoweringErrorKind,
};
use fol_parser::ast::{AstNode, ContainerType, Literal, LoopCondition};
use fol_resolver::{
    MountedSymbolProvenance, PackageIdentity, ResolvedSymbol, ScopeId, SourceUnitId, SymbolId,
    SymbolKind,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoweredValue {
    pub local_id: LoweredLocalId,
    pub type_id: LoweredTypeId,
    pub recoverable_error_type: Option<LoweredTypeId>,
}

#[derive(Debug, Clone)]
pub(crate) struct EntryVariantLowering {
    pub type_id: LoweredTypeId,
    pub payload_type: Option<LoweredTypeId>,
    pub default: Option<AstNode>,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceDeclIndex {
    globals: BTreeMap<(PackageIdentity, SymbolId), LoweredGlobalId>,
    global_effects: BTreeMap<(PackageIdentity, LoweredGlobalId), Option<LoweredTypeId>>,
    routines: BTreeMap<(PackageIdentity, SymbolId), LoweredRoutineId>,
    routine_params: BTreeMap<LoweredRoutineId, Vec<LoweredTypeId>>,
    entry_variants: BTreeMap<(PackageIdentity, SymbolId, String), EntryVariantLowering>,
}

impl WorkspaceDeclIndex {
    pub(crate) fn from_workspace(
        typed: &fol_typecheck::TypedWorkspace,
        packages: &BTreeMap<PackageIdentity, LoweredPackage>,
    ) -> Self {
        let mut index = Self::default();
        for typed_package in typed.packages() {
            let Some(lowered_package) = packages.get(&typed_package.identity) else {
                continue;
            };
            index.extend_package(typed_package, lowered_package);
        }
        index
    }

    pub(crate) fn build(workspace: &LoweredWorkspace) -> Self {
        let mut index = Self::default();
        for package in workspace.packages() {
            for global in package.global_decls.values() {
                index
                    .globals
                    .insert((package.identity.clone(), global.symbol_id), global.id);
                index.global_effects.insert(
                    (package.identity.clone(), global.id),
                    global.recoverable_error_type,
                );
            }
            for routine in package.routine_decls.values() {
                if let Some(symbol_id) = routine.symbol_id {
                    index
                        .routines
                        .insert((package.identity.clone(), symbol_id), routine.id);
                }
                let params = routine
                    .params
                    .iter()
                    .filter_map(|param| routine.locals.get(*param).and_then(|local| local.type_id))
                    .collect::<Vec<_>>();
                index.routine_params.insert(routine.id, params);
            }
        }
        index
    }

    pub(crate) fn extend_package(
        &mut self,
        typed_package: &fol_typecheck::TypedPackage,
        package: &LoweredPackage,
    ) {
        for global in package.global_decls.values() {
            self.globals
                .insert((package.identity.clone(), global.symbol_id), global.id);
            self.global_effects.insert(
                (package.identity.clone(), global.id),
                global.recoverable_error_type,
            );
        }
        for routine in package.routine_decls.values() {
            if let Some(symbol_id) = routine.symbol_id {
                self.routines
                    .insert((package.identity.clone(), symbol_id), routine.id);
            }
            let params = routine
                .params
                .iter()
                .filter_map(|param| routine.locals.get(*param).and_then(|local| local.type_id))
                .collect::<Vec<_>>();
            self.routine_params.insert(routine.id, params);
        }
        self.extend_entry_variants(typed_package, package);
    }

    pub(crate) fn global_id_for_symbol(
        &self,
        identity: &PackageIdentity,
        symbol_id: SymbolId,
    ) -> Option<LoweredGlobalId> {
        self.globals.get(&(identity.clone(), symbol_id)).copied()
    }

    pub(crate) fn routine_id_for_symbol(
        &self,
        identity: &PackageIdentity,
        symbol_id: SymbolId,
    ) -> Option<LoweredRoutineId> {
        self.routines.get(&(identity.clone(), symbol_id)).copied()
    }

    pub(crate) fn global_recoverable_error_type(
        &self,
        identity: &PackageIdentity,
        global_id: LoweredGlobalId,
    ) -> Option<LoweredTypeId> {
        self.global_effects
            .get(&(identity.clone(), global_id))
            .copied()
            .flatten()
    }

    pub(crate) fn routine_param_types(&self, routine_id: LoweredRoutineId) -> Option<&[LoweredTypeId]> {
        self.routine_params.get(&routine_id).map(Vec::as_slice)
    }

    pub(crate) fn entry_variant(
        &self,
        identity: &PackageIdentity,
        symbol_id: SymbolId,
        variant: &str,
    ) -> Option<&EntryVariantLowering> {
        self.entry_variants
            .get(&(identity.clone(), symbol_id, variant.to_string()))
    }

    fn extend_entry_variants(
        &mut self,
        typed_package: &fol_typecheck::TypedPackage,
        package: &LoweredPackage,
    ) {
        for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
            let source_unit_id = SourceUnitId(source_unit_index);
            for item in &source_unit.items {
                let AstNode::TypeDecl {
                    name,
                    type_def: fol_parser::ast::TypeDefinition::Entry {
                        variant_meta, ..
                    },
                    ..
                } = &item.node else {
                    continue;
                };
                let Some(symbol_id) = crate::decls::find_local_symbol_id(
                    &typed_package.program,
                    source_unit_id,
                    SymbolKind::Type,
                    name,
                ) else {
                    continue;
                };
                let Some(type_decl) = package.type_decls.get(&symbol_id) else {
                    continue;
                };
                let crate::LoweredTypeDeclKind::Entry { variants } = &type_decl.kind else {
                    continue;
                };
                for variant in variants {
                    let default = variant_meta
                        .get(&variant.name)
                        .and_then(|meta| meta.default.clone());
                    self.entry_variants.insert(
                        (package.identity.clone(), symbol_id, variant.name.clone()),
                        EntryVariantLowering {
                            type_id: type_decl.runtime_type,
                            payload_type: variant.payload_type,
                            default,
                        },
                    );
                }
            }
        }
    }
}

pub(crate) struct RoutineCursor<'a> {
    routine: &'a mut LoweredRoutine,
    block_id: LoweredBlockId,
    next_local_index: usize,
    next_instr_index: usize,
    next_block_index: usize,
    loop_exit_blocks: Vec<LoweredBlockId>,
}

impl<'a> RoutineCursor<'a> {
    pub(crate) fn new(routine: &'a mut LoweredRoutine, block_id: LoweredBlockId) -> Self {
        Self {
            next_local_index: routine.locals.len(),
            next_instr_index: routine.instructions.len(),
            next_block_index: routine.blocks.len(),
            routine,
            block_id,
            loop_exit_blocks: Vec::new(),
        }
    }

    pub(crate) fn current_block_id(&self) -> LoweredBlockId {
        self.block_id
    }

    pub(crate) fn switch_block(&mut self, block_id: LoweredBlockId) -> Result<(), LoweringError> {
        if self.routine.blocks.get(block_id).is_none() {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "lowered routine '{}' lost block {}",
                    self.routine.name, block_id.0
                ),
            ));
        }
        self.block_id = block_id;
        Ok(())
    }

    pub(crate) fn create_block(&mut self) -> LoweredBlockId {
        let block_id = self.routine.blocks.push(crate::LoweredBlock {
            id: LoweredBlockId(self.next_block_index),
            instructions: Vec::new(),
            terminator: None,
        });
        self.next_block_index += 1;
        block_id
    }

    pub(crate) fn push_loop_exit(&mut self, block_id: LoweredBlockId) {
        self.loop_exit_blocks.push(block_id);
    }

    pub(crate) fn pop_loop_exit(&mut self) -> Option<LoweredBlockId> {
        self.loop_exit_blocks.pop()
    }

    pub(crate) fn current_loop_exit(&self) -> Option<LoweredBlockId> {
        self.loop_exit_blocks.last().copied()
    }

    pub(crate) fn current_block_terminated(&self) -> Result<bool, LoweringError> {
        self.routine
            .blocks
            .get(self.block_id)
            .map(|block| block.is_terminated())
            .ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "lowered routine '{}' lost block {}",
                        self.routine.name, self.block_id.0
                    ),
                )
            })
    }

    pub(crate) fn terminate_current_block(
        &mut self,
        terminator: crate::LoweredTerminator,
    ) -> Result<(), LoweringError> {
        let Some(block) = self.routine.blocks.get_mut(self.block_id) else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "lowered routine '{}' lost block {}",
                    self.routine.name, self.block_id.0
                ),
            ));
        };
        if block.terminator.is_some() {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "lowered routine '{}' attempted to terminate block {} twice",
                    self.routine.name, self.block_id.0
                ),
            ));
        }
        block.terminator = Some(terminator);
        Ok(())
    }

    pub(crate) fn allocate_local(
        &mut self,
        type_id: LoweredTypeId,
        name: Option<String>,
    ) -> LoweredLocalId {
        self.allocate_local_with_effect(type_id, None, name)
    }

    pub(crate) fn allocate_local_with_effect(
        &mut self,
        type_id: LoweredTypeId,
        recoverable_error_type: Option<LoweredTypeId>,
        name: Option<String>,
    ) -> LoweredLocalId {
        let local_id = self.routine.locals.push(LoweredLocal {
            id: LoweredLocalId(self.next_local_index),
            type_id: Some(type_id),
            recoverable_error_type,
            name,
        });
        self.next_local_index += 1;
        local_id
    }

    pub(crate) fn push_instr(
        &mut self,
        result: Option<LoweredLocalId>,
        kind: LoweredInstrKind,
    ) -> Result<LoweredInstrId, LoweringError> {
        if self.current_block_terminated()? {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "lowered routine '{}' attempted to append instructions after block {} terminated",
                    self.routine.name, self.block_id.0
                ),
            ));
        }
        let instr_id = self.routine.instructions.push(LoweredInstr {
            id: LoweredInstrId(self.next_instr_index),
            result,
            kind,
        });
        self.next_instr_index += 1;
        let Some(block) = self.routine.blocks.get_mut(self.block_id) else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("lowered routine '{}' lost block {}", self.routine.name, self.block_id.0),
            ));
        };
        block.instructions.push(instr_id);
        Ok(instr_id)
    }

    pub(crate) fn lower_literal(
        &mut self,
        literal: &Literal,
        type_id: LoweredTypeId,
    ) -> Result<LoweredValue, LoweringError> {
        let operand = match literal {
            Literal::Integer(value) => LoweredOperand::Int(*value),
            Literal::Float(value) => LoweredOperand::Float(value.to_bits()),
            Literal::String(value) => LoweredOperand::Str(value.clone()),
            Literal::Character(value) => LoweredOperand::Char(*value),
            Literal::Boolean(value) => LoweredOperand::Bool(*value),
            Literal::Nil => {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::Unsupported,
                    "nil lowering is part of the shell-lowering phase, not the core expression phase",
                ));
            }
        };
        let local_id = self.allocate_local(type_id, None);
        self.push_instr(Some(local_id), LoweredInstrKind::Const(operand))?;
        Ok(LoweredValue {
            local_id,
            type_id,
            recoverable_error_type: None,
        })
    }

    pub(crate) fn lower_identifier_reference(
        &mut self,
        current_identity: &PackageIdentity,
        decl_index: &WorkspaceDeclIndex,
        resolved_symbol: &ResolvedSymbol,
        result_type: LoweredTypeId,
    ) -> Result<LoweredValue, LoweringError> {
        if let Some(local_id) = self.routine.local_symbols.get(&resolved_symbol.id).copied() {
            let recoverable_error_type = self
                .routine
                .locals
                .get(local_id)
                .and_then(|local| local.recoverable_error_type);
            let result_local =
                self.allocate_local_with_effect(result_type, recoverable_error_type, None);
            self.push_instr(
                Some(result_local),
                LoweredInstrKind::LoadLocal { local: local_id },
            )?;
            return Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type,
            });
        }

        let (owning_identity, owning_symbol_id) =
            canonical_symbol_key(current_identity, resolved_symbol.mounted_from.as_ref(), resolved_symbol.id);
        let Some(global_id) = decl_index.global_id_for_symbol(&owning_identity, owning_symbol_id) else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "value symbol '{}' does not map to a lowered local or global definition",
                    resolved_symbol.name
                ),
            ));
        };
        let recoverable_error_type =
            decl_index.global_recoverable_error_type(&owning_identity, global_id);
        let result_local =
            self.allocate_local_with_effect(result_type, recoverable_error_type, None);
        self.push_instr(Some(result_local), LoweredInstrKind::LoadGlobal { global: global_id })?;
        Ok(LoweredValue {
            local_id: result_local,
            type_id: result_type,
            recoverable_error_type,
        })
    }
}

pub(crate) fn canonical_symbol_key(
    current_identity: &PackageIdentity,
    mounted_from: Option<&MountedSymbolProvenance>,
    symbol_id: SymbolId,
) -> (PackageIdentity, SymbolId) {
    mounted_from
        .map(|provenance| {
            (
                provenance.package_identity.clone(),
                provenance.foreign_symbol,
            )
        })
        .unwrap_or_else(|| (current_identity.clone(), symbol_id))
}

pub(crate) fn lower_routine_bodies(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    decl_index: &WorkspaceDeclIndex,
    lowered_package: &mut LoweredPackage,
) -> Result<(), Vec<LoweringError>> {
    let mut errors = Vec::new();

    for (source_unit_index, source_unit) in typed_package.program.resolved().syntax().source_units.iter().enumerate() {
        let source_unit_id = SourceUnitId(source_unit_index);
        for item in &source_unit.items {
            let (name, body) = match &item.node {
                AstNode::FunDecl { name, body, .. }
                | AstNode::ProDecl { name, body, .. }
                | AstNode::LogDecl { name, body, .. } => (name.as_str(), body.as_slice()),
                AstNode::Commented { node, .. } => match node.as_ref() {
                    AstNode::FunDecl { name, body, .. }
                    | AstNode::ProDecl { name, body, .. }
                    | AstNode::LogDecl { name, body, .. } => (name.as_str(), body.as_slice()),
                    _ => continue,
                },
                _ => continue,
            };
            let Some(symbol_id) = crate::decls::find_local_symbol_id(
                &typed_package.program,
                source_unit_id,
                SymbolKind::Routine,
                name,
            ) else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not retain a local lowering symbol"),
                ));
                continue;
            };
            let Some(scope_id) = typed_package
                .program
                .typed_symbol(symbol_id)
                .map(|symbol| symbol.scope_id)
            else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not retain typed scope information"),
                ));
                continue;
            };
            let Some(routine_id) = lowered_package
                .routine_decls
                .iter()
                .find_map(|(routine_id, routine)| {
                    (routine.symbol_id == Some(symbol_id)).then_some(*routine_id)
                })
            else {
                errors.push(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("routine '{name}' does not map to a lowered routine shell"),
                ));
                continue;
            };
            let Some(routine) = lowered_package.routine_decls.get_mut(&routine_id) else {
                continue;
            };
            if let Err(error) = lower_body_nodes(
                typed_package,
                type_table,
                &lowered_package.checked_type_map,
                lowered_package.identity.clone(),
                decl_index,
                routine,
                source_unit_id,
                scope_id,
                body,
            ) {
                errors.push(error);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn lower_body_nodes(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    routine: &mut LoweredRoutine,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    nodes: &[AstNode],
) -> Result<(), LoweringError> {
    let entry_block = routine.entry_block;
    let mut cursor = RoutineCursor::new(routine, entry_block);
    cursor.routine.body_result = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        &current_identity,
        decl_index,
        &mut cursor,
        source_unit_id,
        scope_id,
        nodes,
    )?
    .map(|value| value.local_id);

    Ok(())
}

fn lower_body_sequence(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    nodes: &[AstNode],
) -> Result<Option<LoweredValue>, LoweringError> {
    let mut final_value = None;

    for node in nodes {
        if let Some(value) = lower_body_node(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        )? {
            final_value = Some(value);
        }
        if cursor.current_block_terminated()? {
            break;
        }
    }

    Ok(final_value)
}

fn lower_body_node(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
) -> Result<Option<LoweredValue>, LoweringError> {
    match node {
        AstNode::Comment { .. } => Ok(None),
        AstNode::Commented { node, .. } => lower_body_node(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        ),
        AstNode::VarDecl { name, value, .. } => lower_local_binding(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            name,
            SymbolKind::ValueBinding,
            value.as_deref(),
        ),
        AstNode::LabDecl { name, value, .. } => lower_local_binding(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            name,
            SymbolKind::LabelBinding,
            value.as_deref(),
        ),
        AstNode::Return { value } => match value.as_deref() {
            Some(value) => {
                let lowered = lower_expression_expected(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    routine_return_type(cursor, type_table),
                    value,
                )?;
                cursor.terminate_current_block(crate::LoweredTerminator::Return {
                    value: Some(lowered.local_id),
                })?;
                Ok(None)
            }
            None => {
                cursor.terminate_current_block(crate::LoweredTerminator::Return { value: None })?;
                Ok(None)
            }
        },
        AstNode::FunctionCall { name, args, .. } if name == "report" => {
            let lowered = match args.as_slice() {
                [value] => Some(
                    lower_expression_expected(
                        typed_package,
                        type_table,
                        checked_type_map,
                        current_identity,
                        decl_index,
                        cursor,
                        source_unit_id,
                        scope_id,
                        routine_error_type(cursor, type_table),
                        value,
                    )?
                    .local_id,
                ),
                [] => None,
                _ => {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("report expects exactly 1 value in lowered V1 bodies, got {}", args.len()),
                    ))
                }
            };
            cursor.terminate_current_block(crate::LoweredTerminator::Report { value: lowered })?;
            Ok(None)
        }
        AstNode::When {
            expr,
            cases,
            default,
        } if default.is_none() || when_always_terminates(cases, default.as_deref()) => {
            lower_when_statement(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                expr,
                cases,
                default.as_deref(),
            )?;
            Ok(None)
        }
        AstNode::Loop { condition, body } => {
            lower_loop_statement(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                condition,
                body,
            )?;
            Ok(None)
        }
        AstNode::Break => {
            let Some(exit_block) = cursor.current_loop_exit() else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "break lowering requires an active loop exit block",
                ));
            };
            cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: exit_block })?;
            Ok(None)
        }
        AstNode::Yield { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "yield lowering is not part of the current V1 lowering milestone",
        )),
        _ => lower_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            node,
        )
        .map(Some),
    }
}

fn lower_local_binding(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    name: &str,
    kind: SymbolKind,
    value: Option<&AstNode>,
) -> Result<Option<LoweredValue>, LoweringError> {
    let Some(symbol_id) = crate::decls::find_symbol_in_scope_or_descendants(
        &typed_package.program,
        source_unit_id,
        scope_id,
        kind,
        name,
    ) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("binding '{name}' does not retain an exact lowering symbol in scope"),
        ));
    };
    let type_id = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|symbol| symbol.declared_type)
        .and_then(|checked_type| checked_type_map.get(&checked_type).copied())
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("binding '{name}' does not retain a lowered storage type"),
            )
        })?;
    let recoverable_error_type = typed_package
        .program
        .typed_symbol(symbol_id)
        .and_then(|symbol| symbol.recoverable_effect)
        .and_then(|effect| checked_type_map.get(&effect.error_type).copied());
    let local_id =
        cursor.allocate_local_with_effect(type_id, recoverable_error_type, Some(name.to_string()));
    cursor.routine.local_symbols.insert(symbol_id, local_id);

    if let Some(value) = value {
        let lowered_value = if recoverable_error_type.is_some() {
            lower_expression_observed(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                Some(type_id),
                value,
            )?
        } else {
            lower_expression_expected(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                Some(type_id),
                value,
            )?
        };
        cursor.push_instr(
            None,
            LoweredInstrKind::StoreLocal {
                local: local_id,
                value: lowered_value.local_id,
            },
        )?;
        Ok(Some(LoweredValue {
            local_id,
            type_id,
            recoverable_error_type,
        }))
    } else {
        Ok(Some(LoweredValue {
            local_id,
            type_id,
            recoverable_error_type,
        }))
    }
}

fn lower_record_initializer(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    fields: &[fol_parser::ast::RecordInitField],
) -> Result<LoweredValue, LoweringError> {
    let Some(type_id) = expected_type else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "record initializer lowering requires an expected record type in V1",
        ));
    };
    let Some(crate::LoweredType::Record { fields: expected_fields }) = type_table.get(type_id) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "record initializer does not map to a lowered record runtime type",
        ));
    };

    let mut lowered_fields = Vec::with_capacity(fields.len());
    for field in fields {
        let Some(field_type) = expected_fields.get(&field.name).copied() else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!(
                    "record initializer field '{}' does not map to a lowered record layout",
                    field.name
                ),
            ));
        };
        let lowered_value = lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            Some(field_type),
            &field.value,
        )?;
        lowered_fields.push((field.name.clone(), lowered_value.local_id));
    }

    let result_local = cursor.allocate_local(type_id, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::ConstructRecord {
            type_id,
            fields: lowered_fields,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id,
        recoverable_error_type: None,
    })
}

fn lower_nil_literal(
    type_table: &crate::LoweredTypeTable,
    cursor: &mut RoutineCursor<'_>,
    expected_type: Option<LoweredTypeId>,
) -> Result<LoweredValue, LoweringError> {
    let Some(type_id) = expected_type else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "nil lowering requires an expected opt[...] or err[...] runtime type in lowered V1",
        ));
    };
    let result_local = cursor.allocate_local(type_id, None);
    match type_table.get(type_id) {
        Some(crate::LoweredType::Optional { .. }) => {
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::ConstructOptional {
                    type_id,
                    value: None,
                },
            )?;
        }
        Some(crate::LoweredType::Error { .. }) => {
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::ConstructError {
                    type_id,
                    value: None,
                },
            )?;
        }
        _ => {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::Unsupported,
                "nil lowering requires an expected opt[...] or err[...] runtime type in lowered V1",
            ))
        }
    }
    Ok(LoweredValue {
        local_id: result_local,
        type_id,
        recoverable_error_type: None,
    })
}

fn apply_expected_shell_wrap(
    type_table: &crate::LoweredTypeTable,
    cursor: &mut RoutineCursor<'_>,
    expected_type: Option<LoweredTypeId>,
    value: LoweredValue,
) -> Result<LoweredValue, LoweringError> {
    let Some(expected_type) = expected_type else {
        return Ok(value);
    };
    if expected_type == value.type_id {
        return Ok(value);
    }
    match type_table.get(expected_type) {
        Some(crate::LoweredType::Optional { inner }) if *inner == value.type_id => {
            let result_local = cursor.allocate_local(expected_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::ConstructOptional {
                    type_id: expected_type,
                    value: Some(value.local_id),
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: expected_type,
                recoverable_error_type: None,
            })
        }
        Some(crate::LoweredType::Error { inner: Some(inner) }) if *inner == value.type_id => {
            let result_local = cursor.allocate_local(expected_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::ConstructError {
                    type_id: expected_type,
                    value: Some(value.local_id),
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: expected_type,
                recoverable_error_type: None,
            })
        }
        _ => Ok(value),
    }
}

fn lower_unwrap_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    operand: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let operand = lower_expression(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        operand,
    )?;
    let inner_type = match type_table.get(operand.type_id) {
        Some(crate::LoweredType::Optional { inner }) => Some(*inner),
        Some(crate::LoweredType::Error { inner }) => *inner,
        _ => None,
    }
    .ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "unwrap lowering requires an opt[...] or typed err[...] runtime operand in lowered V1",
        )
    })?;
    let result_local = cursor.allocate_local(inner_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::UnwrapShell {
            operand: operand.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: inner_type,
        recoverable_error_type: None,
    })
}

fn lower_entry_variant_access(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    object: &AstNode,
    field: &str,
    expected_type: Option<LoweredTypeId>,
) -> Result<Option<LoweredValue>, LoweringError> {
    let Some((owning_identity, owning_symbol_id, variant)) =
        resolve_entry_variant_target(
            typed_package,
            type_table,
            current_identity,
            object,
            field,
            checked_type_map,
        )?
    else {
        return Ok(None);
    };
    let Some(entry_variant) = decl_index.entry_variant(&owning_identity, owning_symbol_id, &variant) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("entry variant '{variant}' does not retain lowered variant metadata"),
        ));
    };

    if expected_type == Some(entry_variant.type_id) {
        let payload = match (&entry_variant.payload_type, &entry_variant.default) {
            (Some(payload_type), Some(default)) => Some(
                lower_expression_expected(
                    typed_package,
                    type_table,
                    checked_type_map,
                    current_identity,
                    decl_index,
                    cursor,
                    source_unit_id,
                    scope_id,
                    Some(*payload_type),
                    default,
                )?
                .local_id,
            ),
            (Some(_), None) => {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::Unsupported,
                    format!(
                        "entry construction for variant '{variant}' requires a lowered default payload expression"
                    ),
                ))
            }
            (None, _) => None,
        };
        let result_local = cursor.allocate_local(entry_variant.type_id, None);
        cursor.push_instr(
            Some(result_local),
            LoweredInstrKind::ConstructEntry {
                type_id: entry_variant.type_id,
                variant,
                payload,
            },
        )?;
        return Ok(Some(LoweredValue {
            local_id: result_local,
            type_id: entry_variant.type_id,
            recoverable_error_type: None,
        }));
    }

    let Some(payload_type) = entry_variant.payload_type else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "entry variant access for '{variant}' requires a payload-bearing variant or an expected entry context"
            ),
        ));
    };
    let Some(default) = entry_variant.default.as_ref() else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "entry variant access for '{variant}' requires a lowered default payload expression"
            ),
        ));
    };
    let payload = lower_expression_expected(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        Some(payload_type),
        default,
    )?;
    Ok(Some(payload))
}

fn lower_container_literal(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    container_type: ContainerType,
    expected_type: Option<LoweredTypeId>,
    elements: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let container_kind = expected_container_kind(type_table, expected_type).unwrap_or(container_type);
    match container_kind {
        ContainerType::Array | ContainerType::Vector | ContainerType::Sequence => {
            lower_linear_container_literal(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                container_kind,
                expected_type,
                elements,
            )
        }
        ContainerType::Set => lower_set_literal(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_type,
            elements,
        ),
        ContainerType::Map => lower_map_literal(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_type,
            elements,
        ),
    }
}

fn lower_linear_container_literal(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    kind: ContainerType,
    expected_type: Option<LoweredTypeId>,
    elements: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let element_nodes = container_elements(elements);
    let mut lowered_elements = Vec::with_capacity(element_nodes.len());
    let mut element_type = expected_linear_element_type(type_table, expected_type, kind.clone());

    for element in element_nodes {
        let lowered = lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            element_type,
            element,
        )?;
        element_type.get_or_insert(lowered.type_id);
        lowered_elements.push(lowered.local_id);
    }

    let Some(type_id) = resolve_linear_container_type(
        type_table,
        kind.clone(),
        expected_type,
        element_type,
        lowered_elements.len(),
    ) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "empty linear container literals require an expected container type in lowered V1",
        ));
    };

    let result_local = cursor.allocate_local(type_id, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::ConstructLinear {
            kind: lowered_linear_kind(kind)?,
            type_id,
            elements: lowered_elements,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id,
        recoverable_error_type: None,
    })
}

fn lower_set_literal(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    elements: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let element_nodes = container_elements(elements);
    let expected_members = expected_set_member_types(type_table, expected_type);

    if element_nodes.is_empty() {
        let Some(type_id) = expected_type else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::Unsupported,
                "empty set literals require an expected set type in lowered V1",
            ));
        };
        let result_local = cursor.allocate_local(type_id, None);
        cursor.push_instr(
            Some(result_local),
            LoweredInstrKind::ConstructSet {
                type_id,
                members: Vec::new(),
            },
        )?;
        return Ok(LoweredValue {
            local_id: result_local,
            type_id,
            recoverable_error_type: None,
        });
    }

    let mut member_types = Vec::with_capacity(element_nodes.len());
    let mut members = Vec::with_capacity(element_nodes.len());
    for (index, element) in element_nodes.iter().enumerate() {
        let expected_member = expected_members
            .as_ref()
            .and_then(|member_types| member_types.get(index))
            .copied();
        let lowered = lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_member,
            element,
        )?;
        member_types.push(expected_member.unwrap_or(lowered.type_id));
        members.push(lowered.local_id);
    }

    let type_id = expected_type.unwrap_or_else(|| find_set_type(type_table, &member_types));
    let result_local = cursor.allocate_local(type_id, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::ConstructSet { type_id, members },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id,
        recoverable_error_type: None,
    })
}

fn lower_map_literal(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    elements: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let element_nodes = container_elements(elements);
    let mut expected_key = expected_map_key_type(type_table, expected_type);
    let mut expected_value = expected_map_value_type(type_table, expected_type);

    if element_nodes.is_empty() {
        let Some(type_id) = expected_type else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::Unsupported,
                "empty map literals require an expected map type in lowered V1",
            ));
        };
        let result_local = cursor.allocate_local(type_id, None);
        cursor.push_instr(
            Some(result_local),
            LoweredInstrKind::ConstructMap {
                type_id,
                entries: Vec::new(),
            },
        )?;
        return Ok(LoweredValue {
            local_id: result_local,
            type_id,
            recoverable_error_type: None,
        });
    }

    let mut entries = Vec::with_capacity(element_nodes.len());
    for pair in element_nodes {
        let (key, value) = map_literal_pair(pair)?;
        let lowered_key = lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_key,
            key,
        )?;
        expected_key.get_or_insert(lowered_key.type_id);

        let lowered_value = lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_value,
            value,
        )?;
        expected_value.get_or_insert(lowered_value.type_id);
        entries.push((lowered_key.local_id, lowered_value.local_id));
    }

    let Some(type_id) = resolve_map_type(type_table, expected_type, expected_key, expected_value) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "map literal could not determine a lowered key/value type",
        ));
    };

    let result_local = cursor.allocate_local(type_id, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::ConstructMap { type_id, entries },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id,
        recoverable_error_type: None,
    })
}

fn when_case_body(case: &fol_parser::ast::WhenCase) -> &[AstNode] {
    match case {
        fol_parser::ast::WhenCase::Case { body, .. }
        | fol_parser::ast::WhenCase::Is { body, .. }
        | fol_parser::ast::WhenCase::In { body, .. }
        | fol_parser::ast::WhenCase::Has { body, .. }
        | fol_parser::ast::WhenCase::On { body, .. }
        | fol_parser::ast::WhenCase::Of { body, .. } => body.as_slice(),
    }
}

fn when_always_terminates(
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> bool {
    let Some(default) = default else {
        return false;
    };
    !cases.is_empty()
        && cases.iter().all(|case| body_always_terminates(when_case_body(case)))
        && body_always_terminates(default)
}

fn body_always_terminates(nodes: &[AstNode]) -> bool {
    nodes.iter()
        .rev()
        .find(|node| !matches!(node, AstNode::Comment { .. }))
        .is_some_and(node_always_terminates)
}

fn node_always_terminates(node: &AstNode) -> bool {
    match node {
        AstNode::Comment { .. } => false,
        AstNode::Commented { node, .. } => node_always_terminates(node),
        AstNode::Return { .. } => true,
        AstNode::FunctionCall { name, .. } if name == "report" => true,
        AstNode::Block { statements } => body_always_terminates(statements),
        AstNode::When { cases, default, .. } => when_always_terminates(cases, default.as_deref()),
        _ => false,
    }
}

fn lower_when_statement(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expr: &AstNode,
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> Result<(), LoweringError> {
    let _ = lower_expression(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expr,
    )?;

    let mut after_block = None;
    let mut has_fallthrough = false;

    for (index, case) in cases.iter().enumerate() {
        let (condition, body) = when_case_condition_and_body(case)?;
        let lowered_condition = lower_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            condition,
        )?;
        let body_block = cursor.create_block();
        let else_block = if index + 1 < cases.len() || default.is_some() {
            cursor.create_block()
        } else {
            ensure_after_block(cursor, &mut after_block)
        };
        cursor.terminate_current_block(crate::LoweredTerminator::Branch {
            condition: lowered_condition.local_id,
            then_block: body_block,
            else_block,
        })?;

        cursor.switch_block(body_block)?;
        let _ = lower_body_sequence(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            body,
        )?;
        if !cursor.current_block_terminated()? {
            let after_block = ensure_after_block(cursor, &mut after_block);
            cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: after_block })?;
            has_fallthrough = true;
        }

        if Some(else_block) != after_block {
            cursor.switch_block(else_block)?;
        }
    }

    if let Some(default) = default {
        has_fallthrough |= lower_default_when_body(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            default,
            &mut after_block,
        )?;
    }

    if let Some(after_block) = after_block.filter(|_| has_fallthrough) {
        cursor.switch_block(after_block)?;
    }

    Ok(())
}

fn lower_loop_statement(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    condition: &LoopCondition,
    body: &[AstNode],
) -> Result<(), LoweringError> {
    match condition {
        LoopCondition::Condition(condition) => {
            let header_block = cursor.create_block();
            let body_block = cursor.create_block();
            let exit_block = cursor.create_block();

            cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                target: header_block,
            })?;

            cursor.switch_block(header_block)?;
            let lowered_condition = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                condition,
            )?;
            cursor.terminate_current_block(crate::LoweredTerminator::Branch {
                condition: lowered_condition.local_id,
                then_block: body_block,
                else_block: exit_block,
            })?;

            cursor.switch_block(body_block)?;
            cursor.push_loop_exit(exit_block);
            let _ = lower_body_sequence(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                body,
            )?;
            cursor.pop_loop_exit();
            if !cursor.current_block_terminated()? {
                cursor.terminate_current_block(crate::LoweredTerminator::Jump {
                    target: header_block,
                })?;
            }

            cursor.switch_block(exit_block)?;
            Ok(())
        }
        LoopCondition::Iteration { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "iteration loop lowering is not part of the current lowered V1 control-flow milestone",
        )),
    }
}

fn lower_default_when_body(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    default: &[AstNode],
    after_block: &mut Option<LoweredBlockId>,
) -> Result<bool, LoweringError> {
    let _ = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        default,
    )?;
    if !cursor.current_block_terminated()? {
        let after_block = ensure_after_block(cursor, after_block);
        cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: after_block })?;
        return Ok(true);
    }
    Ok(false)
}

fn ensure_after_block(
    cursor: &mut RoutineCursor<'_>,
    after_block: &mut Option<LoweredBlockId>,
) -> LoweredBlockId {
    *after_block.get_or_insert_with(|| cursor.create_block())
}

fn lower_when_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expr: &AstNode,
    cases: &[fol_parser::ast::WhenCase],
    default: Option<&[AstNode]>,
) -> Result<LoweredValue, LoweringError> {
    let Some(default) = default else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "value-producing when expressions require a default branch in lowered V1",
        ));
    };

    let _ = lower_expression(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expr,
    )?;

    let join_block = cursor.create_block();
    let mut join_local = None;

    for (index, case) in cases.iter().enumerate() {
        let (condition, body) = when_case_condition_and_body(case)?;
        let lowered_condition = lower_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            condition,
        )?;
        let body_block = cursor.create_block();
        let else_block = if index + 1 < cases.len() || !default.is_empty() {
            cursor.create_block()
        } else {
            join_block
        };
        cursor.terminate_current_block(crate::LoweredTerminator::Branch {
            condition: lowered_condition.local_id,
            then_block: body_block,
            else_block,
        })?;

        cursor.switch_block(body_block)?;
        let branch_value = lower_body_sequence(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            body,
        )?;
        lower_when_branch_value(cursor, &mut join_local, branch_value, join_block)?;

        if else_block != join_block {
            cursor.switch_block(else_block)?;
        }
    }

    let default_value = lower_body_sequence(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        default,
    )?;
    lower_when_branch_value(cursor, &mut join_local, default_value, join_block)?;

    cursor.switch_block(join_block)?;
    join_local.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "value-producing when did not retain a lowered join value",
        )
    })
}

fn lower_when_branch_value(
    cursor: &mut RoutineCursor<'_>,
    join_local: &mut Option<LoweredValue>,
    branch_value: Option<LoweredValue>,
    join_block: LoweredBlockId,
) -> Result<(), LoweringError> {
    match branch_value {
        Some(branch_value) => {
            let destination = if let Some(existing) = join_local {
                if existing.type_id != branch_value.type_id {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        "value-producing when branches do not agree on one lowered join type",
                    ));
                }
                *existing
            } else {
                let local_id = cursor.allocate_local(branch_value.type_id, None);
                let value = LoweredValue {
                    local_id,
                    type_id: branch_value.type_id,
                    recoverable_error_type: None,
                };
                *join_local = Some(value);
                value
            };
            cursor.push_instr(
                None,
                LoweredInstrKind::StoreLocal {
                    local: destination.local_id,
                    value: branch_value.local_id,
                },
            )?;
            if !cursor.current_block_terminated()? {
                cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: join_block })?;
            }
            Ok(())
        }
        None if cursor.current_block_terminated()? => Ok(()),
        None => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "value-producing when branches must yield a value or terminate early",
        )),
    }
}

fn when_case_condition_and_body(
    case: &fol_parser::ast::WhenCase,
) -> Result<(&AstNode, &[AstNode]), LoweringError> {
    match case {
        fol_parser::ast::WhenCase::Case { condition, body }
        | fol_parser::ast::WhenCase::Is {
            value: condition,
            body,
        }
        | fol_parser::ast::WhenCase::In {
            range: condition,
            body,
        }
        | fol_parser::ast::WhenCase::Has {
            member: condition,
            body,
        }
        | fol_parser::ast::WhenCase::On {
            channel: condition,
            body,
        } => Ok((condition, body.as_slice())),
        fol_parser::ast::WhenCase::Of { .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            "type-matching when/of branches are not lowered in this slice yet",
        )),
    }
}

fn lower_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    lower_expression_expected(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        None,
        node,
    )
}

fn materialize_recoverable_value(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    lowered: LoweredValue,
) -> Result<LoweredValue, LoweringError> {
    let Some(error_type) = lowered.recoverable_error_type else {
        return Ok(lowered);
    };
    let Some(routine_error_type) = routine_error_type(cursor, type_table) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "recoverable value reached a plain lowering context outside an error-aware routine",
        ));
    };
    if routine_error_type != error_type {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "recoverable value with lowered error type {} cannot propagate through routine error type {}",
                error_type.0, routine_error_type.0
            ),
        ));
    }

    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while materializing a recoverable value",
            )
        })?;
    let condition_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(condition_local),
        LoweredInstrKind::CheckRecoverable {
            operand: lowered.local_id,
        },
    )?;

    let error_block = cursor.create_block();
    let success_block = cursor.create_block();
    cursor.terminate_current_block(crate::LoweredTerminator::Branch {
        condition: condition_local,
        then_block: error_block,
        else_block: success_block,
    })?;

    cursor.switch_block(error_block)?;
    let error_local = cursor.allocate_local(error_type, None);
    cursor.push_instr(
        Some(error_local),
        LoweredInstrKind::ExtractRecoverableError {
            operand: lowered.local_id,
        },
    )?;
    cursor.terminate_current_block(crate::LoweredTerminator::Report {
        value: Some(error_local),
    })?;

    cursor.switch_block(success_block)?;
    let success_local = cursor.allocate_local(lowered.type_id, None);
    cursor.push_instr(
        Some(success_local),
        LoweredInstrKind::UnwrapRecoverable {
            operand: lowered.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: success_local,
        type_id: lowered.type_id,
        recoverable_error_type: None,
    })
}

fn lower_check_call(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    _syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let [operand] = args else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("check expects exactly 1 value in lowered V1, got {}", args.len()),
        ));
    };
    let observed = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        None,
        operand,
    )?;
    if observed.recoverable_error_type.is_none() {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "check lowering requires a recoverable routine result operand in V1",
        ));
    }
    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while lowering check(...)",
            )
        })?;
    let result_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::CheckRecoverable {
            operand: observed.local_id,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: bool_type,
        recoverable_error_type: None,
    })
}

fn lower_pipe_or_expression(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    left: &AstNode,
    right: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let observed_left = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        None,
        left,
    )?;
    if observed_left.recoverable_error_type.is_none() {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "'||' lowering requires a recoverable expression on the left in V1",
        ));
    }
    let bool_type = checked_type_map
        .get(&typed_package.program.builtin_types().bool_)
        .copied()
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "lowered workspace lost builtin bool while lowering '||'",
            )
        })?;
    let condition_local = cursor.allocate_local(bool_type, None);
    cursor.push_instr(
        Some(condition_local),
        LoweredInstrKind::CheckRecoverable {
            operand: observed_left.local_id,
        },
    )?;

    let error_block = cursor.create_block();
    let success_block = cursor.create_block();
    let join_block = cursor.create_block();
    let result_local = cursor.allocate_local(observed_left.type_id, None);

    cursor.terminate_current_block(crate::LoweredTerminator::Branch {
        condition: condition_local,
        then_block: error_block,
        else_block: success_block,
    })?;

    cursor.switch_block(success_block)?;
    let success_value = cursor.allocate_local(observed_left.type_id, None);
    cursor.push_instr(
        Some(success_value),
        LoweredInstrKind::UnwrapRecoverable {
            operand: observed_left.local_id,
        },
    )?;
    cursor.push_instr(
        None,
        LoweredInstrKind::StoreLocal {
            local: result_local,
            value: success_value,
        },
    )?;
    cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: join_block })?;

    cursor.switch_block(error_block)?;
    let fallback_value = lower_pipe_or_fallback(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        observed_left.type_id,
        right,
    )?;
    if let Some(fallback_value) = fallback_value {
        cursor.push_instr(
            None,
            LoweredInstrKind::StoreLocal {
                local: result_local,
                value: fallback_value.local_id,
            },
        )?;
        cursor.terminate_current_block(crate::LoweredTerminator::Jump { target: join_block })?;
    }

    cursor.switch_block(join_block)?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: observed_left.type_id,
        recoverable_error_type: None,
    })
}

fn lower_pipe_or_fallback(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: LoweredTypeId,
    right: &AstNode,
) -> Result<Option<LoweredValue>, LoweringError> {
    match right {
        AstNode::FunctionCall { name, args, .. } if name == "report" => {
            let lowered = match args.as_slice() {
                [value] => Some(
                    lower_expression_expected(
                        typed_package,
                        type_table,
                        checked_type_map,
                        current_identity,
                        decl_index,
                        cursor,
                        source_unit_id,
                        scope_id,
                        routine_error_type(cursor, type_table),
                        value,
                    )?
                    .local_id,
                ),
                [] => None,
                _ => {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("report expects exactly 1 value in lowered V1 bodies, got {}", args.len()),
                    ))
                }
            };
            cursor.terminate_current_block(crate::LoweredTerminator::Report { value: lowered })?;
            Ok(None)
        }
        AstNode::FunctionCall { name, args, .. } if name == "panic" => {
            let lowered = match args.as_slice() {
                [value] => Some(
                    lower_expression_expected(
                        typed_package,
                        type_table,
                        checked_type_map,
                        current_identity,
                        decl_index,
                        cursor,
                        source_unit_id,
                        scope_id,
                        None,
                        value,
                    )?
                    .local_id,
                ),
                [] => None,
                _ => {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("panic expects at most 1 value in lowered V1, got {}", args.len()),
                    ))
                }
            };
            cursor.terminate_current_block(crate::LoweredTerminator::Panic { value: lowered })?;
            Ok(None)
        }
        AstNode::Return { .. } => {
            let _ = lower_body_node(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                right,
            )?;
            Ok(None)
        }
        _ => lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            Some(expected_type),
            right,
        )
        .map(Some),
    }
}

fn lower_expression_expected(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let lowered = lower_expression_observed(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        expected_type,
        node,
    )?;
    let lowered = materialize_recoverable_value(
        typed_package,
        type_table,
        checked_type_map,
        current_identity,
        decl_index,
        cursor,
        source_unit_id,
        scope_id,
        lowered,
    )?;
    apply_expected_shell_wrap(type_table, cursor, expected_type, lowered)
}

fn lower_expression_observed(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    expected_type: Option<LoweredTypeId>,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    let lowered = match node {
        AstNode::Literal(Literal::Nil) => lower_nil_literal(type_table, cursor, expected_type),
        AstNode::Literal(literal) => {
            let type_id = literal_type_id(typed_package, checked_type_map, literal).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "literal expression does not retain a lowering-owned type",
                )
            })?;
            cursor.lower_literal(literal, type_id)
        }
        AstNode::UnaryOp {
            op: fol_parser::ast::UnaryOperator::Unwrap,
            operand,
        } => lower_unwrap_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            operand,
        ),
        AstNode::UnaryOp { op, .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "unary operator lowering for '{}' lands in a later lowering slice",
                describe_unary_operator(op)
            ),
        )),
        AstNode::BinaryOp {
            op: fol_parser::ast::BinaryOperator::PipeOr,
            left,
            right,
        } => lower_pipe_or_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            left,
            right,
        ),
        AstNode::BinaryOp { op, .. } => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "binary operator lowering for '{}' lands in a later lowering slice",
                describe_binary_operator(op)
            ),
        )),
        AstNode::RecordInit { fields, .. } => {
            lower_record_initializer(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                expected_type,
                fields,
            )
        }
        AstNode::ContainerLiteral {
            container_type,
            elements,
        } => lower_container_literal(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            container_type.clone(),
            expected_type,
            elements,
        ),
        AstNode::Assignment { target, value } => {
            let lowered_value = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                value,
            )?;
            lower_assignment_target(
                typed_package,
                current_identity,
                decl_index,
                cursor,
                target,
                lowered_value,
            )
        }
        AstNode::FunctionCall { syntax_id, name, args } if name == "check" => lower_check_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            *syntax_id,
            args,
        ),
        AstNode::FunctionCall { syntax_id, name, args } => lower_function_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            *syntax_id,
            fol_resolver::ReferenceKind::FunctionCall,
            name,
            args,
        ),
        AstNode::QualifiedFunctionCall { path, args } => lower_function_call(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            path.syntax_id(),
            fol_resolver::ReferenceKind::QualifiedFunctionCall,
            &path.joined(),
            args,
        ),
        AstNode::MethodCall { object, method, args } => {
            let receiver = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
            )?;
            let (callee, result_type, error_type) = resolve_method_target(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                method,
                receiver.type_id,
            )?;
            let mut lowered_args = vec![receiver.local_id];
            let param_types = decl_index
                .routine_param_types(callee)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("method '{method}' does not retain lowered parameter types"),
                    )
                })?
                .to_vec();
            lowered_args.extend(
                args.iter()
                    .enumerate()
                    .map(|(index, arg)| {
                        let expected = param_types.get(index + 1).copied();
                        lower_expression_expected(
                            typed_package,
                            type_table,
                            checked_type_map,
                            current_identity,
                            decl_index,
                            cursor,
                            source_unit_id,
                            scope_id,
                            expected,
                            arg,
                        )
                        .map(|value| value.local_id)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let result_local = cursor.allocate_local_with_effect(result_type, error_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::Call {
                    callee,
                    args: lowered_args,
                    error_type,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: error_type,
            })
        }
        AstNode::FieldAccess { object, field } => {
            if let Some(entry_value) = lower_entry_variant_access(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
                field,
                expected_type,
            )? {
                return apply_expected_shell_wrap(type_table, cursor, expected_type, entry_value);
            }
            let base = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                object,
            )?;
            if let Some(expected_type) = expected_type {
                if base.type_id == expected_type
                    && matches!(type_table.get(base.type_id), Some(crate::LoweredType::Entry { variants }) if variants.contains_key(field))
                {
                    return Err(LoweringError::with_kind(
                        LoweringErrorKind::Unsupported,
                        format!(
                            "entry construction lowering for variant '{}' lands in the pending aggregate slice",
                            field
                        ),
                    ));
                }
            }
            let Some(result_type) = field_access_type(type_table, base.type_id, field) else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("field access '.{field}' does not map to a lowered record field"),
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::FieldAccess {
                    base: base.local_id,
                    field: field.clone(),
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            })
        }
        AstNode::IndexAccess { container, index } => {
            let lowered_container = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                container,
            )?;
            let lowered_index = lower_expression(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                index,
            )?;
            let Some(result_type) = index_access_type(type_table, lowered_container.type_id, index) else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "index access does not map to a lowered container element type",
                ));
            };
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::IndexAccess {
                    container: lowered_container.local_id,
                    index: lowered_index.local_id,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
                recoverable_error_type: None,
            })
        }
        AstNode::When {
            expr,
            cases,
            default,
        } => lower_when_expression(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expr,
            cases,
            default.as_deref(),
        ),
        AstNode::Identifier { syntax_id, name } => {
            let syntax_id = syntax_id.ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a syntax id"),
                )
            })?;
            let Some(reference) = typed_package.program.resolved().references.iter().find(|reference| {
                reference.syntax_id == Some(syntax_id)
                    && reference.kind == fol_resolver::ReferenceKind::Identifier
            }) else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' is missing from resolver output"),
                ));
            };
            let Some(symbol_id) = reference.resolved else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not resolve to a lowered symbol"),
                ));
            };
            let resolved_symbol = typed_package
                .program
                .resolved()
                .symbol(symbol_id)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("identifier '{name}' lost its resolved symbol"),
                    )
                })?;
            let result_type = reference_type_id(typed_package, reference.id).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a lowered reference type"),
                )
            })?;
            let result_type = checked_type_map.get(&result_type).copied().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("identifier '{name}' does not retain a lowered reference type"),
                )
            })?;
            cursor.lower_identifier_reference(current_identity, decl_index, resolved_symbol, result_type)
        }
        AstNode::QualifiedIdentifier { path } => {
            let syntax_id = path.syntax_id().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("qualified identifier '{}' does not retain a syntax id", path.joined()),
                )
            })?;
            let Some(reference) = typed_package.program.resolved().references.iter().find(|reference| {
                reference.syntax_id == Some(syntax_id)
                    && reference.kind == fol_resolver::ReferenceKind::QualifiedIdentifier
            }) else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("qualified identifier '{}' is missing from resolver output", path.joined()),
                ));
            };
            let Some(symbol_id) = reference.resolved else {
                return Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!("qualified identifier '{}' does not resolve to a lowered symbol", path.joined()),
                ));
            };
            let resolved_symbol = typed_package
                .program
                .resolved()
                .symbol(symbol_id)
                .ok_or_else(|| {
                    LoweringError::with_kind(
                        LoweringErrorKind::InvalidInput,
                        format!("qualified identifier '{}' lost its resolved symbol", path.joined()),
                    )
                })?;
            let result_type = reference_type_id(typed_package, reference.id).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not retain a lowered reference type",
                        path.joined()
                    ),
                )
            })?;
            let result_type = checked_type_map.get(&result_type).copied().ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    format!(
                        "qualified identifier '{}' does not retain a lowered reference type",
                        path.joined()
                    ),
                )
            })?;
            cursor.lower_identifier_reference(current_identity, decl_index, resolved_symbol, result_type)
        }
        AstNode::Commented { node, .. } => lower_expression_expected(
            typed_package,
            type_table,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            scope_id,
            expected_type,
            node,
        ),
        other => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "expression lowering for '{}' is not implemented in this slice yet",
                describe_expression(other)
            ),
        )),
    }?;
    apply_expected_shell_wrap(type_table, cursor, expected_type, lowered)
}

fn literal_type_id(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    literal: &Literal,
) -> Option<LoweredTypeId> {
    let builtin = match literal {
        Literal::Integer(_) => typed_package.program.builtin_types().int,
        Literal::Float(_) => typed_package.program.builtin_types().float,
        Literal::String(_) => typed_package.program.builtin_types().str_,
        Literal::Character(_) => typed_package.program.builtin_types().char_,
        Literal::Boolean(_) => typed_package.program.builtin_types().bool_,
        Literal::Nil => return None,
    };
    checked_type_map.get(&builtin).copied()
}

fn reference_type_id(
    typed_package: &fol_typecheck::TypedPackage,
    reference_id: fol_resolver::ReferenceId,
) -> Option<fol_typecheck::CheckedTypeId> {
    typed_package
        .program
        .typed_reference(reference_id)?
        .resolved_type
}

fn describe_expression(node: &AstNode) -> String {
    match node {
        AstNode::Assignment { .. } => "assignment".to_string(),
        AstNode::FunctionCall { name, .. } => format!("function call '{name}'"),
        AstNode::QualifiedFunctionCall { path, .. } => format!("qualified function call '{}'", path.joined()),
        AstNode::MethodCall { method, .. } => format!("method call '{method}'"),
        AstNode::FieldAccess { field, .. } => format!("field access '.{field}'"),
        AstNode::IndexAccess { .. } => "index access".to_string(),
        AstNode::ContainerLiteral { container_type, .. } => match container_type {
            ContainerType::Array => "array literal".to_string(),
            ContainerType::Vector => "vector literal".to_string(),
            ContainerType::Sequence => "sequence literal".to_string(),
            ContainerType::Set => "set literal".to_string(),
            ContainerType::Map => "map literal".to_string(),
        },
        AstNode::Return { .. } => "return".to_string(),
        AstNode::When { .. } => "when".to_string(),
        AstNode::Loop { .. } => "loop".to_string(),
        _ => "expression".to_string(),
    }
}

fn describe_unary_operator(op: &fol_parser::ast::UnaryOperator) -> &'static str {
    match op {
        fol_parser::ast::UnaryOperator::Neg => "neg",
        fol_parser::ast::UnaryOperator::Not => "not",
        fol_parser::ast::UnaryOperator::Ref => "ref",
        fol_parser::ast::UnaryOperator::Deref => "deref",
        fol_parser::ast::UnaryOperator::Unwrap => "unwrap",
    }
}

fn describe_binary_operator(op: &fol_parser::ast::BinaryOperator) -> &'static str {
    match op {
        fol_parser::ast::BinaryOperator::Add => "add",
        fol_parser::ast::BinaryOperator::Sub => "sub",
        fol_parser::ast::BinaryOperator::Mul => "mul",
        fol_parser::ast::BinaryOperator::Div => "div",
        fol_parser::ast::BinaryOperator::Mod => "mod",
        fol_parser::ast::BinaryOperator::Pow => "pow",
        fol_parser::ast::BinaryOperator::Eq => "eq",
        fol_parser::ast::BinaryOperator::Ne => "ne",
        fol_parser::ast::BinaryOperator::Lt => "lt",
        fol_parser::ast::BinaryOperator::Le => "le",
        fol_parser::ast::BinaryOperator::Gt => "gt",
        fol_parser::ast::BinaryOperator::Ge => "ge",
        fol_parser::ast::BinaryOperator::And => "and",
        fol_parser::ast::BinaryOperator::Or => "or",
        fol_parser::ast::BinaryOperator::Xor => "xor",
        fol_parser::ast::BinaryOperator::In => "in",
        fol_parser::ast::BinaryOperator::Has => "has",
        fol_parser::ast::BinaryOperator::Is => "is",
        fol_parser::ast::BinaryOperator::As => "as",
        fol_parser::ast::BinaryOperator::Cast => "cast",
        fol_parser::ast::BinaryOperator::Pipe => "pipe",
        fol_parser::ast::BinaryOperator::PipeOr => "pipe_or",
    }
}

fn resolve_entry_variant_target(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    current_identity: &PackageIdentity,
    object: &AstNode,
    field: &str,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
) -> Result<Option<(PackageIdentity, SymbolId, String)>, LoweringError> {
    let (resolved_symbol, checked_type) = match object {
        AstNode::Identifier { syntax_id, name } => (
            resolve_reference_symbol(
                typed_package,
                *syntax_id,
                fol_resolver::ReferenceKind::Identifier,
                name,
            )?,
            resolve_reference_type_id(
                typed_package,
                checked_type_map,
                *syntax_id,
                fol_resolver::ReferenceKind::Identifier,
            ),
        ),
        AstNode::QualifiedIdentifier { path } => (
            resolve_reference_symbol(
                typed_package,
                path.syntax_id(),
                fol_resolver::ReferenceKind::QualifiedIdentifier,
                &path.joined(),
            )?,
            resolve_reference_type_id(
                typed_package,
                checked_type_map,
                path.syntax_id(),
                fol_resolver::ReferenceKind::QualifiedIdentifier,
            ),
        ),
        AstNode::Commented { node, .. } => {
            return resolve_entry_variant_target(
                typed_package,
                type_table,
                current_identity,
                node,
                field,
                checked_type_map,
            );
        }
        _ => return Ok(None),
    };

    let Some(checked_type) = checked_type else {
        return Ok(None);
    };
    let lowered_type = checked_type;
    if !matches!(resolved_symbol.kind, SymbolKind::Type | SymbolKind::Alias) {
        return Ok(None);
    }
    if !matches!(
        type_table_entry_kind(type_table, lowered_type),
        Some(())
    ) {
        return Ok(None);
    }

    let (owning_identity, owning_symbol_id) = canonical_symbol_key(
        current_identity,
        resolved_symbol.mounted_from.as_ref(),
        resolved_symbol.id,
    );
    Ok(Some((owning_identity, owning_symbol_id, field.to_string())))
}

fn type_table_entry_kind(
    type_table: &crate::LoweredTypeTable,
    lowered_type: LoweredTypeId,
) -> Option<()> {
    matches!(type_table.get(lowered_type), Some(crate::LoweredType::Entry { .. })).then_some(())
}

fn lower_assignment_target(
    typed_package: &fol_typecheck::TypedPackage,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    target: &AstNode,
    lowered_value: LoweredValue,
) -> Result<LoweredValue, LoweringError> {
    let resolved_symbol = match target {
        AstNode::Identifier { syntax_id, name } => resolve_reference_symbol(
            typed_package,
            *syntax_id,
            fol_resolver::ReferenceKind::Identifier,
            name,
        )?,
        AstNode::QualifiedIdentifier { path } => resolve_reference_symbol(
            typed_package,
            path.syntax_id(),
            fol_resolver::ReferenceKind::QualifiedIdentifier,
            &path.joined(),
        )?,
        _ => {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                "assignment targets must lower from plain or qualified identifiers in V1",
            ))
        }
    };

    if let Some(local_id) = cursor.routine.local_symbols.get(&resolved_symbol.id).copied() {
        cursor.push_instr(
            None,
            LoweredInstrKind::StoreLocal {
                local: local_id,
                value: lowered_value.local_id,
            },
        )?;
        return Ok(lowered_value);
    }

    let (owning_identity, owning_symbol_id) =
        canonical_symbol_key(current_identity, resolved_symbol.mounted_from.as_ref(), resolved_symbol.id);
    let Some(global_id) = decl_index.global_id_for_symbol(&owning_identity, owning_symbol_id) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!(
                "assignment target '{}' does not map to a lowered global definition",
                resolved_symbol.name
            ),
        ));
    };
    cursor.push_instr(
        None,
        LoweredInstrKind::StoreGlobal {
            global: global_id,
            value: lowered_value.local_id,
        },
    )?;
    Ok(lowered_value)
}

fn resolve_reference_symbol<'a>(
    typed_package: &'a fol_typecheck::TypedPackage,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: fol_resolver::ReferenceKind,
    display_name: &str,
) -> Result<&'a fol_resolver::ResolvedSymbol, LoweringError> {
    let syntax_id = syntax_id.ok_or_else(|| {
        LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' does not retain a syntax id"),
        )
    })?;
    let Some(reference) = typed_package.program.resolved().references.iter().find(|reference| {
        reference.syntax_id == Some(syntax_id) && reference.kind == kind
    }) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' is missing from resolver output"),
        ));
    };
    let Some(symbol_id) = reference.resolved else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("reference '{display_name}' does not resolve to a lowered symbol"),
        ));
    };
    typed_package
        .program
        .resolved()
        .symbol(symbol_id)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("reference '{display_name}' lost its resolved symbol"),
            )
        })
}

fn routine_return_type(
    cursor: &RoutineCursor<'_>,
    type_table: &crate::LoweredTypeTable,
) -> Option<LoweredTypeId> {
    let signature_id = cursor.routine.signature?;
    match type_table.get(signature_id) {
        Some(crate::LoweredType::Routine(signature)) => signature.return_type,
        _ => None,
    }
}

fn routine_error_type(
    cursor: &RoutineCursor<'_>,
    type_table: &crate::LoweredTypeTable,
) -> Option<LoweredTypeId> {
    let signature_id = cursor.routine.signature?;
    match type_table.get(signature_id) {
        Some(crate::LoweredType::Routine(signature)) => signature.error_type,
        _ => None,
    }
}

fn lower_function_call(
    typed_package: &fol_typecheck::TypedPackage,
    type_table: &crate::LoweredTypeTable,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    scope_id: ScopeId,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: fol_resolver::ReferenceKind,
    display_name: &str,
    args: &[AstNode],
) -> Result<LoweredValue, LoweringError> {
    let resolved_symbol = resolve_reference_symbol(typed_package, syntax_id, kind, display_name)?;
    let (owning_identity, owning_symbol_id) =
        canonical_symbol_key(current_identity, resolved_symbol.mounted_from.as_ref(), resolved_symbol.id);
    let Some(callee) = decl_index.routine_id_for_symbol(&owning_identity, owning_symbol_id) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("call target '{display_name}' does not map to a lowered routine definition"),
        ));
    };
    let Some(result_type) = resolve_reference_type_id(typed_package, checked_type_map, syntax_id, kind) else {
        return Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "procedure-style calls without a value result are not lowered in this slice yet: '{display_name}'"
            ),
        ));
    };
    let param_types = decl_index
        .routine_param_types(callee)
        .ok_or_else(|| {
            LoweringError::with_kind(
                LoweringErrorKind::InvalidInput,
                format!("call target '{display_name}' does not retain lowered parameter types"),
            )
        })?
        .to_vec();
    let lowered_args = args
        .iter()
        .enumerate()
        .map(|(index, arg)| {
            let expected = param_types.get(index).copied();
            lower_expression_expected(
                typed_package,
                type_table,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                scope_id,
                expected,
                arg,
            )
            .map(|value| value.local_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let call_error_type = lowered_symbol_error_type(
        typed_package,
        checked_type_map,
        resolved_symbol.id,
    );
    let result_local = cursor.allocate_local_with_effect(result_type, call_error_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::Call {
            callee,
            args: lowered_args,
            error_type: call_error_type,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
        recoverable_error_type: call_error_type,
    })
}

fn resolve_reference_type_id(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    syntax_id: Option<fol_parser::ast::SyntaxNodeId>,
    kind: fol_resolver::ReferenceKind,
) -> Option<LoweredTypeId> {
    let syntax_id = syntax_id?;
    let reference = typed_package
        .program
        .resolved()
        .references
        .iter()
        .find(|reference| reference.syntax_id == Some(syntax_id) && reference.kind == kind)?;
    let checked_type = reference_type_id(typed_package, reference.id)?;
    checked_type_map.get(&checked_type).copied()
}

fn lowered_symbol_error_type(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    symbol_id: SymbolId,
) -> Option<LoweredTypeId> {
    let declared_type = typed_package.program.typed_symbol(symbol_id)?.declared_type?;
    let fol_typecheck::CheckedType::Routine(signature) =
        typed_package.program.type_table().get(declared_type)?
    else {
        return None;
    };
    signature
        .error_type
        .and_then(|error_type| checked_type_map.get(&error_type).copied())
}

fn resolve_method_target(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    method: &str,
    receiver_type: LoweredTypeId,
) -> Result<(LoweredRoutineId, LoweredTypeId, Option<LoweredTypeId>), LoweringError> {
    let mut matches = Vec::new();

    for (symbol_id, symbol) in typed_package.program.resolved().symbols.iter_with_ids() {
        if symbol.kind != SymbolKind::Routine || symbol.name != method {
            continue;
        }
        let Some(typed_symbol) = typed_package.program.typed_symbol(symbol_id) else {
            continue;
        };
        let Some(receiver_checked_type) = typed_symbol.receiver_type else {
            continue;
        };
        let Some(lowered_receiver_type) = checked_type_map.get(&receiver_checked_type).copied() else {
            continue;
        };
        if lowered_receiver_type != receiver_type {
            continue;
        }

        let (owning_identity, owning_symbol_id) =
            canonical_symbol_key(current_identity, symbol.mounted_from.as_ref(), symbol_id);
        let Some(routine_id) = decl_index.routine_id_for_symbol(&owning_identity, owning_symbol_id) else {
            continue;
        };
        let Some(signature_checked_type) = typed_symbol.declared_type else {
            continue;
        };
        let Some(fol_typecheck::CheckedType::Routine(signature)) =
            typed_package.program.type_table().get(signature_checked_type)
        else {
            continue;
        };
        let Some(return_type) = signature
            .return_type
            .and_then(|return_type| checked_type_map.get(&return_type).copied())
        else {
            return Err(LoweringError::with_kind(
                LoweringErrorKind::Unsupported,
                format!(
                    "procedure-style method calls without a value result are not lowered in this slice yet: '{method}'"
                ),
            ));
        };
        let error_type = signature
            .error_type
            .and_then(|error_type| checked_type_map.get(&error_type).copied());
        matches.push((routine_id, return_type, error_type));
    }

    match matches.len() {
        1 => Ok(matches.remove(0)),
        0 => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("method '{method}' is not available for the lowered receiver type"),
        )),
        _ => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            format!("method '{method}' is ambiguous for the lowered receiver type"),
        )),
    }
}

fn field_access_type(
    type_table: &crate::LoweredTypeTable,
    object_type: LoweredTypeId,
    field: &str,
) -> Option<LoweredTypeId> {
    match type_table.get(object_type) {
        Some(crate::LoweredType::Record { fields }) => fields.get(field).copied(),
        Some(crate::LoweredType::Entry { variants }) => variants.get(field).copied().flatten(),
        _ => None,
    }
}

fn index_access_type(
    type_table: &crate::LoweredTypeTable,
    container_type: LoweredTypeId,
    index: &AstNode,
) -> Option<LoweredTypeId> {
    match type_table.get(container_type) {
        Some(crate::LoweredType::Array { element_type, .. })
        | Some(crate::LoweredType::Vector { element_type })
        | Some(crate::LoweredType::Sequence { element_type }) => Some(*element_type),
        Some(crate::LoweredType::Map { value_type, .. }) => Some(*value_type),
        Some(crate::LoweredType::Set { member_types }) => {
            let index_value = literal_index_value(index)?;
            member_types.get(index_value).copied().or_else(|| {
                let first = member_types.first().copied()?;
                member_types.iter().all(|member| *member == first).then_some(first)
            })
        }
        _ => None,
    }
}

fn expected_linear_element_type(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
    kind: ContainerType,
) -> Option<LoweredTypeId> {
    match (expected_type.and_then(|type_id| type_table.get(type_id)), kind) {
        (Some(crate::LoweredType::Array { element_type, .. }), ContainerType::Array)
        | (Some(crate::LoweredType::Vector { element_type }), ContainerType::Vector)
        | (Some(crate::LoweredType::Sequence { element_type }), ContainerType::Sequence) => {
            Some(*element_type)
        }
        _ => None,
    }
}

fn expected_container_kind(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
) -> Option<ContainerType> {
    match expected_type.and_then(|type_id| type_table.get(type_id)) {
        Some(crate::LoweredType::Array { .. }) => Some(ContainerType::Array),
        Some(crate::LoweredType::Vector { .. }) => Some(ContainerType::Vector),
        Some(crate::LoweredType::Sequence { .. }) => Some(ContainerType::Sequence),
        Some(crate::LoweredType::Set { .. }) => Some(ContainerType::Set),
        Some(crate::LoweredType::Map { .. }) => Some(ContainerType::Map),
        _ => None,
    }
}

fn resolve_linear_container_type(
    type_table: &crate::LoweredTypeTable,
    kind: ContainerType,
    expected_type: Option<LoweredTypeId>,
    element_type: Option<LoweredTypeId>,
    len: usize,
) -> Option<LoweredTypeId> {
    if let Some(type_id) = expected_type {
        return match (type_table.get(type_id), kind) {
            (Some(crate::LoweredType::Array { .. }), ContainerType::Array)
            | (Some(crate::LoweredType::Vector { .. }), ContainerType::Vector)
            | (Some(crate::LoweredType::Sequence { .. }), ContainerType::Sequence) => Some(type_id),
            _ => None,
        };
    }

    let element_type = element_type?;
    Some(match kind {
        ContainerType::Array => find_array_type(type_table, element_type, Some(len)),
        ContainerType::Vector => find_vector_type(type_table, element_type),
        ContainerType::Sequence => find_sequence_type(type_table, element_type),
        ContainerType::Set | ContainerType::Map => return None,
    })
}

fn expected_set_member_types(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
) -> Option<Vec<LoweredTypeId>> {
    match expected_type.and_then(|type_id| type_table.get(type_id)) {
        Some(crate::LoweredType::Set { member_types }) => Some(member_types.clone()),
        _ => None,
    }
}

fn expected_map_key_type(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
) -> Option<LoweredTypeId> {
    match expected_type.and_then(|type_id| type_table.get(type_id)) {
        Some(crate::LoweredType::Map { key_type, .. }) => Some(*key_type),
        _ => None,
    }
}

fn expected_map_value_type(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
) -> Option<LoweredTypeId> {
    match expected_type.and_then(|type_id| type_table.get(type_id)) {
        Some(crate::LoweredType::Map { value_type, .. }) => Some(*value_type),
        _ => None,
    }
}

fn resolve_map_type(
    type_table: &crate::LoweredTypeTable,
    expected_type: Option<LoweredTypeId>,
    key_type: Option<LoweredTypeId>,
    value_type: Option<LoweredTypeId>,
) -> Option<LoweredTypeId> {
    if let Some(type_id) = expected_type {
        return matches!(type_table.get(type_id), Some(crate::LoweredType::Map { .. })).then_some(type_id);
    }
    Some(find_map_type(type_table, key_type?, value_type?))
}

fn lowered_linear_kind(kind: ContainerType) -> Result<LoweredLinearKind, LoweringError> {
    match kind {
        ContainerType::Array => Ok(LoweredLinearKind::Array),
        ContainerType::Vector => Ok(LoweredLinearKind::Vector),
        ContainerType::Sequence => Ok(LoweredLinearKind::Sequence),
        ContainerType::Set | ContainerType::Map => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "set/map container kinds do not lower through linear container instructions",
        )),
    }
}

fn find_array_type(
    type_table: &crate::LoweredTypeTable,
    element_type: LoweredTypeId,
    size: Option<usize>,
) -> LoweredTypeId {
    (0..type_table.len())
        .map(crate::LoweredTypeId)
        .find(|type_id| {
            matches!(
                type_table.get(*type_id),
                Some(crate::LoweredType::Array {
                    element_type: actual_element,
                    size: actual_size,
                }) if *actual_element == element_type && *actual_size == size
            )
        })
        .unwrap_or_else(|| panic!("lowered type table lost array shape for element {}", element_type.0))
}

fn find_vector_type(
    type_table: &crate::LoweredTypeTable,
    element_type: LoweredTypeId,
) -> LoweredTypeId {
    (0..type_table.len())
        .map(crate::LoweredTypeId)
        .find(|type_id| {
            matches!(
                type_table.get(*type_id),
                Some(crate::LoweredType::Vector {
                    element_type: actual_element,
                }) if *actual_element == element_type
            )
        })
        .unwrap_or_else(|| panic!("lowered type table lost vector shape for element {}", element_type.0))
}

fn find_sequence_type(
    type_table: &crate::LoweredTypeTable,
    element_type: LoweredTypeId,
) -> LoweredTypeId {
    (0..type_table.len())
        .map(crate::LoweredTypeId)
        .find(|type_id| {
            matches!(
                type_table.get(*type_id),
                Some(crate::LoweredType::Sequence {
                    element_type: actual_element,
                }) if *actual_element == element_type
            )
        })
        .unwrap_or_else(|| panic!("lowered type table lost sequence shape for element {}", element_type.0))
}

fn find_set_type(type_table: &crate::LoweredTypeTable, member_types: &[LoweredTypeId]) -> LoweredTypeId {
    (0..type_table.len())
        .map(crate::LoweredTypeId)
        .find(|type_id| {
            matches!(
                type_table.get(*type_id),
                Some(crate::LoweredType::Set {
                    member_types: actual_members,
                }) if actual_members == member_types
            )
        })
        .unwrap_or_else(|| panic!("lowered type table lost set shape"))
}

fn find_map_type(
    type_table: &crate::LoweredTypeTable,
    key_type: LoweredTypeId,
    value_type: LoweredTypeId,
) -> LoweredTypeId {
    (0..type_table.len())
        .map(crate::LoweredTypeId)
        .find(|type_id| {
            matches!(
                type_table.get(*type_id),
                Some(crate::LoweredType::Map {
                    key_type: actual_key,
                    value_type: actual_value,
                }) if *actual_key == key_type && *actual_value == value_type
            )
        })
        .unwrap_or_else(|| panic!("lowered type table lost map shape"))
}

fn container_elements(elements: &[AstNode]) -> Vec<&AstNode> {
    elements
        .iter()
        .filter(|element| !matches!(element, AstNode::Comment { .. }))
        .collect()
}

fn map_literal_pair(pair: &AstNode) -> Result<(&AstNode, &AstNode), LoweringError> {
    match pair {
        AstNode::ContainerLiteral { elements, .. } => {
            let pair_items = container_elements(elements);
            if let [key, value] = pair_items.as_slice() {
                Ok((*key, *value))
            } else {
                Err(LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "map literals require each element to be a two-value pair",
                ))
            }
        }
        AstNode::Commented { node, .. } => map_literal_pair(node),
        _ => Err(LoweringError::with_kind(
            LoweringErrorKind::InvalidInput,
            "map literals require each element to be a two-value pair",
        )),
    }
}

fn literal_index_value(node: &AstNode) -> Option<usize> {
    match node {
        AstNode::Literal(Literal::Integer(value)) => usize::try_from(*value).ok(),
        AstNode::Commented { node, .. } => literal_index_value(node),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{RoutineCursor, WorkspaceDeclIndex};
    use crate::{
        types::{LoweredBuiltinType, LoweredTypeTable},
        LoweredBlock, LoweredGlobal, LoweredInstrKind, LoweredOperand, LoweredPackage,
        LoweredRoutine, LoweredTerminator, LoweredWorkspace, LoweringErrorKind,
    };
    use fol_parser::ast::AstParser;
    use fol_parser::ast::Literal;
    use fol_resolver::{resolve_workspace, PackageIdentity, PackageSourceKind, SourceUnitId, SymbolKind};
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;
    use std::collections::BTreeMap;

    fn lower_fixture_error(source: &str) -> crate::LoweringError {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_negative_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(&fixture, source).expect("should write lowering negative fixture");

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
        crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect_err("fixture should fail during lowering")
            .into_iter()
            .next()
            .expect("lowering should emit at least one error")
    }

    fn lower_folder_fixture_error(files: &[(&str, &str)]) -> crate::LoweringError {
        let root = std::env::temp_dir().join(format!(
            "fol_lower_negative_folder_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("should create lowering folder fixture root");
        for (path, source) in files {
            let full_path = root.join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .expect("should create lowering folder fixture parent directories");
            }
            std::fs::write(&full_path, source).expect("should write lowering folder fixture");
        }

        let app_root = root.join("app");
        let mut stream = FileStream::from_folder(app_root.to_str().expect("utf8 temp path"))
            .expect("Should open lowering folder fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering folder fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering folder fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering folder fixture should typecheck");
        crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect_err("folder fixture should fail during lowering")
            .into_iter()
            .next()
            .expect("lowering should emit at least one error")
    }

    fn lower_folder_fixture_workspace(files: &[(&str, &str)]) -> crate::LoweredWorkspace {
        let root = std::env::temp_dir().join(format!(
            "fol_lower_success_folder_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("should create lowering folder fixture root");
        for (path, source) in files {
            let full_path = root.join(path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)
                    .expect("should create lowering folder fixture parent directories");
            }
            std::fs::write(&full_path, source).expect("should write lowering folder fixture");
        }

        let app_root = root.join("app");
        let mut stream = FileStream::from_folder(app_root.to_str().expect("utf8 temp path"))
            .expect("Should open lowering folder fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering folder fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering folder fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering folder fixture should typecheck");
        crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("folder fixture should lower successfully")
    }

    fn lower_fixture_panic_message(source: &str) -> String {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_panic_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(&fixture, source).expect("should write lowering panic fixture");

        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
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
            let _ = crate::LoweringSession::new(typed).lower_workspace();
        }))
        .expect_err("fixture should currently panic during lowering");

        if let Some(message) = panic.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = panic.downcast_ref::<&str>() {
            (*message).to_string()
        } else {
            "non-string panic payload".to_string()
        }
    }

    fn lower_fixture_workspace(source: &str) -> crate::LoweredWorkspace {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_success_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(&fixture, source).expect("should write lowering success fixture");

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
        crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("fixture should lower successfully")
    }

    #[test]
    fn literal_lowering_emits_constant_instructions_into_the_current_block() {
        let mut types = LoweredTypeTable::new();
        let int_type = types.intern_builtin(LoweredBuiltinType::Int);
        let float_type = types.intern_builtin(LoweredBuiltinType::Float);
        let str_type = types.intern_builtin(LoweredBuiltinType::Str);

        let mut routine = LoweredRoutine::new(crate::LoweredRoutineId(0), "main", crate::LoweredBlockId(0));
        let entry = routine.blocks.push(LoweredBlock {
            id: crate::LoweredBlockId(0),
            instructions: Vec::new(),
            terminator: None,
        });
        routine.entry_block = entry;
        let mut cursor = RoutineCursor::new(&mut routine, entry);

        let int_value = cursor
            .lower_literal(&Literal::Integer(7), int_type)
            .expect("integer literals should lower");
        let float_value = cursor
            .lower_literal(&Literal::Float(3.5), float_type)
            .expect("float literals should lower");
        let str_value = cursor
            .lower_literal(&Literal::String("ok".to_string()), str_type)
            .expect("string literals should lower");

        assert_eq!(routine.blocks.get(entry).expect("entry block should exist").instructions.len(), 3);
        assert_eq!(routine.locals.len(), 3);
        assert_eq!(int_value.local_id.0, 0);
        assert_eq!(float_value.local_id.0, 1);
        assert_eq!(str_value.local_id.0, 2);
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::Const(LoweredOperand::Int(7)))
        );
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(1)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::Const(LoweredOperand::Float(3.5f64.to_bits())))
        );
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(2)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::Const(LoweredOperand::Str("ok".to_string())))
        );
    }

    #[test]
    fn lowering_repro_keeps_same_name_parameters_distinct_per_routine_scope() {
        let lowered = lower_folder_fixture_workspace(&[
            (
                "shared/lib.fol",
                concat!(
                    "typ[exp] User: rec = {\n",
                    "    count: int;\n",
                    "}\n",
                    "fun[exp] fallback(): int = {\n",
                    "    return 2;\n",
                    "}\n",
                    "fun[exp] (User)read(): int = {\n",
                    "    return 7;\n",
                    "}\n",
                ),
            ),
            (
                "app/main.fol",
                concat!(
                    "use shared: loc = {\"../shared\"};\n",
                    "fun[] decide(flag: bol, user: User): int = {\n",
                    "    when(flag) {\n",
                    "        case(true) { user.read() }\n",
                    "        * { fallback() }\n",
                    "    }\n",
                    "}\n",
                ),
            ),
        ]);
        let entry_package = lowered.entry_package();
        for routine_name in ["build_user", "choose_count", "main"] {
            let routine = entry_package
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .expect("routine shell should exist");
            let param_names = routine
                .params
                .iter()
                .filter_map(|local_id| routine.locals.get(*local_id).and_then(|local| local.name.clone()))
                .collect::<Vec<_>>();
            assert!(
                param_names.iter().any(|name| name == "flag"),
                "routine '{routine_name}' should keep its own lowered flag parameter",
            );
        }
    }

    #[test]
    fn lowering_repro_lowers_non_empty_seq_literals_in_typed_v1_contexts() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] take(values: seq[str]): seq[str] = {\n",
                "    return values\n",
                "}\n",
                "fun[] from_binding(): seq[str] = {\n",
                "    var names: seq[str] = {\"Ada\", \"Lin\"}\n",
                "    return names\n",
                "}\n",
                "fun[] from_return(): seq[str] = {\n",
                "    return {\"Ada\", \"Lin\"}\n",
                "}\n",
                "fun[] from_arg(): seq[str] = {\n",
                "    return take({\"Ada\", \"Lin\"})\n",
                "}\n",
            ),
        );

        for routine_name in ["from_binding", "from_return", "from_arg"] {
            let routine = lowered
                .entry_package()
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .expect("sequence lowering routine should exist");
            let construct = routine.instructions.iter().find_map(|instr| match &instr.kind {
                LoweredInstrKind::ConstructLinear { kind, elements, .. } => {
                    Some((*kind, elements.len()))
                }
                _ => None,
            });

            assert_eq!(
                construct,
                Some((crate::control::LoweredLinearKind::Sequence, 2)),
                "typed sequence literals should lower as sequence instructions in {routine_name}",
            );
        }
    }

    #[test]
    fn lowering_repro_lowers_non_empty_set_and_map_literals_in_typed_v1_contexts() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] set_return(): set[int, str] = {\n",
                "    return {1, \"two\"}\n",
                "}\n",
                "fun[] map_return(): map[str, int] = {\n",
                "    return {{\"US\", 45}, {\"DE\", 82}}\n",
                "}\n",
                "fun[] from_set_index(): str = {\n",
                "    var parts: set[int, str] = {1, \"two\"}\n",
                "    return parts[1]\n",
                "}\n",
                "fun[] from_map_index(): int = {\n",
                "    var counts: map[str, int] = {{\"US\", 45}, {\"DE\", 82}}\n",
                "    return counts[\"DE\"]\n",
                "}\n",
            ),
        );

        let expected = [
            ("set_return", "set", 2usize),
            ("map_return", "map", 2usize),
            ("from_set_index", "set", 2usize),
            ("from_map_index", "map", 2usize),
        ];
        for (routine_name, aggregate_kind, expected_len) in expected {
            let routine = lowered
                .entry_package()
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .expect("aggregate lowering routine should exist");
            let lowered_members = routine.instructions.iter().find_map(|instr| match (&instr.kind, aggregate_kind) {
                (LoweredInstrKind::ConstructSet { members, .. }, "set") => Some(members.len()),
                (LoweredInstrKind::ConstructMap { entries, .. }, "map") => Some(entries.len()),
                _ => None,
            });

            assert_eq!(
                lowered_members,
                Some(expected_len),
                "typed {aggregate_kind} literals should lower in {routine_name}",
            );
        }
    }

    #[test]
    fn lowering_repro_keeps_exact_typed_container_instruction_shapes() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] seq_return(): seq[str] = {\n",
                "    return {\"Ada\", \"Lin\"}\n",
                "}\n",
                "fun[] map_return(): map[str, int] = {\n",
                "    return {{\"US\", 45}, {\"DE\", 82}}\n",
                "}\n",
            ),
        );

        let seq_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "seq_return")
            .expect("sequence return routine should exist");
        assert_eq!(seq_routine.instructions.len(), 3);
        assert!(matches!(
            seq_routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Str(value))) if value == "Ada"
        ));
        assert!(matches!(
            seq_routine.instructions.get(crate::LoweredInstrId(1)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Str(value))) if value == "Lin"
        ));
        assert!(matches!(
            seq_routine.instructions.get(crate::LoweredInstrId(2)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::ConstructLinear {
                kind: crate::control::LoweredLinearKind::Sequence,
                elements,
                ..
            }) if elements.len() == 2
        ));

        let map_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "map_return")
            .expect("map return routine should exist");
        assert_eq!(map_routine.instructions.len(), 5);
        assert!(matches!(
            map_routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Str(value))) if value == "US"
        ));
        assert!(matches!(
            map_routine.instructions.get(crate::LoweredInstrId(1)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Int(45)))
        ));
        assert!(matches!(
            map_routine.instructions.get(crate::LoweredInstrId(2)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Str(value))) if value == "DE"
        ));
        assert!(matches!(
            map_routine.instructions.get(crate::LoweredInstrId(3)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::Const(LoweredOperand::Int(82)))
        ));
        assert!(matches!(
            map_routine.instructions.get(crate::LoweredInstrId(4)).map(|instr| &instr.kind),
            Some(LoweredInstrKind::ConstructMap { entries, .. }) if entries.len() == 2
        ));
    }

    #[test]
    fn lowering_repro_lowers_early_return_when_branches_as_statement_control_flow() {
        let lowered = lower_fixture_workspace(
            concat!(
                "var enabled: bol = true\n",
                "var default_name: str = \"Ada\"\n",
                "var low_count: int = 1\n",
                "var high_count: int = 7\n",
                "typ NameTag: rec = {\n",
                "    label: str;\n",
                "    code: int\n",
                "}\n",
                "typ Audit: rec = {\n",
                "    active: bol;\n",
                "    marker: NameTag\n",
                "}\n",
                "typ User: rec = {\n",
                "    name: str;\n",
                "    count: int;\n",
                "    audit: Audit\n",
                "}\n",
                "fun[] build_tag(): NameTag = {\n",
                "    return { label = \"stable\", code = high_count }\n",
                "}\n",
                "fun[] build_user(): User = {\n",
                "    return {\n",
                "        name = default_name,\n",
                "        count = high_count,\n",
                "        audit = {\n",
                "            active = enabled,\n",
                "            marker = build_tag(),\n",
                "        },\n",
                "    }\n",
                "}\n",
                "fun[] choose_count(): int = {\n",
                "    when(enabled) {\n",
                "        case(true) { high_count }\n",
                "        * { low_count }\n",
                "    }\n",
                "}\n",
                "fun[] main(): int = {\n",
                "    var current: User = build_user()\n",
                "    loop(enabled) {\n",
                "        break\n",
                "    }\n",
                "    when(enabled) {\n",
                "        case(true) { return current.audit.marker.code }\n",
                "        * { return choose_count() }\n",
                "    }\n",
                "}\n",
            ),
        );
        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main lowering routine should exist");
        let return_blocks = routine
            .blocks
            .iter()
            .filter(|block| matches!(block.terminator, Some(crate::LoweredTerminator::Return { .. })))
            .count();

        assert_eq!(routine.body_result, None);
        assert_eq!(
            return_blocks, 2,
            "early-return when lowering should preserve both branch returns without synthesizing a join value",
        );
    }

    #[test]
    fn lowering_repro_keeps_exact_cfg_shape_for_early_return_when_branches() {
        let lowered = lower_fixture_workspace(
            "fun[] main(flag: bol): int = {\n    when(flag) {\n        case(true) { return 1 }\n        * { return 2 }\n    }\n}\n",
        );
        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main lowering routine should exist");

        assert_eq!(routine.blocks.len(), 3);
        assert_eq!(routine.body_result, None);
        assert!(matches!(
            routine.blocks.get(crate::LoweredBlockId(0)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Branch {
                then_block: crate::LoweredBlockId(1),
                else_block: crate::LoweredBlockId(2),
                ..
            })
        ));
        assert!(matches!(
            routine.blocks.get(crate::LoweredBlockId(1)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Return { value: Some(_) })
        ));
        assert!(matches!(
            routine.blocks.get(crate::LoweredBlockId(2)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Return { value: Some(_) })
        ));
    }

    #[test]
    fn nil_literal_lowering_stays_deferred_to_shell_lowering() {
        let mut types = LoweredTypeTable::new();
        let int_type = types.intern_builtin(LoweredBuiltinType::Int);
        let mut routine = LoweredRoutine::new(crate::LoweredRoutineId(0), "main", crate::LoweredBlockId(0));
        let entry = routine.blocks.push(LoweredBlock {
            id: crate::LoweredBlockId(0),
            instructions: Vec::new(),
            terminator: None,
        });
        routine.entry_block = entry;

        let error = RoutineCursor::new(&mut routine, entry)
            .lower_literal(&Literal::Nil, int_type)
            .expect_err("nil should stay out of the core literal slice");

        assert_eq!(error.kind(), LoweringErrorKind::Unsupported);
    }

    #[test]
    fn identifier_lowering_loads_parameter_locals_and_top_level_globals() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_identifier_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "var count: int = 1\nfun[] main(value: int): int = { value }",
        )
        .expect("should write lowering identifier fixture");

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

        let package = lowered_workspace.entry_package();
        let mut routine = package
            .routine_decls
            .values()
            .next()
            .expect("routine shell should exist")
            .clone();
        let decl_index = WorkspaceDeclIndex::build(&lowered_workspace);
        let int_type = package
            .checked_type_map
            .get(&fol_typecheck::CheckedTypeId(0))
            .copied()
            .expect("int builtin should map into lowering types");

        let param_symbol = typed
            .entry_program()
            .resolved()
            .symbols
            .iter_with_ids()
            .find(|(_, symbol)| symbol.kind == SymbolKind::Parameter && symbol.name == "value")
            .map(|(symbol_id, _)| symbol_id)
            .expect("parameter symbol should exist");
        let global_symbol = package
            .global_decls
            .values()
            .find(|global| global.name == "count")
            .map(|global| global.symbol_id)
            .expect("global symbol should exist");

        let mut cursor = RoutineCursor::new(&mut routine, routine.entry_block);
        let param_value = cursor
            .lower_identifier_reference(
                lowered_workspace.entry_identity(),
                &decl_index,
                typed.entry_program()
                    .resolved()
                    .symbol(param_symbol)
                    .expect("parameter symbol should resolve"),
                int_type,
            )
            .expect("parameter references should lower to local loads");
        let global_value = cursor
            .lower_identifier_reference(
                lowered_workspace.entry_identity(),
                &decl_index,
                typed.entry_program()
                    .resolved()
                    .symbol(global_symbol)
                    .expect("global symbol should resolve"),
                int_type,
            )
            .expect("global references should lower to global loads");

        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::LoadLocal {
                local: routine.local_symbols[&param_symbol],
            })
        );
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(1)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::LoadGlobal {
                global: package.globals[0],
            })
        );
        assert_eq!(param_value.local_id.0, routine.locals.len() - 2);
        assert_eq!(global_value.local_id.0, routine.locals.len() - 1);
    }

    #[test]
    fn declaration_index_tracks_globals_and_routines_by_owning_package() {
        let identity = PackageIdentity {
            source_kind: PackageSourceKind::Entry,
            canonical_root: "/workspace/app".to_string(),
            display_name: "app".to_string(),
        };
        let mut package = LoweredPackage::new(crate::LoweredPackageId(0), identity.clone());
        package.global_decls.insert(
            crate::LoweredGlobalId(0),
            LoweredGlobal {
                id: crate::LoweredGlobalId(0),
                symbol_id: fol_resolver::SymbolId(1),
                source_unit_id: SourceUnitId(0),
                name: "answer".to_string(),
                type_id: crate::LoweredTypeId(0),
                recoverable_error_type: None,
                mutable: false,
            },
        );
        package.routine_decls.insert(
            crate::LoweredRoutineId(0),
            LoweredRoutine::new(crate::LoweredRoutineId(0), "main", crate::LoweredBlockId(0)),
        );
        package
            .routine_decls
            .get_mut(&crate::LoweredRoutineId(0))
            .expect("routine should exist")
            .symbol_id = Some(fol_resolver::SymbolId(2));
        let mut packages = BTreeMap::new();
        packages.insert(identity.clone(), package);
        let mut type_table = crate::LoweredTypeTable::new();
        let recoverable_abi = crate::LoweredRecoverableAbi::v1(
            type_table.intern_builtin(crate::LoweredBuiltinType::Bool),
        );
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            packages,
            vec![crate::LoweredEntryCandidate {
                package_identity: identity.clone(),
                routine_id: crate::LoweredRoutineId(0),
                name: "main".to_string(),
            }],
            type_table,
            crate::LoweredSourceMap::new(),
            recoverable_abi,
        );

        let index = WorkspaceDeclIndex::build(&workspace);

        assert_eq!(
            index.global_id_for_symbol(&identity, fol_resolver::SymbolId(1)),
            Some(crate::LoweredGlobalId(0))
        );
        assert_eq!(
            index.routine_id_for_symbol(&identity, fol_resolver::SymbolId(2)),
            Some(crate::LoweredRoutineId(0))
        );
    }

    #[test]
    fn routine_body_lowering_keeps_local_initializers_and_final_expression_results() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_body_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(): int = {\n    var value: int = 1\n    value\n}",
        )
        .expect("should write lowering body fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("body lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .next()
            .expect("lowered routine should exist");
        let entry_block = routine
            .blocks
            .get(routine.entry_block)
            .expect("entry block should exist");

        assert_eq!(entry_block.instructions.len(), 3);
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::Const(LoweredOperand::Int(1)))
        );
        assert!(
            matches!(
                routine.instructions.get(crate::LoweredInstrId(1)).map(|instr| &instr.kind),
                Some(LoweredInstrKind::StoreLocal { .. })
            ),
            "local binding initializer should lower into a store"
        );
        assert!(
            matches!(
                routine.instructions.get(crate::LoweredInstrId(2)).map(|instr| &instr.kind),
                Some(LoweredInstrKind::LoadLocal { .. })
            ),
            "final body expression should lower into a local load"
        );
        assert!(routine.body_result.is_some());
    }

    #[test]
    fn assignment_lowering_emits_local_and_global_store_instructions() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_assignment_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "var count: int = 0\nfun[] main(): int = {\n    var value: int = 1\n    value = 2\n    count = value\n    value\n}",
        )
        .expect("should write lowering assignment fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("assignment lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .next()
            .expect("lowered routine should exist");

        assert!(
            routine
                .instructions
                .iter()
                .any(|instr| matches!(instr.kind, LoweredInstrKind::StoreLocal { .. })),
            "assignment to local bindings should lower into StoreLocal"
        );
        assert!(
            routine
                .instructions
                .iter()
                .any(|instr| matches!(instr.kind, LoweredInstrKind::StoreGlobal { .. })),
            "assignment to globals should lower into StoreGlobal"
        );
    }

    #[test]
    fn call_lowering_emits_direct_callee_calls_for_plain_and_qualified_forms() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_call_exprs_{stamp}"));
        let app_dir = root.join("app");
        let math_dir = app_dir.join("math");
        fs::create_dir_all(&math_dir).expect("should create nested namespace dir");
        fs::write(
            app_dir.join("main.fol"),
            "fun[] helper(): int = { 1 }\nfun[] main(): int = {\n    helper()\n    math::triple()\n}",
        )
        .expect("should write entry file");
        fs::write(math_dir.join("lib.fol"), "fun[exp] triple(): int = { 3 }\n")
            .expect("should write nested namespace file");

        let mut stream = FileStream::from_folder(app_dir.to_str().expect("utf8 temp path"))
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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("call lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let call_instrs = routine
            .instructions
            .iter()
            .filter(|instr| matches!(instr.kind, LoweredInstrKind::Call { .. }))
            .collect::<Vec<_>>();

        assert_eq!(call_instrs.len(), 2);
    }

    #[test]
    fn method_call_lowering_rewrites_receivers_into_direct_call_arguments() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_method_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun (int)double(): int = { 2 }\nfun[] main(): int = {\n    var value: int = 1\n    value.double()\n}",
        )
        .expect("should write lowering method fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("method call lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let call = routine
            .instructions
            .iter()
            .find_map(|instr| match &instr.kind {
                LoweredInstrKind::Call { callee, args, .. } => Some((*callee, args.clone())),
                _ => None,
            })
            .expect("method body should contain a lowered call");

        assert_eq!(call.1.len(), 1);
    }

    #[test]
    fn errorful_call_lowering_retains_explicit_error_type_metadata() {
        let lowered = lower_fixture_workspace(
            "fun[] load(): int / str = {\n\
                 report \"bad\";\n\
                 return 1;\n\
             }\n\
             fun[] main(): int / str = {\n\
                 return load();\n\
             }\n",
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let call_error_type = routine
            .instructions
            .iter()
            .find_map(|instr| match &instr.kind {
                LoweredInstrKind::Call { error_type, .. } => *error_type,
                _ => None,
            })
            .expect("errorful call should retain an explicit lowered error type");

        assert_eq!(
            lowered.type_table().get(call_error_type),
            Some(&crate::LoweredType::Builtin(crate::LoweredBuiltinType::Str))
        );
        let signature = routine
            .signature
            .and_then(|signature| lowered.type_table().get(signature))
            .expect("main routine should retain a lowered signature");
        match signature {
            crate::LoweredType::Routine(signature) => {
                assert_eq!(
                    signature
                        .error_type
                        .and_then(|error_type| lowered.type_table().get(error_type)),
                    Some(&crate::LoweredType::Builtin(crate::LoweredBuiltinType::Str))
                );
            }
            other => panic!("expected lowered routine signature, got {other:?}"),
        }
    }

    #[test]
    fn propagation_lowering_branches_and_reports_recoverable_calls() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int / str = {\n",
                "    return load(flag)\n",
                "}\n",
            ),
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::Call { error_type: Some(_), .. }
        )));
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::CheckRecoverable { .. }
        )));
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::UnwrapRecoverable { .. }
        )));
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::ExtractRecoverableError { .. }
        )));
        assert!(routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Branch { .. })
        )));
        assert!(routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Report { .. })
        )));
    }

    #[test]
    fn check_lowering_observes_recoverable_bindings_without_propagation() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): bol = {\n",
                "    var attempt = load(flag)\n",
                "    return check(attempt)\n",
                "}\n",
            ),
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let attempt_local = routine
            .locals
            .iter()
            .find(|local| local.name.as_deref() == Some("attempt"))
            .expect("attempt local should exist");

        assert!(attempt_local.recoverable_error_type.is_some());
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::CheckRecoverable { .. }
        )));
        assert!(!routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Report { .. })
        )));
    }

    #[test]
    fn pipe_or_default_lowering_branches_to_a_plain_fallback_value() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    return load(flag) || 5\n",
                "}\n",
            ),
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::CheckRecoverable { .. }
        )));
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::UnwrapRecoverable { .. }
        )));
        assert!(routine.instructions.iter().any(|instr| matches!(
            instr.kind,
            LoweredInstrKind::Const(LoweredOperand::Int(5))
        )));
        assert!(!routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Report { .. } | LoweredTerminator::Panic { .. })
        )));
    }

    #[test]
    fn pipe_or_report_lowering_uses_error_branch_reports() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int / str = {\n",
                "    return load(flag) || report \"fallback\"\n",
                "}\n",
            ),
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Report { .. })
        )));
    }

    #[test]
    fn pipe_or_panic_lowering_uses_error_branch_panics() {
        let lowered = lower_fixture_workspace(
            concat!(
                "fun[] load(flag: bol): int / str = {\n",
                "    when(flag) {\n",
                "        case(true) { report \"bad\" }\n",
                "        * { return 7 }\n",
                "    }\n",
                "}\n",
                "fun[] main(flag: bol): int = {\n",
                "    return load(flag) || panic \"fallback\"\n",
                "}\n",
            ),
        );

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(routine.blocks.iter().any(|block| matches!(
            block.terminator,
            Some(LoweredTerminator::Panic { .. })
        )));
    }

    #[test]
    fn field_access_lowering_emits_explicit_extraction_instructions() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_field_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "typ Point: { x: int, y: int }\nfun[] main(point: Point): int = {\n    point.x\n}",
        )
        .expect("should write lowering field fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("field access lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(
            routine
                .instructions
                .iter()
                .any(|instr| matches!(instr.kind, LoweredInstrKind::FieldAccess { .. })),
            "record field access should lower into an explicit FieldAccess instruction"
        );
    }

    #[test]
    fn index_access_lowering_emits_explicit_container_access_instructions() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_index_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] head(values: vec[int]): int = {\n    values[0]\n}",
        )
        .expect("should write lowering index fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("index access lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "head")
            .expect("head routine should exist");

        assert!(
            routine
                .instructions
                .iter()
                .any(|instr| matches!(instr.kind, LoweredInstrKind::IndexAccess { .. })),
            "container index access should lower into an explicit IndexAccess instruction"
        );
    }

    #[test]
    fn expression_lowering_keeps_local_and_imported_value_call_parity() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_expr_parity_{stamp}"));
        let app_dir = root.join("app");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&app_dir).expect("should create app dir");
        fs::create_dir_all(&shared_dir).expect("should create shared dir");
        fs::write(
            app_dir.join("main.fol"),
            "use shared: loc = {\"../shared\"}\nfun[] local_helper(): int = { 1 }\nfun[] main(): int = {\n    local_helper()\n    shared::twice(answer)\n}",
        )
        .expect("should write app entry");
        fs::write(
            shared_dir.join("lib.fol"),
            "var[exp] answer: int = 7\nfun[exp] twice(value: int): int = { value }",
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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("expression lowering parity should succeed");

        let main_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let shared_package = lowered
            .packages()
            .find(|package| package.identity.display_name == "shared")
            .expect("shared package should exist");

        assert!(
            main_routine
                .instructions
                .iter()
                .any(|instr| matches!(instr.kind, LoweredInstrKind::LoadGlobal { global } if shared_package.global_decls.contains_key(&global))),
            "entry routine should lower imported value references into foreign global loads"
        );
        assert!(
            main_routine
                .instructions
                .iter()
                .filter(|instr| matches!(instr.kind, LoweredInstrKind::Call { .. }))
                .count()
                >= 2,
            "entry routine should keep both local and imported call sites as direct Call instructions"
        );
    }

    #[test]
    fn return_lowering_emits_explicit_return_terminators_and_skips_trailing_body_nodes() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_return_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(): int = {\n    return 1\n    2\n}",
        )
        .expect("should write lowering return fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("return lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let entry_block = routine
            .blocks
            .get(routine.entry_block)
            .expect("entry block should exist");

        assert_eq!(entry_block.instructions.len(), 1);
        assert_eq!(
            entry_block.terminator,
            Some(LoweredTerminator::Return {
                value: Some(crate::LoweredLocalId(0)),
            })
        );
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::Const(LoweredOperand::Int(1)))
        );
        assert!(
            routine.body_result.is_none(),
            "early returns should not leave a trailing body_result behind"
        );
    }

    #[test]
    fn report_lowering_emits_explicit_report_terminators_and_skips_trailing_body_nodes() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_report_exprs_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(flag: bol): int / bol = {\n    report flag\n    return 1\n}",
        )
        .expect("should write lowering report fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("report lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let entry_block = routine
            .blocks
            .get(routine.entry_block)
            .expect("entry block should exist");

        assert_eq!(entry_block.instructions.len(), 1);
        assert_eq!(
            routine.instructions.get(crate::LoweredInstrId(0)).map(|instr| &instr.kind),
            Some(&LoweredInstrKind::LoadLocal {
                local: routine.params[0],
            })
        );
        assert_eq!(
            entry_block.terminator,
            Some(LoweredTerminator::Report {
                value: Some(crate::LoweredLocalId(1)),
            })
        );
        assert!(
            routine.body_result.is_none(),
            "early reports should not leave a trailing body_result behind"
        );
    }

    #[test]
    fn when_statement_lowering_emits_branch_blocks_and_falls_through_afterward() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_when_stmt_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(flag: bol): int = {\n    when(flag) {\n        case(true) { 1 }\n    }\n    return 2\n}",
        )
        .expect("should write lowering when fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("statement-style when lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(
            routine
                .blocks
                .iter()
                .any(|block| matches!(block.terminator, Some(LoweredTerminator::Branch { .. }))),
            "statement-style when should lower into an explicit branch terminator"
        );
        assert!(
            routine
                .blocks
                .iter()
                .any(|block| matches!(block.terminator, Some(LoweredTerminator::Jump { .. }))),
            "when bodies should jump into a shared continuation block"
        );
        assert!(
            routine
                .blocks
                .iter()
                .any(|block| matches!(block.terminator, Some(LoweredTerminator::Return { .. }))),
            "control should fall through after the when into the trailing return"
        );
    }

    #[test]
    fn when_expression_lowering_stores_branch_values_into_one_join_local() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_when_expr_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "var yes: int = 1\nvar no: int = 2\nfun[] main(flag: bol): int = {\n    when(flag) {\n        case(true) { yes }\n        * { no }\n    }\n}",
        )
        .expect("should write lowering when-expression fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("value-producing when lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        let mut stored_join_locals = routine
            .instructions
            .iter()
            .filter_map(|instr| match instr.kind {
                LoweredInstrKind::StoreLocal { local, .. } => Some(local),
                _ => None,
            })
            .collect::<Vec<_>>();
        stored_join_locals.sort_by_key(|local| local.0);
        stored_join_locals.dedup_by_key(|local| local.0);

        assert_eq!(stored_join_locals.len(), 1);
        assert_eq!(routine.body_result, Some(stored_join_locals[0]));
        assert!(
            routine
                .blocks
                .iter()
                .any(|block| matches!(block.terminator, Some(LoweredTerminator::Branch { .. }))),
            "value-producing when should branch explicitly"
        );
        assert!(
            routine
                .blocks
                .iter()
                .filter(|block| matches!(block.terminator, Some(LoweredTerminator::Jump { .. })))
                .count()
                >= 2,
            "value-producing when branches should jump into a shared join block"
        );
    }

    #[test]
    fn when_statement_lowering_keeps_a_three_block_shape_for_single_case_fallthrough() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_when_stmt_shape_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(flag: bol): int = {\n    when(flag) {\n        case(true) { 1 }\n    }\n    return 2\n}",
        )
        .expect("should write lowering when shape fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("statement-style when lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert_eq!(routine.blocks.len(), 3);
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(0)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Branch {
                condition: crate::LoweredLocalId(2),
                then_block: crate::LoweredBlockId(1),
                else_block: crate::LoweredBlockId(2),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(1)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(2),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(2)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Return {
                value: Some(crate::LoweredLocalId(4)),
            })
        );
    }

    #[test]
    fn when_expression_lowering_keeps_branch_default_and_join_block_shape() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_when_expr_shape_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "var yes: int = 1\nvar no: int = 2\nfun[] main(flag: bol): int = {\n    when(flag) {\n        case(true) { yes }\n        * { no }\n    }\n}",
        )
        .expect("should write lowering when-expression shape fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("value-producing when lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert_eq!(routine.blocks.len(), 4);
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(0)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Branch {
                condition: crate::LoweredLocalId(2),
                then_block: crate::LoweredBlockId(2),
                else_block: crate::LoweredBlockId(3),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(2)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(1),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(3)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(1),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(1)).and_then(|block| block.terminator.clone()),
            None
        );
        assert_eq!(routine.body_result, Some(crate::LoweredLocalId(3)));
    }

    #[test]
    fn loop_condition_lowering_keeps_header_body_and_exit_blocks() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_loop_shape_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(flag: bol, limit: int): int = {\n    loop(flag) {\n        var current: int = limit\n    }\n    return limit\n}",
        )
        .expect("should write lowering loop shape fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("condition loop lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert_eq!(routine.blocks.len(), 4);
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(0)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(1),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(1)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Branch {
                condition: crate::LoweredLocalId(2),
                then_block: crate::LoweredBlockId(2),
                else_block: crate::LoweredBlockId(3),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(2)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(1),
            })
        );
        assert!(matches!(
            routine.blocks.get(crate::LoweredBlockId(3)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Return { .. })
        ));
    }

    #[test]
    fn break_lowering_jumps_directly_to_the_loop_exit_block() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_break_shape_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] main(flag: bol, limit: int): int = {\n    loop(flag) {\n        break\n    }\n    return limit\n}",
        )
        .expect("should write lowering break shape fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("break lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert_eq!(routine.blocks.len(), 4);
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(0)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(1),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(1)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Branch {
                condition: crate::LoweredLocalId(2),
                then_block: crate::LoweredBlockId(2),
                else_block: crate::LoweredBlockId(3),
            })
        );
        assert_eq!(
            routine.blocks.get(crate::LoweredBlockId(2)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Jump {
                target: crate::LoweredBlockId(3),
            })
        );
        assert!(matches!(
            routine.blocks.get(crate::LoweredBlockId(3)).and_then(|block| block.terminator.clone()),
            Some(LoweredTerminator::Return { .. })
        ));
    }

    #[test]
    fn record_initializer_lowering_constructs_records_in_binding_and_call_contexts() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_record_init_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "typ User: { name: str, count: int }\nfun[] echo(user: User): User = { return user }\nfun[] main(): User = {\n    var current: User = { name = \"ok\", count = 1 }\n    return echo({ name = \"next\", count = 2 })\n}",
        )
        .expect("should write lowering record fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("record initializer lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let construct_types = routine
            .instructions
            .iter()
            .filter_map(|instr| match &instr.kind {
                LoweredInstrKind::ConstructRecord { type_id, fields } => {
                    Some((*type_id, fields.len()))
                }
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(construct_types.len(), 2);
        assert_eq!(construct_types[0], construct_types[1]);
        assert_eq!(construct_types[0].1, 2);
    }

    #[test]
    fn linear_container_lowering_constructs_array_vector_and_sequence_values() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_linear_container_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] make_arr(): arr[int, 3] = { return {1, 2, 3} }\nfun[] make_vec(): vec[int] = { return {1, 2, 3} }\nfun[] make_seq(): seq[int] = { return {1, 2, 3} }\n",
        )
        .expect("should write lowering linear-container fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("linear container lowering should succeed");

        for (routine_name, expected_kind, expected_len) in [
            ("make_arr", crate::control::LoweredLinearKind::Array, 3usize),
            ("make_vec", crate::control::LoweredLinearKind::Vector, 3usize),
            ("make_seq", crate::control::LoweredLinearKind::Sequence, 3usize),
        ] {
            let routine = lowered
                .entry_package()
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .expect("lowered routine should exist");
            let construct = routine.instructions.iter().find_map(|instr| match &instr.kind {
                LoweredInstrKind::ConstructLinear {
                    kind,
                    type_id: _,
                    elements,
                } => Some((*kind, elements.len())),
                _ => None,
            });

            assert_eq!(construct, Some((expected_kind, expected_len)));
        }
    }

    #[test]
    fn set_and_map_lowering_construct_explicit_aggregate_instructions() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_set_map_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "fun[] take_set(items: set[int, str]): str = { return items[1] }\nfun[] take_map(items: map[str, int]): int = { return items[\"US\"] }\nfun[] main(): int = {\n    var parts: set[int, str] = {1, \"two\"}\n    var counts: map[str, int] = {{\"US\", 45}, {\"DE\", 82}}\n    var current: str = take_set(parts)\n    return take_map(counts)\n}\n",
        )
        .expect("should write lowering set/map fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("set/map lowering should succeed");

        let routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let set_instr = routine.instructions.iter().find_map(|instr| match &instr.kind {
            LoweredInstrKind::ConstructSet { members, .. } => Some(members.len()),
            _ => None,
        });
        let map_instr = routine.instructions.iter().find_map(|instr| match &instr.kind {
            LoweredInstrKind::ConstructMap { entries, .. } => Some(entries.len()),
            _ => None,
        });

        assert_eq!(set_instr, Some(2));
        assert_eq!(map_instr, Some(2));
    }

    #[test]
    fn entry_variant_lowering_supports_payload_access_and_entry_construction() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_entry_variant_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "typ Color: ent = {\n    var BLUE: str = \"#0037cd\";\n    var RED: str = \"#ff0000\";\n}\nfun[] payload(): str = {\n    return Color.BLUE;\n}\nfun[] typed(): Color = {\n    return Color.RED;\n}\n",
        )
        .expect("should write lowering entry fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("entry variant lowering should succeed");

        let payload_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "payload")
            .expect("payload routine should exist");
        let typed_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "typed")
            .expect("typed routine should exist");

        assert!(
            payload_routine.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::Const(LoweredOperand::Str(_))
            )),
            "entry payload access should lower the default payload expression"
        );
        assert!(
            typed_routine.instructions.iter().any(|instr| matches!(
                &instr.kind,
                LoweredInstrKind::ConstructEntry {
                    variant,
                    payload: Some(_),
                    ..
                } if variant == "RED"
            )),
            "typed entry construction should lower to an explicit ConstructEntry instruction"
        );
    }

    #[test]
    fn nil_lowering_constructs_optional_and_error_shell_values() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_nil_shells_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "ali MaybeText: opt[str]\nali Failure: err[str]\nfun[] make(): MaybeText = { return nil }\nfun[] fail(): int / Failure = { report nil }\n",
        )
        .expect("should write lowering nil fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("nil lowering should succeed");

        let make_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "make")
            .expect("make routine should exist");
        let fail_routine = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "fail")
            .expect("fail routine should exist");

        assert!(
            make_routine.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructOptional { value: None, .. }
            )),
            "nil in an optional context should lower to an explicit empty optional constructor"
        );
        assert!(
            fail_routine.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructError { value: None, .. }
            )),
            "nil in a typed error context should lower to an explicit empty error constructor"
        );
    }

    #[test]
    fn unwrap_lowering_uses_explicit_shell_unwrap_instructions() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_unwrap_shells_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "ali MaybeText: opt[str]\nali Failure: err[str]\nfun[] from_optional(value: MaybeText): str = { return value! }\nfun[] from_error(value: Failure): str = { return value! }\n",
        )
        .expect("should write lowering unwrap fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("unwrap lowering should succeed");

        for routine_name in ["from_optional", "from_error"] {
            let routine = lowered
                .entry_package()
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .unwrap_or_else(|| panic!("{routine_name} routine should exist"));

            assert!(
                routine.instructions.iter().any(|instr| matches!(
                    instr.kind,
                    LoweredInstrKind::UnwrapShell { .. }
                )),
                "{routine_name} should lower postfix unwrap into an explicit shell-unwrapping instruction"
            );
        }
    }

    #[test]
    fn alias_shell_contexts_lower_to_concrete_runtime_shell_operations() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_shell_alias_{stamp}"));
        let app_dir = root.join("app");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&app_dir).expect("should create app dir");
        fs::create_dir_all(&shared_dir).expect("should create shared dir");
        fs::write(
            shared_dir.join("lib.fol"),
            "ali RemoteText: opt[str]\nali RemoteFailure: err[str]\nfun[exp] imported_wrap(): RemoteText = { return \"shared\" }\nfun[exp] imported_fail(): int / RemoteFailure = { report \"shared\" }\n",
        )
        .expect("should write shared package");
        fs::write(
            app_dir.join("main.fol"),
            "use shared: loc = {\"../shared\"}\nali LocalText: opt[str]\nali LocalFailure: err[str]\nfun[] local_wrap(): LocalText = { return \"local\" }\nfun[] local_fail(): int / LocalFailure = { report \"local\" }\nfun[] imported_wrap_main(): shared::RemoteText = { return \"entry\" }\nfun[] imported_fail_main(): int / shared::RemoteFailure = { report \"entry\" }\n",
        )
        .expect("should write app package");

        let mut stream =
            FileStream::from_folder(app_dir.to_str().expect("utf8 temp path")).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("shell alias lowering should succeed");

        let app_package = lowered.entry_package();
        for routine_name in [
            "local_wrap",
            "local_fail",
            "imported_wrap_main",
            "imported_fail_main",
        ] {
            let routine = app_package
                .routine_decls
                .values()
                .find(|routine| routine.name == routine_name)
                .unwrap_or_else(|| panic!("{routine_name} routine should exist"));

            let has_shell_ctor = routine.instructions.iter().any(|instr| {
                matches!(
                    instr.kind,
                    LoweredInstrKind::ConstructOptional { value: Some(_), .. }
                        | LoweredInstrKind::ConstructError { value: Some(_), .. }
                )
            });
            assert!(
                has_shell_ctor,
                "{routine_name} should lower alias-backed shell contexts into concrete runtime shell construction"
            );
        }
    }

    #[test]
    fn shell_payload_lifting_lowers_to_explicit_runtime_wrappers() {
        let fixture = std::env::temp_dir().join(format!(
            "fol_lower_shell_lifts_{}.fol",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system clock should be monotonic enough for tmp names")
                .as_nanos()
        ));
        std::fs::write(
            &fixture,
            "ali MaybeText: opt[str]\nali Failure: err[str]\nfun[] echo(value: MaybeText): MaybeText = { return value }\nfun[] direct(): MaybeText = { return \"return\" }\nfun[] main(): MaybeText = {\n    var local: MaybeText = \"bind\"\n    return echo(\"call\")\n}\nfun[] fail(): int / Failure = { report \"broken\" }\n",
        )
        .expect("should write lowering shell lift fixture");

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
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("shell lifting lowering should succeed");

        let direct = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "direct")
            .expect("direct routine should exist");
        let main = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");
        let fail = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "fail")
            .expect("fail routine should exist");

        assert!(
            direct.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructOptional { value: Some(_), .. }
            )),
            "return payload lifting should lower to an explicit optional constructor"
        );
        assert!(
            main.instructions.iter().filter(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructOptional { value: Some(_), .. }
            )).count() >= 2,
            "binding and call payload lifting should each lower to explicit optional constructors"
        );
        assert!(
            fail.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructError { value: Some(_), .. }
            )),
            "report payload lifting should lower to an explicit error constructor"
        );
    }

    #[test]
    fn aggregate_container_and_shell_lowering_stays_aligned_across_local_and_imported_surfaces() {
        use std::fs;
        use std::time::{SystemTime, UNIX_EPOCH};

        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock should be monotonic enough for tmp path")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("fol_lower_parity_mix_{stamp}"));
        let app_dir = root.join("app");
        let shared_dir = root.join("shared");
        fs::create_dir_all(&app_dir).expect("should create app dir");
        fs::create_dir_all(&shared_dir).expect("should create shared dir");
        fs::write(
            shared_dir.join("lib.fol"),
            "ali RemoteText: opt[str]\ntyp RemoteUser: { name: str, count: int }\nfun[exp] keep_remote(user: RemoteUser): RemoteUser = { return user }\n",
        )
        .expect("should write shared package");
        fs::write(
            app_dir.join("main.fol"),
            "use shared: loc = {\"../shared\"}\nali LocalText: opt[str]\ntyp LocalUser: { name: str, count: int }\nfun[] main(): shared::RemoteUser = {\n    var local: LocalText = \"ok\"\n    var remote_label: shared::RemoteText = \"shared\"\n    var local_user: LocalUser = { name = \"local\", count = 1 }\n    var remote_user: shared::RemoteUser = { name = \"remote\", count = 2 }\n    var ids: seq[int] = {1, 2, 3}\n    return shared::keep_remote(remote_user)\n}\n",
        )
        .expect("should write app package");

        let mut stream =
            FileStream::from_folder(app_dir.to_str().expect("utf8 temp path")).expect("Should open lowering fixture");
        let mut lexer = fol_lexer::lexer::stage3::Elements::init(&mut stream);
        let mut parser = AstParser::new();
        let syntax = parser
            .parse_package(&mut lexer)
            .expect("Lowering fixture should parse");
        let resolved = resolve_workspace(syntax).expect("Lowering fixture should resolve");
        let typed = Typechecker::new()
            .check_resolved_workspace(resolved)
            .expect("Lowering fixture should typecheck");
        let lowered = crate::LoweringSession::new(typed)
            .lower_workspace()
            .expect("mixed parity lowering should succeed");

        let main = lowered
            .entry_package()
            .routine_decls
            .values()
            .find(|routine| routine.name == "main")
            .expect("main routine should exist");

        assert!(
            main.instructions.iter().filter(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructOptional { value: Some(_), .. }
            )).count() >= 2,
            "local and imported shell aliases should both lower to explicit shell constructors"
        );
        assert!(
            main.instructions.iter().filter(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructRecord { .. }
            )).count() >= 2,
            "local and imported record contexts should both lower to explicit record constructors"
        );
        assert!(
            main.instructions.iter().any(|instr| matches!(
                instr.kind,
                LoweredInstrKind::ConstructLinear { .. }
            )),
            "container literals should keep lowering alongside aggregate and shell surfaces"
        );
    }

    #[test]
    fn unsupported_lowering_surfaces_report_explicit_boundary_messages() {
        let nil_error = lower_fixture_error(
            "fun[] main(): int = {\n    return nil;\n}\n",
        );
        assert_eq!(nil_error.kind(), LoweringErrorKind::Unsupported);
        assert!(
            nil_error
                .message()
                .contains("nil lowering requires an expected opt[...] or err[...] runtime type in lowered V1")
        );

        let operator_error = lower_fixture_error(
            "fun[] main(): int = {\n    return 1 + 2;\n}\n",
        );
        assert_eq!(operator_error.kind(), LoweringErrorKind::Unsupported);
        assert!(
            operator_error
                .message()
                .contains("binary operator lowering for 'add' lands in a later lowering slice")
        );

        let loop_error = lower_fixture_error(
            "fun[] main(items: seq[int]): int = {\n    loop(item in items) {\n        break;\n    }\n    return 0;\n}\n",
        );
        assert_eq!(loop_error.kind(), LoweringErrorKind::Unsupported);
        assert!(
            loop_error
                .message()
                .contains("iteration loop lowering is not part of the current lowered V1 control-flow milestone")
        );

        let entry_error = lower_fixture_error(
            "typ Status: ent = {\n    var OK: int = 1;\n}\nfun[] main(): Status = {\n    return Status.OK;\n}\n",
        );
        assert_eq!(entry_error.kind(), LoweringErrorKind::Unsupported);
        assert!(
            entry_error
                .message()
                .contains("entry construction lowering for variant 'OK' lands in the pending aggregate slice")
        );
    }

    #[test]
    fn audited_v1_lowering_boundaries_fail_with_explicit_messages() {
        let cases = [
            (
                crate::UnsupportedLoweringSurface::UnaryOperators,
                "fun[] main(): int = {\n    return -1;\n}\n",
                "unary operator lowering for 'neg' lands in a later lowering slice",
            ),
            (
                crate::UnsupportedLoweringSurface::BinaryOperators,
                "fun[] main(): int = {\n    return 1 + 2;\n}\n",
                "binary operator lowering for 'add' lands in a later lowering slice",
            ),
            (
                crate::UnsupportedLoweringSurface::TypeMatchingWhenOf,
                "fun classify(value: any): int = {\n    when(value) {\n        of(int) { return 1; }\n        { return 0; }\n    }\n}\n",
                "type-matching when/of branches are not lowered in this slice yet",
            ),
            (
                crate::UnsupportedLoweringSurface::IterationLoops,
                "fun[] main(items: seq[int]): int = {\n    loop(item in items) {\n        break;\n    }\n    return 0;\n}\n",
                "iteration loop lowering is not part of the current lowered V1 control-flow milestone",
            ),
            (
                crate::UnsupportedLoweringSurface::ProcedureStyleFreeCalls,
                "pro finish(): non = {\n    return;\n}\nfun[] main(): int = {\n    finish();\n    return 0;\n}\n",
                "procedure-style calls without a value result are not lowered in this slice yet: 'finish'",
            ),
            (
                crate::UnsupportedLoweringSurface::ProcedureStyleMethodCalls,
                "typ Box: { value: int }\npro (Box)touch(): non = {\n    return;\n}\nfun[] main(box: Box): int = {\n    box.touch();\n    return box.value;\n}\n",
                "procedure-style method calls without a value result are not lowered in this slice yet: 'touch'",
            ),
            (
                crate::UnsupportedLoweringSurface::EntryVariantConstruction,
                "typ Status: ent = {\n    var OK: int = 1;\n}\nfun[] main(): Status = {\n    return Status.OK;\n}\n",
                "entry construction lowering for variant 'OK' lands in the pending aggregate slice",
            ),
        ];

        assert_eq!(crate::v1_lowering_boundaries().len(), cases.len());

        for (surface, source, expected_message) in cases {
            let error = lower_fixture_error(source);
            assert_eq!(
                error.kind(),
                LoweringErrorKind::Unsupported,
                "expected unsupported lowering for boundary '{}'",
                surface.label()
            );
            assert!(
                error.message().contains(expected_message),
                "expected lowering boundary '{}' to mention '{expected_message}', got: {:?}",
                surface.label(),
                error
            );
        }
    }
}
