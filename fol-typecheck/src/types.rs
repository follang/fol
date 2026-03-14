use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CheckedTypeId(pub usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum BuiltinType {
    Int,
    Float,
    Bool,
    Char,
    Str,
    Never,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CheckedType {
    Builtin(BuiltinType),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TypeTable {
    types: Vec<CheckedType>,
    canonical_ids: BTreeMap<CheckedType, CheckedTypeId>,
}

impl TypeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn len(&self) -> usize {
        self.types.len()
    }

    pub fn is_empty(&self) -> bool {
        self.types.is_empty()
    }

    pub fn get(&self, id: CheckedTypeId) -> Option<&CheckedType> {
        self.types.get(id.0)
    }

    pub fn intern(&mut self, ty: CheckedType) -> CheckedTypeId {
        if let Some(id) = self.canonical_ids.get(&ty) {
            return *id;
        }

        let id = CheckedTypeId(self.types.len());
        self.types.push(ty.clone());
        self.canonical_ids.insert(ty, id);
        id
    }

    pub fn intern_builtin(&mut self, builtin: BuiltinType) -> CheckedTypeId {
        self.intern(CheckedType::Builtin(builtin))
    }
}

#[cfg(test)]
mod tests {
    use super::{BuiltinType, CheckedType, TypeTable};

    #[test]
    fn type_table_interns_builtin_types_canonically() {
        let mut table = TypeTable::new();

        let first = table.intern_builtin(BuiltinType::Int);
        let second = table.intern_builtin(BuiltinType::Int);
        let third = table.intern_builtin(BuiltinType::Str);

        assert_eq!(first, second);
        assert_ne!(first, third);
        assert_eq!(table.len(), 2);
        assert_eq!(table.get(first), Some(&CheckedType::Builtin(BuiltinType::Int)));
        assert_eq!(table.get(third), Some(&CheckedType::Builtin(BuiltinType::Str)));
    }
}
