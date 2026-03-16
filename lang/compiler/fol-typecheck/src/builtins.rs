use crate::types::{BuiltinType, CheckedTypeId, TypeTable};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuiltinTypeIds {
    pub int: CheckedTypeId,
    pub float: CheckedTypeId,
    pub bool_: CheckedTypeId,
    pub char_: CheckedTypeId,
    pub str_: CheckedTypeId,
    pub never: CheckedTypeId,
}

impl BuiltinTypeIds {
    pub fn install(table: &mut TypeTable) -> Self {
        Self {
            int: table.intern_builtin(BuiltinType::Int),
            float: table.intern_builtin(BuiltinType::Float),
            bool_: table.intern_builtin(BuiltinType::Bool),
            char_: table.intern_builtin(BuiltinType::Char),
            str_: table.intern_builtin(BuiltinType::Str),
            never: table.intern_builtin(BuiltinType::Never),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BuiltinTypeIds;
    use crate::types::{BuiltinType, CheckedType, TypeTable};

    #[test]
    fn builtin_type_ids_install_all_v1_scalar_types_once() {
        let mut table = TypeTable::new();
        let builtins = BuiltinTypeIds::install(&mut table);

        assert_eq!(table.len(), 6);
        assert_eq!(
            table.get(builtins.int),
            Some(&CheckedType::Builtin(BuiltinType::Int))
        );
        assert_eq!(
            table.get(builtins.float),
            Some(&CheckedType::Builtin(BuiltinType::Float))
        );
        assert_eq!(
            table.get(builtins.bool_),
            Some(&CheckedType::Builtin(BuiltinType::Bool))
        );
        assert_eq!(
            table.get(builtins.char_),
            Some(&CheckedType::Builtin(BuiltinType::Char))
        );
        assert_eq!(
            table.get(builtins.str_),
            Some(&CheckedType::Builtin(BuiltinType::Str))
        );
        assert_eq!(
            table.get(builtins.never),
            Some(&CheckedType::Builtin(BuiltinType::Never))
        );
    }

    #[test]
    fn builtin_type_ids_reuse_existing_builtin_slots() {
        let mut table = TypeTable::new();
        let first = BuiltinTypeIds::install(&mut table);
        let second = BuiltinTypeIds::install(&mut table);

        assert_eq!(first, second);
        assert_eq!(table.len(), 6);
    }
}
