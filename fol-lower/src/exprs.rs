use crate::{
    control::{LoweredInstr, LoweredInstrKind, LoweredLocal, LoweredOperand},
    ids::{LoweredBlockId, LoweredInstrId, LoweredLocalId, LoweredTypeId},
    LoweredGlobalId, LoweredPackage, LoweredRoutine, LoweredRoutineId, LoweredWorkspace,
    LoweringError, LoweringErrorKind,
};
use fol_parser::ast::{AstNode, Literal};
use fol_resolver::{
    MountedSymbolProvenance, PackageIdentity, ResolvedSymbol, ScopeId, SourceUnitId, SymbolId,
    SymbolKind,
};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoweredValue {
    pub local_id: LoweredLocalId,
    pub type_id: LoweredTypeId,
}

#[derive(Debug, Default)]
pub(crate) struct WorkspaceDeclIndex {
    globals: BTreeMap<(PackageIdentity, SymbolId), LoweredGlobalId>,
    routines: BTreeMap<(PackageIdentity, SymbolId), LoweredRoutineId>,
}

impl WorkspaceDeclIndex {
    pub(crate) fn from_packages(packages: &BTreeMap<PackageIdentity, LoweredPackage>) -> Self {
        let mut index = Self::default();
        for package in packages.values() {
            index.extend_package(package);
        }
        index
    }

    pub(crate) fn build(workspace: &LoweredWorkspace) -> Self {
        let mut index = Self::default();
        for package in workspace.packages() {
            index.extend_package(package);
        }
        index
    }

    pub(crate) fn extend_package(&mut self, package: &LoweredPackage) {
        for global in package.global_decls.values() {
            self.globals
                .insert((package.identity.clone(), global.symbol_id), global.id);
        }
        for routine in package.routine_decls.values() {
            if let Some(symbol_id) = routine.symbol_id {
                self.routines
                    .insert((package.identity.clone(), symbol_id), routine.id);
            }
        }
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
}

pub(crate) struct RoutineCursor<'a> {
    routine: &'a mut LoweredRoutine,
    block_id: LoweredBlockId,
    next_local_index: usize,
    next_instr_index: usize,
}

impl<'a> RoutineCursor<'a> {
    pub(crate) fn new(routine: &'a mut LoweredRoutine, block_id: LoweredBlockId) -> Self {
        Self {
            next_local_index: routine.locals.len(),
            next_instr_index: routine.instructions.len(),
            routine,
            block_id,
        }
    }

