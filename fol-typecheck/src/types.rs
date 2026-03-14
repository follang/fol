use std::collections::BTreeMap;

use fol_resolver::SymbolId;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DeclaredTypeKind {
    Type,
    Alias,
    GenericParameter,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RoutineType {
    pub params: Vec<CheckedTypeId>,
    pub return_type: CheckedTypeId,
    pub error_type: Option<CheckedTypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CheckedType {
    Builtin(BuiltinType),
    Declared {
        symbol: SymbolId,
        name: String,
        kind: DeclaredTypeKind,
    },
    Array {
        element_type: CheckedTypeId,
        size: Option<usize>,
    },
    Vector {
        element_type: CheckedTypeId,
    },
    Sequence {
        element_type: CheckedTypeId,
    },
    Set {
        member_types: Vec<CheckedTypeId>,
    },
    Map {
        key_type: CheckedTypeId,
        value_type: CheckedTypeId,
    },
    Optional {
        inner: CheckedTypeId,
    },
    Error {
        inner: Option<CheckedTypeId>,
    },
    Record {
        fields: BTreeMap<String, CheckedTypeId>,
    },
    Entry {
        variants: BTreeMap<String, Option<CheckedTypeId>>,
    },
    Routine(RoutineType),
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
    use super::{BuiltinType, CheckedType, DeclaredTypeKind, RoutineType, TypeTable};
    use fol_resolver::SymbolId;
    use std::collections::BTreeMap;

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

    #[test]
    fn type_table_canonicalizes_declared_and_structural_shapes() {
        let mut table = TypeTable::new();
        let int_id = table.intern_builtin(BuiltinType::Int);
        let declared = table.intern(CheckedType::Declared {
            symbol: SymbolId(4),
            name: "Point".to_string(),
            kind: DeclaredTypeKind::Type,
        });

        let mut fields = BTreeMap::new();
        fields.insert("x".to_string(), int_id);
        fields.insert("y".to_string(), int_id);
        let record_first = table.intern(CheckedType::Record {
            fields: fields.clone(),
        });
        let record_second = table.intern(CheckedType::Record { fields });
        let routine = table.intern(CheckedType::Routine(RoutineType {
            params: vec![declared, int_id],
            return_type: declared,
            error_type: None,
        }));

        assert_eq!(record_first, record_second);
        assert_ne!(declared, routine);
        assert_eq!(
            table.get(declared),
            Some(&CheckedType::Declared {
                symbol: SymbolId(4),
                name: "Point".to_string(),
                kind: DeclaredTypeKind::Type,
            })
        );
    }
}
