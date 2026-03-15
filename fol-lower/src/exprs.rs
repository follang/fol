use crate::{
    control::{LoweredInstr, LoweredInstrKind, LoweredLocal, LoweredOperand},
    ids::{LoweredBlockId, LoweredInstrId, LoweredLocalId, LoweredTypeId},
    LoweredRoutine, LoweringError, LoweringErrorKind,
};
use fol_parser::ast::Literal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct LoweredValue {
    pub local_id: LoweredLocalId,
    pub type_id: LoweredTypeId,
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
}

#[cfg(test)]
mod tests {
    use super::RoutineCursor;
    use crate::{
        types::{LoweredBuiltinType, LoweredTypeTable},
        LoweredBlock, LoweredInstrKind, LoweredOperand, LoweredRoutine, LoweringErrorKind,
    };
    use fol_parser::ast::Literal;

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
}