    pub(crate) fn allocate_local(
        &mut self,
        type_id: LoweredTypeId,
        name: Option<String>,
    ) -> LoweredLocalId {
        let local_id = self.routine.locals.push(LoweredLocal {
            id: LoweredLocalId(self.next_local_index),
            type_id: Some(type_id),
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
        Ok(LoweredValue { local_id, type_id })
    }

    pub(crate) fn lower_identifier_reference(
        &mut self,
        current_identity: &PackageIdentity,
        decl_index: &WorkspaceDeclIndex,
        resolved_symbol: &ResolvedSymbol,
        result_type: LoweredTypeId,
    ) -> Result<LoweredValue, LoweringError> {
        if let Some(local_id) = self.routine.local_symbols.get(&resolved_symbol.id).copied() {
            let result_local = self.allocate_local(result_type, None);
            self.push_instr(
                Some(result_local),
                LoweredInstrKind::LoadLocal { local: local_id },
            )?;
            return Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
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
        let result_local = self.allocate_local(result_type, None);
        self.push_instr(Some(result_local), LoweredInstrKind::LoadGlobal { global: global_id })?;
        Ok(LoweredValue {
            local_id: result_local,
            type_id: result_type,
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

    for node in nodes {
        if let Some(value) = lower_body_node(
            typed_package,
            checked_type_map,
            &current_identity,
            decl_index,
            &mut cursor,
            source_unit_id,
            scope_id,
            node,
        )? {
            cursor.routine.body_result = Some(value.local_id);
        }
    }

    Ok(())
}

fn lower_body_node(
    typed_package: &fol_typecheck::TypedPackage,
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
            Some(value) => lower_expression(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                value,
            )
            .map(Some),
            None => Ok(None),
        },
        _ => lower_expression(
            typed_package,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            node,
        )
        .map(Some),
    }
}

fn lower_local_binding(
    typed_package: &fol_typecheck::TypedPackage,
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
    let Some(symbol_id) = crate::decls::find_symbol_in_exact_scope(
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
    let local_id = cursor.allocate_local(type_id, Some(name.to_string()));
    cursor.routine.local_symbols.insert(symbol_id, local_id);

    if let Some(value) = value {
        let lowered_value = lower_expression(
            typed_package,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            value,
        )?;
        cursor.push_instr(
            None,
            LoweredInstrKind::StoreLocal {
                local: local_id,
                value: lowered_value.local_id,
            },
        )?;
        Ok(Some(LoweredValue { local_id, type_id }))
    } else {
        Ok(Some(LoweredValue { local_id, type_id }))
    }
}

fn lower_expression(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
    node: &AstNode,
) -> Result<LoweredValue, LoweringError> {
    match node {
        AstNode::Literal(literal) => {
            let type_id = literal_type_id(typed_package, checked_type_map, literal).ok_or_else(|| {
                LoweringError::with_kind(
                    LoweringErrorKind::InvalidInput,
                    "literal expression does not retain a lowering-owned type",
                )
            })?;
            cursor.lower_literal(literal, type_id)
        }
        AstNode::Assignment { target, value } => {
            let lowered_value = lower_expression(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
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
        AstNode::FunctionCall { syntax_id, name, args } => lower_function_call(
            typed_package,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            *syntax_id,
            fol_resolver::ReferenceKind::FunctionCall,
            name,
            args,
        ),
        AstNode::QualifiedFunctionCall { path, args } => lower_function_call(
            typed_package,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            path.syntax_id(),
            fol_resolver::ReferenceKind::QualifiedFunctionCall,
            &path.joined(),
            args,
        ),
        AstNode::MethodCall { object, method, args } => {
            let receiver = lower_expression(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                object,
            )?;
            let (callee, result_type) = resolve_method_target(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                method,
                receiver.type_id,
            )?;
            let mut lowered_args = vec![receiver.local_id];
            lowered_args.extend(
                args.iter()
                    .map(|arg| {
                        lower_expression(
                            typed_package,
                            checked_type_map,
                            current_identity,
                            decl_index,
                            cursor,
                            source_unit_id,
                            arg,
                        )
                        .map(|value| value.local_id)
                    })
                    .collect::<Result<Vec<_>, _>>()?,
            );
            let result_local = cursor.allocate_local(result_type, None);
            cursor.push_instr(
                Some(result_local),
                LoweredInstrKind::Call {
                    callee,
                    args: lowered_args,
                },
            )?;
            Ok(LoweredValue {
                local_id: result_local,
                type_id: result_type,
            })
        }
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
        AstNode::Commented { node, .. } => lower_expression(
            typed_package,
            checked_type_map,
            current_identity,
            decl_index,
            cursor,
            source_unit_id,
            node,
        ),
        other => Err(LoweringError::with_kind(
            LoweringErrorKind::Unsupported,
            format!(
                "expression lowering for '{}' is not implemented in this slice yet",
                describe_expression(other)
            ),
        )),
    }
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
        AstNode::Return { .. } => "return".to_string(),
        AstNode::When { .. } => "when".to_string(),
        AstNode::Loop { .. } => "loop".to_string(),
        _ => "expression".to_string(),
    }
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

fn lower_function_call(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    cursor: &mut RoutineCursor<'_>,
    source_unit_id: SourceUnitId,
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
    let lowered_args = args
        .iter()
        .map(|arg| {
            lower_expression(
                typed_package,
                checked_type_map,
                current_identity,
                decl_index,
                cursor,
                source_unit_id,
                arg,
            )
            .map(|value| value.local_id)
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result_local = cursor.allocate_local(result_type, None);
    cursor.push_instr(
        Some(result_local),
        LoweredInstrKind::Call {
            callee,
            args: lowered_args,
        },
    )?;
    Ok(LoweredValue {
        local_id: result_local,
        type_id: result_type,
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

fn resolve_method_target(
    typed_package: &fol_typecheck::TypedPackage,
    checked_type_map: &BTreeMap<fol_typecheck::CheckedTypeId, LoweredTypeId>,
    current_identity: &PackageIdentity,
    decl_index: &WorkspaceDeclIndex,
    method: &str,
    receiver_type: LoweredTypeId,
) -> Result<(LoweredRoutineId, LoweredTypeId), LoweringError> {
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
        matches.push((routine_id, return_type));
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

#[cfg(test)]
mod tests {
    use super::{RoutineCursor, WorkspaceDeclIndex};
    use crate::{
        types::{LoweredBuiltinType, LoweredTypeTable},
        LoweredBlock, LoweredGlobal, LoweredInstrKind, LoweredOperand, LoweredPackage,
        LoweredRoutine, LoweredWorkspace, LoweringErrorKind,
    };
    use fol_parser::ast::AstParser;
    use fol_parser::ast::Literal;
    use fol_resolver::{resolve_workspace, PackageIdentity, PackageSourceKind, SourceUnitId, SymbolKind};
    use fol_stream::FileStream;
    use fol_typecheck::Typechecker;
    use std::collections::BTreeMap;

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
        let workspace = LoweredWorkspace::new(
            identity.clone(),
            packages,
            crate::LoweredTypeTable::new(),
            crate::LoweredSourceMap::new(),
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
                LoweredInstrKind::Call { callee, args } => Some((*callee, args.clone())),
                _ => None,
            })
            .expect("method body should contain a lowered call");

        assert_eq!(call.1.len(), 1);
    }
}
