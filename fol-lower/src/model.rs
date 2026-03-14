use crate::ids::{LoweredGlobalId, LoweredPackageId, LoweredRoutineId, LoweredTypeId};
use fol_parser::ast::SyntaxOrigin;
use fol_resolver::PackageIdentity;
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
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweredWorkspace {
    entry_identity: PackageIdentity,
    packages: BTreeMap<PackageIdentity, LoweredPackage>,
    source_map: LoweredSourceMap,
}

impl LoweredWorkspace {
    pub fn new(
        entry_identity: PackageIdentity,
        packages: BTreeMap<PackageIdentity, LoweredPackage>,
        source_map: LoweredSourceMap,
    ) -> Self {
        Self {
            entry_identity,
            packages,
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

    pub fn source_map(&self) -> &LoweredSourceMap {
        &self.source_map
    }
}

#[cfg(test)]
mod tests {
    use super::{LoweredPackage, LoweredSourceMap, LoweredWorkspace};
    use crate::ids::LoweredPackageId;
    use fol_resolver::{PackageIdentity, PackageSourceKind};
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

        let mut packages = BTreeMap::new();
        packages.insert(
            entry_identity.clone(),
            LoweredPackage::new(LoweredPackageId(0), entry_identity.clone()),
        );
        packages.insert(
            shared_identity.clone(),
            LoweredPackage::new(LoweredPackageId(1), shared_identity),
        );

        let workspace = LoweredWorkspace::new(
            entry_identity.clone(),
            packages,
            LoweredSourceMap::default(),
        );

        assert_eq!(workspace.entry_identity(), &entry_identity);
        assert_eq!(workspace.entry_package().id, LoweredPackageId(0));
        assert_eq!(workspace.package_count(), 2);
    }

    #[test]
    fn lowered_source_map_shell_starts_empty() {
        let source_map = LoweredSourceMap::new();

        assert!(source_map.is_empty());
        assert!(source_map.entries().is_empty());
    }
}
