use crate::{
    ids::{LoweredGlobalId, LoweredPackageId, LoweredRoutineId, LoweredTypeId},
    types::LoweredTypeTable,
};
use fol_parser::ast::SyntaxOrigin;
use fol_resolver::{MountedSymbolProvenance, PackageIdentity, SourceUnitId, SymbolId};
use fol_typecheck::CheckedTypeId;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum LoweredSourceSymbol {
    Package(LoweredPackageId),
    Global(LoweredGlobalId),
    Routine(LoweredRoutineId),
    Type(LoweredTypeId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredSourceMapEntry {
    pub symbol: LoweredSourceSymbol,
    pub origin: SyntaxOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredFieldLayout {
    pub name: String,
    pub type_id: LoweredTypeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredVariantLayout {
    pub name: String,
    pub payload_type: Option<LoweredTypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoweredTypeDeclKind {
    Alias { target_type: LoweredTypeId },
    Record { fields: Vec<LoweredFieldLayout> },
    Entry { variants: Vec<LoweredVariantLayout> },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredTypeDecl {
    pub symbol_id: SymbolId,
    pub source_unit_id: SourceUnitId,
    pub name: String,
    pub runtime_type: LoweredTypeId,
    pub kind: LoweredTypeDeclKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredSourceUnit {
    pub source_unit_id: SourceUnitId,
    pub path: String,
    pub package: String,
    pub namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredSymbolOwnership {
    pub symbol_id: SymbolId,
    pub source_unit_id: SourceUnitId,
    pub owning_package: PackageIdentity,
    pub mounted_from: Option<MountedSymbolProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LoweredSourceMap {
    entries: Vec<LoweredSourceMapEntry>,
}

impl LoweredSourceMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entries(&self) -> &[LoweredSourceMapEntry] {
        &self.entries
    }

    pub fn push(&mut self, entry: LoweredSourceMapEntry) {
        self.entries.push(entry);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredPackage {
    pub id: LoweredPackageId,
    pub identity: PackageIdentity,
    pub globals: Vec<LoweredGlobalId>,
    pub routines: Vec<LoweredRoutineId>,
    pub types: Vec<LoweredTypeId>,
    pub exported_symbols: Vec<String>,
    pub source_units: Vec<LoweredSourceUnit>,
    pub symbol_ownership: BTreeMap<SymbolId, LoweredSymbolOwnership>,
    pub checked_type_map: BTreeMap<CheckedTypeId, LoweredTypeId>,
    pub routine_signatures: BTreeMap<SymbolId, LoweredTypeId>,
    pub type_decls: BTreeMap<SymbolId, LoweredTypeDecl>,
}

impl LoweredPackage {
    pub fn new(id: LoweredPackageId, identity: PackageIdentity) -> Self {
        Self {
            id,
            identity,
            globals: Vec::new(),
            routines: Vec::new(),
            types: Vec::new(),
            exported_symbols: Vec::new(),
            source_units: Vec::new(),
            symbol_ownership: BTreeMap::new(),
            checked_type_map: BTreeMap::new(),
            routine_signatures: BTreeMap::new(),
            type_decls: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredWorkspace {
    entry_identity: PackageIdentity,
    packages: BTreeMap<PackageIdentity, LoweredPackage>,
    type_table: LoweredTypeTable,
    source_map: LoweredSourceMap,
}

impl LoweredWorkspace {
    pub fn new(
        entry_identity: PackageIdentity,
        packages: BTreeMap<PackageIdentity, LoweredPackage>,
        type_table: LoweredTypeTable,
        source_map: LoweredSourceMap,
    ) -> Self {
        Self {
            entry_identity,
            packages,
            type_table,
            source_map,
        }
    }

    pub fn entry_identity(&self) -> &PackageIdentity {
        &self.entry_identity
    }

    pub fn entry_package(&self) -> &LoweredPackage {
        self.packages
            .get(&self.entry_identity)
            .expect("lowered workspace should retain its entry package")
    }

    pub fn package(&self, identity: &PackageIdentity) -> Option<&LoweredPackage> {
        self.packages.get(identity)
    }

    pub fn packages(&self) -> impl Iterator<Item = &LoweredPackage> {
        self.packages.values()
    }

    pub fn package_count(&self) -> usize {
        self.packages.len()
    }

    pub fn type_table(&self) -> &LoweredTypeTable {
        &self.type_table
    }

    pub fn source_map(&self) -> &LoweredSourceMap {
        &self.source_map
    }
}

#[cfg(test)]
mod tests {
    use super::{LoweredPackage, LoweredSourceMap, LoweredSourceUnit, LoweredWorkspace};
    use crate::ids::LoweredPackageId;
    use crate::types::LoweredTypeTable;
    use fol_resolver::{PackageIdentity, PackageSourceKind, SourceUnitId};
    use std::collections::BTreeMap;

    fn identity(name: &str, kind: PackageSourceKind) -> PackageIdentity {
        PackageIdentity {
            source_kind: kind,
            canonical_root: format!("/workspace/{name}"),
            display_name: name.to_string(),
        }
    }

    #[test]
    fn lowered_workspace_shell_keeps_entry_package_and_count() {
        let entry_identity = identity("app", PackageSourceKind::Entry);
        let shared_identity = identity("shared", PackageSourceKind::Local);

        let mut entry_package = LoweredPackage::new(LoweredPackageId(0), entry_identity.clone());
        entry_package.source_units.push(LoweredSourceUnit {
            source_unit_id: SourceUnitId(0),
            path: "app/main.fol".to_string(),
            package: "app".to_string(),
            namespace: "app".to_string(),
        });
        let mut packages = BTreeMap::new();
        packages.insert(entry_identity.clone(), entry_package);
        packages.insert(
            shared_identity.clone(),
            LoweredPackage::new(LoweredPackageId(1), shared_identity),
        );

        let workspace = LoweredWorkspace::new(
            entry_identity.clone(),
            packages,
            LoweredTypeTable::new(),
            LoweredSourceMap::default(),
        );

        assert_eq!(workspace.entry_identity(), &entry_identity);
        assert_eq!(workspace.entry_package().id, LoweredPackageId(0));
        assert_eq!(workspace.package_count(), 2);
        assert_eq!(workspace.entry_package().source_units.len(), 1);
        assert!(workspace.type_table().is_empty());
    }

    #[test]
    fn lowered_source_map_shell_starts_empty() {
        let source_map = LoweredSourceMap::new();

        assert!(source_map.is_empty());
        assert!(source_map.entries().is_empty());
    }
}
