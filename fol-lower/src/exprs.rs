use crate::{
    control::{LoweredInstr, LoweredInstrKind, LoweredLocal, LoweredOperand},
    ids::{LoweredBlockId, LoweredInstrId, LoweredLocalId, LoweredTypeId},
    LoweredGlobalId, LoweredPackage, LoweredRoutine, LoweredRoutineId, LoweredWorkspace,
    LoweringError, LoweringErrorKind,
};
use fol_parser::ast::Literal;
use fol_resolver::{MountedSymbolProvenance, PackageIdentity, ResolvedSymbol, SymbolId};
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
}
